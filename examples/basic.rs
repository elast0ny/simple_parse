// Run with :
//  RUST_LOG=debug cargo run --example basic --features verbose
use std::io::{Cursor, Write};

use simple_parse::{SpRead, SpWrite};

use ::env_logger::Builder;

#[derive(Debug)]
#[derive(SpRead, SpWrite)]
pub enum Message {
    ClientLogin(
        #[sp(var_size)]
        LoginInfo
    ),
    Logout(u16, u16),
    Chat(String),
    File { name: String, contents: Vec<u8> },
}

#[derive(Debug)]
#[derive(SpRead, SpWrite)]
pub struct LoginInfo {
    username: String,
    password_len: u8,
    #[sp(count="password_len")]
    password: String,
}

macro_rules! dump_optimization_hints {
    ($typ:ty) => {
        println!("{} {{\n    IS_VAR_SIZE: {}\n    STATIC_SIZE: {}\n}}", stringify!($typ), <$typ as ::simple_parse::SpOptHints>::IS_VAR_SIZE, <$typ as ::simple_parse::SpOptHints>::STATIC_SIZE);
    };
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    let data: &[u8] = &[
        0x00, 
        0x04, 0x00, 0x00, 0x00, 
        b'T', b'o', b'n', b'y',
        0x03, 
        b'a', b'b', b'c'
    ];
    println!("Original Bytes == {:X?}", data);
    let data_len = data.len() as u64;
    let mut cursor = Cursor::new(data);
    let mut dst = Vec::new();

    dump_optimization_hints!(Message);
    dump_optimization_hints!(LoginInfo);

    loop {
        let obj = <Message>::from_reader(&mut cursor);
        
        match obj {
            Ok(v) => {
                println!("SpRead == {:X?}", v);
                dst.clear();
                v.to_writer(&mut dst)?;
                println!("SpWrite == {:X?}", dst);
            },
            Err(e) => {
                if data_len - cursor.position() > 0 {
                    println!("{}", e);
                }
                break;
            }
        }
    }

    println!("{} bytes left", data_len - cursor.position());
    Ok(())
}
