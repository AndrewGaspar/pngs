use std::clone::Clone;
use std::fs::File;
use std::io;
use std::io::Read;
use std::result::Result;
use std::vec::Vec;

use util;

pub trait RawChunk {
    fn length(&self) -> u32;
    fn chunk_type(&self) -> ChunkTypePrimitive;
    fn chunk_data(&self) -> &[u8];
    fn crc(&self) -> u32;
}

pub type ChunkTypePrimitive = [u8; 4];

pub struct ManagedRawChunk {
    chunk_type: ChunkTypePrimitive,
    chunk_data: Vec<u8>,
    crc: u32,
}

impl RawChunk for ManagedRawChunk {
    fn length(&self) -> u32 {
        self.chunk_type.len() as u32
    }

    fn chunk_type(&self) -> ChunkTypePrimitive {
        self.chunk_type
    }

    fn chunk_data(&self) -> &[u8] {
        &self.chunk_data[..]
    }

    fn crc(&self) -> u32 {
        self.crc
    }
}

pub enum PngParseError {
    IoError(io::Error),
    IncorrectSignature,
    UnexpectedEnd,
    ParseError
}

macro_rules! iotry {
    ($expr:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            return Err($crate::raw::PngParseError::IoError(err));
        }  
    })
}

fn fill_buffer(buffer: &mut Read, bytes: &mut [u8]) -> Result<(), PngParseError> {
    let bytes_read = iotry!(buffer.read(&mut bytes[..]));

    if bytes_read != bytes.len() {
        return Err(PngParseError::UnexpectedEnd);
    }

    return Ok(());
}

fn make_vec<T : Clone>(size: usize, default_value: T) -> Vec<T> {
    let mut list = Vec::with_capacity(size);

    for _ in 0..size {
        list.push(default_value.clone())
    }

    list
}

pub fn read_png_raw_from_file(path: &str) ->  Result<Vec<ManagedRawChunk>, PngParseError> {

    let mut chunks = Vec::new();

    let mut f = iotry!(File::open(&path));

    let mut signature = [0; 8];
    let png_sig = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let size = iotry!(f.read(&mut signature));

    if size != 8 || signature != png_sig {
        return Err(PngParseError::IncorrectSignature);
    }

    loop {
        let mut word = [0; 4];
        
        let bytes_read = iotry!(f.read(&mut word));

        if bytes_read == 0 {
            return Ok(chunks);
        }

        let length = util::bytes_as_be_u32(&word);

        try!(fill_buffer(&mut f, &mut word));

        let chunk_type = word.clone();

        let mut chunk = make_vec(length as usize, 0u8);

        assert_eq!(length as usize, chunk.len());

        try!(fill_buffer(&mut f, &mut chunk));

        try!(fill_buffer(&mut f, &mut word));

        let crc = util::bytes_as_be_u32(&word);

        chunks.push(ManagedRawChunk {
            chunk_type: chunk_type,
            chunk_data: chunk,
            crc: crc
        })
    }
}