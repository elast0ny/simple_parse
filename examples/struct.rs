use simple_parse::{SpRead, SpWrite};

#[derive(Debug, SpRead, SpWrite)]
pub struct SomeStruct {
    some_field: u8,
    some_string: String,
    num_dwords: u16,
    #[sp(count = "num_dwords", endian = "big")]
    dwords: Vec<u32>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dst = Vec::new();
    let data: &[u8] = &[
        0x12, //some_field
        0x02, 0x00, 0x00, 0x00, // string length
        b'A', b'B',
        0x03, 0x00, //num_dwords
        0x11, 0x22, 0x33, 0x44, //dword[0]
        0x55, 0x66, 0x77, 0x88, //dword[1]
        0x99, 0xAA, 0xBB, 0xCC, //dword[2]
    ];

    // Parse the arbitrary bytes into our struct
    let (_, mut s) = SomeStruct::from_bytes(data)?;
    println!("{:X?}", s);

    // Dump struct as bytes
    let len = s.to_bytes(&mut dst)?;
    println!("{} bytes : {:X?}", len, dst);

    assert_eq!(&dst, &data);
    dst.clear();

    // Add field and dump again
    s.dwords.push(0xDDEEFFFF);
    let len = s.to_bytes(&mut dst)?;
    println!("{} bytes : {:X?}", len, dst);
    assert_eq!(
        &dst,
        &[
            0x12, //some_field
            0x02, 0x00, 0x00, 0x00, // string length
            b'A', b'B',
            0x04, 0x00, //num_dwords
            0x11, 0x22, 0x33, 0x44, //dword[0]
            0x55, 0x66, 0x77, 0x88, //dword[1]
            0x99, 0xAA, 0xBB, 0xCC, //dword[2]
            0xDD, 0xEE, 0xFF, 0xFF, //dword[3]
        ]
    );
    dst.clear();

    Ok(())
}
