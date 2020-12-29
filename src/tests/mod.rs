use std::io::Cursor;
use std::mem::size_of;

use crate::*;

const NUM_ITEMS: usize = 3;

#[macro_use]
macro_rules! test_primitive {
    // Defaults to using Self::STATIC_SIZE
    ($typ:ty, $new:ident $(, $ref_test:ident)?) => {{
        let mut ctx = SpCtx::default();
        let mut i = 0;
        let mut empty: [u8; 0] = [0; 0];
        let mut tmp: [u8; size_of::<$typ>() * NUM_ITEMS] = [0; size_of::<$typ>() * NUM_ITEMS];

        println!("<{}>", stringify!($typ));

        println!("\tEmpty Slice");
        assert_eq!(<$typ>::inner_from_reader(&mut Cursor::new(&empty), &mut ctx), Err(SpError::NotEnoughSpace));
        assert_eq!(<$typ>::inner_from_slice(&mut Cursor::new(&empty), &mut ctx), Err(SpError::NotEnoughSpace));
        assert_eq!(<$typ>::inner_from_mut_slice(&mut Cursor::new(&mut empty), &mut ctx), Err(SpError::NotEnoughSpace));

        println!("\tOne byte short");
        assert_eq!(<$typ>::inner_from_reader(&mut Cursor::new(&tmp[..size_of::<$typ>()-1]), &mut ctx), Err(SpError::NotEnoughSpace));
        assert_eq!(<$typ>::inner_from_slice(&mut Cursor::new(&tmp[..size_of::<$typ>()-1]), &mut ctx), Err(SpError::NotEnoughSpace));
        assert_eq!(<$typ>::inner_from_mut_slice(&mut Cursor::new(&mut tmp[..size_of::<$typ>()-1]), &mut ctx), Err(SpError::NotEnoughSpace));

        println!("\tOne byte extra");
        let mut cur = Cursor::new(&tmp[..size_of::<$typ>()+1]);
        assert_eq!(<$typ>::inner_from_reader(&mut cur, &mut ctx).unwrap(), $new(0));
        assert_eq!(cur.position() as usize, size_of::<$typ>());
        let mut cur = Cursor::new(&tmp[..size_of::<$typ>()+1]);
        assert_eq!(<$typ>::inner_from_slice(&mut cur, &mut ctx).unwrap(), $new(0));
        assert_eq!(cur.position() as usize, size_of::<$typ>());
        let mut cur = Cursor::new(&mut tmp[..size_of::<$typ>()+1]);
        assert_eq!(<$typ>::inner_from_mut_slice(&mut cur, &mut ctx).unwrap(), $new(0));
        assert_eq!(cur.position() as usize, size_of::<$typ>());

        println!("\tLittle Endian");
        ctx.is_little_endian = true;
        tmp[0] = 1;
        assert_eq!(<$typ>::inner_from_reader(&mut Cursor::new(&tmp), &mut ctx).unwrap(), $new(1));
        assert_eq!(<$typ>::inner_from_slice(&mut Cursor::new(&tmp), &mut ctx).unwrap(), $new(1));
        assert_eq!(<$typ>::inner_from_mut_slice(&mut Cursor::new(&mut tmp), &mut ctx).unwrap(), $new(1));

        println!("\tBig Endian");
        tmp[0] = 0;
        tmp[size_of::<$typ>()-1] = 1;
        ctx.is_little_endian = false;
        assert_eq!(<$typ>::inner_from_reader(&mut Cursor::new(&tmp), &mut ctx).unwrap(), $new(1));
        assert_eq!(<$typ>::inner_from_slice(&mut Cursor::new(&tmp), &mut ctx).unwrap(), $new(1));
        assert_eq!(<$typ>::inner_from_mut_slice(&mut Cursor::new(&mut tmp), &mut ctx).unwrap(), $new(1));

        println!("\t{} * {}", stringify!($typ), NUM_ITEMS);
        ctx.is_little_endian = true;
        let mut cur = Cursor::new(&tmp[..]);
        while let Ok(_) = <$typ>::inner_from_reader(&mut cur, &mut ctx) {
            i += 1;
        }
        assert_eq!(i, NUM_ITEMS);
        assert_eq!(cur.position() as usize, size_of::<$typ>() * NUM_ITEMS);
        i = 0;
        let mut cur = Cursor::new(&tmp[..]);
        while let Ok(_) = <$typ>::inner_from_slice(&mut cur, &mut ctx) {
            i += 1;
        }
        assert_eq!(i, NUM_ITEMS);
        assert_eq!(cur.position() as usize, size_of::<$typ>() * NUM_ITEMS);
        i = 0;
        let mut cur = Cursor::new(&mut tmp[..]);
        while let Ok(_) = <$typ>::inner_from_mut_slice(&mut cur, &mut ctx) {
            i += 1;
        }
        assert_eq!(i, NUM_ITEMS);
        assert_eq!(cur.position() as usize, size_of::<$typ>() * NUM_ITEMS);

        $($ref_test!($typ, $new);)?
    }};
}

#[macro_use]
macro_rules! test_references {
    // Defaults to using Self::STATIC_SIZE
    ($typ:ty, $new:ident) => {{
        let mut ctx = SpCtx::default();
        let mut tmp: [u8; size_of::<$typ>() * NUM_ITEMS] = [0; size_of::<$typ>() * NUM_ITEMS];

        tmp[0] = 2;

        println!("\t&{}", stringify!($typ));
        // References
        let mut cur = Cursor::new(&tmp[..]);
        assert_eq!(
            *<&$typ>::inner_from_slice(&mut cur, &mut ctx).unwrap(),
            $new(2)
        );
        assert_eq!(
            *<&$typ>::inner_from_slice(&mut cur, &mut ctx).unwrap(),
            $new(0)
        );
        let mut cur = Cursor::new(&mut tmp[..]);
        assert_eq!(
            *<&$typ>::inner_from_mut_slice(&mut cur, &mut ctx).unwrap(),
            $new(2)
        );
        assert_eq!(
            *<&$typ>::inner_from_mut_slice(&mut cur, &mut ctx).unwrap(),
            $new(0)
        );
        let mut cur = Cursor::new(&mut tmp[..]);
        assert_eq!(
            *<&mut $typ>::inner_from_mut_slice(&mut cur, &mut ctx).unwrap(),
            $new(2)
        );
        assert_eq!(
            *<&mut $typ>::inner_from_mut_slice(&mut cur, &mut ctx).unwrap(),
            $new(0)
        );

        // Slice
        println!("\t&[{}] with count", stringify!($typ));

        ctx.count = Some(0);
        let mut cur = Cursor::new(&tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_slice(&mut cur, &mut ctx)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(cur.position(), 0);
        let mut cur = Cursor::new(&mut tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_mut_slice(&mut cur, &mut ctx)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(cur.position(), 0);

        ctx.count = Some(NUM_ITEMS);
        let mut cur = Cursor::new(&tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_slice(&mut cur, &mut ctx)
                .unwrap()
                .len(),
            NUM_ITEMS
        );
        assert_eq!(cur.position() as usize, size_of::<$typ>() * NUM_ITEMS);
        let mut cur = Cursor::new(&mut tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_mut_slice(&mut cur, &mut ctx)
                .unwrap()
                .len(),
            NUM_ITEMS
        );
        assert_eq!(cur.position() as usize, size_of::<$typ>() * NUM_ITEMS);

        ctx.count = Some(NUM_ITEMS - 1);
        let mut cur = Cursor::new(&tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_slice(&mut cur, &mut ctx)
                .unwrap()
                .len(),
            NUM_ITEMS - 1
        );
        assert_eq!(cur.position() as usize, size_of::<$typ>() * (NUM_ITEMS - 1));
        let mut cur = Cursor::new(&mut tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_mut_slice(&mut cur, &mut ctx)
                .unwrap()
                .len(),
            NUM_ITEMS - 1
        );
        assert_eq!(cur.position() as usize, size_of::<$typ>() * (NUM_ITEMS - 1));

        ctx.count = Some(NUM_ITEMS + 1);
        let mut cur = Cursor::new(&tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_slice(&mut cur, &mut ctx),
            Err(SpError::NotEnoughSpace)
        );
        let mut cur = Cursor::new(&mut tmp[..]);
        assert_eq!(
            <&[$typ]>::inner_from_mut_slice(&mut cur, &mut ctx),
            Err(SpError::NotEnoughSpace)
        );
    }};
}

fn new_primitive<T>(val: T) -> T {
    val
}

fn new_float32(val: usize) -> f32 {
    if val == 1 {
        0.000000000000000000000000000000000000000000001
    } else if val == 2 {
        0.000000000000000000000000000000000000000000003
    } else {
        0.0
    }
}

fn new_float64(val: usize) -> f64 {
    if val == 1 {
        0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005
    } else if val == 2 {
        0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001
    } else {
        0.0
    }
}

fn new_bool(val: usize) -> bool {
    val > 0
}

#[test]
fn primitive_types() {
    test_primitive!(u8, new_primitive, test_references);
    test_primitive!(u16, new_primitive, test_references);
    test_primitive!(u32, new_primitive, test_references);
    test_primitive!(u64, new_primitive, test_references);
    test_primitive!(u128, new_primitive, test_references);
    test_primitive!(usize, new_primitive, test_references);
    test_primitive!(i8, new_primitive, test_references);
    test_primitive!(i16, new_primitive, test_references);
    test_primitive!(i32, new_primitive, test_references);
    test_primitive!(i64, new_primitive, test_references);
    test_primitive!(i128, new_primitive, test_references);
    test_primitive!(isize, new_primitive, test_references);

    test_primitive!(f32, new_float32, test_references);
    test_primitive!(f64, new_float64, test_references);

    test_primitive!(bool, new_bool);
}
