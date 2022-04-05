// Run with `RUST_LOG=debug cargo run --example enum_derive --features=verbose`

use std::{io::Write, mem::MaybeUninit};

use ::simple_parse::{SpRead, SpWrite};
use env_logger::Builder;

#[derive(Debug, SpRead, SpWrite)]
pub enum SomeEnum {
    Var1,
    Var2(
        #[sp(endian="big")]
        u32,
        String
    ),
    Var3 {
        field1: u16,
        #[sp(len = "field1")]
        items: Vec<u16>,
    },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    // Simulate bytes coming from a socket
    let mut recv_sock: &[u8] = &[
        0x00, // SomeEnum::Var1
        0x01, // SomeEnum::Var2
        0xDE, 0xAD, 0xBE, 0xEF, // u32
        0x02, 0, 0, 0, // str_len
        b'H', b'i', // string
    ];

    let mut dst = MaybeUninit::uninit();
    // Read data from "socket"

    loop {
        match SomeEnum::from_reader(&mut recv_sock, &mut dst) {
            Ok(v) => println!("{v:02X?}"),
            Err(e) => {
                println!("{e}");
                break;
            },
        }
    }

    /* STDOUT
     * [DEBUG] Read enum SomeEnum
     * [DEBUG]   read(1)
     * [DEBUG] 0x0
     * [DEBUG] Self::Var1
     * [DEBUG]   total : 1 bytes
     * Var1
     * [DEBUG] Read enum SomeEnum
     * [DEBUG]   read(1)
     * [DEBUG] 0x1
     * [DEBUG] Self::Var2
     * [DEBUG]   read(4)
     * [DEBUG] 0xDEADBEEF
     * [DEBUG]   read(4)
     * [DEBUG] 0x2
     * [DEBUG]   read(2)
     * [DEBUG] 0x48
     * [DEBUG] 0x69
     * [DEBUG] 'Hi'
     * [DEBUG]   total : 11 bytes
     * Var2(DEADBEEF, "Hi")
     * [DEBUG] Read enum SomeEnum
     * Failed to read more bytes : failed to fill whole buffer
     */

    Ok(())
}
