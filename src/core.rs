use crate::Error;
use global::IndexSet;
use global::errors::GenericError;
use std::io::{Cursor, Read, Seek, Write};

#[derive(Debug, Clone, Default)]
pub struct File {
    pub(crate) interner: StringInterner,
    pub(crate) data: Cursor<Vec<u8>>,
}

impl File {
    pub fn new<T: AsRef<[u8]>>(bytes: T) -> global::Result<Self> {
        let mut data = Cursor::new(bytes.as_ref().to_vec());
        let mut interner_len = [0u8; 8];
        data.read_exact(&mut interner_len)?;
        let interner_len = u64::from_le_bytes(interner_len);
        let mut interner = vec![0u8; interner_len as usize];
        data.read_exact(&mut interner)?;
        let interner = StringInterner::new(interner);
        let mut _data = Vec::new();
        data.read_to_end(&mut _data)?;
        Ok(Self {
            interner,
            data: Cursor::new(_data),
        })
    }
    pub fn writer(&mut self) -> &mut (impl Write + Seek) {
        &mut self.data
    }
    pub fn reader(&mut self) -> &mut (impl Read + Seek) {
        &mut self.data
    }
    pub fn string_position_of(&mut self, s: &str) -> global::Result<u64, GenericError<Error>> {
        self.interner.position_of(s)
    }
    pub fn get_string(&self, i: u64) -> global::Result<&str, GenericError<Error>> {
        self.interner.get(i)
    }
    pub fn to_bytes(&self) -> global::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let interner_bytes = self.interner.to_bytes();
        buf.extend_from_slice(&(interner_bytes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&self.interner.to_bytes());
        buf.extend_from_slice(self.data.get_ref());
        let this = Self::new(&buf)?;
        Ok(buf)
    }
}

#[derive(Debug, Clone)]
pub struct StringInterner {
    set: IndexSet<String>,
}

impl Default for StringInterner {
    fn default() -> Self {
        let mut set = IndexSet::new();
        set.insert(String::new());
        Self { set }
    }
}

impl StringInterner {
    pub fn new<T: AsRef<[u8]>>(bytes: T) -> Self {
        let bytes = bytes.as_ref();
        let mut set = IndexSet::new();
        if !bytes.starts_with(b"\0") {
            set.insert("".to_owned());
        }
        for s in bytes.split(|x| x.eq(&b'\0')) {
            let s = unsafe { String::from_utf8_unchecked(Vec::from(s)) };
            set.insert(s);
        }
        Self { set }
    }
    pub fn position_of(&mut self, s: &str) -> global::Result<u64, GenericError<Error>> {
        match self.set.iter().position(|x| x.eq(s)) {
            Some(pos) => Ok(pos as u64),
            None => {
                let l = self.set.len();
                self.set.insert(s.to_owned());
                Ok(l as u64)
            }
        }
    }
    #[track_caller]
    pub fn get(&self, i: u64) -> global::Result<&str, GenericError<Error>> {
        let mut x = i as usize;
        for s in self.set.iter() {
            if x == 0 {
                return Ok(s);
            }
            x -= 1;
        }
        Err(Error::StringNotFound { index: i }.throw())
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut s = self.set.iter().fold(Vec::<u8>::new(), |mut a, b| {
            let mut s = b.as_bytes().to_vec();
            s.push(b'\0');
            a.extend_from_slice(&s);
            a
        });
        s.pop();
        s.extend_from_slice(&vec![0; s.len() % 8]);
        s
    }
}
