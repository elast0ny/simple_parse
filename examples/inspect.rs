use ::env_logger::Builder;
use simple_parse::{SpRead, SpWrite};
/**
 * This example is for those who are currious to see what kind of code simple_parse generates.
 * Run this example with :
 *      RUST_LOG=debug cargo run --example inspect --features verbose
 * To see everytime a read() call would be performed on the [socket|file|etc...]
 *
 * Or run with :
 *      RUST_LOG=debug cargo run --example inspect --features verbose --features print-generated
 * To get a dump of the actual parsing code that got generated
 */
use std::io::{Cursor, Write};

#[derive(Debug, SpRead, SpWrite)]
pub enum Message {
    ClientLogin(
        // simple_parse does not know about LoginInfo while generating code for `enum Message`
        // We must explicitly declare that it is variably sized (because it contains a variably sized type)
        // Or else compilation will fail
        #[sp(var_size)] LoginInfo,
    ),
    Logout(u16, u16),
    Chat(String),
    File {
        name: String,
        creation_time: Option<u32>,
        contents: Vec<u8>,
    },
}

#[derive(Debug, SpRead, SpWrite)]
pub struct LoginInfo {
    username_len: u8,
    is_admin: bool,
    #[sp(endian = "big")]
    secret_iv: u16,
    #[sp(count = "username_len")] // The length of `username` is the u8 field `username_len`
    username: String,
    password: String,
    got_session: bool,
    #[sp(count = "got_session")]
    session: Option<u32>,
}

macro_rules! dump_optimization_hints {
    ($typ:ty) => {
        println!(
            "{} {{\n    IS_VAR_SIZE: {}\n    STATIC_SIZE: {}\n    COUNT_SIZE: {}\n}}",
            stringify!($typ),
            <$typ as ::simple_parse::SpOptHints>::IS_VAR_SIZE,
            <$typ as ::simple_parse::SpOptHints>::STATIC_SIZE,
            <$typ as ::simple_parse::SpOptHints>::COUNT_SIZE,
        );
    };
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    // Emulate data coming from a socket
    let data: &[u8] = &[
        0x00, // 1st variant (ClientLogin)
        0x04, // username_len
        0x01, // is_admin
        0xC0, 0xFE, // secret_iv
        b'T', b'o', b'n', b'y', // username
        0x03, 0, 0, 0, // password length
        b'a', b'b', b'c', // password
        0x00, // None
        0x02, // 3rd variant (Chat)
        0x05, 0, 0, 0, b'H', b'e', b'l', b'l', b'o', // Chat(message)
        0x03, // 4th variant (File)
        0x06, 0, 0, 0, b'h', b'i', b'.', b't', b'x', b't', // File.name
        0x00, // None
        0, 0, 0, 0, // contents.len() == 0, empty file contents
    ];
    println!("Original Bytes == {:X?}", data);
    let data_len = data.len() as u64;
    let mut cursor = Cursor::new(data);
    let mut dst = Vec::new();

    // Dump the SpOptHints for these structs
    dump_optimization_hints!(Message);
    dump_optimization_hints!(LoginInfo);

    loop {
        // Parse messages coming from our fake socket
        let obj = <Message>::from_reader(&mut cursor);
        match obj {
            Ok(v) => {
                //Dump parsed message
                println!("SpRead\tMessage::{:X?}", v);

                // Convert back to bytes to show SpWrite
                dst.clear();
                v.to_writer(&mut dst)?;
                println!("SpWrite\t{:X?}", dst);
            }
            Err(e) => {
                // Unexpected error, not at end of stream
                if data_len - cursor.position() > 0 {
                    println!("{}", e);
                } else {
                    println!("Done !");
                }
                break;
            }
        }
    }

    println!("{} bytes left", data_len - cursor.position());
    Ok(())
}

/***
 * EXAMPLE STDOUT
 *
Original Bytes == [0, 4, 1, C0, FE, 54, 6F, 6E, 79, 3, 0, 0, 0, 61, 62, 63, 2, 5, 0, 0, 0, 48, 65, 6C, 6C, 6F, 3, 6, 0, 0, 0, 68, 69, 2E, 74, 78, 74, 0, 0, 0, 0, 0]
Message {
    IS_VAR_SIZE: true
    STATIC_SIZE: 5
    COUNT_SIZE: 0
}
LoginInfo {
    IS_VAR_SIZE: true
    STATIC_SIZE: 4
    COUNT_SIZE: 0
}
[DEBUG] Read enum Message
[DEBUG] Read(5)
[DEBUG] [0x0] : 0x00 [u8]
[DEBUG] Read variant ClientLogin
[DEBUG] [0x1] : 0x04 [u8]
[DEBUG] [0x2] : 0x01 [u8]
[DEBUG] [0x3] : 0xFEC0 [u16]
[DEBUG] swap to native (little) endian
[DEBUG] Read(4)
[DEBUG] [0x5] : 0x54 [u8]
[DEBUG] [0x6] : 0x6F [u8]
[DEBUG] [0x7] : 0x6E [u8]
[DEBUG] [0x8] : 0x79 [u8]
[DEBUG] Read(4)
[DEBUG] [0x9] : 0x00000003 [u32]
[DEBUG] Read(3)
[DEBUG] [0xD] : 0x61 [u8]
[DEBUG] [0xE] : 0x62 [u8]
[DEBUG] [0xF] : 0x63 [u8]
SpRead  Message::ClientLogin(LoginInfo { username_len: 4, is_admin: true, secret_iv: C0FE, username: "Tony", password: "abc" })
SpWrite [0, 4, 1, C0, FE, 54, 6F, 6E, 79, 3, 0, 0, 0, 61, 62, 63]
[DEBUG] Read enum Message
[DEBUG] Read(5)
[DEBUG] [0x0] : 0x02 [u8]
[DEBUG] Read variant Chat
[DEBUG] [0x1] : 0x00000005 [u32]
[DEBUG] Read(5)
[DEBUG] [0x5] : 0x48 [u8]
[DEBUG] [0x6] : 0x65 [u8]
[DEBUG] [0x7] : 0x6C [u8]
[DEBUG] [0x8] : 0x6C [u8]
[DEBUG] [0x9] : 0x6F [u8]
SpRead  Message::Chat("Hello")
SpWrite [2, 5, 0, 0, 0, 48, 65, 6C, 6C, 6F]
[DEBUG] Read enum Message
[DEBUG] Read(5)
[DEBUG] [0x0] : 0x03 [u8]
[DEBUG] Read variant File
[DEBUG] [0x1] : 0x00000006 [u32]
[DEBUG] Read(6)
[DEBUG] [0x5] : 0x68 [u8]
[DEBUG] [0x6] : 0x69 [u8]
[DEBUG] [0x7] : 0x2E [u8]
[DEBUG] [0x8] : 0x74 [u8]
[DEBUG] [0x9] : 0x78 [u8]
[DEBUG] [0xA] : 0x74 [u8]
[DEBUG] Read(1)
[DEBUG] [0xB] : 0x00 [u8]
[DEBUG] None
[DEBUG] Read(4)
[DEBUG] [0xC] : 0x00000000 [u32]
SpRead  Message::File { name: "hi.txt", creation_time: None, contents: [] }
SpWrite [3, 6, 0, 0, 0, 68, 69, 2E, 74, 78, 74, 0, 0, 0, 0, 0]
[DEBUG] Read enum Message
[DEBUG] Read(5)
Done !
0 bytes left
 */
