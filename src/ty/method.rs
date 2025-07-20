use crate::ty::GenericBinding;
use global::attrs::MethodAttr;
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use global::instruction::StringInstruction;
use global::{IndexMap, StringName, StringTypeReference};
use proc_macros::{ReadFromFile, WriteToFile};

#[derive(Clone, Debug, Getters, CopyGetters, ctor, ReadFromFile, WriteToFile)]
#[allow(unused)]
#[getset(get = "pub")]
#[ctor(pub new)]
pub struct Method {
    name: StringName,
    #[getset(skip)]
    #[get_copy = "pub"]
    attr: MethodAttr,
    instructions: Vec<StringInstruction>,
    ret_type: StringTypeReference,
    args: Vec<StringTypeReference>,
    type_vars: IndexMap<StringName, GenericBinding>,
}
