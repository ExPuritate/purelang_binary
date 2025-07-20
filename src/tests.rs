use crate::assembly::Assembly;
use crate::method::Method;
use crate::ty::class::ClassDef;
use crate::ty::{GenericBinding, TypeDef, class};
use enumflags2::make_bitflags;
use global::StringMethodReference;
use global::attrs::MethodAttr;
use global::attrs::MethodImplementationFlags;
use global::attrs::{
    ClassImplementationFlags, FieldAttr, FieldImplementationFlags, TypeAttr, TypeSpecificAttr,
    Visibility,
};
use global::instruction::StringInstruction;
use global::{StringName, StringTypeReference, indexmap, string_name};
use std::io::Write;
use std::sync::Arc;

#[test]
fn test_emit_get() -> global::Result<()> {
    const TEST_CLASS_NAME: StringTypeReference =
        StringTypeReference::make_static_single("Test", "Test.Test");
    let mut assem = Assembly::default();
    *assem.name_mut() = StringName::from_static_str("Test");
    assem.type_defs_mut().insert(
        StringName::from_static_str("Test.Test"),
        TypeDef::Class(ClassDef::new(
            Some(StringTypeReference::from_string_repr("[!]System.Object")?),
            indexmap! {
                string_name!("@T") => GenericBinding::new(
                    vec![StringTypeReference::from_string_repr("[!]System.IDisposable")?],
                    Some(StringTypeReference::from_string_repr("[!]System.Array`1[@T:[!]System.Object]")?)
                ),
                string_name!("@U") => GenericBinding::new(
                    vec![],
                    None,
                )
            },
            TypeAttr::new(
                Visibility::Public,
                TypeSpecificAttr::Class(make_bitflags!(ClassImplementationFlags::{})),
            ),
            StringName::from_static_str("Test.Test"),
            indexmap! {
                string_name!("PrintStaticsAndGenericType()") => Method::new(
                    string_name!("PrintStaticsAndGenericType()"),
                    MethodAttr::new(
                        Visibility::Public,
                        make_bitflags!(MethodImplementationFlags::{Static}),
                        10,
                    ),
                    vec![
                        StringInstruction::LoadStatic {
                            register_addr: 0,
                            ty: TEST_CLASS_NAME.clone(),
                            name: string_name!("__test"),
                        },
                        StringInstruction::StaticCall {
                            ty: StringTypeReference::core_static_single_type("System.Console"),
                            method: StringMethodReference::Single(string_name!("WriteLine([!]System.String)")),
                            args: vec![0],
                            ret_at: 1,
                        },
                    ],
                    StringTypeReference::core_static_single_type("System.Void"),
                    vec![],
                    Default::default(),
                ),
                string_name!("Main([!]System.Array`1[@T:[!]System.String])") => Method::new(
                    string_name!("Main([!]System.Array`1[@T:[!]System.String])"),
                    MethodAttr::new(
                        Visibility::Public,
                        make_bitflags!(MethodImplementationFlags::{Static}),
                        10,
                    ),
                    vec![
                        StringInstruction::StaticCall {
                            ty: TEST_CLASS_NAME,
                            method: StringMethodReference::Single(string_name!("PrintStaticsAndGenericType()")),
                            args: vec![],
                            ret_at: 1,
                        },
                        StringInstruction::Load_u64 {
                            register_addr: 1,
                            val: 0,
                        },
                        StringInstruction::ReturnVal {
                            register_addr: 1,
                        }
                    ],
                    StringTypeReference::core_static_single_type("System.Void"),
                    vec![
                        StringTypeReference::WithGeneric {
                            assem: string_name!("!"),
                            ty: string_name!("System.Array`1"),
                            type_vars: Arc::new(indexmap! {
                                string_name!("@T") => StringTypeReference::core_static_single_type("System.String"),
                            })
                        }
                    ],
                    Default::default(),
                ),
                StringMethodReference::STATIC_CTOR_REF.unwrap_single() => Method::new(
                    StringMethodReference::STATIC_CTOR_REF.unwrap_single(),
                    MethodAttr::new(
                        Visibility::Public,
                        make_bitflags!(MethodImplementationFlags::{Static}),
                        10,
                    ),
                    vec![
                        StringInstruction::Load_u64 {
                            register_addr: 0,
                            val: 10,
                        },
                        StringInstruction::InstanceCall {
                            val: 0,
                            method: StringMethodReference::Single(string_name!("ToString()")),
                            args: vec![],
                            ret_at: 1,
                        },
                        StringInstruction::SetField {
                            register_addr: 1,
                            field: string_name!("__test"),
                        },
                    ],
                    StringTypeReference::core_static_single_type("System.Void"),
                    vec![],
                    Default::default(),
                ),
            },
            indexmap! {
                StringName::from_static_str("__test") => class::Field {
                    name: StringName::from_static_str("__test"),
                    attr: FieldAttr::new(Visibility::Public, make_bitflags!(FieldImplementationFlags::{Static})),
                    ty: StringTypeReference::make_static_single("!", "System.String"),
                }
            }
        )),
    );
    let b = assem.to_file_bytes()?;
    print!("Out to file?[Y/n] ");
    std::io::stdout().flush()?;
    let mut s = String::new();
    dbg!(std::io::stdin().read_line(&mut s)?);
    if s.to_ascii_lowercase().starts_with("y") {
        std::fs::write("./test.plb", &b)?;
        assert_eq!(std::fs::read("./test.plb")?, b);
    }
    let assem_gotten = Assembly::from_bytes(b)?;
    dbg!(&assem_gotten);
    Ok(())
}

#[test]
fn test_get_only() -> global::Result<()> {
    let assem = Assembly::from_file("./test.plb")?;
    dbg!(&assem);
    Ok(())
}
