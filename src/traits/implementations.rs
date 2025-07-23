use crate::core::File;
use crate::traits::{ReadFromFile, WriteToFile};
use const_for::const_for;
use enumflags2::{BitFlag, BitFlags};
use global::attrs::{
    FieldAttr, MethodAttr, TypeAttr, TypeSpecificAttr, TypeSpecificAttrType, Visibility,
};
use global::instruction::{StringInstruction, StringInstructionType};
use global::{IndexMap, StringMethodReference, StringName, StringTypeReference};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::MaybeUninit;

macro primitive_impl($($t:ty)+) {$(
    impl super::ToBytes for $t {
        fn to_bytes(&self) -> [u8; Self::SIZE] {
            self.to_le_bytes()
        }
    }
    impl super::FromBytes for $t {
        fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
            Self::from_le_bytes(bytes)
        }
    }
)+}

primitive_impl! {
    u8
    u16
    u32
    u64
    u128
    i8
    i16
    i32
    i64
    i128
}

impl ReadFromFile for String {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let i = u64::read_from_file(file)?;
        Ok(file.get_string(i)?.to_owned())
    }
}

impl WriteToFile for str {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        let i = file.string_position_of(self)?;
        i.write_to_file(file)
    }
}

impl ReadFromFile for StringName {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        Ok(Self::from_string(String::read_from_file(file)?))
    }
}
impl WriteToFile for StringName {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        self.as_str().write_to_file(file)
    }
}

impl ReadFromFile for StringTypeReference {
    #[track_caller]
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        Ok(Self::from_string_repr(String::read_from_file(file)?)?)
    }
}

impl WriteToFile for StringTypeReference {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        self.string_name_repr().write_to_file(file)
    }
}

impl ReadFromFile for StringMethodReference {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        Self::from_string_repr(String::read_from_file(file)?)
    }
}

impl WriteToFile for StringMethodReference {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        self.string_name_repr().write_to_file(file)
    }
}

impl<T: ReadFromFile> ReadFromFile for Vec<T> {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let i = u64::read_from_file(file)?;
        let mut vec = Vec::new();
        for _ in 0..i {
            vec.push(T::read_from_file(file)?);
        }
        Ok(vec)
    }
}

impl<T: WriteToFile> WriteToFile for Vec<T> {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        self.iter().try_for_each(|item| item.write_to_file(file))
    }
}

impl<T: ReadFromFile, const N: usize> ReadFromFile for [T; N] {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let mut this = std::array::from_fn(|_| MaybeUninit::<T>::uninit());
        const_for! {
            i in (0..N) => {
                this[i] = MaybeUninit::new(T::read_from_file(file)?);
            }
        }
        unsafe { Ok(MaybeUninit::array_assume_init(this)) }
    }
}

impl<T: WriteToFile, const N: usize> WriteToFile for [T; N] {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        const_for! {
            i in (0..N) => {
                self[i].write_to_file(file)?;
            }
        }
        Ok(())
    }
}

impl<K: ReadFromFile + Eq + Hash, V: ReadFromFile> ReadFromFile for HashMap<K, V> {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let i = u64::read_from_file(file)?;
        let mut map = Self::with_capacity(i as usize);
        for _ in 0..i {
            let k = K::read_from_file(file)?;
            let v = V::read_from_file(file)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

impl<K: WriteToFile, V: WriteToFile> WriteToFile for HashMap<K, V> {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        self.iter()
            .try_for_each(|(k, v)| k.write_to_file(file).and_then(|_| v.write_to_file(file)))
    }
}

impl<K: ReadFromFile + Eq + Hash, V: ReadFromFile> ReadFromFile for IndexMap<K, V> {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let i = u64::read_from_file(file)?;
        let mut map = Self::with_capacity(i as usize);
        for _ in 0..i {
            let k = K::read_from_file(file)?;
            let v = V::read_from_file(file)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

impl<K: WriteToFile + Any, V: WriteToFile + Any> WriteToFile for IndexMap<K, V> {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        (self.len() as u64).write_to_file(file)?;
        self.iter()
            .try_for_each(|(k, v)| k.write_to_file(file).and_then(|_| v.write_to_file(file)))
    }
}

impl<T: BitFlag> ReadFromFile for BitFlags<T>
where
    T::Numeric: ReadFromFile + Send + Sync + Debug,
    T: Send + Sync + Debug,
{
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let num = T::Numeric::read_from_file(file)?;
        Ok(Self::from_bits(num)?)
    }
}

impl<T: BitFlag> WriteToFile for BitFlags<T>
where
    T::Numeric: WriteToFile,
{
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        let num = self.bits();
        num.write_to_file(file)
    }
}

impl<T: ReadFromFile> ReadFromFile for Option<T> {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let b = u8::read_from_file(file)?;
        if b == 1 {
            Ok(Some(T::read_from_file(file)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: WriteToFile> WriteToFile for Option<T> {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        match self {
            Some(val) => 1u8
                .write_to_file(file)
                .and_then(|_| val.write_to_file(file)),
            None => 0u8.write_to_file(file),
        }
    }
}

impl ReadFromFile for Visibility {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        Ok(
            <Self as global::num_enum::TryFromPrimitive>::try_from_primitive(
                <Self as global::num_enum::TryFromPrimitive>::Primitive::read_from_file(file)?,
            )?,
        )
    }
}
impl WriteToFile for Visibility {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        let x: <Self as global::num_enum::TryFromPrimitive>::Primitive = (*self).into();
        x.write_to_file(file)
    }
}

impl ReadFromFile for TypeSpecificAttr {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let t: TypeSpecificAttrType = u8::read_from_file(file)?.try_into()?;
        match t {
            TypeSpecificAttrType::Class => Ok(Self::Class(ReadFromFile::read_from_file(file)?)),
            TypeSpecificAttrType::Struct => Ok(Self::Struct(ReadFromFile::read_from_file(file)?)),
            TypeSpecificAttrType::Interface => {
                Ok(Self::Interface(ReadFromFile::read_from_file(file)?))
            }
        }
    }
}

impl WriteToFile for TypeSpecificAttr {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        (self.to_type() as u8).write_to_file(file)?;
        match self {
            TypeSpecificAttr::Class(flags) => flags.write_to_file(file),
            TypeSpecificAttr::Struct(flags) => flags.write_to_file(file),
            TypeSpecificAttr::Interface(flags) => flags.write_to_file(file),
        }
    }
}

macro rw_file_foreign($($t:ty: $($n:ident)+ ;)+) {$(
    impl WriteToFile for $t {
        fn write_to_file(&self, file: &mut File) -> global::Result<()> {
            $(
                self.$n ().write_to_file(file)?;
            )+
            Ok(())
        }
    }
    impl ReadFromFile for $t {
        fn read_from_file(file: &mut File) -> global::Result<Self> {
            Ok(Self::new(
                $(
                    ReadFromFile::read_from_file(file)?,
                    ${ignore($n)}
                )+
            ))
        }
    }
)+}

macro many_rw_file_foreign($($t:ty: $($n:ident)+ ;)+) {
    $(
        rw_file_foreign!($t: $($n)+ ;);
    )+
}

many_rw_file_foreign! {
    TypeAttr: vis specific;
    MethodAttr: vis impl_flags register_len;
    FieldAttr: vis impl_flags;
}

impl ReadFromFile for StringInstruction {
    fn read_from_file(file: &mut File) -> global::Result<Self> {
        let t = StringInstructionType::try_from(u64::read_from_file(file)?)?;
        macro matcher($($i:ident => $(@$t_i:ident)|* $(|)?)+) {
            match t {
                $(
                    ::global::instruction::StringInstructionType::$i =>
                        Ok(
                            ::global::instruction::StringInstruction::$i {
                                $($t_i: ReadFromFile::read_from_file(file)?,)*
                            }
                        ),
                )+
            }
        }
        #[allow(deprecated)]
        {
            matcher! {
                LoadTrue => @register_addr
                LoadFalse => @register_addr
                Load_u8 => @register_addr | @val
                Load_u8_0 => @register_addr
                Load_u8_1 => @register_addr
                Load_u8_2 => @register_addr
                Load_u8_3 => @register_addr
                Load_u8_4 => @register_addr
                Load_u8_5 => @register_addr
                Load_u64 => @register_addr | @val
                NewObject => @ty | @ctor_name | @args | @register_addr
                InstanceCall => @val | @method | @args | @ret_at
                StaticCall => @ty | @method | @args | @ret_at
                LoadArg => @register_addr | @arg
                LoadAllArgsAsArray => @register_addr
                LoadStatic => @register_addr | @ty | @name
                ReturnVal => @register_addr
                SetField => @register_addr | @field
            }
        }
    }
}

impl WriteToFile for StringInstructionType {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        let x: <Self as global::num_enum::TryFromPrimitive>::Primitive = (*self).into();
        x.write_to_file(file)
    }
}

impl WriteToFile for StringInstruction {
    fn write_to_file(&self, file: &mut File) -> global::Result<()> {
        macro matcher($($i:ident => $(@$t_i:ident)|* $(|)?)+) {
            match self {
                $(
                    ::global::instruction::StringInstruction::$i {
                        $($t_i,)*
                    } => {
                        $($t_i.write_to_file(file)?;)*
                    }
                )+
            }
        }
        self.to_type().write_to_file(file)?;
        #[allow(deprecated)]
        {
            matcher! {
                LoadTrue => @register_addr
                LoadFalse => @register_addr
                Load_u8 => @register_addr | @val
                Load_u8_0 => @register_addr
                Load_u8_1 => @register_addr
                Load_u8_2 => @register_addr
                Load_u8_3 => @register_addr
                Load_u8_4 => @register_addr
                Load_u8_5 => @register_addr
                Load_u64 => @register_addr | @val
                NewObject => @ty | @ctor_name | @args | @register_addr
                InstanceCall => @val | @method | @args | @ret_at
                StaticCall => @ty | @method | @args | @ret_at
                LoadArg => @register_addr | @arg
                LoadAllArgsAsArray => @register_addr
                LoadStatic => @register_addr | @ty | @name
                ReturnVal => @register_addr
                SetField => @register_addr | @field
            }
        }
        Ok(())
    }
}
