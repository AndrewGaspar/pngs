use std::mem;

pub fn bytes_as_be_u32(bytes: &[u8; 4]) -> u32 {
    u32::from_be(unsafe { mem::transmute_copy(bytes) })
}