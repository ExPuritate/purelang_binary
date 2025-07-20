use global::attrs::FieldAttr;
use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use global::{StringName, StringTypeReference};
use proc_macros::{ReadFromFile, WriteToFile};

#[derive(Clone, Debug, Getters, CopyGetters, ReadFromFile, WriteToFile, ctor)]
#[getset(get = "pub")]
pub struct Field {
    pub(crate) name: StringName,
    #[getset(skip)]
    #[get_copy = "pub"]
    pub(crate) attr: FieldAttr,
    pub(crate) ty: StringTypeReference,
}
