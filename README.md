# simple_parse

[![crates.io](https://img.shields.io/crates/v/simple_parse.svg)](https://crates.io/crates/simple_parse)
[![mio](https://docs.rs/simple_parse/badge.svg)](https://docs.rs/simple_parse/)
![Lines of Code](https://tokei.rs/b1/github/elast0ny/simple_parse)

This crate is heavily based of [deku](https://github.com/sharksforarms/deku) with the bit level precision removed.

## Usage

```Rust
use simple_parse::{SpRead, SpWrite};

#[derive(Debug, SpRead, SpWrite)]
pub struct SomeStruct {
    some_field: u8,
    num_items: u16,
    #[sp(count="num_items", endian="big")]
    items: Vec<u32>,
}

/// <...>

// Parse arbitrary byte buffer into our struct
let (rest, mut res) = SomeStruct::from_bytes(byte_slice)?;
/// Modify the struct
res.dwords.push(0x12345678);
/// Dump modified struct into Vec<u8>
let generated_bytes = res.to_bytes()?;

```

For complete examples see : [examples](examples/)