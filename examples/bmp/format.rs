use simple_parse::{validate_reader_exact, SpCtx, SpError, SpRead, SpWrite};
use std::io::{Read, Write};

#[derive(Debug, SpRead, SpWrite)]
#[sp(endian = "little")] // The BMP format explicitely needs little endian
pub struct BmpHeader {
    #[sp(validate = "validate_magic_header")]
    // Call `validate_magic_header()` with the contents of `magic`
    pub magic: u16,
    pub size: u32,
    reserved1: u16,
    reserved2: u16,
    pixel_array_offset: u32,
    #[sp(var_size)]
    // We must tell simple_parse that this custom type has a variable size or this wont compile
    dib: DIBHeader,
}

// You differentiate DIB header types by their size...
#[derive(Debug, SpRead, SpWrite)]
#[sp(id_type = "u32", endian = "little")] // The header size is a u32
pub enum DIBHeader {
    // Only support BITMAPINFOHEADER for this toy example
    #[sp(id = 40)]
    BitmapInfo {
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        size: u32,
        horizontal_res: i32,
        vertical_res: i32,
        clr_used: u32,
        important: u32,
        // The logic for parsing the two fields bellow is complicated. We must use custom reader/writer
        #[sp(
            reader="BitmapCompression::read, compression", // will generate : `self.compression_info = BitmapCompression::read(&compression, ...)?;`
            writer="BitmapCompression::write"
        )]
        compression_info: BitmapCompression,
        #[sp(
            // Regular functions are also allowed
            reader="parse_color_table, clr_used, compression_info, bit_count",
            writer="write_color_table"
        )]
        color_palette: Vec<RgbQuad>,
    },
}

#[derive(Debug)]
pub enum BitmapCompression {
    None,
    BitFields {
        red: u32,
        green: u32,
        blue: u32,
    },
    AlphaBitFields {
        alpha: u32,
        red: u32,
        green: u32,
        blue: u32,
    },
}

/// holds a color from the bmp color table
#[derive(Debug, SpRead, SpWrite)]
pub struct RgbQuad {
    blue: u8,
    green: u8,
    red: u8,
    reserved: u8,
}

impl BitmapCompression {
    // Custom SpRead parser based on the compression value
    fn read<R: Read + ?Sized>(
        compression_bitmask: &u32,
        src: &mut R,
        ctx: &mut SpCtx,
    ) -> Result<Self, SpError> {
        const BI_RGB: u32 = 0;
        const BI_BITFIELDS: u32 = 3;
        const BI_ALPHABITFIELDS: u32 = 6;
        // The compression method is encoded into the compression header field
        match *compression_bitmask {
            BI_RGB => Ok(Self::None),
            BI_BITFIELDS => {
                // Call read() for every u32
                let red = u32::inner_from_reader(src, ctx)?;
                let green = u32::inner_from_reader(src, ctx)?;
                let blue = u32::inner_from_reader(src, ctx)?;
                Ok(Self::BitFields { red, green, blue })
            }
            BI_ALPHABITFIELDS => {
                // Call read() once for 16 bytes
                let mut tmp = [0u8, 16];
                // Use simple_parse provided helper
                validate_reader_exact(&mut tmp, src)?;
                let mut checked_bytes = tmp.as_mut_ptr();
                let alpha: u32;
                let red: u32;
                let green: u32;
                let blue: u32;
                // Use unchecked versions to consume the pre-validated 16 bytes
                unsafe {
                    alpha = u32::inner_from_reader_unchecked(checked_bytes, src, ctx)?;
                    checked_bytes = checked_bytes.add(4);
                    red = u32::inner_from_reader_unchecked(checked_bytes, src, ctx)?;
                    checked_bytes = checked_bytes.add(4);
                    green = u32::inner_from_reader_unchecked(checked_bytes, src, ctx)?;
                    checked_bytes = checked_bytes.add(4);
                    blue = u32::inner_from_reader_unchecked(checked_bytes, src, ctx)?;
                }
                Ok(Self::AlphaBitFields {
                    alpha,
                    red,
                    green,
                    blue,
                })
            }
            _ => Err(SpError::InvalidBytes),
        }
    }
    // Custom SpWrite
    fn write<W: Write + ?Sized>(
        this: &BitmapCompression,
        ctx: &mut SpCtx,
        dst: &mut W,
    ) -> Result<usize, SpError> {
        match this {
            &Self::None => Ok(0),
            &Self::AlphaBitFields {
                alpha,
                red,
                green,
                blue,
            } => {
                alpha.inner_to_writer(ctx, dst)?;
                red.inner_to_writer(ctx, dst)?;
                green.inner_to_writer(ctx, dst)?;
                blue.inner_to_writer(ctx, dst)?;
                Ok(16)
            }
            &Self::BitFields { red, green, blue } => {
                red.inner_to_writer(ctx, dst)?;
                green.inner_to_writer(ctx, dst)?;
                blue.inner_to_writer(ctx, dst)?;
                Ok(12)
            }
        }
    }
}

/// This function is called as soon as the magic u16 byte is read
fn validate_magic_header(magic: &u16, _ctx: &mut SpCtx) -> Result<(), SpError> {
    // BMP headers must start with two bytes containing B and M
    if *magic != 0x4D42 {
        Err(SpError::InvalidBytes)
    } else {
        Ok(())
    }
}

/// Parses a BMP color table based on header values
fn parse_color_table<R: Read + ?Sized>(
    clr_used: &u32,
    compression: &BitmapCompression,
    bit_count: &u16,
    src: &mut R,
    ctx: &mut SpCtx,
) -> Result<Vec<RgbQuad>, SpError> {
    // The bitmap is not compressed which means every pixel contains the actual color information.
    // The Color table is supposed to be empty
    if let BitmapCompression::None = compression {
        return Ok(Vec::new());
    }

    let mut res = Vec::new();
    match *bit_count {
        // 1 bit_count means there must be 2 colors
        // 2 bit_count meants the must be 4 colors
        1 | 2 => {
            for _i in 0..(2u8).pow(*bit_count as _) {
                res.push(RgbQuad::inner_from_reader(src, ctx)?);
            }
        }
        // 4 bit_count and up encode how many colors are used in clr_used. This value has a max of 2^bit_count
        4 | 8 | 16 | 24 | 32 => {
            // Make sure not too many colors are provided
            if *clr_used > (2u32).pow(*bit_count as _) {
                return Err(SpError::InvalidBytes);
            }
            // Add the colors to our array
            for _i in 0..*clr_used {
                res.push(RgbQuad::inner_from_reader(src, ctx)?);
            }
        }
        _ => return Err(SpError::InvalidBytes),
    }

    Ok(res)
}

/// Writes BMP a color table
fn write_color_table<W: Write + ?Sized>(
    this: &Vec<RgbQuad>,
    ctx: &mut SpCtx,
    dst: &mut W,
) -> Result<usize, SpError> {
    let mut size_written = 0;
    // extra validation could be done here to ensure clr_used and bit_count is valid for these colors
    for color in this.iter() {
        size_written += color.inner_to_writer(ctx, dst)?;
    }
    Ok(size_written)
}
