use std::{io::Cursor, mem::MaybeUninit};

use simple_parse::*;

#[test]
fn collections() {
    let s = "Hello World !";
    let mut orig = (s.len() as simple_parse::DefaultCountType)
        .to_le_bytes()
        .to_vec();
    orig.extend(s.as_bytes());

    let mut tmp = MaybeUninit::uninit();
    assert_eq!(
        <String>::from_reader(&mut Cursor::new(&orig), &mut tmp).unwrap(),
        s
    );
}
