# simple_parse

[![crates.io](https://img.shields.io/crates/v/simple_parse.svg)](https://crates.io/crates/simple_parse)
[![mio](https://docs.rs/simple_parse/badge.svg)](https://docs.rs/simple_parse/)
![Lines of Code](https://tokei.rs/b1/github/elast0ny/simple_parse)

`simple_parse` is a declarative binary stream parser that aims to generate the most efficient parsing code possible for your custom types while remaining safe.


| Features | Description |
|:----:|:----|
| Fast| The generated parsing code is often faster than "idiomatic" C implementations|
| [No copy](examples/no_copy.rs) | Able to return references into byte slices |
| Built-in endianness support | Annotating structs/fields with `endian` gives control over how numbers will be parsed |
| Convert back to bytes | In addition to parsing arbitrary bytes, `simple_parse` also allows dumping structs back into binary form |

***

If `simple_parse` is unable to describe your complex/non-standard binary formats, take a look at [deku](https://github.com/sharksforarms/deku).

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


## Project Goals
In vague order of priority, `simple_parse` aims to provide :

 1. Safety
 2. Performance
 3. Ease of use
 4. Adaptability

In other words, `simple_parse` will try to generate the most performant code while never compromising on safety.

Secondly, priority will be given to ease of use by providing default implementations that work well in most cases while also allowing *some* customisation to accomodate for binary formats we cannot control (see the bmp image parsing [example](examples/bmp/)).

## Advanced Usage
`simple_parse` provides a few ways to enhance the generate parsing code. See [attributes.rs](simple_parse-derive/src/attributes.rs) for an exhaustive list of options.
### __Validation__
It is possible to insert validation "hooks" at any point in the parsing/writing process.

For example, BMP image headers must always start with the two first bytes being `'BM'` :
```Rust
#[derive(SpRead, SpWrite)]
struct BmpHeader {
    #[sp(validate = "validate_header")]
    magic: u16,
    size: u32,
    reserved1: u16,
    reserved2: u16,
    pixel_array_offset: u32,
    // ...
```
(Taken from [bmp example](examples/bmp/main.rs))

This tells `simple_parse` to insert a call to `validate_header(magic: &u16, ctx: &mut SpCtx)` directly after having populated the `u16` when reading and before dumping the struct as bytes when writing.

### __Custom Length (for TLV style)__
`simple_parse` provides default implementations for dynamically sized types by simply prepending the number of elements (`count`) followed by the elements.

i.e. A Vec<u8> with three values turns into :
```Rust
// [count] | [count] * [elements]
[3u32][val1][val2][val3]
```
When parsing binary formats that dont follow this layout, you can annotate your dynamically sized field with `count` :
```Rust
pub struct File {
    pub content_len: u16,
    pub filename: String, // Use the default prepended count
    #[sp(count="content_len")]
    pub contents: Vec<u8>, // Use an existing field as the count
```
The `content_len` field will be used to populate `contents` and `contents.len()` will be written at that offset when writing.
### __Custom Read/Write__
When `simple_parse`'s default reading and writing implementations are not well suited for your formats, you can override them with the `reader` and `writer` attributes.
```Rust
struct BmpHeader {
    comp_bitmask: u32,
    #[sp(
        reader="BmpComp::read, comp_bitmask",
        writer="BmpComp::write",
    )]
    compression_info: BmpComp,
    //...
```
When reading, this will generate code like :

```Rust
compression_info = BmpComp::read(comp_bitmask: &u32, src: &mut Read, ctx: &mut SpCtx)?;
```

And when writing :

```Rust
written_sz += BmpComp::write(&self.compression_info, ctx: &mut SpCtx, dst: &mut Write)?;
```

Note : Using `reader` will generate suboptimal parsing code as `simple_parse` cannot make any assumptions about the custom `reader`'s impact on the input bytes.

## License

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.