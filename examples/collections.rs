// Run with `RUST_LOG=debug cargo run --example cstring --features=verbose`

use std::{io::Write, collections::HashMap};

use ::simple_parse::SpRead;
use env_logger::Builder;
use simple_parse::SpWrite;

#[derive(SpRead, SpWrite, Debug)]
struct TestStruct {
    _num_vals: u16,
    some_bool: bool,

    // The length of a collection can come from another field
    #[sp(len="_num_vals")]
    values: Vec<u8>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    
    // The default format for collections is to use a u32 for
    // it's length followed by the items
    let mut bytes: &[u8] = &[
        0x04, 0x00, 0x00, 0x00,
        0xAA, 0xBB,
        0xCC, 0xDD,
        0x44, 0x33,
        0x22, 0x11,
    ];
    println!("{:X?}", <Vec<u16>>::from_reader(&mut bytes));
    // Ok([BBAA, DDCC, 3344, 1122])

    // HashMap & co. work the same
    let mut bytes: &[u8] = &[
        0x02, 0x00, 0x00, 0x00,
        0xAA, 0xBB,
            0x05, 0x00, 0x00, 0x00, b'H', b'e', b'l', b'l', b'o',
        0x11, 0x22,
            0x02, 0x00, 0x00, 0x00, b'H', b'i',
    ];
    println!("{:X?}", <HashMap<u16, String>>::from_reader(&mut bytes));
    // Ok({2211: "Hi", BBAA: "Hello"})

    // Collections can also depend on another field in its struct/enum
    // for its length
    let mut some_file: &[u8] = &[
        0x02, 0x00,
        0x01,
        0xAA, 0xBB
    ];
    println!("{:X?}", TestStruct::from_reader(&mut some_file));
    // Ok(TestStruct { num_vals: 2, some_bool: true, values: [AA, BB] })

    Ok(())
}
