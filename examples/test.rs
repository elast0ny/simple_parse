use std::io::Cursor;

use simple_parse::{SpReadRawMut};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Box::new([0x0F, 0, 0, 0, 0, 0, 0, 0x0F]);

    println!("{:p}[{}] = {:02X?}", data.as_ptr(), data.len(), data);

    let mut cursor = Cursor::new(&mut data[..]);
    let val = <u8>::from_mut_slice(&mut cursor)?;
    println!("u8\t{:02X} ({:p})", val, &val);

    cursor.set_position(0);
    while let Ok(val) = <&u8>::from_mut_slice(&mut cursor) {
        println!("&u8\t{:02X} ({:p})", *val, val);
    }
    cursor.set_position(0);
    while let Ok(val) = <&u16>::from_mut_slice(&mut cursor) {
        println!("&u16\t{:04X} ({:p})", *val, val);
    }
    cursor.set_position(0);
    while let Ok(val) = <&u32>::from_mut_slice(&mut cursor) {
        println!("&u32\t{:08X} ({:p})", *val, val);
    }
    cursor.set_position(0);
    while let Ok(val) = <&u64>::from_mut_slice(&mut cursor) {
        println!("&u64\t{:016X} ({:p})", *val, val);
    }

    // Change the memory
    cursor.set_position(0);
    let val = <&mut u8>::from_mut_slice(&mut cursor)?;
    println!("&mut u8\t{:02X} ({:p}) = 0x2", *val, val);
    *val = 0x2;
    println!("&mut u8\t{:02X} ({:p})", *val, val);

    /*
    cursor.set_position(0);
    let val = <&mut u64>::from_mut_slice(&mut cursor)?;
    drop(cursor);
    // This should fail as `val` still borrows into &data
    data = Box::new([0x0F, 0, 0, 0, 0, 0, 0, 0x0F]);
    println!("&u64\t{:016X} ({:p})", *val, val);
    */

    Ok(())
}
