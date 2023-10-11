use std::{cell::RefCell, cmp, fmt, rc::Rc};

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use miden_hir::{Felt, FieldElement, FunctionIdent, Ident, OperandStack, Stack, StarkField};

use crate::{BlockId, Function, Module, Op, Program};

/// The type signature for native Rust functions callable from MASM IR
pub type NativeFn = dyn FnMut(&mut Emulator, &[Felt]) -> Result<(), EmulationError>;

/// The size/type of pointers in the emulator
type Addr = u32;

/// This type represents the various sorts of errors which can occur when
/// running the emulator on a MASM program. Some errors may result in panics,
/// but those which we can handle are represented here.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum EmulationError {
    /// The given module is already loaded
    #[error("unable to load module: '{0}' is already loaded")]
    AlreadyLoaded(Ident),
    /// The given function is already loaded
    #[error("unable to load function: '{0}' is already loaded")]
    DuplicateFunction(FunctionIdent),
    /// The given function cannot be found
    #[error("unable to invoke function: '{0}' is not defined")]
    UndefinedFunction(FunctionIdent),
    /// The emulator ran out of available memory
    #[error("system limit: out of memory")]
    OutOfMemory,
    /// The emulator was terminated due to a program failing to terminate in its budgeted time
    #[error("execution terminated prematurely: maximum cycle count reached")]
    CycleBudgetExceeded,
    /// A breakpoint was reached, so execution was suspended and can be resumed
    #[error("execution suspended by breakpoint")]
    BreakpointHit,
}

/// We allow functions in the emulator to be defined in either MASM IR, or native Rust.
///
/// Functions implemented in Rust are given a mutable reference to the emulator, so they
/// have virtually unlimited power, but are correspondingly very unsafe. With great
/// power comes great responsibility, etc.
#[derive(Clone)]
enum Stub {
    /// This function has a definition in Miden Assembly
    Asm(Rc<Function>),
    /// This function has a native Rust implementation
    Native(Rc<RefCell<Box<NativeFn>>>),
}

#[derive(Copy, Clone)]
pub enum Breakpoint {
    /// Break after one cycle
    Step,
    /// Break after `n` cycles
    StepN(usize),
    /// Break when the given function is called
    Call(FunctionIdent),
    /// Break when a write touches the region specified
    MemoryWrite { addr: usize, size: usize },
}

/// [Emulator] provides us with a means to execute our MASM IR directly
/// without having to emit "real" MASM and run it via the Miden VM.
/// In other words, it's a convenient way to run tests to verify the
/// expected behavior of a program without all of the baggage of the
/// Miden VM.
///
/// [Emulator] is necessarily a more limited execution environment:
///
/// * It only handles instructions which are defined in the [Op] enum
/// * Anything related to proving, calling contracts, etc. is not supported
/// * The default environment is empty, i.e. there are no Miden VM standard
/// library functions available. Users must emit Miden IR for all functions
/// they wish to call, or alternatively, provide native stubs.
pub struct Emulator {
    functions: FxHashMap<FunctionIdent, Stub>,
    locals: FxHashMap<FunctionIdent, Addr>,
    modules_loaded: FxHashSet<Ident>,
    modules_pending: FxHashSet<Ident>,
    memory: Vec<[Felt; 4]>,
    stack: OperandStack<Felt>,
    callstack: Vec<Activation>,
    hp: u32,
    lp: u32,
    bp: Option<Breakpoint>,
    clk: usize,
    clk_limit: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InstructionPointer {
    /// The block in which the instruction pointer is located
    pub block: BlockId,
    /// The index of the instruction pointed to
    pub index: usize,
}
impl InstructionPointer {
    pub const fn new(block: BlockId) -> Self {
        Self { block, index: 0 }
    }
}

/// Represents the current state of the program being executed for use in debugging/troubleshooting
pub struct DebugInfo<'a> {
    /// The current function being executed
    pub function: FunctionIdent,
    /// The address at which locals for the current function begin
    pub fp: Addr,
    /// The current instruction pointer value
    pub ip: InstructionPointer,
    /// The instruction under the instruction pointer
    pub ix: Option<Op>,
    /// The current state of the operand stack
    pub stack: &'a OperandStack<Felt>,
}
impl<'a> fmt::Debug for DebugInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DebugInfo")
            .field("function", &self.function)
            .field("fp", &self.fp)
            .field("ip", &self.ip)
            .field("ix", &self.ix)
            .field("stack", &self.stack.debug())
            .finish()
    }
}

struct Activation {
    function: Rc<Function>,
    ip: InstructionPointer,
    fp: Addr,
    repeat_stack: SmallVec<[Option<usize>; 2]>,
    ip_stack: SmallVec<[InstructionPointer; 2]>,
}
impl Activation {
    pub fn new(function: Rc<Function>, fp: Addr) -> Self {
        let block = function.body;
        Self {
            function,
            ip: InstructionPointer::new(block),
            fp,
            repeat_stack: Default::default(),
            ip_stack: Default::default(),
        }
    }

    // Peek at the next instruction to be executed
    fn peek_instruction(&self) -> Option<Op> {
        // Get code for this activation record
        let code = self.function.blocks[self.ip.block].ops.as_slice();
        // If we've reached the end of the current code block, attempt
        // to resume execution of the parent code block, if there is one
        if self.ip.index == code.len() {
            if let Some(Some(count)) = self.repeat_stack.last().copied() {
                if count > 0 {
                    let next = code.get(0).copied();
                    return next;
                }
            }
            let ip = self.ip_stack.last().copied()?;
            self.function.blocks[ip.block].ops.get(ip.index).copied()
        } else {
            code.get(self.ip.index).copied()
        }
    }

    // Get the instruction under the instruction pointer, and move the instruction pointer forward
    fn next_instruction(&mut self) -> Option<Op> {
        // Get code for this activation record
        let code = self.function.blocks[self.ip.block].ops.as_slice();
        // If we've reached the end of the current code block, attempt
        // to resume execution of the parent code block, if there is one
        if self.ip.index == code.len() {
            if let Some(Some(count)) = self.repeat_stack.pop() {
                if count > 0 {
                    self.repeat_stack.push(Some(count - 1));
                    self.ip.index = 1;
                    return code.get(0).copied();
                }
            }
            self.ip = self.ip_stack.pop()?;
            let next = self.function.blocks[self.ip.block]
                .ops
                .get(self.ip.index)
                .copied();
            self.ip.index += 1;
            next
        } else {
            let next = code.get(self.ip.index).copied();
            self.ip.index += 1;
            next
        }
    }

    fn enter_block(&mut self, block: BlockId) {
        self.ip_stack.push(self.ip);
        self.repeat_stack.push(None);
        self.ip = InstructionPointer::new(block);
    }

    fn enter_while_loop(&mut self, block: BlockId) {
        // We must revisit the while.true instruction on each iteration,
        // so move the instruction pointer back one
        let ip = InstructionPointer {
            block: self.ip.block,
            index: self.ip.index - 1,
        };
        self.ip_stack.push(ip);
        self.repeat_stack.push(None);
        self.ip = InstructionPointer::new(block);
    }

    fn repeat_block(&mut self, block: BlockId, count: usize) {
        self.ip_stack.push(self.ip);
        self.repeat_stack.push(Some(count - 1));
        self.ip = InstructionPointer::new(block);
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new(
            Self::DEFAULT_HEAP_SIZE,
            Self::DEFAULT_HEAP_START,
            Self::DEFAULT_LOCALS_START,
        )
    }
}
impl Emulator {
    const DEFAULT_HEAP_SIZE: u32 = (64 * 1024 * 4) / 32;
    const DEFAULT_HEAP_START: u32 = (64 * 1024 * 2) / 32;
    const DEFAULT_LOCALS_START: u32 = (64 * 1024 * 3) / 32;
    const EMPTY_WORD: [Felt; 4] = [Felt::ZERO; 4];

    /// Construct a new, empty emulator with:
    ///
    /// * A linear memory heap of `memory_size` words
    /// * The start of the usable heap set to `hp` (an address in words)
    /// * The start of the reserved heap used for locals set to `lp` (an address in words)
    ///
    pub fn new(memory_size: u32, hp: u32, lp: u32) -> Self {
        let memory = vec![Self::EMPTY_WORD; memory_size as usize];
        Self {
            functions: Default::default(),
            locals: Default::default(),
            modules_loaded: Default::default(),
            modules_pending: Default::default(),
            memory,
            stack: Default::default(),
            callstack: vec![],
            hp,
            lp,
            bp: None,
            clk: 0,
            clk_limit: usize::MAX,
        }
    }

    /// Place a cap on the number of cycles the emulator will execute before failing with an error
    pub fn set_max_cycles(&mut self, max: usize) {
        self.clk_limit = max;
    }

    /// Sets the next breakpoint for the emulator
    pub fn set_breakpoint(&mut self, bp: Breakpoint) {
        self.bp = Some(bp);
    }

    /// Clears any active breakpoint
    pub fn clear_breakpoint(&mut self) {
        self.bp = None;
    }

    /// Get's debug information about the current emulator state
    pub fn info(&self) -> Option<DebugInfo<'_>> {
        let current = self.callstack.last()?;
        Some(DebugInfo {
            function: current.function.name,
            fp: current.fp,
            ip: current.ip,
            ix: current.peek_instruction(),
            stack: &self.stack,
        })
    }

    /// Load `program` into this emulator
    pub fn load_program(&mut self, program: Program) -> Result<(), EmulationError> {
        for module in program.modules.into_iter() {
            self.load_module(module)?;
        }

        // TODO: Load data segments

        Ok(())
    }

    /// Load `module` into this emulator
    pub fn load_module(&mut self, mut module: Module) -> Result<(), EmulationError> {
        if !self.modules_loaded.insert(module.name) {
            return Err(EmulationError::AlreadyLoaded(module.name));
        }

        // Register module dependencies
        for import in module.imports.iter() {
            let name = Ident::with_empty_span(import.name);
            if self.modules_loaded.contains(&name) {
                continue;
            }
            self.modules_pending.insert(name);
        }
        self.modules_pending.remove(&module.name);

        // Load functions from this module
        while let Some(function) = module.functions.pop_front() {
            let function = Rc::from(function);
            self.load_function(function)?;
        }

        Ok(())
    }

    /// Load `function` into this emulator
    fn load_function(&mut self, function: Rc<Function>) -> Result<(), EmulationError> {
        let id = function.name;
        if self.functions.contains_key(&id) {
            return Err(EmulationError::DuplicateFunction(id));
        }
        let fp = self.lp;
        self.lp += function.locals().len() as u32;
        self.functions.insert(id, Stub::Asm(function));
        self.locals.insert(id, fp);

        Ok(())
    }

    /// Load `function` into this emulator, with the given identifier
    ///
    /// Because we don't know the set of [FuncId] that have already been allocated,
    /// we leave the the choice up to the caller. We assert that functions do
    /// not get defined twice to catch conflicts, just in case.
    pub fn load_nif(
        &mut self,
        id: FunctionIdent,
        function: Box<NativeFn>,
    ) -> Result<(), EmulationError> {
        if self.functions.contains_key(&id) {
            return Err(EmulationError::DuplicateFunction(id));
        }
        self.functions
            .insert(id, Stub::Native(Rc::new(RefCell::new(function))));

        Ok(())
    }

    /// Allocate space for `value` on the emulator heap, and copy it's contents there.
    ///
    /// NOTE: The smallest unit of addressable memory is 4 bytes (32 bits). If you provide
    /// a value that is smaller than this, or is not a multiple of 4, the data will be padded
    /// with zeroes to ensure that it is.
    pub fn write_bytes_to_memory(&mut self, value: &[u8]) -> u32 {
        let addr = self.hp;
        if value.is_empty() {
            return addr;
        }

        let mut elem_idx = 0;
        for chunk in value.chunks(4) {
            let elem = match chunk.len() {
                4 => u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                3 => u32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]),
                2 => u32::from_le_bytes([chunk[0], chunk[1], 0, 0]),
                1 => u32::from_le_bytes([chunk[0], 0, 0, 0]),
                0 => 0,
                _ => unreachable!(),
            };
            if elem_idx == 4 {
                elem_idx = 0;
                assert!(
                    self.hp + 1 < self.lp,
                    "heap has overflowed into reserved region"
                );
                self.hp += 1;
            }
            self.memory[self.hp as usize][elem_idx] = Felt::new(elem as u64);
            elem_idx += 1;
        }

        addr
    }

    /// Allocate enough words to hold `size` bytes of memory
    pub fn malloc(&mut self, size: usize) -> u32 {
        let addr = self.hp;

        if size == 0 {
            return addr;
        }

        let size = size as u32;
        let extra = size % 16;
        let words = (size / 16) + (extra > 0) as u32;
        assert!(
            self.hp + words < self.lp,
            "heap has overflowed into reserved region"
        );
        self.hp += words;

        addr
    }

    /// Write `value` to the word at `addr`, and element `index`
    pub fn store(&mut self, addr: usize, index: usize, value: Felt) {
        assert!(addr < self.memory.len(), "invalid address");
        self.memory[addr][index] = value;
    }

    /// Run the emulator by invoking `callee` with `args` placed on the
    /// operand stack in FIFO order.
    ///
    /// If a fatal error occurs during emulation, `Err` is returned,
    /// e.g. if `callee` has not been loaded.
    ///
    /// When `callee` returns, it's result will be returned wrapped in `Ok`.
    /// For functions with no return value, this will be `Ok(None)`, or all
    /// others it will be `Ok(Some(value))`.
    pub fn invoke(
        &mut self,
        callee: FunctionIdent,
        args: &[Felt],
    ) -> Result<OperandStack<Felt>, EmulationError> {
        let fun = self
            .functions
            .get(&callee)
            .cloned()
            .ok_or(EmulationError::UndefinedFunction(callee))?;
        match fun {
            Stub::Asm(ref function) => self.invoke_function(function.clone(), args),
            Stub::Native(function) => {
                let mut function = function.borrow_mut();
                function(self, args)?;
                Ok(self.stack.clone())
            }
        }
    }

    /// Invoke a function defined in MASM IR, placing the given arguments on the
    /// operand stack in FIFO order.
    #[inline]
    fn invoke_function(
        &mut self,
        function: Rc<Function>,
        args: &[Felt],
    ) -> Result<OperandStack<Felt>, EmulationError> {
        // Place the arguments on the operand stack
        assert_eq!(args.len(), function.arity());
        for arg in args.iter().copied().rev() {
            self.stack.push(arg);
        }

        // Schedule `function`
        let fp = self.locals[&function.name];
        let state = Activation::new(function.clone(), fp);
        self.callstack.push(state);

        self.resume()
    }

    /// Resume execution when the emulator suspended due to a breakpoint
    pub fn resume(&mut self) -> Result<OperandStack<Felt>, EmulationError> {
        self.run()?;

        Ok(self.stack.clone())
    }

    /// Run the emulator until all calls are completed, the cycle budget is exhausted,
    /// or a breakpoint is hit.
    ///
    /// It is expected that the caller has set up the operand stack with the correct
    /// number of arguments. If not, undefined behavior (from the perspective of the
    /// MASM program) will result.
    #[inline(never)]
    fn run(&mut self) -> Result<(), EmulationError> {
        // This is the core interpreter loop for MASM IR, it runs until one of the
        // following occurs:
        //
        // * We run out of code to execute, i.e. the function is returning normally
        // * Execution was explicitly aborted from within the function
        // * Execution traps due to a MASM invariant being violated, indicating the
        // code is malformed.
        // * Execution traps due to a runtime system error, e.g. out of memory
        'outer: loop {
            // Terminate execution early if we reach a predetermined number of cycles
            self.clk += 1;
            if self.clk > self.clk_limit {
                return Err(EmulationError::CycleBudgetExceeded);
            }

            match self.bp {
                None => {
                    if self.step()? {
                        break;
                    }
                }
                Some(Breakpoint::Step) => {
                    if self.step()? {
                        break;
                    } else {
                        return Err(EmulationError::BreakpointHit);
                    }
                }
                Some(Breakpoint::StepN(count)) => {
                    // Execute `count` instructions unless we reach the end or an error
                    let mut count = count;
                    while count > 0 {
                        count -= 1;
                        if self.step()? {
                            break 'outer;
                        }
                        self.clk += 1;
                    }
                    return Err(EmulationError::BreakpointHit);
                }
                Some(_) => {
                    // Handled by the step function
                    if self.step()? {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn step(&mut self) -> Result<bool, EmulationError> {
        const U32_P: u64 = 2u64.pow(32);
        const U32_BITS: u64 = 32;

        // If there are no more activation records, we're done
        if self.callstack.is_empty() {
            return Ok(true);
        }

        let mut state = self.callstack.pop().unwrap();
        if let Some(ix) = state.next_instruction() {
            match ix {
                Op::Padw => {
                    self.stack.padw();
                }
                Op::Push(v) => {
                    self.stack.push(v);
                }
                Op::Push2([a, b]) => {
                    self.stack.push(a);
                    self.stack.push(b);
                }
                Op::Pushw(word) => {
                    self.stack.pushw(word);
                }
                Op::PushU8(i) => {
                    self.stack.push_u8(i);
                }
                Op::PushU16(i) => {
                    self.stack.push_u16(i);
                }
                Op::PushU32(i) => {
                    self.stack.push_u32(i);
                }
                Op::Drop => {
                    self.stack.drop();
                }
                Op::Dropw => {
                    self.stack.dropw();
                }
                Op::Dup(pos) => {
                    self.stack.dup(pos as usize);
                }
                Op::Dupw(pos) => {
                    self.stack.dupw(pos as usize);
                }
                Op::Swap(pos) => {
                    self.stack.swap(pos as usize);
                }
                Op::Swapw(pos) => {
                    self.stack.swapw(pos as usize);
                }
                Op::Movup(pos) => {
                    self.stack.movup(pos as usize);
                }
                Op::Movupw(pos) => {
                    self.stack.movupw(pos as usize);
                }
                Op::Movdn(pos) => {
                    self.stack.movdn(pos as usize);
                }
                Op::Movdnw(pos) => {
                    self.stack.movdnw(pos as usize);
                }
                Op::Cswap => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true || cond == Felt::ZERO, "invalid boolean value");
                    if is_true {
                        self.stack.swap(1);
                    }
                }
                Op::Cswapw => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true || cond == Felt::ZERO, "invalid boolean value");
                    if is_true {
                        self.stack.swapw(1);
                    }
                }
                Op::Cdrop => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true || cond == Felt::ZERO, "invalid boolean value");
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    if is_true {
                        self.stack.push(b);
                    } else {
                        self.stack.push(a);
                    }
                }
                Op::Cdropw => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true || cond == Felt::ZERO, "invalid boolean value");
                    let b = self.stack.popw().expect("operand stack is empty");
                    let a = self.stack.popw().expect("operand stack is empty");
                    if is_true {
                        self.stack.pushw(b);
                    } else {
                        self.stack.pushw(a);
                    }
                }
                Op::Assert => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true, "assertion failed: got {cond}");
                }
                Op::Assertz => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_false = cond == Felt::ZERO;
                    assert!(is_false, "assertion failed: got {cond}");
                }
                Op::AssertEq => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert_eq!(a, b, "equality assertion failed");
                }
                Op::AssertEqw => {
                    let b = self.stack.popw().expect("operand stack is empty");
                    let a = self.stack.popw().expect("operand stack is empty");
                    assert_eq!(a, b, "equality assertion failed");
                }
                Op::LocAddr(id) => {
                    let addr = state.fp + id.as_usize() as u32;
                    debug_assert!(addr < self.memory.len() as u32);
                    self.stack.push_u32(addr);
                }
                Op::MemLoad => {
                    let addr = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(
                        addr < u32::MAX as u64,
                        "expected valid 32-bit address, got {addr}"
                    );
                    let addr = addr as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.stack.push(self.memory[addr][0]);
                }
                Op::MemLoadOffset => {
                    let offset = self.stack.pop().expect("operand stack is empty").as_int();
                    let addr = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(offset < 4, "expected valid element offset, got {offset}");
                    assert!(
                        addr < u32::MAX as u64,
                        "expected valid 32-bit address, got {addr}"
                    );
                    let addr = addr as usize;
                    let offset = offset as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.stack.push(self.memory[addr][offset]);
                }
                Op::MemLoadImm(addr) => {
                    let addr = addr as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.stack.push(self.memory[addr][0]);
                }
                Op::MemLoadOffsetImm(addr, offset) => {
                    let addr = addr as usize;
                    let offset = offset as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.stack.push(self.memory[addr][offset]);
                }
                Op::MemLoadw => {
                    let addr = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(
                        addr < u32::MAX as u64,
                        "expected valid 32-bit address, got {addr}"
                    );
                    let addr = addr as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.stack.dropw();
                    self.stack.pushw(self.memory[addr]);
                }
                Op::MemLoadwImm(addr) => {
                    let addr = addr as usize;
                    assert!(addr < self.memory.len() - 4, "out of bounds memory access");
                    self.stack.dropw();
                    self.stack.pushw(self.memory[addr]);
                }
                Op::MemStore => {
                    let addr = self.stack.pop().expect("operand stack is empty").as_int();
                    let value = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        addr < u32::MAX as u64,
                        "expected valid 32-bit address, got {addr}"
                    );
                    let addr = addr as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.memory[addr][0] = value;
                }
                Op::MemStoreOffset => {
                    let offset = self.stack.pop().expect("operand stack is empty").as_int();
                    let addr = self.stack.pop().expect("operand stack is empty").as_int();
                    let value = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        addr < u32::MAX as u64,
                        "expected valid 32-bit address, got {addr}"
                    );
                    assert!(offset < 4, "expected valid element offset, got {offset}");
                    let addr = addr as usize;
                    let offset = offset as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.memory[addr][offset] = value;
                }
                Op::MemStoreImm(addr) => {
                    let addr = addr as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    let value = self.stack.pop().expect("operand stack is empty");
                    self.memory[addr][0] = value;
                }
                Op::MemStoreOffsetImm(addr, offset) => {
                    let addr = addr as usize;
                    let offset = offset as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    let value = self.stack.pop().expect("operand stack is empty");
                    self.memory[addr][offset] = value;
                }
                Op::MemStorew => {
                    let addr = self.stack.pop().expect("operand stack is empty").as_int();
                    let word = self.stack.peekw().expect("operand stack is empty");
                    assert!(
                        addr < u32::MAX as u64,
                        "expected valid 32-bit address, got {addr}"
                    );
                    let addr = addr as usize;
                    assert!(addr < self.memory.len(), "out of bounds memory access");
                    self.memory[addr] = word;
                }
                Op::MemStorewImm(addr) => {
                    let addr = addr as usize;
                    assert!(addr < self.memory.len() - 4, "out of bounds memory access");
                    let word = self.stack.peekw().expect("operand stack is empty");
                    self.memory[addr] = word;
                }
                Op::If(then_blk, else_blk) => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true || cond == Felt::ZERO, "invalid boolean value");
                    if is_true {
                        state.enter_block(then_blk);
                    } else {
                        state.enter_block(else_blk);
                    }
                }
                Op::While(body_blk) => {
                    let cond = self.stack.pop().expect("operand stack is empty");
                    let is_true = cond == Felt::ONE;
                    assert!(is_true || cond == Felt::ZERO, "invalid boolean value");

                    if is_true {
                        state.enter_while_loop(body_blk);
                    }
                }
                Op::Repeat(n, body_blk) => {
                    state.repeat_block(body_blk, n as usize);
                }
                Op::Exec(callee) => {
                    let callee = callee;
                    let fun = self
                        .functions
                        .get(&callee)
                        .cloned()
                        .ok_or(EmulationError::UndefinedFunction(callee))?;
                    match fun {
                        Stub::Asm(ref function) => {
                            let fp = self.locals[&function.name];
                            let callee_state = Activation::new(function.clone(), fp);
                            // Suspend caller
                            self.callstack.push(state);
                            // Schedule callee next
                            self.callstack.push(callee_state);
                            if let Some(Breakpoint::Call(bp)) = self.bp {
                                if callee == bp {
                                    return Err(EmulationError::BreakpointHit);
                                }
                            }
                            return Ok(false);
                        }
                        Stub::Native(_function) => unimplemented!(),
                    }
                }
                Op::Syscall(_callee) => unimplemented!(),
                Op::Add => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a + b);
                }
                Op::AddImm(imm) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a + imm);
                }
                Op::Sub => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a - b);
                }
                Op::SubImm(imm) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a - imm);
                }
                Op::Mul => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a * b);
                }
                Op::MulImm(imm) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a * imm);
                }
                Op::Div => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a / b);
                }
                Op::DivImm(imm) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a / imm);
                }
                Op::Neg => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(-a);
                }
                Op::Inv => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a.inv());
                }
                Op::Incr => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push(a + Felt::ONE);
                }
                Op::Pow2 => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let two = Felt::new(2);
                    assert!(
                        a < 64,
                        "invalid power of two: expected {a} to be a value less than 64"
                    );
                    self.stack.push(two.exp(a));
                }
                Op::Exp => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        b < 64,
                        "invalid power of two: expected {b} to be a value less than 64"
                    );
                    self.stack.push(a.exp(b));
                }
                Op::ExpImm(pow) => {
                    let pow = pow as u64;
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        pow < 64,
                        "invalid power of two: expected {pow} to be a value less than 64"
                    );
                    self.stack.push(a.exp(pow));
                }
                Op::Not => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    let a = !(a == 1);
                    self.stack.push_u8(a as u8);
                }
                Op::And => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    assert!(b < 2, "invalid boolean value");
                    let result = (a == 1) & (b == 1);
                    self.stack.push_u8(result as u8);
                }
                Op::AndImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    let result = (a == 1) & b;
                    self.stack.push_u8(result as u8);
                }
                Op::Or => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    assert!(b < 2, "invalid boolean value");
                    let result = (a == 1) | (b == 1);
                    self.stack.push_u8(result as u8);
                }
                Op::OrImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    let a = a == 1;
                    let result = a | b;
                    self.stack.push_u8(result as u8);
                }
                Op::Xor => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    assert!(b < 2, "invalid boolean value");
                    let result = (a == 1) ^ (b == 1);
                    self.stack.push_u8(result as u8);
                }
                Op::XorImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < 2, "invalid boolean value");
                    let result = (a == 1) ^ b;
                    self.stack.push_u8(result as u8);
                }
                Op::Eq => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push_u8((a == b) as u8);
                }
                Op::EqImm(imm) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push_u8((a == imm) as u8);
                }
                Op::Neq => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push_u8((a != b) as u8);
                }
                Op::NeqImm(imm) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    self.stack.push_u8((a != imm) as u8);
                }
                Op::Gt => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a > b) as u8);
                }
                Op::GtImm(b) => {
                    let b = b.as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a > b) as u8);
                }
                Op::Gte => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a >= b) as u8);
                }
                Op::GteImm(b) => {
                    let b = b.as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a >= b) as u8);
                }
                Op::Lt => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a < b) as u8);
                }
                Op::LtImm(b) => {
                    let b = b.as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a < b) as u8);
                }
                Op::Lte => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a <= b) as u8);
                }
                Op::LteImm(b) => {
                    let b = b.as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a <= b) as u8);
                }
                Op::IsOdd => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a % 2 == 0) as u8);
                }
                Op::Eqw => {
                    let b = self.stack.popw().expect("operand stack is empty");
                    let a = self.stack.popw().expect("operand stack is empty");
                    self.stack.push_u8((a == b) as u8);
                }
                Op::Clk => {
                    self.stack.push(Felt::new(self.clk as u64));
                }
                Op::U32Test => {
                    let top = self.stack.peek().expect("operand stack is empty").as_int();
                    self.stack.push_u8((top < U32_P) as u8);
                }
                Op::U32Testw => {
                    let word = self.stack.peekw().expect("operand stack is empty");
                    let is_true = word.iter().all(|elem| elem.as_int() < U32_P);
                    self.stack.push_u8(is_true as u8);
                }
                Op::U32Assert => {
                    let top = self.stack.peek().expect("operand stack is empty").as_int();
                    assert!(top < U32_P, "assertion failed: {top} is larger than 2^32");
                }
                Op::U32Assert2 => {
                    let a = self.stack.peek().expect("operand stack is empty").as_int();
                    let b = self.stack.peek().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                }
                Op::U32Assertw => {
                    let word = self.stack.peekw().expect("operand stack is empty");
                    for elem in word.into_iter() {
                        assert!(
                            elem.as_int() < U32_P,
                            "assertion failed: {elem} is larger than 2^32"
                        );
                    }
                }
                Op::U32Cast => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push(Felt::new(a % U32_P));
                }
                Op::U32Split => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let hi = a / U32_P;
                    let lo = a % U32_P;
                    self.stack.push(Felt::new(lo));
                    self.stack.push(Felt::new(hi));
                }
                Op::U32CheckedAdd => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let result = a + b;
                    assert!(
                        result < U32_P,
                        "assertion failed: {result} is larger than 2^32"
                    );
                    self.stack.push(Felt::new(result));
                }
                Op::U32CheckedAddImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let result = a + b as u64;
                    assert!(
                        result < U32_P,
                        "assertion failed: {result} is larger than 2^32"
                    );
                    self.stack.push(Felt::new(result));
                }
                Op::U32OverflowingAdd => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let (result, overflowed) = (a as u32).overflowing_add(b as u32);
                    self.stack.push_u32(result);
                    self.stack.push_u8(overflowed as u8);
                }
                Op::U32OverflowingAddImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let (result, overflowed) = (a as u32).overflowing_add(b);
                    self.stack.push_u32(result);
                    self.stack.push_u8(overflowed as u8);
                }
                Op::U32WrappingAdd => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let result = (a as u32).wrapping_add(b as u32);
                    self.stack.push_u32(result);
                }
                Op::U32WrappingAddImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let result = (a as u32).wrapping_add(b);
                    self.stack.push_u32(result);
                }
                Op::U32OverflowingAdd3 => todo!(),
                Op::U32WrappingAdd3 => todo!(),
                Op::U32CheckedSub => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert!(a > b, "assertion failed: subtraction underflow: {a} - {b}");
                    self.stack.push(Felt::new(a - b));
                }
                Op::U32CheckedSubImm(b) => {
                    let b = b as u64;
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(a > b, "assertion failed: subtraction underflow: {a} - {b}");
                    self.stack.push(Felt::new(a - b));
                }
                Op::U32OverflowingSub => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let (result, underflowed) = (a as u32).overflowing_sub(b as u32);
                    self.stack.push_u32(result);
                    self.stack.push_u8(underflowed as u8);
                }
                Op::U32OverflowingSubImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let (result, underflowed) = (a as u32).overflowing_sub(b);
                    self.stack.push_u32(result);
                    self.stack.push_u8(underflowed as u8);
                }
                Op::U32WrappingSub => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let result = (a as u32).wrapping_sub(b as u32);
                    self.stack.push_u32(result);
                }
                Op::U32WrappingSubImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let result = (a as u32).wrapping_sub(b);
                    self.stack.push_u32(result);
                }
                Op::U32CheckedMul => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let result = a * b;
                    assert!(
                        result < U32_P,
                        "assertion failed: {result} is larger than 2^32"
                    );
                    self.stack.push(Felt::new(result));
                }
                Op::U32CheckedMulImm(b) => {
                    let b = b as u64;
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let result = a * b;
                    assert!(
                        result < U32_P,
                        "assertion failed: {result} is larger than 2^32"
                    );
                    self.stack.push(Felt::new(result));
                }
                Op::U32OverflowingMul => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let (result, overflowed) = (a as u32).overflowing_mul(b as u32);
                    self.stack.push_u32(result);
                    self.stack.push_u8(overflowed as u8);
                }
                Op::U32OverflowingMulImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let (result, overflowed) = (a as u32).overflowing_mul(b);
                    self.stack.push_u32(result);
                    self.stack.push_u8(overflowed as u8);
                }
                Op::U32WrappingMul => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let result = (a as u32).wrapping_mul(b as u32);
                    self.stack.push_u32(result);
                }
                Op::U32WrappingMulImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let result = (a as u32).wrapping_mul(b);
                    self.stack.push_u32(result);
                }
                Op::U32OverflowingMadd => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let c = self.stack.pop().expect("operand stack is empty").as_int();
                    let result = a * b + c;
                    let d = result % 2u64.pow(32);
                    let e = result / 2u64.pow(32);
                    self.stack.push_u32(d as u32);
                    self.stack.push_u32(e as u32);
                }
                Op::U32WrappingMadd => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let c = self.stack.pop().expect("operand stack is empty").as_int();
                    let d = (a * b + c) % 2u64.pow(32);
                    self.stack.push_u32(d as u32);
                }
                Op::U32CheckedDiv => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    self.stack.push(Felt::new(a / b));
                }
                Op::U32CheckedDivImm(b) => {
                    let b = b as u64;
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    self.stack.push(Felt::new(a / b));
                }
                Op::U32UncheckedDiv => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert_ne!(b, Felt::ZERO, "assertion failed: division by zero");
                    self.stack.push(a / b);
                }
                Op::U32UncheckedDivImm(b) => {
                    let b = b as u64;
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    let a = self.stack.pop().expect("operand stack is empty");
                    let b = Felt::new(b);
                    self.stack.push(a / b);
                }
                Op::U32CheckedMod => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32CheckedModImm(b) => {
                    let b = b as u64;
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32UncheckedMod => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32UncheckedModImm(b) => {
                    let b = b as u64;
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32CheckedDivMod => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    self.stack.push(Felt::new(a / b));
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32CheckedDivModImm(b) => {
                    let b = b as u64;
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    self.stack.push(Felt::new(a / b));
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32UncheckedDivMod => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    self.stack.push(Felt::new(a / b));
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32UncheckedDivModImm(b) => {
                    let b = b as u64;
                    assert_ne!(b, 0, "assertion failed: division by zero");
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push(Felt::new(a / b));
                    self.stack.push(Felt::new(a % b));
                }
                Op::U32And => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a & b);
                }
                Op::U32Or => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a | b);
                }
                Op::U32Xor => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert!(b < U32_BITS, "assertion failed: {b} is larger than 2^32");
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a ^ b);
                }
                Op::U32Not => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let a = a as u32;
                    self.stack.push_u32(!a);
                }
                Op::U32CheckedShl => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert!(
                        b < U32_BITS,
                        "assertion failed: {b} exceeds maximum shift of 31"
                    );
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a << b);
                }
                Op::U32CheckedShlImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < 32, "assertion failed: {b} exceeds maximum shift of 31");
                    let a = a as u32;
                    self.stack.push_u32(a << b);
                }
                Op::U32UncheckedShl => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a << b);
                }
                Op::U32UncheckedShlImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    self.stack.push_u32(a << b);
                }
                Op::U32CheckedShr => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert!(
                        b < U32_BITS,
                        "assertion failed: {b} exceeds maximum shift of 31"
                    );
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a >> b);
                }
                Op::U32CheckedShrImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < 32, "assertion failed: {b} exceeds maximum shift of 31");
                    let a = a as u32;
                    self.stack.push_u32(a >> b);
                }
                Op::U32UncheckedShr => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a >> b);
                }
                Op::U32UncheckedShrImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    self.stack.push_u32(a >> b);
                }
                Op::U32CheckedRotl => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert!(
                        b < U32_BITS,
                        "assertion failed: {b} exceeds maximum shift of 31"
                    );
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a.rotate_left(b));
                }
                Op::U32CheckedRotlImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < 32, "assertion failed: {b} exceeds maximum shift of 31");
                    let a = a as u32;
                    self.stack.push_u32(a.rotate_left(b));
                }
                Op::U32UncheckedRotl => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a.rotate_left(b));
                }
                Op::U32UncheckedRotlImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    self.stack.push_u32(a.rotate_left(b));
                }
                Op::U32CheckedRotr => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    assert!(
                        b < U32_BITS,
                        "assertion failed: {b} exceeds maximum shift of 31"
                    );
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a.rotate_right(b));
                }
                Op::U32CheckedRotrImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < 32, "assertion failed: {b} exceeds maximum shift of 31");
                    let a = a as u32;
                    self.stack.push_u32(a.rotate_right(b));
                }
                Op::U32UncheckedRotr => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    let b = b as u32;
                    self.stack.push_u32(a.rotate_right(b));
                }
                Op::U32UncheckedRotrImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    self.stack.push_u32(a.rotate_right(b));
                }
                Op::U32CheckedPopcnt => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    let a = a as u32;
                    self.stack.push_u32(a.count_ones());
                }
                Op::U32UncheckedPopcnt => {
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = a as u32;
                    self.stack.push_u32(a.count_ones());
                }
                Op::U32Eq => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        a.as_int() < U32_P,
                        "assertion failed: {a} is larger than 2^32"
                    );
                    assert!(
                        b.as_int() < U32_P,
                        "assertion failed: {b} is larger than 2^32"
                    );
                    self.stack.push_u8((a == b) as u8);
                }
                Op::U32EqImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        a.as_int() < U32_P,
                        "assertion failed: {a} is larger than 2^32"
                    );
                    let b = Felt::new(b as u64);
                    self.stack.push_u8((a == b) as u8);
                }
                Op::U32Neq => {
                    let b = self.stack.pop().expect("operand stack is empty");
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        a.as_int() < U32_P,
                        "assertion failed: {a} is larger than 2^32"
                    );
                    assert!(
                        b.as_int() < U32_P,
                        "assertion failed: {b} is larger than 2^32"
                    );
                    self.stack.push_u8((a != b) as u8);
                }
                Op::U32NeqImm(b) => {
                    let a = self.stack.pop().expect("operand stack is empty");
                    assert!(
                        a.as_int() < U32_P,
                        "assertion failed: {a} is larger than 2^32"
                    );
                    let b = Felt::new(b as u64);
                    self.stack.push_u8((a != b) as u8);
                }
                Op::U32CheckedGt => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    self.stack.push_u8((a > b) as u8);
                }
                Op::U32UncheckedGt => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a > b) as u8);
                }
                Op::U32CheckedGte => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    self.stack.push_u8((a >= b) as u8);
                }
                Op::U32UncheckedGte => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a >= b) as u8);
                }
                Op::U32CheckedLt => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    self.stack.push_u8((a < b) as u8);
                }
                Op::U32UncheckedLt => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a < b) as u8);
                }
                Op::U32CheckedLte => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    self.stack.push_u8((a <= b) as u8);
                }
                Op::U32UncheckedLte => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push_u8((a <= b) as u8);
                }
                Op::U32CheckedMin => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    self.stack.push(Felt::new(cmp::min(a, b)));
                }
                Op::U32UncheckedMin => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push(Felt::new(cmp::min(a, b)));
                }
                Op::U32CheckedMax => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    assert!(a < U32_P, "assertion failed: {a} is larger than 2^32");
                    assert!(b < U32_P, "assertion failed: {b} is larger than 2^32");
                    self.stack.push(Felt::new(cmp::max(a, b)));
                }
                Op::U32UncheckedMax => {
                    let b = self.stack.pop().expect("operand stack is empty").as_int();
                    let a = self.stack.pop().expect("operand stack is empty").as_int();
                    self.stack.push(Felt::new(cmp::max(a, b)));
                }
            }
        }

        // Suspend the current activation record
        self.callstack.push(state);

        Ok(false)
    }
}
