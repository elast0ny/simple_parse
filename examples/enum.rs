use simple_parse::{SpRead, SpWrite};

#[derive(SpRead, SpWrite, Debug)]
pub enum Message {
    #[sp(id = "1")]
    ServerHello(u32, u32),

    #[sp(id = "2")]
    ClientLogin,

    #[sp(id = "3")]
    ServerDisconnect {
        #[sp(endian = "big")]
        timestamp: u32,
        _noptions: u8,
        #[sp(count = "_noptions")]
        options: Vec<u8>,
    },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Buffer that contains 2 messages
    let orig_buf: &[u8] = &[
        0x01, // ServerHello
        0x12, 0x23, 0x34, 0x45, // ServerHello.0
        0x12, 0x34, 0x56, 0x78, // ServerHello.1
        0x03, // ServerDisconnect
        0xDE, 0xAD, 0xBE, 0xEF, //ServerDisconnect.timestamp
        0x2,  //ServerDisconnect.num_options
        0x11, //ServerDisconnect.option[0]
        0x22, //ServerDisconnect.option[1]
    ];

    let mut start_idx = 0;
    let mut cursor = orig_buf;
    let mut dst: Vec<u8> = Vec::new();

    println!("Decoding : {:X?}", orig_buf);
    let mut msg = Message::from_reader(&mut cursor)?;
    println!("Got : {:X?}", msg);

    {
        // Make sure encoding generated the exact same bytes
        dst.clear();
        msg.to_writer(&mut dst)?;
        assert_eq!(&orig_buf[start_idx..start_idx + dst.len()], dst);
        start_idx += dst.len();
    }

    println!("Decoding : {:X?}", cursor);
    msg = Message::from_reader(&mut cursor)?;
    println!("Got : {:X?}", msg);

    {
        // Make sure encoding generated the exact same bytes
        dst.clear();
        msg.to_writer(&mut dst)?;
        assert_eq!(&orig_buf[start_idx..start_idx + dst.len()], dst);
    }

    // Modify the last message
    if let Message::ServerDisconnect { options, .. } = &mut msg {
        options.push(0x12);
        println!("Added option to last message : {:X?}", options);
    } else {
        panic!("Did not parse as ServerDisconnect !?");
    }

    dst.clear();
    msg.to_writer(&mut dst)?;
    println!("Encoding : {:X?}", dst);

    {
        // Make sure encoding generated the exact same bytes
        dst.clear();
        msg.to_writer(&mut dst)?;
        assert_eq!(
            &dst,
            &[
                0x03, // ServerDisconnect
                0xDE, 0xAD, 0xBE, 0xEF, //ServerDisconnect.timestamp
                0x3,  //ServerDisconnect.num_options
                0x11, //ServerDisconnect.option[0]
                0x22, //ServerDisconnect.option[1]
                0x12, //ServerDisconnect.option[2]
            ]
        );
    }
    Ok(())
}
