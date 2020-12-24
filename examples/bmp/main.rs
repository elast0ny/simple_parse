/**
 * This example shows the use of simple_parse for a more "complicated" use case, parsing a BMP header.
 *
 * This code is in no way a complete BMP parser and will most likely fail to parse different variants of the spec.
 *
 */
use clap::{App, Arg};
use env_logger::Builder;
use simple_parse::{SpRead, SpWrite};
use std::io::{Read, Seek, Write};

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

    // Parse bmp header
    let header = BmpHeader::from_reader(&mut file).expect("Failed to read header");

    // Print parsed values
    println!("{:?}", header);

    // Get number of bytes we've read so far
    let last_pos = file.seek(std::io::SeekFrom::Current(0))? as usize;
    let mut orig_data = Vec::with_capacity(last_pos);
    unsafe { orig_data.set_len(last_pos) };

    // Read back original bytes as a Vec<u8>
    file.seek(std::io::SeekFrom::Start(0))?;
    file.read_exact(orig_data.as_mut_slice())?;

    // Write header back into bytes
    let mut generated_data = Vec::new();
    header.to_writer(&mut generated_data)?;
    println!("Back into bytes :\n{:X?}", generated_data);

    // The generated data should match the original data
    assert_eq!(orig_data, generated_data.as_slice());

    Ok(())
}
