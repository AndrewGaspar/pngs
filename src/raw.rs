use std::clone::Clone;
use std::convert::From;
use std::fs::File;
use std::io;
use std::io::Read;
use std::vec::Vec;
use std::iter::Iterator;

use util;

pub trait RawChunk {
    fn length(&self) -> u32;
    fn chunk_type(&self) -> ChunkTypePrimitive;
    fn chunk_data(&self) -> &[u8];
    fn crc(&self) -> u32;
}

pub type SignatureTypePrimitive = [u8; 8];
pub type ChunkTypePrimitive = [u8; 4];

pub struct ManagedRawChunk {
    chunk_type: ChunkTypePrimitive,
    chunk_data: Vec<u8>,
    crc: u32,
}

impl RawChunk for ManagedRawChunk {
    fn length(&self) -> u32 {
        self.chunk_data.len() as u32
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
    InvalidChunkType(ChunkTypePrimitive),
    IncorrectSignature(SignatureTypePrimitive),
    UnexpectedEnd,
    ParseError
}

impl From<io::Error> for PngParseError {
    fn from(err: io::Error) -> PngParseError {
        PngParseError::IoError(err)
    }
}

pub type Result<T> = ::std::result::Result<T, PngParseError>;

fn fill_buffer(buffer: &mut Read, bytes: &mut [u8]) -> Result<()> {
    let bytes_read = try!(buffer.read(&mut bytes[..]));

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

fn ensure_valid_chunk_type(chunk_type: ChunkTypePrimitive) -> Result<()> {
    for b in &chunk_type[..] {
        if !((*b >= 65 && *b <= 90) || (*b >= 97 && *b <= 122)) {
            return Err(PngParseError::InvalidChunkType(chunk_type.clone()));
        }
    }

    Ok(())
}


fn ensure_valid_signature(sig: SignatureTypePrimitive) -> Result<()> {
    let png_sig = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    if sig != png_sig {
        return Err(PngParseError::IncorrectSignature(sig));
    }

    Ok(())
}

pub struct RawChunks<R: Read> {
    reader: R,
    has_signature: bool,
    has_finished: bool,
}

impl<R: Read> RawChunks<R> {
    fn new(reader: R) -> RawChunks<R> {
        RawChunks {
            reader: reader,
            has_signature: false,
            has_finished: false,
        }
    }

    fn ensure_signed(&mut self) -> Result<()> {
        if self.has_signature {
            return Ok(());
        }

        let mut signature = [0; 8];
        try!(fill_buffer(&mut self.reader, &mut signature));
        try!(ensure_valid_signature(signature));

        self.has_signature = true;

        Ok(())
    }

    fn try_next(&mut self) -> Option<Result<ManagedRawChunk>> {
        match self.ensure_signed() {
            Err(err) => {
                return Some(Err(err));
            },
            _ => {},
        };

        let mut word = [0; 4];
        
        let bytes_read = match self.reader.read(&mut word) {
            Ok(len) => len,
            Err(err) => {
                return Some(Err(PngParseError::IoError(err)));
            }
        };

        if bytes_read == 0 {
            return None;
        }

        Some((|| {
            let length = util::bytes_as_be_u32(&word);
            
            try!(fill_buffer(&mut self.reader, &mut word));

            let chunk_type = word.clone();

            try!(ensure_valid_chunk_type(chunk_type));

            let mut chunk = make_vec(length as usize, 0u8);

            assert_eq!(length as usize, chunk.len());

            try!(fill_buffer(&mut self.reader, &mut chunk));

            try!(fill_buffer(&mut self.reader, &mut word));

            let crc = util::bytes_as_be_u32(&word);

            Ok(ManagedRawChunk {
                chunk_type: chunk_type,
                chunk_data: chunk,
                crc: crc
            })
        })())
    }
}

impl<R: Read> Iterator for RawChunks<R> {
    type Item = Result<ManagedRawChunk>;

    fn next(&mut self) -> Option<Result<ManagedRawChunk>> {
        if self.has_finished {
            return None;
        }

        match self.try_next() {
            Some(result) => {
                Some(match result {
                    Err(err) => {
                        self.has_finished = true;
                        Err(err)
                    },
                    s => s
                })
            },
            None => {
                self.has_finished = true;
                None
            }
        }
    }
}

pub fn read_png_raw<R: Read>(reader: R) -> RawChunks<R> {
    RawChunks::new(reader)
}

pub fn read_png_raw_from_file(path: &str) -> ::std::io::Result<RawChunks<File>> {
    let file = try!(File::open(&path));

    Ok(read_png_raw(file))
}



#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid_chunk_type(chunk_type: & ChunkTypePrimitive) -> bool {
        super::ensure_valid_chunk_type(chunk_type.clone()).is_ok()
    }

    #[test]
    fn standard_chunks() {
        let specified_chunks 
            = [b"IHDR",
               b"PLTE",
               b"IDAT",
               b"IEND",
               b"gBKD",
               b"cHRM",
               b"gAMA",
               b"hIST",
               b"pHYs",
               b"sBIT",
               b"tEXt",
               b"tIME",
               b"tRNS",
               b"zTXt",
             ];

        for spec in specified_chunks.iter() {
            assert!(is_valid_chunk_type(spec));
        }
    }
}