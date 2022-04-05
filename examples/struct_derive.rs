// Run with `RUST_LOG=debug cargo run --example basic_derive --features=verbose`

/**
 * Demonstrates the use of #[derive] on structs
 * This example also shows how simple_parse is able to optimize read() calls
 * by aggregating statically sized types.
 */
use std::{io::Write, mem::MaybeUninit};

use ::simple_parse::{SpRead, SpWrite};
use env_logger::Builder;

#[derive(SpRead, SpWrite)]
pub struct SomeStruct {
    pub field1: u8,
    pub field2: u16,
    pub field3: u32,
    #[sp(endian = "big")]
    pub items: Vec<u32>,

    // Dynamically sized collections can depend on other fields for their lengths
    pub custom_len: u8,
    #[sp(len="custom_len")]
    pub msg: String,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    // Simulate bytes coming from a socket
    let mut recv_sock: &[u8] = &[
        1, // field1
        2, 0, // field2
        3, 0, 0, 0, // field3
        0, 0, 0, 3, // num_items (big endian)
        0xDE, 0xAD, 0xBE, 0xEF, // items[0]
        0xBA, 0xDC, 0x0F, 0xFE, // items[1]
        0x11, 0x22, 0x33, 0x44, // items[2]
        0x05, // custom_len
        b'H', b'e', b'l', b'l', b'o', 
    ];

    let mut dst = MaybeUninit::uninit();
    // Read data from "socket"
    let my_struct = SomeStruct::from_reader(&mut recv_sock, &mut dst)?;

    // Modify our struct
    my_struct.items.push(0xFFFFFFFF);

    // Convert struct back into bytes
    let mut send_sock: Vec<u8> = Vec::new();
    my_struct.to_writer(&mut send_sock)?;

    /* STDOUT
     * [DEBUG] Read struct SomeStruct
     * [DEBUG]   read(1)
     * [DEBUG] 0x1
     * [DEBUG]   read(2)
     * [DEBUG] 0x2
     * [DEBUG]   read(4)
     * [DEBUG] 0x3
     * [DEBUG]   read(4)
     * [DEBUG] 0x3
     * [DEBUG]   read(12)
     * [DEBUG] 0xDEADBEEF
     * [DEBUG] 0xBADC0FFE
     * [DEBUG] 0x11223344
     * [DEBUG]   read(1)
     * [DEBUG] 0x5
     * [DEBUG]   read(5)
     * [DEBUG] 0x48
     * [DEBUG] 0x65
     * [DEBUG] 0x6C
     * [DEBUG] 0x6C
     * [DEBUG] 0x6F
     * [DEBUG] 'Hello'
     * [DEBUG]   total : 29 bytes
     * [01, 02, 00, 03, 00, 00, 00, 00, 00, 00, 04, DE, AD, BE, EF, BA, DC, 0F, FE, 11, 22, 33, 44, FF, FF, FF, FF, 05, 48, 65, 6C, 6C, 6F]
     */
    println!("{send_sock:02X?}");

    Ok(())
}
