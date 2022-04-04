use std::{io::Cursor, num::*};
use core::mem::size_of;

use simple_parse::*;

/// returns whether every byte in the slice is the same
/// Used to check if endianess changes should change the resulting value
fn all_same_bytes(buf: &[u8]) -> bool {
    if buf.len() == 1 {
        return true;
    }

    for i in 1..buf.len() {
        if buf[i] != buf[0] {
            return false
        }
    }

    return true;
}

macro_rules! test_primitive {
    ($typ:ty) => {
        primitive_read!($typ, $typ);
    };
    ($typ:ty, $init_val: expr) => {{
        let orig: $typ = $init_val;
        let orig_bytes = unsafe {
            core::slice::from_raw_parts(
                &orig as *const _  as *const u8,
                size_of::<$typ>(),
            )
        };

        // Try to parse buffer with 1 byte less than what this type needs
        assert!(<$typ>::from_reader(&mut Cursor::new(&orig_bytes[..size_of::<$typ>() -1])).is_err(), "{}.from_reader() with not enough bytes should fail", stringify!($typ));

        // Make sure we can parse bytes properly
        assert_eq!(<$typ>::from_reader(&mut Cursor::new(orig_bytes)).unwrap(), orig, "Failed to parse bytes into {}", stringify!($typ));
        let mut dst = Vec::with_capacity(size_of::<$typ>());
        // Make sure we write the expected number of bytes
        assert_eq!(orig.to_writer(&mut dst).unwrap(), size_of::<$typ>(), "Write type {} wrote invalid number of bytes", stringify!($typ));
        // Make sure the written bytes match the original
        assert_eq!(&dst, orig_bytes, "{}.to_writer() didnt generate same data", stringify!($typ));

        // Test reading data that isnt in native endianness
        let mut ctx = SpCtx::default();
        ctx.is_little_endian = !cfg!(target_endian = "little");

        let v = <$typ>::inner_from_reader(&mut Cursor::new(orig_bytes), &mut ctx).unwrap();

        if !all_same_bytes(orig_bytes) {
            assert_ne!(v, $init_val, "{}.from_reader() parsed value with different endianness should be different", stringify!($typ));
        }
        assert_eq!(ctx.cursor, size_of::<$typ>(), "{}.from_reader() didnt increment cursor properly", stringify!($typ));

        dst.clear();
        ctx.cursor = 0;
        v.inner_to_writer(&mut ctx, &mut dst).unwrap();
        assert_eq!(ctx.cursor, size_of::<$typ>(), "{}.to_writer() didnt increment cursor properly", stringify!($typ));
        assert_eq!(&dst, orig_bytes, "{}.to_writer() didnt generate same data using non-native endianness", stringify!($typ));
    }};
}


#[test]
fn primitives_unsigned() {
      
    test_primitive!(u8, 0);
    test_primitive!(u8, u8::MAX/2);
    test_primitive!(u8, u8::MAX);
    test_primitive!(u16, 0);
    test_primitive!(u16, u16::MAX/2);
    test_primitive!(u16, u16::MAX);
    test_primitive!(u32, 0);
    test_primitive!(u32, u32::MAX/2);
    test_primitive!(u32, u32::MAX);
    test_primitive!(u64, 0);
    test_primitive!(u64, u64::MAX/2);
    test_primitive!(u64, u64::MAX);
    test_primitive!(u128, 0);
    test_primitive!(u128, u128::MAX/2);
    test_primitive!(u128, u128::MAX);
    test_primitive!(usize, 0);
    test_primitive!(usize, usize::MAX/2);
    test_primitive!(usize, usize::MAX);
}

#[test]
fn primitives_signed() {
    test_primitive!(i8, 0);
    test_primitive!(i8, i8::MAX/2);
    test_primitive!(i8, i8::MAX);
    test_primitive!(i16, 0);
    test_primitive!(i16, i16::MAX/2);
    test_primitive!(i16, i16::MAX);
    test_primitive!(i32, 0);
    test_primitive!(i32, i32::MAX/2);
    test_primitive!(i32, i32::MAX);
    test_primitive!(i64, 0);
    test_primitive!(i64, i64::MAX/2);
    test_primitive!(i64, i64::MAX);
    test_primitive!(i128, 0);
    test_primitive!(i128, i128::MAX/2);
    test_primitive!(i128, i128::MAX);
    test_primitive!(isize, 0);
    test_primitive!(isize, isize::MAX/2);
    test_primitive!(isize, isize::MAX);
}

#[test]
fn primitives_special() {
    test_primitive!(f32, 0.0);
    test_primitive!(f64, 0.0);

    test_primitive!(bool, true);
    test_primitive!(bool, false);

    // Make sure you cant construct a NonZero from 0
    let zero: &[u8] = &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let non_zero: &[u8] = &[1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    
    assert!(<NonZeroU8>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroU8>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroU16>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroU16>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroU32>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroU32>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroU64>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroU64>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroU128>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroU128>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroUsize>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroUsize>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroI8>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroI8>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroI16>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroI16>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroI32>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroI32>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroI64>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroI64>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroI128>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroI128>::from_reader(&mut Cursor::new(non_zero)).is_ok());
    assert!(<NonZeroIsize>::from_reader(&mut Cursor::new(zero)).is_err());
    assert!(<NonZeroIsize>::from_reader(&mut Cursor::new(non_zero)).is_ok());
}