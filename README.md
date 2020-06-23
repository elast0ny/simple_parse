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
    num_dwords: u16,
    #[sp(count="num_dwords", endian="big")]
    dwords: Vec<u32>,
}

/// <...>

let (rest, mut res) = SomeStruct::from_bytes(byte_slice)?;
res.dwords.push(0x12345678);
let generated_bytes = res.to_bytes()?;

```

For complete examples see : [examples](examples/)