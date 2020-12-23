use simple_parse::{SpRead, SpError, validate_reader_exact};

#[derive(Debug, SpRead)]
#[sp(endian = "little")] // The BMP format forces little endian
pub struct BmpHeader {
    pub magic1: u8,
    pub magic2: u8,
    pub size: u32,
    reserved1: u16,
    reserved2: u16,
    pixel_array_offset: u32,
    #[sp(var_size)]
    dib: DIBHeader,
}

// You differentiate DIB header types by their size...
#[derive(Debug, SpRead)]
#[sp(id_type = "u32", endian = "little")] // The BMP format forces little endian
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
        // The logic for parsing the two fields bellow is complicated. We must use custom readers.
        #[sp(reader="BitmapCompression::read, compression")]
        compression_info: BitmapCompression,
        
        #[sp(reader="parse_color_table, clr_used, compression_info, bit_count")]
        color_palette: Vec<RgbQuad>,
    },
}

#[derive(Debug)]
pub enum BitmapCompression {
    None,
    BitFields{
        red: u32,
        green: u32,
        blue: u32,
    },
    AlphaBitFields{
        alpha: u32,
        red: u32,
        green: u32,
        blue: u32,
    }
}
impl BitmapCompression {
    // Custom SpRead parser based on the compression value
    fn read<R: std::io::Read + ?Sized>(compression_bitmask: &u32, src: &mut R, is_input_le: bool, count: Option<usize>) -> Result<Self, SpError> {
        const BI_RGB: u32 = 0;
        const BI_BITFIELDS : u32 = 3;
        const BI_ALPHABITFIELDS: u32 = 6;
        // The compression method is encoded into the compression header field
        match *compression_bitmask {
            BI_RGB => {
                Ok(Self::None)
            },
            BI_BITFIELDS => {
                // Call read() for every u32
                let red = u32::inner_from_reader(src, is_input_le, count)?;
                let green = u32::inner_from_reader(src, is_input_le, count)?;
                let blue = u32::inner_from_reader(src, is_input_le, count)?;
                Ok(Self::BitFields{red, green, blue})
            },
            BI_ALPHABITFIELDS => {
                // Call read() once for 16 bytes
                let mut tmp = [0u8, 16];
                validate_reader_exact(&mut tmp, src)?;
                let mut checked_bytes = tmp.as_mut_ptr();
                let alpha: u32;
                let red: u32;
                let green: u32;
                let blue: u32;
                // Use unchecked versions to consume the pre-validated 16 bytes
                unsafe {
                    alpha = u32::inner_from_reader_unchecked(checked_bytes, src, is_input_le, count)?;
                    checked_bytes = checked_bytes.add(4);
                    red = u32::inner_from_reader_unchecked(checked_bytes, src, is_input_le, count)?;
                    checked_bytes = checked_bytes.add(4);
                    green = u32::inner_from_reader_unchecked(checked_bytes, src, is_input_le, count)?;
                    checked_bytes = checked_bytes.add(4);
                    blue = u32::inner_from_reader_unchecked(checked_bytes, src, is_input_le, count)?;
                }
                Ok(Self::AlphaBitFields{alpha,red, green, blue})
            },
            _ => Err(SpError::InvalidBytes)
        }
    }
}

/// holds a color from the bmp color table
#[derive(Debug, SpRead)]
pub struct RgbQuad {
    blue: u8,
    green: u8,
    red: u8,
    reserved: u8,
}

/// Parses a BMP color table based on header values
fn parse_color_table<R: std::io::Read + ?Sized>(clr_used: &u32, compression: &BitmapCompression, bit_count: &u16, src: &mut R, is_input_le: bool, count: Option<usize>) -> Result<Vec<RgbQuad>, SpError> {
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
                res.push(RgbQuad::inner_from_reader(src, is_input_le, count)?);
            }
        },
        // 4 bit_count and up encode how many colors are used in clr_used. This value has a max of 2^bit_count
        4 | 8 | 16 | 24 | 32 => {
            // Make sure not too many colors are provided
            if *clr_used > (2u32).pow(*bit_count as _) {
                return Err(SpError::InvalidBytes);
            }
            // Add the colors to our array
            for _i in 0..*clr_used {
                res.push(RgbQuad::inner_from_reader(src, is_input_le, count)?);
            }
        },
        _ => return Err(SpError::InvalidBytes),
    }

    Ok(res)
}