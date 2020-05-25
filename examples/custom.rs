use simple_parse::{SpError, SpRead, SpWrite};

#[derive(SpRead, SpWrite, Debug)]
pub enum Message {
    #[sp(id = "1")]
    ServerHello {
        #[sp(reader = "string_read(input, 8)", writer = "string_write(input, 8, dst)")]
        banner: String,
    },

    #[sp(id = "2")]
    ClientLogin {
        #[sp(reader = "string_read(input, 8)", writer = "string_write(input, 8, dst)")]
        username: String,
    },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg_stream: &[u8] = &[
        0x01, // ServerHello
        'H' as _, 'i' as _, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // banner
        0x02, // ClientLogin
        'E' as _, 'l' as _, 'a' as _, 's' as _, 't' as _, '0' as _, 'n' as _,
        'y' as _, // username
    ];

    let mut dst = Vec::new();
    let rest = msg_stream;

    let (rest, msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);

    let (_rest, mut msg) = Message::from_bytes(rest)?;
    println!("{:X?}", msg);

    msg.to_bytes(&mut dst)?;
    println!("{:X?}", dst);
    dst.clear();

    if let Message::ClientLogin {
        ref mut username, ..
    } = msg
    {
        *username = String::from("H4ck3r");
    }
    println!("{:X?}", msg);

    msg.to_bytes(&mut dst)?;
    println!("{:X?}", dst);

    Ok(())
}

// Parses utf8 characters up to a null terminator or num_bytes
fn string_read(input: &[u8], num_bytes: usize) -> Result<(&[u8], String), SpError> {
    // Makes sure theres at least num_bytes
    if input.len() < num_bytes {
        return Err(SpError::NotEnoughBytes);
    }

    let (ascii_bytes, rest) = input.split_at(num_bytes);

    // Attempt to find null terminator
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
fn string_write(s: &String, num_bytes: usize, dst: &mut Vec<u8>) -> Result<(), SpError> {
    let mut bytes = s.clone().into_bytes();

    // Make sure string is exactly num_bytes
    bytes.resize(num_bytes, 0x00);

    dst.append(&mut bytes);

    Ok(())
}
