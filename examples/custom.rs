use simple_parse::{SpError, SpRead, SpWrite};

#[derive(SpRead, SpWrite, Debug)]
#[sp(id_type = "u8")]
pub enum Message {
    #[sp(id = "1")]
    ServerHello {
        #[sp(reader = "string_read(input, 8)", writer = "string_write(input, 8)")]
        banner: String,
    },

    #[sp(id = "2")]
    ClientLogin {
        #[sp(reader = "string_read(input, 8)", writer = "string_write(input, 8)")]
        username: String,
    },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg_stream: &[u8] = &[
        0x01, // ServerHello
        'H' as _, 'i' as _, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // server_name
        0x02, // ClientLogin
        'E' as _, 'l' as _, 'a' as _, 's' as _, 't' as _, '0' as _, 'n' as _,
        'y' as _, // username
    ];

    let rest = msg_stream;

    let (rest, msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);

    let (_rest, mut msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);
    println!("{:X?}", msg.to_bytes()?);
    if let Message::ClientLogin {
        ref mut username, ..
    } = msg
    {
        *username = String::from("H4ck3r");
    }
    println!("{:X?}", msg);
    println!("{:X?}", msg.to_bytes()?);

    Ok(())
}

// Parses utf8 characters up to a null terminator or max_bytes
fn string_read(input: &[u8], max_bytes: usize) -> Result<(&[u8], String), SpError> {
    let (ascii_bytes, rest) = input.split_at(max_bytes);

    let mut sz = 0;
    for b in ascii_bytes.iter() {
        if *b == 0x00 {
            break;
        }
        sz += 1;
    }

    let res = match std::str::from_utf8(&ascii_bytes[..sz]) {
        Ok(s) => s.to_string(),
        Err(_) => return Err(SpError::InvalidBytes),
    };

    Ok((rest, res))
}

// Converts a string into bytes writing up to num_bytes. If the string
// is shorter, it is padded with null terminators
fn string_write(s: &String, num_bytes: usize) -> Result<Vec<u8>, SpError> {
    let mut bytes = s.clone().into_bytes();

    // Make sure string is exactly num_bytes
    bytes.resize(num_bytes, 0x00);

    Ok(bytes)
}
