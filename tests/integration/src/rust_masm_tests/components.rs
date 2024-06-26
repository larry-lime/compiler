use expect_test::expect_file;
use miden_core::crypto::hash::RpoDigest;
use miden_frontend_wasm::{ImportMetadata, WasmTranslationConfig};
use miden_hir::{FunctionType, Ident, InterfaceFunctionIdent, InterfaceIdent, Symbol, Type};

use crate::CompilerTest;

#[test]
fn wcm_add() {
    // Has no imports
    let config = Default::default();
    let mut test = CompilerTest::rust_source_cargo_component("add-comp", config);
    let artifact_name = test.source.artifact_name();
    test.expect_wasm(expect_file![format!("../../expected/components/{artifact_name}.wat")]);
    test.expect_ir(expect_file![format!("../../expected/components/{artifact_name}.hir")]);
}

#[test]
fn wcm_inc() {
    // Imports an add component used in the above test

    let interface_function_ident = InterfaceFunctionIdent {
        interface: InterfaceIdent::from_full_ident(
            "miden:add-package/add-interface@1.0.0".to_string(),
        ),
        function: Symbol::intern("add"),
    };
    let import_metadata = [(
        interface_function_ident.clone(),
        ImportMetadata {
            digest: RpoDigest::default(),
        },
    )]
    .into_iter()
    .collect();

    let config = WasmTranslationConfig {
        import_metadata,
        ..Default::default()
    };
    let mut test = CompilerTest::rust_source_cargo_component("inc-comp", config);
    let artifact_name = test.source.artifact_name();
    test.expect_wasm(expect_file![format!("../../expected/components/{artifact_name}.wat")]);
    test.expect_ir(expect_file![format!("../../expected/components/{artifact_name}.hir")]);

    let ir_component = test.hir().unwrap_component();

    assert!(!ir_component.modules().is_empty());

    let export_name_sym = Symbol::intern("inc");
    let export = ir_component.exports().get(&export_name_sym.into()).unwrap();
    assert_eq!(export.function.function.as_symbol(), export_name_sym);

    let expected_export_func_ty = FunctionType::new_wasm(vec![Type::U32], vec![Type::U32]);
    assert_eq!(export.function_ty, expected_export_func_ty);
    let module = ir_component.modules().first().unwrap().1;
    dbg!(&module.imports());
    let import_info = module.imports();
    let function_id = import_info
        .imported(&Ident::from("miden:add-package/add-interface@1.0.0"))
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>()
        .first()
        .cloned()
        .unwrap()
        .clone();
    let component_import =
        ir_component.imports().get(&function_id).unwrap().unwrap_canon_abi_import();
    assert_eq!(component_import.interface_function, interface_function_ident);
    assert!(!component_import.function_ty.params.is_empty());
    let expected_import_func_ty =
        FunctionType::new_wasm(vec![Type::U32, Type::U32], vec![Type::U32]);
    assert_eq!(component_import.function_ty, expected_import_func_ty);
}
