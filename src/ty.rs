use crate::ty::class::ClassDef;
use global::getset::Getters;
use global::{StringName, StringTypeReference, WithType};
use proc_macros::{ReadFromFile, WriteToFile};

pub mod class;
pub mod field;
pub mod method;

#[derive(Debug, Clone, WithType, ReadFromFile, WriteToFile)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromFile, WriteToFile))]
#[allow(clippy::large_enum_variant)]
pub enum TypeDef {
    Class(ClassDef),
    Struct,
}

#[derive(Clone, Debug, Getters, ReadFromFile, WriteToFile)]
#[getset(get = "pub")]
pub struct GenericBinding {
    pub(crate) implemented_interfaces: Vec<StringTypeReference>,
    pub(crate) parent: Option<StringTypeReference>,
}

impl GenericBinding {
    pub fn new(
        implemented_interfaces: Vec<StringTypeReference>,
        parent: Option<StringTypeReference>,
    ) -> Self {
        Self {
            implemented_interfaces,
            parent,
        }
    }
}
