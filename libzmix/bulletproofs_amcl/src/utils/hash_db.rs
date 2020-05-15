// Interface and an in-memory for key-value database where the key is bytes and is intended to be hash.
// Used to store merkle tree nodes.

use crate::errors::{BulletproofError, BulletproofErrorKind};
use std::collections::HashMap;

pub trait HashDb<T: Clone> {
    fn insert(&mut self, hash: Vec<u8>, value: T);

    fn get(&self, hash: &[u8]) -> Result<T, BulletproofError>;
}

#[derive(Clone, Debug)]
pub struct InMemoryHashDb<T: Clone> {
    db: HashMap<Vec<u8>, T>,
}

impl<T: Clone> HashDb<T> for InMemoryHashDb<T> {
    fn insert(&mut self, hash: Vec<u8>, value: T) {
        self.db.insert(hash, value);
    }

    fn get(&self, hash: &[u8]) -> Result<T, BulletproofError> {
        match self.db.get(hash) {
            Some(val) => Ok(val.clone()),
            None => Err(BulletproofErrorKind::HashNotFoundInDB {
                hash: hash.to_vec(),
            }
            .into()),
        }
    }
}

impl<T: Clone> InMemoryHashDb<T> {
    pub fn new() -> Self {
        let db = HashMap::<Vec<u8>, T>::new();
        Self { db }
    }
}

impl<T: Clone> InMemoryHashDb<T> {
    pub fn len(&self) -> usize {
        self.db.len()
    }
}

impl<T: Clone> InMemoryHashDb<T> {
    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.db.contains_key(key)
    }
}

use std::io;
use std::path::Path;
use crate::r1cs::gadgets::helper_constraints::sparse_merkle_tree_8_ary::DbVal8ary;
use amcl_wrapper::field_elem::FieldElement;

impl InMemoryHashDb<DbVal8ary> {
    pub fn save(&self, path: &Path, root: &FieldElement) -> Result<(), io::Error> {
        use std::fs::File;
        use std::io::Write;
        //extern crate zip;
        //extern crate hex;
        //use zip::write::{ZipWriter, FileOptions};
        use byteorder::{LittleEndian, WriteBytesExt};

        //let mut i = 0u32;
        let f = File::create(path)?;
        let mut writer = f; // undo when we're compressing again
        //let mut writer = ZipWriter::new(f);
        //writer.start_file(&root.to_hex(), FileOptions::default())?;
        writer.write(&root.to_bytes())?;
        writer.write_u32::<LittleEndian>(self.db.len() as u32)?;
        for (key, value) in &self.db {
            //io::stdout().write(format!("{}: {} --", i, hex::encode(&key[40..])).as_bytes());
            //for j in 0..8 {
            //    io::stdout().write(format!(" {}", hex::encode(&value[j].to_bytes()[40..])).as_bytes());
            //}
            //io::stdout().write(b"\n"); io::stdout().flush();
            writer.write(&key)?;
            for item in value {
                writer.write(&item.to_bytes())?;
            }
            //i += 1;
        }
        //println!("Wrote {} out of {} pairs to compressed file.", i, self.db.len());
        //writer.finish()?;
        Ok(())
    }

    pub fn load(&mut self, path: &Path) -> Result<FieldElement, io::Error> {
        use std::fs::File;
        use std::io::Read;
        //extern crate zip;
        //use zip::read::ZipArchive;
        use byteorder::{LittleEndian, ReadBytesExt};
        //use std::io::{Write, stdout};

        let f = File::open(path)?;
        //let mut zip = ZipArchive::new(f)?;
        let mut file = f;//zip.by_index(0).unwrap();
        let mut key_buf = FieldElement::new().to_bytes();
        file.read_exact(&mut key_buf)?;
        let root = FieldElement::from_bytes(&key_buf).unwrap();
        //let root = FieldElement::from_hex(file.name().to_string()).unwrap();
        let count = file.read_u32::<LittleEndian>().unwrap();
        let mut value: DbVal8ary = [
            FieldElement::zero(),
            FieldElement::zero(),
            FieldElement::zero(),
            FieldElement::zero(),
            FieldElement::zero(),
            FieldElement::zero(),
            FieldElement::zero(),
            FieldElement::zero(),
        ];
        let mut value_buf = key_buf.clone();
        for _ in 0..count {
            if file.read_exact(&mut key_buf).is_err() {
                break;
            }
            for j in 0..8 {
                file.read_exact(&mut value_buf).unwrap();
                value[j] = FieldElement::from_bytes(&value_buf).unwrap();
            }
            //io::stdout().write(format!("{}: {} --", i, hex::encode(&key_buf[40..])).as_bytes());
            //for j in 0..8 {
            //    io::stdout().write(format!(" {}", hex::encode(&value[j].to_bytes()[40..])).as_bytes());
            //}
            //io::stdout().write(b"\n"); io::stdout().flush();
            self.db.insert(key_buf.clone(), value.clone());
        }
        Ok(root)
    }
}
