# simple_parse

[![crates.io](https://img.shields.io/crates/v/simple_parse.svg)](https://crates.io/crates/simple_parse)
[![mio](https://docs.rs/simple_parse/badge.svg)](https://docs.rs/simple_parse/)
![Lines of Code](https://tokei.rs/b1/github/elast0ny/simple_parse)

`simple_parse` is a declarative binary stream parser that aims to generate the most efficient parsing code possible for your custom types while remaining safe.


| Features | Description |
|:----:|:----|
| Single "copy" | The data is read directly into it's final destination whenever possible |
| Built-in endianness support | Annotating structs/fields with `endian` gives control over how numbers will be parsed |
| Convert back to bytes | In addition to parsing arbitrary bytes, `simple_parse` also allows dumping structs back into binary form |

***

If `simple_parse` is unable to describe your complex/non-standard binary formats, take a look at [deku](https://github.com/sharksforarms/deku) or [binrw](https://github.com/jam1garner/binrw).

## Usage

See [client_server](examples/client_server.rs) for the complete example.

```Rust
use ::simple_parse::{SpRead, SpWrite};

#[derive(SpRead, SpWrite)]
pub enum Message {
    Ping,
    Pong,
    Chat(String),
    Key {
        private: Vec<u8>,
        public: Vec<u8>,
    },
    Disconnect,
}

pub fn main() {
    /* <...> */
    
    // Declare a destination buffer to use when parsing
    let mut dst: MaybeUninit<Message> = MaybeUninit::uninit();

    loop {
        // Receive & parse bytes from the socket as a `Message` using SpRead
        let msg = Message::from_reader(&mut sock, &mut dst).expect("[server] Failed to receive message");

        match msg {
            Message::Ping => {
                println!("[server] Got Ping ! Sending Pong...");
                // Respond with a Pong using SpWrite
                (Message::Pong).to_writer(&mut sock).expect("[server] Failed to send Pong");
            },
            Message::Pong => println!("[server] got pong !"),
            Message::Chat(s) => println!("[server] Received chat : '{s}'"),
            Message::Key{private, public} => println!("[server] got keys : {private:X?}:{public:X?}"),
            Message::Disconnect => break,
        }
    }

    /* <...> */
}
```

For more examples see : [examples/](examples/)


## Project Goals
In vague order of priority, `simple_parse` aims to provide :

 1. Safety
 2. Performance
 3. Ease of use
 4. Adaptability

In other words, `simple_parse` will try to generate the most performant code while never compromising on safety.

Secondly, priority will be given to ease of use by providing default implementations that work well in most cases while also allowing *some* customisation to accomodate for binary formats we cannot control.

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
    #[sp(endian="big")]
    size: u32,
    reserved1: u16,
    reserved2: u16,
    pixel_array_offset: u32,
    // ...
```
This tells `simple_parse` to insert a call to `validate_header(magic: &u16, ctx: &mut SpCtx)` directly after having populated the `u16` when reading and before dumping the struct as bytes when writing.

### __Custom Length (for TLV style)__
`simple_parse` provides default implementations for dynamically sized types by simply prepending the number of elements (`len`) followed by the elements.

i.e. By default, a `Vec<u8>` with three values will map to :
```Rust
// [len] | [len] * [elements]
[3u32][val1][val2][val3]
```
When parsing binary formats that dont follow this layout, you can annotate your dynamically sized field with `len` :
```Rust
pub struct File {
    pub content_len: u16,
    pub filename: String, // Use the default prepended len
    #[sp(len="content_len")]
    pub contents: Vec<u8>, // Use an existing field as the len
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

## License

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.