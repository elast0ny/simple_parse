use ::simple_parse::{SpOptHints, SpReadRaw, SpWrite};

// You must derive SpOptHints manualy when not using SpRead
#[derive(SpOptHints, SpReadRaw)]
pub struct File<'a> {
    pub filename: &'a str,
    pub contents: &'a [u8],
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    // Generate bytes for filename
    "Some_very_long_filename_we_dont_want_to_copy.txt".to_writer(&mut data)?;
    // Generate bytes for file contents
    [1u8; 4096].to_writer(&mut data)?;

    // Print the address of the input buffer in memory
    println!("Data starts at 0x{:p}", data.as_ptr());

    let f = File::from_slice(&mut std::io::Cursor::new(&data))?;

    // Print the address of each field's references
    println!("File {{");
    println!("\tfilename: 0x{:p} [{}]", f.filename.as_ptr(), f.filename.len());
    println!("\tcontents: 0x{:p} [{}]", f.contents.as_ptr(), f.contents.len());
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