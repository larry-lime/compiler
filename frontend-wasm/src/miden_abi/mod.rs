pub(crate) mod transform;
pub(crate) mod tx_kernel;

use miden_core::crypto::hash::RpoDigest;
use miden_hir::{FunctionType, Symbol};

/// Parse the stable import function name and the hex encoded digest from the function name
pub fn parse_import_function_digest(import_name: &str) -> Result<(String, RpoDigest), String> {
    // parse the hex encoded digest from the function name in the angle brackets
    // and the function name (before the angle brackets) example:
    // "miden:tx_kernel/note.get_inputs"
    let mut parts = import_name.split('<');
    let function_name = parts.next().unwrap();
    let digest = parts
        .next()
        .and_then(|s| s.strip_suffix('>'))
        .ok_or_else(|| "Import name parsing error: missing closing angle bracket in import name")?;
    Ok((
        function_name.to_string(),
        RpoDigest::try_from(digest).map_err(|e| e.to_string())?,
    ))
}

pub fn is_miden_sdk_module(module_id: Symbol) -> bool {
    tx_kernel::types().contains_key(module_id.as_str())
}

/// Get the target Miden ABI tx kernel function type for the given module and function id
pub fn miden_sdk_function_type(module_id: Symbol, function_id: Symbol) -> FunctionType {
    let funcs = tx_kernel::types()
        .get(module_id.as_str())
        .expect(format!("No Miden ABI function types found for module {}", module_id).as_str());
    funcs.get(function_id.as_str()).cloned().expect(
        format!(
            "No Miden ABI function type found for function {} in module {}",
            function_id, module_id
        )
        .as_str(),
    )
}
