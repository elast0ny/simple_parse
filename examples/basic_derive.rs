// Run with `RUST_LOG=debug cargo run --example basic_derive --features=verbose`

/**
 * Demonstrates the use of #[derive] on structs
 * This example also shows how simple_parse is able to optimize read() calls
 * by aggregating statically sized types.
 */
use std::io::Write;

use ::simple_parse::{SpRead, SpWrite};
use env_logger::Builder;

#[derive(SpRead, SpWrite)]
pub struct SomeStruct {
    pub field1: u8,
    pub field2: u16,
    pub field3: u32,
    #[sp(endian = "big")]
    pub items: Vec<u32>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    // Simulate bytes coming from a socket
    let mut recv_sock: &[u8] = &[
        1, 2, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0xDE, 0xAD, 0xBE, 0xEF, 0xBA, 0xDC, 0x0F, 0xFE, 0x11,
        0x22, 0x33, 0x44,
    ];

    // Read data from "socket"
    let mut my_struct = SomeStruct::from_reader(&mut recv_sock)?;

    // Modify our struct
    my_struct.items.push(0xFFFFFFFF);

    // Convert struct back into bytes
    let mut send_sock: Vec<u8> = Vec::new();
    my_struct.to_writer(&mut send_sock)?;

    /* STDOUT
     * [DEBUG] Read struct SomeStruct
     * [DEBUG] Read(11)
     * [DEBUG]   (u8)	1
     * [DEBUG]   (u16)	2
     * [DEBUG]   (u32)	3
     * [DEBUG]   (u32)	3
     * [DEBUG] Read(12)
     * [DEBUG]   (u32)	3735928559
     * [DEBUG]   (u32)	3134984190
     * [DEBUG]   (u32)	287454020
     * [1, 2, 0, 3, 0, 0, 0, 0, 0, 0, 4, DE, AD, BE, EF, BA, DC, F, FE, 11, 22, 33, 44, FF, FF, FF, FF]
     */
    println!("{:X?}", send_sock);

    Ok(())
}
