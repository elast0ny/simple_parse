// Run with `RUST_LOG=debug cargo run --example cstring --features=verbose`

use std::{collections::HashMap, io::Write, mem::MaybeUninit};

use ::simple_parse::SpRead;
use env_logger::Builder;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    // The default format for collections is to use a u32 for
    // it's length followed by the items
    let mut bytes: &[u8] = &[
        0x04, 0x00, 0x00, 0x00, // Number of values
        0xAA, 0xBB, //val[0]
        0xCC, 0xDD, //val[1]
        0x44, 0x33, //val[2]
        0x22, 0x11, //val[3]
    ];

    let mut v = MaybeUninit::uninit();
    println!("{:X?}", <Vec<u16>>::from_reader(&mut bytes, &mut v));
    // Ok([BBAA, DDCC, 3344, 1122])

    // HashMap & co. work the same
    let mut bytes: &[u8] = &[
        0x02, 0x00, 0x00, 0x00, // Number of HashMap entries
        0xBB, 0xAA, // key[0]
        0x05, 0x00, 0x00, 0x00, b'H', b'e', b'l', b'l', b'o', // val[0]
        0x22, 0x11, // key[1]
        0x02, 0x00, 0x00, 0x00, b'H', b'i', // val[1]
    ];
    let mut v = MaybeUninit::uninit();
    println!(
        "{:X?}",
        <HashMap<u16, String>>::from_reader(&mut bytes, &mut v)
    );
    // Ok({2211: "Hi", BBAA: "Hello"})
    Ok(())
}
