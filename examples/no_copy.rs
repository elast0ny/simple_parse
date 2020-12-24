/**
 * Demonstrates the use of simple_parse's no-copy capabilities
 */
use ::simple_parse::{SpOptHints, SpReadRaw, SpWrite};
use ::env_logger::Builder;

// You must derive SpOptHints manualy when not using SpRead
#[derive(Debug, SpOptHints, SpReadRaw)]
pub struct File<'a> {
    pub content_len: &'a u32,
    
    pub filename: &'a str,
    
    #[sp(count="content_len")]
    pub contents: &'a [u8],
    
    pub timestamp: u32, // Not all values need to be references
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .init();

    let mut data = Vec::new();
    // Write content len
    (128).to_writer(&mut data)?;
    // Generate bytes for filename
    "Some_very_long_filename_we_dont_want_to_copy.txt".to_writer(&mut data)?;
    // Generate bytes for file contents
    [1u8; 124].to_writer(&mut data)?; // Default implementation prepends a u32 for the lenght so only write 4092 of content
    // Write fake timestamp
    (0xffffffffu32).to_writer(&mut data)?;

    let f = File::from_slice(&mut std::io::Cursor::new(&data))?;

    // Print the address of the input buffer in memory
    println!("Data starts at 0x{:p}", data.as_ptr());

    // Print the contents of the struct
    println!("{:?}", f);
    
    // Print the address of each field's references
    println!("File {{");
    println!(
        "\tfilename: 0x{:p} [{}]",
        f.filename.as_ptr(),
        f.filename.len()
    );
    println!(
        "\tcontents: 0x{:p} [{}]",
        f.contents.as_ptr(),
        f.contents.len()
    );
    println!("}}");

    /*
     * Prints to STDOUT something like :
     *
     *      Data starts at 0x0x55d14fae3ba0
     *      File {
     *          filename: 0x0x55d14fae3ba4 [48]
     *          contents: 0x0x55d14fae3bd8 [4096]
     *      }
     *
     * The result struct points directly into the input data
     */

    Ok(())
}