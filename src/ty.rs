use crate::ty::class::ClassDef;
use crate::ty::r#struct::StructDef;
use global::getset::Getters;
use global::{StringTypeReference, WithType};
use proc_macros::{ReadFromFile, WriteToFile};

pub mod class;
pub mod field;
pub mod method;
pub mod r#struct;

#[derive(Debug, Clone, WithType, ReadFromFile, WriteToFile)]
#[with_type(repr = u8)]
#[with_type(derive = (Clone, Copy, ReadFromFile, WriteToFile))]
#[allow(clippy::large_enum_variant)]
pub enum TypeDef {
    Class(ClassDef),
    Struct(StructDef),
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
