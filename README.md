# simple_parse

This crate is essentially a copy paste from [deku](https://github.com/sharksforarms/deku) with the bit level precision removed.


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