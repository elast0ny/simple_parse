# simple_parse

[![crates.io](https://img.shields.io/crates/v/simple_parse.svg)](https://crates.io/crates/simple_parse)
[![mio](https://docs.rs/simple_parse/badge.svg)](https://docs.rs/simple_parse/)
![Lines of Code](https://tokei.rs/b1/github/elast0ny/simple_parse)

simple_parse is a declarative encoder/decoder for Rust structs to/from binary.

It provides basic implementations for most [standard Rust types](#Default-Impls) and also provides a derive macro to implement it one your own structs !

For lower level control, take a look at [deku](https://github.com/sharksforarms/deku).

## Usage

```Rust
use ::simple_parse::{SpRead, SpWrite};

#[derive(SpRead, SpWrite)]
pub struct SomeStruct {
    pub some_field: u8,
    nitems: u16,
    #[sp(count="nitems", endian="big")]
    pub items: Vec<u32>,
}

let mut cursor: &[u8] = &[
    0x01,                   // some_field
    0x00,0x02,              // nitems (Field holding the number of Vec items)
    0xDE,0xAD,0xBE,0xEF,    // items[0]
    0xBA,0xDC,0x0F,0xFE     // items[1]
];

// Decode bytes into a struct
let mut my_struct = SomeStruct::from_bytes(&mut cursor)?;

/// Modify the struct
my_struct.items.push(0xFFFFFFFF);

/// Encode modified struct into bytes
let mut dst_buf: Vec<u8> = Vec::new();
my_struct.to_bytes(&mut new_buf)?;
//dst_buf == [0x01,0x00,0x03,0xDE,0xAD,0xBE,0xEF,0xBA,0xDC,0x0F,0xFE,0xFF,0xFF,0xFF,0xFF]
```

For complete examples see : [examples](examples/)


## Default Impls
| Type | Encoded size |
|:------:|:------:|
|u8\|u16\|u32\|u64\|u128\|usize| 1,2,4,8,16,sizeof(usize) |
|i8\|i16\|i32\|i64\|i128\|isize| 1,2,4,8,16,sizeof(usize) |
|raw ptr| sizeof(usize) |
|bool| 1 |
| String | sizeof(u64) + str.len()|
| CString | str.len() + 1 |
| Vec<T> | sizeof(u64) + [vec.len() * sizeof(T)] |
| HashSet<K> | sizeof(u64) + [set.len() * sizeof(K)] |
| HashMap<K,V> | sizeof(u64) + [map.len() * (sizeof(K) + sizeof(V))] |