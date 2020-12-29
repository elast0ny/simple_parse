use crate::*;

/// Consumes bytes until the specified offset is reached then calls T::from_reader
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
        validate_reader_exact(&mut dst[..bytes_to_read], src)?;
        ctx.cursor += bytes_to_read;
    }

    T::inner_from_reader(src, ctx)
}

/// Consumes bytes until the specified offset is reached then calls T::from_reader with `count` set to Some(total_sz - offset)
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
        validate_reader_exact(&mut dst[..bytes_to_read], src)?;
        ctx.cursor += bytes_to_read;
    }

    ctx.count = Some((total - ctx.cursor) / std::mem::size_of::<T>());
    T::inner_from_reader(src, ctx)
}

/// Writes null bytes into the writer until offset is reached
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
    tmp_ctx.count = Some(0);

    while ctx.cursor + bytes_written != offset {
        let bytes_to_write = std::cmp::min(MAX_ALLOC_SIZE, offset - ctx.cursor);
        bytes_written += (&src[..bytes_to_write]).inner_to_writer(&mut tmp_ctx, dst)?;
    }
    ctx.cursor += bytes_written;

    Ok(bytes_written + this.inner_to_writer(ctx, dst)?)
}

/// Writes null bytes into the writer until offset is reached then calls T.to_writer() with count set to Some()
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
    ctx.count = Some(usize::MAX);
    write_at_offset(this, offset, ctx, dst)
}
