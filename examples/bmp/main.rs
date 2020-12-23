/**
 * This example shows the use of simple_parse for a more "complicated" use case, parsing a BMP header.
 * 
 * This code is in no way a complete BMP parser and will most likely fail to parse different variants of the spec.
 * 
 */

use clap::{App, Arg};
use env_logger::Builder;
use simple_parse::{SpRead};
use std::io::Write;

mod format;
use format::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    let matches = App::new("BMP Dump")
        .about("Dumps the contents of a BMP header")
        .arg(
            Arg::with_name("fpath")
                .help("Path to the input BMP")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    // Open file
    let file_path: &str = matches.value_of("fpath").unwrap();
    let mut file = std::fs::File::open(file_path).expect("Failed to read input file");
    // get filesize
    let file_size = file.metadata()?.len();

    // Parse bmp header
    let header = BmpHeader::from_reader(&mut file).expect("Failed to read header");

    // BMP should start with 'BM'
    if header.magic1 != b'B' || header.magic2 != b'M' {
        return Err(From::from(format!(
            "Invalid BMP magic header: {:X}{:X}",
            header.magic1, header.magic2
        )));
    }

    // Filesizes should match
    if header.size as u64 != file_size {
        return Err(From::from(format!(
            "Header filesize doesnt match real file size : {:X} != {:X}",
            header.size, file_size
        )));
    }

    println!("{:?}", header);

    Ok(())
}
