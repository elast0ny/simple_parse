use simple_parse::{SpRead, SpWrite};

#[derive(Debug, SpRead, SpWrite)]
pub struct SomeStruct {
    some_field: u8,
    some_string: String,
    _ndwords: u16,
    #[sp(count = "_ndwords", endian = "big")]
    dwords: Vec<u32>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let data: Vec<u8> = vec![
        0x12, //some_field
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // string.len()
        b'A', b'B', 0x03, 0x00, //num_dwords
        0x11, 0x22, 0x33, 0x44, //dword[0]
        0x55, 0x66, 0x77, 0x88, //dword[1]
        0x99, 0xAA, 0xBB, 0xCC, //dword[2]
    ];

    let mut cursor = data.as_slice();
    println!("Data[{}] : {:X?}", cursor.len(), cursor);

    // Parse the arbitrary bytes into our struct
    let mut s = SomeStruct::from_bytes(&mut cursor)?;
    println!("Decoded : {:X?}", s);

    // Make sure re-encoding equals original
    let mut dst = Vec::new();
    s.to_bytes(&mut dst)?;
    assert_eq!(&dst, &data);

    // Add field and dump again
    s.dwords.push(0xDDEEFFFF);
    println!("Added number to vec : {:X?}", s.dwords);
    dst.clear();
    s.to_bytes(&mut dst)?;
    println!("Data[{}] : {:X?}", dst.len(), dst);
    assert_eq!(
        &dst,
        &[
            0x12, //some_field
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // string.len()
            b'A', b'B', 0x04, 0x00, //num_dwords
            0x11, 0x22, 0x33, 0x44, //dword[0]
            0x55, 0x66, 0x77, 0x88, //dword[1]
            0x99, 0xAA, 0xBB, 0xCC, //dword[2]
            0xDD, 0xEE, 0xFF, 0xFF, //dword[3]
        ]
    );
    dst.clear();

    Ok(())
}
