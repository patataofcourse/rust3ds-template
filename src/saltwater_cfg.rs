use bytestream::*;
use std::{
    collections::HashMap,
    ffi::CString,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

pub struct Config {
    pub tickflows: HashMap<u16, Vec<u8>>,
}

const MAGIC: &[u8; 4] = b"SCF\x02";

impl Config {
    pub fn from_file(file: impl Into<PathBuf>) -> io::Result<Self> {
        let mut file = File::open(file.into())?; //todo: use ctru_rs instead ffs
        let mut magic_buffer = [0u8; 4];
        file.read(&mut magic_buffer)?;
        if &magic_buffer != MAGIC {
            Err(io::Error::new(io::ErrorKind::Other, "invalid file"))?;
        }
        todo!();
    }

    pub fn to_file(&self, file: impl Into<PathBuf>) -> io::Result<()> {
        let mut file = File::create(file.into())?; //todo: use ctru_rs instead ffs
        file.write(MAGIC)?;
        for (index, string) in &self.tickflows {
            index.write_to(&mut file, ByteOrder::LittleEndian)?;
            (string.len() as u16).write_to(&mut file, ByteOrder::LittleEndian)?;
            for chr in string {
                chr.write_to(&mut file, ByteOrder::LittleEndian)?;
            }
        }
        0xC000u16.write_to(&mut file, ByteOrder::LittleEndian)?;
        Ok(())
    }
}
