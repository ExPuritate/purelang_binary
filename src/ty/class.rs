use crate::ty::GenericBinding;
use crate::ty::method::Method;
use global::attrs::{FieldAttr, TypeAttr};
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use global::{IndexMap, StringName, StringTypeReference};
use proc_macros::{ReadFromFile, WriteToFile};

#[derive(ctor, Debug, Clone, Getters, CopyGetters, ReadFromFile, WriteToFile)]
#[getset(get = "pub")]
pub struct ClassDef {
    pub(crate) parent: Option<StringTypeReference>,
    pub(crate) type_vars: IndexMap<StringName, GenericBinding>,
    #[getset(skip)]
    #[get_copy = "pub"]
    pub(crate) attr: TypeAttr,
    pub(crate) name: StringName,
    pub(crate) methods: IndexMap<StringName, Method>,
    pub(crate) fields: IndexMap<StringName, Field>,
}

pub type Field = super::field::Field;
