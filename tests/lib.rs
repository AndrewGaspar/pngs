extern crate pngs;

use pngs::raw;
use pngs::raw::RawChunk;

use std::io::Cursor;
use std::vec::Vec;

fn assert_length_and_type(maybe_chunk: &raw::Result<raw::ManagedRawChunk>, chunk_type: &raw::ChunkTypePrimitive, len: usize) {

    assert!(maybe_chunk.is_ok());

    let chunk = maybe_chunk.as_ref().ok().expect("Asserted above.");

    assert_eq!(*chunk_type, chunk.chunk_type());
    assert_eq!(len, chunk.length() as usize);
}

#[test]
fn known_png() {
    let sample_png = include_bytes!("Sample.png");

    let cursor = Cursor::new(&sample_png[..]);

    let chunks: Vec<_> = ::pngs::raw::read_png_raw(cursor).collect();

    assert_eq!(6, chunks.len());

    assert_length_and_type(&chunks[0], b"IHDR", 13);
    assert_length_and_type(&chunks[1], b"sRGB", 1);
    assert_length_and_type(&chunks[2], b"gAMA", 4);
    assert_length_and_type(&chunks[3], b"pHYs", 9);
    assert_length_and_type(&chunks[4], b"IDAT", 2828);
    assert_length_and_type(&chunks[5], b"IEND", 0);
}