use simple_parse::{SpError, SpRead, SpWrite};

#[derive(SpRead, SpWrite, Debug)]
#[sp(id_type = "u8")]
pub enum Message {
    #[sp(id = "1")]
    ServerHello(u32, u32),

    #[sp(id = "2")]
    ClientLogin,

    #[sp(id = "3")]
    ServerDisconnect {
        #[sp(endian = "big")]
        timestamp: u32,
        num_options: u8,
        #[sp(count = "num_options")]
        options: Vec<u8>,
    },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg_stream: &[u8] = &[
        0x01, // ServerHello
        0x12, 0x23, 0x34, 0x45, // ServerHello.0
        0x12, 0x34, 0x56, 0x78, // ServerHello.1
        0x03, // ServerDisconnect
        0xDE, 0xAD, 0xBE, 0xEF, //ServerDisconnect.timestamp
        0x2,  //ServerDisconnect.num_options
        0x11, //ServerDisconnect.option[0]
        0x22, //ServerDisconnect.option[1]
        0x02, // ClientLogin
    ];

    let rest = msg_stream;

    let (rest, mut msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);
    let generated_bytes = msg.to_bytes()?;
    println!("{:X?}", generated_bytes);

    let (rest, mut msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);

    // Add an option
    if let Message::ServerDisconnect {
        ref mut options, ..
    } = msg
    {
        options.push(0x12);
    } else {
        panic!("Did not parse as ServerDisconnect !?");
    }
    let generated_bytes = msg.to_bytes()?;
    assert_eq!(
        generated_bytes,
        &[
            0x03, // ServerDisconnect
            0xDE, 0xAD, 0xBE, 0xEF, //ServerDisconnect.timestamp
            0x3,  //ServerDisconnect.num_options
            0x11, //ServerDisconnect.option[0]
            0x22, //ServerDisconnect.option[1]
            0x12, //ServerDisconnect.option[2]
        ]
    );
    println!("{:X?}", generated_bytes);

    // Parse last message
    let (rest, mut msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);
    let generated_bytes = msg.to_bytes()?;
    assert_eq!(generated_bytes, &[0x02]);
    println!("{:X?}", generated_bytes);

    if let Err(SpError::NotEnoughBytes) = Message::from_bytes(rest) {
        println!("Done !");
    } else {
        panic!("Should've failed !");
    }

    Ok(())
}
