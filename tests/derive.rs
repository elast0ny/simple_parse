
use simple_parse::*;
use std::{io::Cursor, mem::MaybeUninit};

#[test]
fn derive_static() {

    let bytes: &[u8] = &[
        1,0,0,0,
        2,
        3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,
    ];
    
    #[derive(Debug, SpRead, SpWrite, PartialEq)]
    struct StaticStruct {
        v1: u32,
        v2: u8,
        v3: u128,
        v4: f32,
    }
    const STATIC_STRUCT_SIZE: usize = 4 + 1 + 16 + 4;

    // Try to parse with 1 byte missing
    let mut tmp = MaybeUninit::uninit();
    assert!(StaticStruct::from_reader(&mut Cursor::new(&bytes[..bytes.len()-1]), &mut tmp).is_err());   

    let mut ctx = SpCtx::default();
    // Make sure every field is parsed properly
    let v = StaticStruct::inner_from_reader(&mut Cursor::new(bytes), &mut ctx, &mut tmp).unwrap();
    assert_eq!(v, &mut StaticStruct {
        v1: 1,
        v2: 2,
        v3: 3,
        v4: 0.0,
    });
    assert_eq!(ctx.cursor, STATIC_STRUCT_SIZE, "ctx.cursor was not advanced properly while reading");
    
    // Convert the struct back to bytes
    ctx.is_reading = false;
    ctx.cursor = 0;
    let mut dst = Vec::new();
    v.inner_to_writer(&mut ctx, &mut dst).unwrap();
    assert_eq!(&dst, bytes);
    assert_eq!(ctx.cursor, STATIC_STRUCT_SIZE, "ctx.cursor was not advanced properly while writing");

    let bytes: &[u8] = &[
        123,
        
        1,0,0,0,
        2,
        3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,
    ];
    #[derive(Debug, SpRead, SpWrite, PartialEq)]
    enum Nested {
        #[sp(id=123)]
        Nest(StaticStruct),
    }

    // Try to parse with 1 byte missing
    let mut tmp = MaybeUninit::uninit();
    assert!(Nested::from_reader(&mut Cursor::new(&bytes[..bytes.len()-1]), &mut tmp).is_err());   

    let mut ctx = SpCtx::default();
    // Make sure every field is parsed properly
    let v = Nested::inner_from_reader(&mut Cursor::new(bytes), &mut ctx, &mut tmp).unwrap();
    assert_eq!(v, &mut Nested::Nest(
        StaticStruct {
            v1: 1,
            v2: 2,
            v3: 3,
            v4: 0.0,
        })
    );
    assert_eq!(ctx.cursor, 1 + STATIC_STRUCT_SIZE, "ctx.cursor was not advanced properly while reading");

    // Convert the struct back to bytes
    ctx.is_reading = false;
    ctx.cursor = 0;
    let mut dst = Vec::new();
    v.inner_to_writer(&mut ctx, &mut dst).unwrap();
    assert_eq!(&dst, bytes);
    assert_eq!(ctx.cursor, 1 + STATIC_STRUCT_SIZE, "ctx.cursor was not advanced properly while writing");
}

#[test]
fn derive_var_size() {
    let bytes: &[u8] = &[
        1,0,0,0,
        5,0,0,0,
        1,2,3,4,5
    ];
    
    #[derive(Debug, SpRead, SpWrite, PartialEq)]
    struct BasicVar {
        v1: u32,
        v2: Vec<u8>,
    }

    // Try to parse with 1 byte missing
    let mut tmp = MaybeUninit::uninit();
    assert!(BasicVar::from_reader(&mut Cursor::new(&bytes[..bytes.len()-1]), &mut tmp).is_err());   

    let mut ctx = SpCtx::default();
    // Make sure every field is parsed properly
    let v = BasicVar::inner_from_reader(&mut Cursor::new(bytes), &mut ctx, &mut tmp).unwrap();
    assert_eq!(v, &mut BasicVar {
        v1: 1,
        v2: vec![1,2,3,4,5],
    });
    assert_eq!(ctx.cursor, 4 + 4 + 5, "ctx.cursor was not advanced properly while reading");
    
    // Convert the struct back to bytes
    ctx.is_reading = false;
    ctx.cursor = 0;
    let mut dst = Vec::new();
    v.inner_to_writer(&mut ctx, &mut dst).unwrap();
    assert_eq!(&dst, bytes);
    assert_eq!(ctx.cursor, 4 + 4 + 5, "ctx.cursor was not advanced properly while writing");
}

#[test]
fn derive_var_size_custom_len() {
    let bytes: &[u8] = &[
        5,0,0,0,    
        1,0,
        1,2,3,4,5
    ];
    
    #[derive(Debug, SpRead, SpWrite, PartialEq)]
    struct CustomLenVar {
        v1: u32,
        v2: u16,
        #[sp(len="v1")]
        v3: Vec<u8>,
    }

    // Try to parse with 1 byte missing
    let mut tmp = MaybeUninit::uninit();
    assert!(CustomLenVar::from_reader(&mut Cursor::new(&bytes[..bytes.len()-1]), &mut tmp).is_err());   

    let mut ctx = SpCtx::default();
    // Make sure every field is parsed properly
    let v = CustomLenVar::inner_from_reader(&mut Cursor::new(bytes), &mut ctx, &mut tmp).unwrap();
    assert_eq!(v, &mut CustomLenVar {
        v1: 5,
        v2: 1,
        v3: vec![1,2,3,4,5],
    });
    assert_eq!(ctx.cursor, 4 + 2 + 5, "ctx.cursor was not advanced properly while reading");
    
    // Convert the struct back to bytes
    ctx.is_reading = false;
    ctx.cursor = 0;
    let mut dst = Vec::new();
    v.inner_to_writer(&mut ctx, &mut dst).unwrap();
    assert_eq!(&dst, bytes);
    assert_eq!(ctx.cursor, 4 + 2 + 5, "ctx.cursor was not advanced properly while writing");
}
