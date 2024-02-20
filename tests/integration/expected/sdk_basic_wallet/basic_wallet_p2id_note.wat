(component
  (type (;0;)
    (instance
      (type (;0;) (record (field "inner" float64)))
      (export (;1;) "felt" (type (eq 0)))
      (type (;2;) (record (field "inner" 1)))
      (export (;3;) "account-id" (type (eq 2)))
      (type (;4;) (tuple 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1))
      (export (;5;) "note-inputs" (type (eq 4)))
      (type (;6;) (tuple 1 1 1 1))
      (export (;7;) "word" (type (eq 6)))
      (type (;8;) (record (field "inner" 7)))
      (export (;9;) "core-asset" (type (eq 8)))
      (type (;10;) (func (param "felt" 1) (result 3)))
      (export (;0;) "account-id-from-felt" (func (type 10)))
    )
  )
  (import "miden:base/core-types@1.0.0" (instance (;0;) (type 0)))
  (alias export 0 "account-id" (type (;1;)))
  (type (;2;)
    (instance
      (alias outer 1 1 (type (;0;)))
      (export (;1;) "account-id" (type (eq 0)))
      (type (;2;) (func (result 1)))
      (export (;0;) "get-id" (func (type 2)))
    )
  )
  (import "miden:base/account@1.0.0" (instance (;1;) (type 2)))
  (alias export 0 "note-inputs" (type (;3;)))
  (alias export 0 "core-asset" (type (;4;)))
  (type (;5;)
    (instance
      (alias outer 1 3 (type (;0;)))
      (export (;1;) "note-inputs" (type (eq 0)))
      (alias outer 1 4 (type (;2;)))
      (export (;3;) "core-asset" (type (eq 2)))
      (type (;4;) (func (result 1)))
      (export (;0;) "get-inputs" (func (type 4)))
      (type (;5;) (list 3))
      (type (;6;) (func (result 5)))
      (export (;1;) "get-assets" (func (type 6)))
    )
  )
  (import "miden:base/note@1.0.0" (instance (;2;) (type 5)))
  (alias export 0 "core-asset" (type (;6;)))
  (type (;7;)
    (instance
      (alias outer 1 6 (type (;0;)))
      (export (;1;) "core-asset" (type (eq 0)))
      (type (;2;) (func (param "core-asset" 1)))
      (export (;0;) "receive-asset" (func (type 2)))
    )
  )
  (import "miden:basic-wallet/basic-wallet@1.0.0" (instance (;3;) (type 7)))
  (core module (;0;)
    (type (;0;) (func (param i32)))
<<<<<<< HEAD
    (type (;1;) (func (result i64)))
    (type (;2;) (func (param i32 i64 i64 i64 i64)))
    (type (;3;) (func))
    (type (;4;) (func (param i32 i32 i32 i32)))
    (type (;5;) (func (param i32 i32)))
    (type (;6;) (func (param i32 i32) (result i32)))
    (type (;7;) (func (param i32 i32 i32)))
    (type (;8;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;9;) (func (param i32 i32 i32) (result i32)))
    (import "miden:base/tx-kernel@1.0.0" "get-inputs" (func $basic_wallet_p2id_note::bindings::miden::base::tx_kernel::get_inputs::wit_import (;0;) (type 0)))
    (import "miden:base/tx-kernel@1.0.0" "get-id" (func $basic_wallet_p2id_note::bindings::miden::base::tx_kernel::get_id::wit_import (;1;) (type 1)))
    (import "miden:base/tx-kernel@1.0.0" "get-assets" (func $basic_wallet_p2id_note::bindings::miden::base::tx_kernel::get_assets::wit_import (;2;) (type 0)))
    (import "miden:basic-wallet/basic-wallet@1.0.0" "receive-asset" (func $basic_wallet_p2id_note::bindings::miden::basic_wallet::basic_wallet::receive_asset::wit_import (;3;) (type 2)))
    (func $__wasm_call_ctors (;4;) (type 3))
    (func $alloc::raw_vec::finish_grow (;5;) (type 4) (param i32 i32 i32 i32)
      (local i32 i32)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 1
            i32.eqz
            br_if 0 (;@3;)
            local.get 2
            i32.const -1
            i32.le_s
            br_if 1 (;@2;)
            block ;; label = @4
              block ;; label = @5
                block ;; label = @6
                  block ;; label = @7
                    local.get 3
                    i32.load offset=4
                    i32.eqz
                    br_if 0 (;@7;)
                    block ;; label = @8
                      local.get 3
                      i32.const 8
                      i32.add
                      i32.load
                      local.tee 4
                      br_if 0 (;@8;)
                      block ;; label = @9
                        local.get 2
                        br_if 0 (;@9;)
                        local.get 1
                        local.set 3
                        br 3 (;@6;)
                      end
                      i32.const 0
                      i32.load8_u offset=1048580
                      drop
                      i32.const 1048576
                      local.get 1
                      local.get 2
                      call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc
                      local.set 3
                      br 2 (;@6;)
                    end
                    local.get 3
                    i32.load
                    local.set 5
                    i32.const 1048576
                    local.get 1
                    local.get 2
                    call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc
                    local.tee 3
                    i32.eqz
                    br_if 3 (;@4;)
                    local.get 3
                    local.get 5
                    local.get 4
                    memory.copy
                    i32.const 1048576
                    local.get 5
                    local.get 1
                    local.get 4
                    call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::dealloc
                    br 2 (;@5;)
                  end
                  block ;; label = @7
                    local.get 2
                    br_if 0 (;@7;)
                    local.get 1
                    local.set 3
                    br 1 (;@6;)
                  end
                  i32.const 0
                  i32.load8_u offset=1048580
                  drop
                  i32.const 1048576
                  local.get 1
                  local.get 2
                  call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc
                  local.set 3
                end
                local.get 3
                i32.eqz
                br_if 1 (;@4;)
              end
              local.get 0
              local.get 3
              i32.store offset=4
              local.get 0
              i32.const 8
              i32.add
              local.get 2
              i32.store
              local.get 0
              i32.const 0
              i32.store
              return
            end
            local.get 0
            local.get 1
            i32.store offset=4
            local.get 0
            i32.const 8
            i32.add
            local.get 2
            i32.store
            br 2 (;@1;)
          end
          local.get 0
          i32.const 0
          i32.store offset=4
          local.get 0
          i32.const 8
          i32.add
          local.get 2
          i32.store
          br 1 (;@1;)
        end
        local.get 0
        i32.const 0
        i32.store offset=4
      end
      local.get 0
      i32.const 1
      i32.store
    )
    (func $alloc::raw_vec::RawVec<T,A>::reserve_for_push (;6;) (type 5) (param i32 i32)
      (local i32 i32 i32 i32)
=======
    (type (;1;) (func (param f64) (result f64)))
    (type (;2;) (func (result f64)))
    (type (;3;) (func (param f64 f64 f64 f64)))
    (type (;4;) (func))
    (type (;5;) (func (param i32 i32) (result i32)))
    (type (;6;) (func (param i32 i32 i32)))
    (type (;7;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;8;) (func (param i32) (result i32)))
    (type (;9;) (func (param i32 i32)))
    (type (;10;) (func (param i32 i32 i32) (result i32)))
    (import "miden:base/note@1.0.0" "get-inputs" (func $basic_wallet_p2id_note::bindings::miden::base::note::get_inputs::wit_import (;0;) (type 0)))
    (import "miden:base/core-types@1.0.0" "account-id-from-felt" (func $basic_wallet_p2id_note::bindings::miden::base::core_types::account_id_from_felt::wit_import (;1;) (type 1)))
    (import "miden:base/account@1.0.0" "get-id" (func $basic_wallet_p2id_note::bindings::miden::base::account::get_id::wit_import (;2;) (type 2)))
    (import "miden:base/note@1.0.0" "get-assets" (func $basic_wallet_p2id_note::bindings::miden::base::note::get_assets::wit_import (;3;) (type 0)))
    (import "miden:basic-wallet/basic-wallet@1.0.0" "receive-asset" (func $basic_wallet_p2id_note::bindings::miden::basic_wallet::basic_wallet::receive_asset::wit_import (;4;) (type 3)))
    (func $__wasm_call_ctors (;5;) (type 4))
    (func $miden:base/note-script@1.0.0#note-script (;6;) (type 4)
      (local i32 i32 i32 i32 i32)
>>>>>>> 886aab3 (refactor: make note's `get_inputs` return a `tuple` instead of `list` in Miden SDK;)
      global.get $__stack_pointer
      i32.const 128
      i32.sub
      local.tee 0
      global.set $__stack_pointer
      call $wit_bindgen::rt::run_ctors_once
      local.get 0
<<<<<<< HEAD
      i32.const 8
      i32.add
      call $basic_wallet_p2id_note::bindings::miden::base::tx_kernel::get_inputs::wit_import
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                local.get 0
                i32.const 24
                i32.add
                i32.load
                local.tee 1
                i32.eqz
                br_if 0 (;@5;)
                local.get 0
                i32.load offset=20
                local.tee 2
                i64.load
                call $basic_wallet_p2id_note::bindings::miden::base::tx_kernel::get_id::wit_import
                i64.ne
                br_if 0 (;@5;)
                local.get 0
                i32.const 12
                i32.add
                call $basic_wallet_p2id_note::bindings::miden::base::tx_kernel::get_assets::wit_import
                local.get 0
                i32.load offset=12
                local.set 3
                block ;; label = @6
                  local.get 0
                  i32.const 16
                  i32.add
                  i32.load
                  local.tee 4
                  i32.eqz
                  br_if 0 (;@6;)
                  local.get 4
                  i32.const 53687091
                  i32.gt_u
                  br_if 2 (;@4;)
                  local.get 4
                  i32.const 40
                  i32.mul
                  local.tee 5
                  i32.const -1
                  i32.le_s
                  br_if 2 (;@4;)
                  i32.const 0
                  local.set 6
                  i32.const 0
                  i32.load8_u offset=1048580
                  drop
                  i32.const 1048576
                  i32.const 8
                  local.get 5
                  call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc
                  local.tee 7
                  i32.eqz
                  br_if 3 (;@3;)
                  local.get 0
                  i32.const 0
                  i32.store offset=28
                  local.get 0
                  local.get 4
                  i32.store offset=24
                  local.get 0
                  local.get 7
                  i32.store offset=20
                  local.get 3
                  local.set 8
                  loop ;; label = @7
                    block ;; label = @8
                      block ;; label = @9
                        local.get 8
                        i32.load8_u
                        br_if 0 (;@9;)
                        i64.const 0
                        local.set 9
                        br 1 (;@8;)
                      end
                      local.get 8
                      i32.const 32
                      i32.add
                      i64.load
                      local.set 10
                      local.get 8
                      i32.const 24
                      i32.add
                      i64.load
                      local.set 11
                      i64.const 1
                      local.set 9
                    end
                    local.get 8
                    i32.const 8
                    i32.add
                    i64.load
                    local.set 12
                    local.get 8
                    i32.const 16
                    i32.add
                    i64.load
                    local.set 13
                    block ;; label = @8
                      local.get 6
                      local.get 0
                      i32.load offset=24
                      i32.ne
                      br_if 0 (;@8;)
                      local.get 0
                      i32.const 20
                      i32.add
                      local.get 6
                      call $alloc::raw_vec::RawVec<T,A>::reserve_for_push
                      local.get 0
                      i32.load offset=20
                      local.set 7
                      local.get 0
                      i32.load offset=28
                      local.set 6
                    end
                    local.get 7
                    local.get 6
                    i32.const 40
                    i32.mul
                    i32.add
                    local.tee 14
                    local.get 10
                    i64.store offset=32
                    local.get 14
                    local.get 11
                    i64.store offset=24
                    local.get 14
                    local.get 13
                    i64.store offset=16
                    local.get 14
                    local.get 12
                    i64.store offset=8
                    local.get 14
                    local.get 9
                    i64.store
                    local.get 0
                    local.get 6
                    i32.const 1
                    i32.add
                    local.tee 6
                    i32.store offset=28
                    local.get 8
                    i32.const 40
                    i32.add
                    local.set 8
                    local.get 4
                    i32.const -1
                    i32.add
                    local.tee 4
                    br_if 0 (;@7;)
                  end
                  local.get 0
                  i32.load offset=24
                  local.set 7
                  local.get 0
                  i32.load offset=20
                  local.set 4
                  local.get 3
                  local.get 5
                  i32.const 8
                  call $wit_bindgen::rt::dealloc
                  local.get 6
                  i32.eqz
                  br_if 4 (;@2;)
                  local.get 4
                  local.get 6
                  i32.const 40
                  i32.mul
                  i32.add
                  local.set 14
                  local.get 4
                  local.set 8
                  loop ;; label = @7
                    local.get 8
                    i64.load
                    local.tee 9
                    i64.const 2
                    i64.eq
                    br_if 5 (;@2;)
                    local.get 9
                    i64.const 0
                    i64.ne
                    local.tee 6
                    local.get 8
                    i64.load offset=8
                    local.get 8
                    i64.load offset=16
                    local.get 8
                    i64.load offset=24
                    i64.const 0
                    local.get 6
                    select
                    local.get 8
                    i64.load offset=32
                    i64.const 0
                    local.get 6
                    select
                    call $basic_wallet_p2id_note::bindings::miden::basic_wallet::basic_wallet::receive_asset::wit_import
                    local.get 8
                    i32.const 40
                    i32.add
                    local.tee 8
                    local.get 14
                    i32.ne
                    br_if 0 (;@7;)
                    br 5 (;@2;)
                  end
                end
                local.get 3
                i32.const 0
                i32.const 8
                call $wit_bindgen::rt::dealloc
                br 4 (;@1;)
              end
              unreachable
              unreachable
            end
            call $alloc::raw_vec::capacity_overflow
            unreachable
          end
          i32.const 8
          local.get 5
          call $alloc::alloc::handle_alloc_error
          unreachable
        end
        local.get 7
        i32.eqz
        br_if 0 (;@1;)
        i32.const 1048576
        local.get 4
        i32.const 8
        local.get 7
        i32.const 40
        i32.mul
        call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::dealloc
=======
      call $basic_wallet_p2id_note::bindings::miden::base::note::get_inputs::wit_import
      block ;; label = @1
        local.get 0
        f64.load
        call $basic_wallet_p2id_note::bindings::miden::base::core_types::account_id_from_felt::wit_import
        call $basic_wallet_p2id_note::bindings::miden::base::account::get_id::wit_import
        f64.ne
        br_if 0 (;@1;)
        local.get 0
        call $basic_wallet_p2id_note::bindings::miden::base::note::get_assets::wit_import
        block ;; label = @2
          local.get 0
          i32.const 4
          i32.add
          i32.load
          local.tee 1
          i32.eqz
          br_if 0 (;@2;)
          local.get 0
          i32.load
          local.tee 2
          local.get 1
          i32.const 5
          i32.shl
          i32.add
          local.set 3
          local.get 2
          local.set 4
          loop ;; label = @3
            local.get 4
            f64.load
            local.get 4
            f64.load offset=8
            local.get 4
            f64.load offset=16
            local.get 4
            f64.load offset=24
            call $basic_wallet_p2id_note::bindings::miden::basic_wallet::basic_wallet::receive_asset::wit_import
            local.get 4
            i32.const 32
            i32.add
            local.tee 4
            local.get 3
            i32.ne
            br_if 0 (;@3;)
          end
          local.get 2
          local.get 1
          i32.const 5
          i32.shl
          i32.const 8
          call $__rust_dealloc
        end
        local.get 0
        i32.const 128
        i32.add
        global.set $__stack_pointer
        return
>>>>>>> 886aab3 (refactor: make note's `get_inputs` return a `tuple` instead of `list` in Miden SDK;)
      end
      i32.const 1048576
      local.get 2
      i32.const 8
      local.get 1
      i32.const 3
      i32.shl
      call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::dealloc
      local.get 0
      i32.const 32
      i32.add
      global.set $__stack_pointer
    )
    (func $__rust_alloc (;8;) (type 6) (param i32 i32) (result i32)
      i32.const 1048576
      local.get 1
      local.get 0
      call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc
    )
    (func $__rust_dealloc (;9;) (type 7) (param i32 i32 i32)
      i32.const 1048576
      local.get 0
      local.get 2
      local.get 1
      call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::dealloc
    )
    (func $__rust_realloc (;9;) (type 7) (param i32 i32 i32 i32) (result i32)
      (local i32)
      block ;; label = @1
        i32.const 1048576
        local.get 2
        local.get 3
        call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc
        local.tee 4
        i32.eqz
        br_if 0 (;@1;)
        local.get 4
        local.get 0
        local.get 1
        local.get 3
        local.get 1
        local.get 3
        i32.lt_u
        select
        memory.copy
        i32.const 1048576
        local.get 0
        local.get 2
        local.get 1
        call $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::dealloc
      end
      local.get 4
    )
    (func $wee_alloc::alloc_first_fit (;11;) (type 9) (param i32 i32 i32) (result i32)
      (local i32 i32 i32 i32 i32 i32 i32)
      block ;; label = @1
        local.get 2
        i32.load
        local.tee 3
        br_if 0 (;@1;)
        i32.const 0
        return
      end
      local.get 1
      i32.const -1
      i32.add
      local.set 4
      i32.const 0
      local.get 1
      i32.sub
      local.set 5
      local.get 0
      i32.const 2
      i32.shl
      local.set 6
      loop ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 3
            i32.load offset=8
            local.tee 1
            i32.const 1
            i32.and
            br_if 0 (;@3;)
            local.get 3
            local.set 7
            br 1 (;@2;)
          end
          loop ;; label = @3
            local.get 3
            local.get 1
            i32.const -2
            i32.and
            i32.store offset=8
            block ;; label = @4
              block ;; label = @5
                local.get 3
                i32.load offset=4
                local.tee 8
                i32.const -4
                i32.and
                local.tee 1
                br_if 0 (;@5;)
                i32.const 0
                local.set 7
                br 1 (;@4;)
              end
              i32.const 0
              local.get 1
              local.get 1
              i32.load8_u
              i32.const 1
              i32.and
              select
              local.set 7
            end
            block ;; label = @4
              local.get 3
              i32.load
              local.tee 0
              i32.const -4
              i32.and
              local.tee 9
              i32.eqz
              br_if 0 (;@4;)
              local.get 0
              i32.const 2
              i32.and
              br_if 0 (;@4;)
              local.get 9
              local.get 9
              i32.load offset=4
              i32.const 3
              i32.and
              local.get 1
              i32.or
              i32.store offset=4
              local.get 3
              i32.load offset=4
              local.tee 8
              i32.const -4
              i32.and
              local.set 1
              local.get 3
              i32.load
              local.set 0
            end
            block ;; label = @4
              local.get 1
              i32.eqz
              br_if 0 (;@4;)
              local.get 1
              local.get 1
              i32.load
              i32.const 3
              i32.and
              local.get 0
              i32.const -4
              i32.and
              i32.or
              i32.store
              local.get 3
              i32.load offset=4
              local.set 8
              local.get 3
              i32.load
              local.set 0
            end
            local.get 3
            local.get 8
            i32.const 3
            i32.and
            i32.store offset=4
            local.get 3
            local.get 0
            i32.const 3
            i32.and
            i32.store
            block ;; label = @4
              local.get 0
              i32.const 2
              i32.and
              i32.eqz
              br_if 0 (;@4;)
              local.get 7
              local.get 7
              i32.load
              i32.const 2
              i32.or
              i32.store
            end
            local.get 2
            local.get 7
            i32.store
            local.get 7
            local.set 3
            local.get 7
            i32.load offset=8
            local.tee 1
            i32.const 1
            i32.and
            br_if 0 (;@3;)
          end
        end
        block ;; label = @2
          local.get 7
          i32.load
          i32.const -4
          i32.and
          local.tee 0
          local.get 7
          i32.const 8
          i32.add
          local.tee 3
          i32.sub
          local.get 6
          i32.lt_u
          br_if 0 (;@2;)
          block ;; label = @3
            block ;; label = @4
              local.get 3
              i32.const 72
              i32.add
              local.get 0
              local.get 6
              i32.sub
              local.get 5
              i32.and
              local.tee 0
              i32.le_u
              br_if 0 (;@4;)
              local.get 4
              local.get 3
              i32.and
              br_if 2 (;@2;)
              local.get 2
              local.get 7
              i32.load offset=8
              i32.const -4
              i32.and
              i32.store
              local.get 7
              i32.load
              local.set 1
              local.get 7
              local.set 3
              br 1 (;@3;)
            end
            i32.const 0
            local.set 1
            local.get 0
            i32.const 0
            i32.store
            local.get 0
            i32.const -8
            i32.add
            local.tee 3
            i64.const 0
            i64.store align=4
            local.get 3
            local.get 7
            i32.load
            i32.const -4
            i32.and
            i32.store
            block ;; label = @4
              local.get 7
              i32.load
              local.tee 8
              i32.const -4
              i32.and
              local.tee 0
              i32.eqz
              br_if 0 (;@4;)
              local.get 8
              i32.const 2
              i32.and
              br_if 0 (;@4;)
              local.get 0
              local.get 0
              i32.load offset=4
              i32.const 3
              i32.and
              local.get 3
              i32.or
              i32.store offset=4
              local.get 3
              i32.load offset=4
              i32.const 3
              i32.and
              local.set 1
            end
            local.get 3
            local.get 1
            local.get 7
            i32.or
            i32.store offset=4
            local.get 7
            local.get 7
            i32.load offset=8
            i32.const -2
            i32.and
            i32.store offset=8
            local.get 7
            local.get 7
            i32.load
            local.tee 1
            i32.const 3
            i32.and
            local.get 3
            i32.or
            local.tee 0
            i32.store
            block ;; label = @4
              local.get 1
              i32.const 2
              i32.and
              br_if 0 (;@4;)
              local.get 3
              i32.load
              local.set 1
              br 1 (;@3;)
            end
            local.get 7
            local.get 0
            i32.const -3
            i32.and
            i32.store
            local.get 3
            i32.load
            i32.const 2
            i32.or
            local.set 1
          end
          local.get 3
          local.get 1
          i32.const 1
          i32.or
          i32.store
          local.get 3
          i32.const 8
          i32.add
          return
        end
        local.get 2
        local.get 1
        i32.store
        local.get 1
        local.set 3
        local.get 1
        br_if 0 (;@1;)
      end
      i32.const 0
    )
    (func $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::alloc (;12;) (type 9) (param i32 i32 i32) (result i32)
      (local i32 i32 i32)
      global.get $__stack_pointer
      i32.const 16
      i32.sub
      local.tee 3
      global.set $__stack_pointer
      block ;; label = @1
        block ;; label = @2
          local.get 2
          br_if 0 (;@2;)
          local.get 1
          local.set 2
          br 1 (;@1;)
        end
        local.get 3
        local.get 0
        i32.load
        i32.store offset=12
        block ;; label = @2
          local.get 2
          i32.const 3
          i32.add
          local.tee 4
          i32.const 2
          i32.shr_u
          local.tee 5
          local.get 1
          local.get 3
          i32.const 12
          i32.add
          call $wee_alloc::alloc_first_fit
          local.tee 2
          br_if 0 (;@2;)
          block ;; label = @3
            local.get 4
            i32.const -4
            i32.and
            local.tee 2
            local.get 1
            i32.const 3
            i32.shl
            i32.const 512
            i32.add
            local.tee 4
            local.get 2
            local.get 4
            i32.gt_u
            select
            i32.const 65543
            i32.add
            local.tee 4
            i32.const 16
            i32.shr_u
            memory.grow
            local.tee 2
            i32.const -1
            i32.ne
            br_if 0 (;@3;)
            i32.const 0
            local.set 2
            br 1 (;@2;)
          end
          local.get 2
          i32.const 16
          i32.shl
          local.tee 2
          i32.const 0
          i32.store offset=4
          local.get 2
          local.get 3
          i32.load offset=12
          i32.store offset=8
          local.get 2
          local.get 2
          local.get 4
          i32.const -65536
          i32.and
          i32.add
          i32.const 2
          i32.or
          i32.store
          local.get 3
          local.get 2
          i32.store offset=12
          local.get 5
          local.get 1
          local.get 3
          i32.const 12
          i32.add
          call $wee_alloc::alloc_first_fit
          local.set 2
        end
        local.get 0
        local.get 3
        i32.load offset=12
        i32.store
      end
      local.get 3
      i32.const 16
      i32.add
      global.set $__stack_pointer
      local.get 2
    )
    (func $<wee_alloc::WeeAlloc as core::alloc::global::GlobalAlloc>::dealloc (;13;) (type 4) (param i32 i32 i32 i32)
      (local i32 i32 i32 i32 i32 i32 i32)
      block ;; label = @1
        local.get 1
        i32.eqz
        br_if 0 (;@1;)
        local.get 3
        i32.eqz
        br_if 0 (;@1;)
        local.get 0
        i32.load
        local.set 4
        local.get 1
        i32.const 0
        i32.store
        local.get 1
        i32.const -8
        i32.add
        local.tee 3
        local.get 3
        i32.load
        local.tee 5
        i32.const -2
        i32.and
        local.tee 6
        i32.store
        block ;; label = @2
          block ;; label = @3
            block ;; label = @4
              block ;; label = @5
                local.get 3
                i32.const 4
                i32.add
                local.tee 7
                i32.load
                i32.const -4
                i32.and
                local.tee 8
                i32.eqz
                br_if 0 (;@5;)
                local.get 8
                i32.load
                local.tee 9
                i32.const 1
                i32.and
                br_if 0 (;@5;)
                block ;; label = @6
                  block ;; label = @7
                    block ;; label = @8
                      local.get 5
                      i32.const -4
                      i32.and
                      local.tee 10
                      br_if 0 (;@8;)
                      local.get 8
                      local.set 1
                      br 1 (;@7;)
                    end
                    local.get 8
                    local.set 1
                    local.get 5
                    i32.const 2
                    i32.and
                    br_if 0 (;@7;)
                    local.get 10
                    local.get 10
                    i32.load offset=4
                    i32.const 3
                    i32.and
                    local.get 8
                    i32.or
                    i32.store offset=4
                    local.get 3
                    i32.load
                    local.set 6
                    local.get 7
                    i32.load
                    local.tee 5
                    i32.const -4
                    i32.and
                    local.tee 1
                    i32.eqz
                    br_if 1 (;@6;)
                    local.get 1
                    i32.load
                    local.set 9
                  end
                  local.get 1
                  local.get 6
                  i32.const -4
                  i32.and
                  local.get 9
                  i32.const 3
                  i32.and
                  i32.or
                  i32.store
                  local.get 7
                  i32.load
                  local.set 5
                  local.get 3
                  i32.load
                  local.set 6
                end
                local.get 7
                local.get 5
                i32.const 3
                i32.and
                i32.store
                local.get 3
                local.get 6
                i32.const 3
                i32.and
                i32.store
                local.get 6
                i32.const 2
                i32.and
                i32.eqz
                br_if 1 (;@4;)
                local.get 8
                local.get 8
                i32.load
                i32.const 2
                i32.or
                i32.store
                br 1 (;@4;)
              end
              local.get 5
              i32.const -4
              i32.and
              local.tee 8
              i32.eqz
              br_if 1 (;@3;)
              local.get 5
              i32.const 2
              i32.and
              br_if 1 (;@3;)
              local.get 8
              i32.load8_u
              i32.const 1
              i32.and
              br_if 1 (;@3;)
              local.get 1
              local.get 8
              i32.load offset=8
              i32.const -4
              i32.and
              i32.store
              local.get 8
              local.get 3
              i32.const 1
              i32.or
              i32.store offset=8
            end
            local.get 4
            local.set 3
            br 1 (;@2;)
          end
          local.get 1
          local.get 4
          i32.store
        end
        local.get 0
        local.get 3
        i32.store
      end
    )
    (func $wit_bindgen::rt::run_ctors_once (;14;) (type 3)
      block ;; label = @1
        i32.const 0
        i32.load8_u offset=1048581
        br_if 0 (;@1;)
        call $__wasm_call_ctors
        i32.const 0
        i32.const 1
        i32.store8 offset=1048581
      end
    )
    (func $cabi_realloc (;15;) (type 8) (param i32 i32 i32 i32) (result i32)
      block ;; label = @1
        block ;; label = @2
          block ;; label = @3
            local.get 1
            br_if 0 (;@3;)
            local.get 3
            i32.eqz
            br_if 2 (;@1;)
            i32.const 0
            i32.load8_u offset=1048580
            drop
            local.get 3
            local.get 2
            call $__rust_alloc
            local.set 2
            br 1 (;@2;)
          end
          local.get 0
          local.get 1
          local.get 2
          local.get 3
          call $__rust_realloc
          local.set 2
        end
        local.get 2
        br_if 0 (;@1;)
        unreachable
        unreachable
      end
      local.get 2
    )
    (func $wit_bindgen::rt::dealloc (;16;) (type 7) (param i32 i32 i32)
      block ;; label = @1
        local.get 1
        i32.eqz
        br_if 0 (;@1;)
        local.get 0
        local.get 1
        local.get 2
        call $__rust_dealloc
      end
    )
    (func $alloc::alloc::handle_alloc_error (;17;) (type 5) (param i32 i32)
      unreachable
      unreachable
    )
    (func $alloc::raw_vec::capacity_overflow (;18;) (type 3)
      unreachable
      unreachable
    )
    (table (;0;) 1 1 funcref)
    (memory (;0;) 17)
    (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
    (export "memory" (memory 0))
    (export "miden:base/note-script@1.0.0#note-script" (func $miden:base/note-script@1.0.0#note-script))
    (export "cabi_realloc" (func $cabi_realloc))
  )
  (core module (;1;)
    (type (;0;) (func (param i32)))
    (func $indirect-miden:base/note@1.0.0-get-inputs (;0;) (type 0) (param i32)
      local.get 0
      i32.const 0
      call_indirect (type 0)
    )
    (func $indirect-miden:base/note@1.0.0-get-assets (;1;) (type 0) (param i32)
      local.get 0
      i32.const 1
      call_indirect (type 0)
    )
    (table (;0;) 2 2 funcref)
    (export "0" (func $indirect-miden:base/note@1.0.0-get-inputs))
    (export "1" (func $indirect-miden:base/note@1.0.0-get-assets))
    (export "$imports" (table 0))
  )
  (core module (;2;)
    (type (;0;) (func (param i32)))
    (import "" "0" (func (;0;) (type 0)))
    (import "" "1" (func (;1;) (type 0)))
    (import "" "$imports" (table (;0;) 2 2 funcref))
    (elem (;0;) (i32.const 0) func 0 1)
  )
  (core instance (;0;) (instantiate 1))
  (alias core export 0 "0" (core func (;0;)))
  (alias core export 0 "1" (core func (;1;)))
  (core instance (;1;)
    (export "get-inputs" (func 0))
    (export "get-assets" (func 1))
  )
  (alias export 0 "account-id-from-felt" (func (;0;)))
  (core func (;2;) (canon lower (func 0)))
  (core instance (;2;)
    (export "account-id-from-felt" (func 2))
  )
  (alias export 1 "get-id" (func (;1;)))
  (core func (;3;) (canon lower (func 1)))
  (core instance (;3;)
    (export "get-id" (func 3))
  )
  (alias export 3 "receive-asset" (func (;2;)))
  (core func (;4;) (canon lower (func 2)))
  (core instance (;4;)
    (export "receive-asset" (func 4))
  )
  (core instance (;5;) (instantiate 0
      (with "miden:base/note@1.0.0" (instance 1))
      (with "miden:base/core-types@1.0.0" (instance 2))
      (with "miden:base/account@1.0.0" (instance 3))
      (with "miden:basic-wallet/basic-wallet@1.0.0" (instance 4))
    )
  )
  (alias core export 5 "memory" (core memory (;0;)))
  (alias core export 5 "cabi_realloc" (core func (;5;)))
  (alias core export 0 "$imports" (core table (;0;)))
  (alias export 2 "get-inputs" (func (;3;)))
  (core func (;6;) (canon lower (func 3) (memory 0)))
  (alias export 2 "get-assets" (func (;4;)))
  (core func (;7;) (canon lower (func 4) (memory 0) (realloc 5)))
  (core instance (;6;)
    (export "$imports" (table 0))
    (export "0" (func 6))
    (export "1" (func 7))
  )
  (core instance (;7;) (instantiate 2
      (with "" (instance 6))
    )
  )
  (type (;8;) (func))
  (alias core export 5 "miden:base/note-script@1.0.0#note-script" (core func (;8;)))
  (func (;5;) (type 8) (canon lift (core func 8)))
  (component (;0;)
    (type (;0;) (func))
    (import "import-func-note-script" (func (;0;) (type 0)))
    (type (;1;) (func))
    (export (;1;) "note-script" (func 0) (func (type 1)))
  )
  (instance (;4;) (instantiate 0
      (with "import-func-note-script" (func 5))
    )
  )
  (export (;5;) "miden:base/note-script@1.0.0" (instance 4))
)