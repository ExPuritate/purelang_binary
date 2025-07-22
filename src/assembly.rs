use crate::Error;
use crate::core::File;
use crate::traits::{ReadFromFile, WriteToFile};
use crate::ty::TypeDef;
use global::StringName;
use proc_macros::{ReadFromFile, WriteToFile};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default, Debug, Clone, ReadFromFile, WriteToFile)]
pub struct Assembly {
    name: StringName,
    type_defs: HashMap<StringName, TypeDef>,
}

#[allow(unused)]
const MAGIC: [u8; 2] = *b"PL";

#[allow(unused)]
#[derive(Debug, Default, Clone, ReadFromFile, WriteToFile)]
struct Header {
    magic: [u8; 2],
}

#[allow(unused)]
impl Header {
    fn check(&self) -> global::Result<()> {
        if self.magic != MAGIC {
            return Err(Error::WrongFileFormat.into());
        }
        Ok(())
    }
}

impl Assembly {
    pub fn name(&self) -> &StringName {
        &self.name
    }
    pub fn type_defs(&self) -> &HashMap<StringName, TypeDef> {
        &self.type_defs
    }
    pub fn name_mut(&mut self) -> &mut StringName {
        &mut self.name
    }
    pub fn type_defs_mut(&mut self) -> &mut HashMap<StringName, TypeDef> {
        &mut self.type_defs
    }
}

impl Assembly {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> global::Result<Self> {
        let mut file = File::new(bytes)?;
        Self::read_from_file(&mut file)
    }
    pub fn from_file<P: AsRef<Path>>(p: P) -> global::Result<Self> {
        Self::from_bytes(std::fs::read(p)?)
    }

    pub fn to_file_bytes(&self) -> global::Result<Vec<u8>> {
        let mut file = File::default();
        self.write_to_file(&mut file)?;
        file.to_bytes()
    }
}
