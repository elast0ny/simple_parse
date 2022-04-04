use crate::*;

/// Consumes bytes until the specified `offset` is reached then calls `T::from_reader`
pub fn read_at_offset<T, V, R>(offset: &V, src: &mut R, ctx: &mut SpCtx) -> Result<T, SpError>
where
    V: Copy + Into<u64>,
    T: SpRead + SpOptHints,
    R: Read + ?Sized,
{
    let offset: usize = (*offset).into() as usize;
    if offset < ctx.cursor {
        // The offset is invalid as we are already farther
        return Err(SpError::InvalidBytes);
    }

    #[cfg(feature = "verbose")]
    crate::debug!(
        "[0x{:X}] : Skipping 0x{:X} bytes to offset 0x{:X}",
        ctx.cursor,
        offset - ctx.cursor,
        offset
    );

    let mut dst = [0u8; MAX_ALLOC_SIZE];
    while ctx.cursor != offset {
        let bytes_to_read = std::cmp::min(MAX_ALLOC_SIZE, offset - ctx.cursor);
        validate_reader_exact(ctx, &mut dst[..bytes_to_read], src)?;
    }

    T::inner_from_reader(src, ctx)
}

/// Consumes bytes until the specified `offset` is reached then calls `T::from_reader` with `len` set to `Some(total_sz - offset)`
pub fn readall_at_offset<T, V, R>(
    offset: &V,
    total: &V,
    src: &mut R,
    ctx: &mut SpCtx,
) -> Result<T, SpError>
where
    V: Copy + Into<u64>,
    T: SpRead + SpOptHints,
    R: Read + ?Sized,
{
    let offset: usize = (*offset).into() as usize;
    let total: usize = (*total).into() as usize;

    if offset < ctx.cursor || offset > total {
        // The offset is invalid as we are already farther
        return Err(SpError::InvalidBytes);
    }

    #[cfg(feature = "verbose")]
    crate::debug!(
        "[0x{:X}] : Skipping 0x{:X} bytes to offset 0x{:X}",
        ctx.cursor,
        offset - ctx.cursor,
        offset
    );

    let mut dst = [0u8; MAX_ALLOC_SIZE];
    while ctx.cursor != offset {
        let bytes_to_read = std::cmp::min(MAX_ALLOC_SIZE, offset - ctx.cursor);
        validate_reader_exact(ctx, &mut dst[..bytes_to_read], src)?;
        ctx.cursor += bytes_to_read;
    }

    ctx.len = Some((total - ctx.cursor) / std::mem::size_of::<T>());
    T::inner_from_reader(src, ctx)
}

/// Writes null bytes into the writer until `offset` is reached
pub fn write_at_offset<T, V, W>(
    this: &T,
    offset: &V,
    ctx: &mut SpCtx,
    dst: &mut W,
) -> Result<usize, SpError>
where
    V: Copy + Into<u64>,
    T: SpWrite,
    W: Write + ?Sized,
{
    let offset: usize = (*offset).into() as usize;
    if offset < ctx.cursor {
        // The offset is invalid as we are already farther
        return Err(SpError::InvalidBytes);
    }

    #[cfg(feature = "verbose")]
    crate::debug!(
        "[0x{:X}] : Writing 0x{:X} bytes to reach offset 0x{:X}",
        ctx.cursor,
        offset - ctx.cursor,
        offset
    );

    let mut bytes_written = 0;
    let src = [0u8; MAX_ALLOC_SIZE];
    let mut tmp_ctx = SpCtx::default();
    tmp_ctx.len = Some(0);

    while ctx.cursor + bytes_written != offset {
        let bytes_to_write = std::cmp::min(MAX_ALLOC_SIZE, offset - ctx.cursor);
        bytes_written += (&src[..bytes_to_write]).inner_to_writer(&mut tmp_ctx, dst)?;
    }
    ctx.cursor += bytes_written;

    Ok(bytes_written + this.inner_to_writer(ctx, dst)?)
}

/// Writes null bytes into the writer until `offset` is reached then calls `T.to_writer()` with `len` set to `Some(usize::MAX)`
pub fn writeall_at_offset<T, V, W>(
    this: &T,
    offset: &V,
    ctx: &mut SpCtx,
    dst: &mut W,
) -> Result<usize, SpError>
where
    V: Copy + Into<u64>,
    T: SpWrite,
    W: Write + ?Sized,
{
    ctx.len = Some(usize::MAX);
    write_at_offset(this, offset, ctx, dst)
}

/// Reads `static_size` bytes from `src` into `dst`
///
/// This function can be used with untrusted `static_size` as it will
/// consume at most [MAX_ALLOC_SIZE] chunks at a time to prevent OOM.
#[inline(always)]
pub fn validate_reader<R: Read + ?Sized>(
    ctx: &mut SpCtx,
    static_size: usize,
    dst: &mut Vec<u8>,
    src: &mut R,
) -> Result<(), SpError> {
    let mut bytes_read = 0;

    while bytes_read < static_size {
        let cur_len = dst.len();
        let cur_chunk_len = std::cmp::min(MAX_ALLOC_SIZE, static_size - bytes_read);

        // Allocate an extra chunk at end of vec
        dst.reserve(cur_chunk_len);

        // Increase len and get slice to new chunk
        unsafe {
            dst.set_len(cur_len + cur_chunk_len);
        }
        let dst_slice = &mut dst.as_mut_slice()[cur_len..];

        // Read chunk into slice
        if let Err(e) = validate_reader_exact(ctx, dst_slice, src) {
            // Remove potentially uninit data from dst vec
            unsafe {
                dst.set_len(cur_len);
            }
            return Err(e);
        }

        bytes_read += cur_chunk_len;
    }

    Ok(())
}

/// Read the exact number of bytes required to fill `buf`.
/// When reading an untrusted number of bytes, use [validate_reader]
#[inline(always)]
pub fn validate_reader_exact<R: Read + ?Sized>(ctx: &mut SpCtx, dst: &mut [u8], src: &mut R) -> Result<(), SpError> {
    #[cfg(feature = "verbose")]
    crate::debug!("Read({})", dst.len());

    // Copy from reader into our stack variable
    if src.read_exact(dst).is_err() {
        return Err(SpError::NotEnoughSpace);
    }
    ctx.cursor += dst.len();

    Ok(())
}


// This only exists to work around const generic issues. It allows a "default" implementation for the SpRead trait
#[doc(hidden)]
#[inline(always)]
pub fn default_from_reader<const STATIC_SIZE: usize, V: SpRead, R: Read + ?Sized>(
    src: &mut R,
    ctx: &mut SpCtx,
) -> Result<V, crate::SpError> {
    let mut dst = [0u8; STATIC_SIZE];

    validate_reader_exact(ctx, &mut dst, src)?;
    unsafe {
        V::inner_from_reader(src, ctx, &dst)
    }
}
