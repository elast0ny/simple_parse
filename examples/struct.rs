use ::simple_parse::{SpRead, SpWrite};

#[derive(SpRead, SpWrite)]
pub struct SomeStruct {
    pub some_field: u8,
    #[sp(endian="big")]
    pub items: Vec<u32>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut srv_sock: &[u8] = &[
        1,                      // some_field
        0,0,0,2,                // items.len()
        0xDE,0xAD,0xBE,0xEF,    // items[0]
        0xBA,0xDC,0x0F,0xFE     // items[1]
    ];

    // Read data from "socket"
    let mut my_struct = SomeStruct::from_reader(&mut srv_sock)?;

    // Modify our struct
    my_struct.items.push(0xFFFFFFFF);

    // Send modified data on "socket"
    let mut cli_sock: Vec<u8> = Vec::new();
    my_struct.to_writer(&mut cli_sock)?;

    println!("{:X?}", cli_sock);

    Ok(())
}