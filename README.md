# simple_parse

[![crates.io](https://img.shields.io/crates/v/simple_parse.svg)](https://crates.io/crates/simple_parse)
[![mio](https://docs.rs/simple_parse/badge.svg)](https://docs.rs/simple_parse/)
![Lines of Code](https://tokei.rs/b1/github/elast0ny/simple_parse)

simple_parse is a declarative binary stream parser that aims to generate the most efficient parsing code possible for your custom types while remaining safe.


| Features | Description |
|:----:|:----|
| Fast| The generated parsing code is often faster than "idiomatic" C implementations|
| [No copy](examples/no_copy.rs) | Able to return references into byte slices |
| Built-in endianness support | Annotating structs/fields with `endian` gives control over how numbers will be parsed |
|Custom implementations | Custom parsers can be writen for individual fields when simple_parse doesnt have the adequate default implementation|



If simple_parse is unable to describe your complex/non-standard binary formats, take a look at [deku](https://github.com/sharksforarms/deku).

## Usage

Snippets taken from [examples/struct.rs](examples/struct.rs)
```Rust
use ::simple_parse::{SpRead, SpWrite};

#[derive(SpRead, SpWrite)]
pub struct SomeStruct {
    pub some_field: u8,
    #[sp(endian="big")]
    pub items: Vec<u32>,
}

// Emulate data coming from a socket
let mut srv_sock: &[u8] = &[
    1,                      // some_field
    0,0,0,2,                // items.len()
    0xDE,0xAD,0xBE,0xEF,    // items[0]
    0xBA,0xDC,0x0F,0xFE     // items[1]
];

// Parse incoming bytes into SomeStruct
let mut my_struct = SomeStruct::from_reader(&mut srv_sock)?;

/// Modify the struct
my_struct.items.push(0xFFFFFFFF);

/// Encode our struct back into bytes
let mut cli_sock: Vec<u8> = Vec::new();
my_struct.to_writer(&mut cli_sock)?;
//dst_buf == [1, 0, 0, 0, 3, DE, AD, BE, EF, BA, DC, F, FE, FF, FF, FF, FF]
```

For complete examples see : [examples](examples/)
