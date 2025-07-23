use crate::method::Method;

use global::derive_ctor::ctor;
use global::getset::{CopyGetters, Getters};
use global::{IndexMap, StringName, StringTypeReference};
use proc_macros::{ReadFromFile, WriteToFile};

#[derive(ctor, Debug, Clone, Getters, CopyGetters, ReadFromFile, WriteToFile)]
#[getset(get = "pub")]
pub struct Implementation {
    ty: StringTypeReference,
    interface: Option<StringTypeReference>,
    methods: IndexMap<StringName, Method>,
}
