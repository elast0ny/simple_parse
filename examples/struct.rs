//! This example shows the use of no copy parsing through SpReadRaw and references.
//! It also uses some annotations on the Vec to demonstrate available customizations.

use simple_parse::{SpReadRaw, SpWrite};
use std::io::Cursor;

#[derive(Debug, SpReadRaw, SpWrite)]
pub struct SomeStruct<'a> {
    _ndwords: &'a u16,
    some_field: u8,
    some_string: &'a str,
    #[sp(count = "_ndwords", endian = "big")]
    dwords: Vec<u32>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let data: Vec<u8> = vec![
        0x03, 0x00, //num_dwords
        0x12, //some_field
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // string.len()
        b'A', b'B',
        0x11, 0x22, 0x33, 0x44, //dword[0]
        0x55, 0x66, 0x77, 0x88, //dword[1]
        0x99, 0xAA, 0xBB, 0xCC, //dword[2]
    ];

    let mut cursor = Cursor::new(data.as_slice());
    println!("Data[{}] : {:X?}", data.len(), &data);

    // Parse the arbitrary bytes into our struct
    let mut s = SomeStruct::from_slice(&mut cursor)?;
    println!("Decoded : {:X?}", s);

    // Make sure re-encoding equals original
    let mut dst = Vec::new();
    s.to_writer(&mut dst)?;
    assert_eq!(&dst, &data);

    // Add field and dump again
    s.dwords.push(0xDDEEFFFF);
    println!("Added number to vec : {:X?}", s.dwords);
    dst.clear();
    s.to_writer(&mut dst)?;
    println!("New[{}] : {:X?}", dst.len(), dst);
    assert_eq!(
        &dst,
        &[
            0x04, 0x00, //num_dwords
            0x12, //some_field
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // string.len()
            b'A', b'B',
            0x11, 0x22, 0x33, 0x44, //dword[0]
            0x55, 0x66, 0x77, 0x88, //dword[1]
            0x99, 0xAA, 0xBB, 0xCC, //dword[2]
            0xDD, 0xEE, 0xFF, 0xFF, //dword[3]
        ]
    );
    dst.clear();

    Ok(())
}
