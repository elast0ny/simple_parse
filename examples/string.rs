// Run with `RUST_LOG=debug cargo run --example cstring --features=verbose`

use std::{ffi::CString, io::Write};

use ::simple_parse::SpRead;
use env_logger::Builder;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    // Strings are encoded as : [str_len][str_bytes...]
    // This format is efficient as only 2 read() calls are needed
    let mut some_file: &[u8] = b"\x0B\x00\x00\x00Hello World";

    /* STDOUT
     * [DEBUG] Read(4)
     * [DEBUG]   (u32)	11
     * [DEBUG] Read(11)
     * [DEBUG]   (u8)	72
     * [DEBUG]   (u8)	101
     * [DEBUG]   (u8)	108
     * [DEBUG]   (u8)	108
     * [DEBUG]   (u8)	111
     * [DEBUG]   (u8)	32
     * [DEBUG]   (u8)	87
     * [DEBUG]   (u8)	111
     * [DEBUG]   (u8)	114
     * [DEBUG]   (u8)	108
     * [DEBUG]   (u8)	100
     * Ok("Hello World")
    */
    println!("{:?}", String::from_reader(&mut some_file));

    // CStrings generate a read() call for every byte until the null terminator is reached
    let mut recv_sock: &[u8] = b"Hello World\0";

    /* STDOUT
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * [DEBUG] Read(1)
     * Ok("Hello World")
    */
    println!("{:?}", CString::from_reader(&mut recv_sock));

    Ok(())
}
