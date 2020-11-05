use simple_parse::{SpError, SpRead, SpWrite};

/* 
    Demonstrates decoding and encoding of a binary format that
    has fixed string sizes that may or may not be null terminated
*/

#[derive(SpRead, SpWrite, Debug)]
pub enum Message {
    #[sp(id = "1")]
    ServerHello {
        #[sp(
            reader = "string_read(src, 8)",
            writer = "string_write(input, 8, dst)"
        )]
        banner: String,
    },

    #[sp(id = "2")]
    ClientLogin {
        #[sp(
            reader = "string_read(src, 8)",
            writer = "string_write(input, 8, dst)"
        )]
        username: String,
    },
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg_stream: &[u8] = &[
        0x01, // ServerHello
        b'H', b'i', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // banner
        0x02, // ClientLogin
        b'E', b'l', b'a', b's', b't', b'0', b'n', b'y', // username
    ];

    let mut cursor = msg_stream;
    let mut dst = Vec::new();

    println!("Data[{}] : {:X?}", cursor.len(), cursor);
    // Parse first message
    let mut msg = Message::from_reader(&mut cursor)?;
    println!("{:X?}", msg);

    // Parse second message
    msg = Message::from_reader(&mut cursor)?;
    println!("{:X?}", msg);

    {
        // Validity check
        msg.to_writer(&mut dst)?;
        assert_eq!(&dst, &[0x02, // ClientLogin
            b'E', b'l', b'a', b's', b't', b'0', b'n', b'y']);
    }

    println!("Modifying username !");
    // Change the login username
    if let Message::ClientLogin{username, ..} = &mut msg {
        username.clear();
        username.push_str("H4ck3r");
    }
    println!("{:X?}", msg);

    dst.clear();
    msg.to_writer(&mut dst)?;
    println!("Data[{}] : {:X?}", dst.len(), dst);

    Ok(())
}

// Parses utf8 characters up to a null terminator or num_bytes
fn string_read<R: std::io::Read + ?Sized>(src: &mut R, num_bytes: usize) -> Result<String, SpError> {
    

    let mut vec: Vec<u8> = vec![0; num_bytes];
    
    if src.read(&mut vec).is_err() {
        return Err(SpError::NotEnoughSpace);
    }

    // Remove trailing null bytes
    while !vec.is_empty() && vec[vec.len() - 1] == 0x00 {
        vec.pop();
    }

    // Parse string as UTF8
    match String::from_utf8(vec) {
        Ok(s) => Ok(s),
        Err(_) => Err(SpError::InvalidBytes),
    }
}

// Converts a string into bytes writing up to num_bytes. If the string
// is shorter, it is padded with null terminators
fn string_write<W: std::io::Write + ?Sized>(s: &str, max_bytes: usize, dst: &mut W) -> Result<usize, SpError> {
    let null_byte = &[0u8];
    let bytes = s.as_bytes();

    let mut len_written = std::cmp::min(max_bytes, bytes.len());
    
    if dst.write(&bytes).is_err() {
        return Err(SpError::NotEnoughSpace);
    }

    // Pad tail with null bytes
    while len_written < max_bytes {
        
        if dst.write(null_byte).is_err() {
            return Err(SpError::NotEnoughSpace);
        }
        len_written += 1;
    }

    Ok(len_written)
}
