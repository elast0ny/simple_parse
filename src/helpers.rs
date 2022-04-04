use crate::*;

/// Puts size_of::<V>() bytes into the MaybeUninit<V>
///
/// # Safety
/// This function does no validation on the internal representation of V
#[doc(hidden)]
pub fn static_size_from_reader<V: SpRead, R: Read + ?Sized, const STATIC_SIZE: usize>(
    // Data source
    src: &mut R,
    // Parsing context
    ctx: &mut SpCtx,
    // Data that has already been read from src. Use this first
    dst: &mut MaybeUninit<V>,
) -> Result<(), crate::SpError> {
    let raw_bytes =
        unsafe { core::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut u8, STATIC_SIZE) };

    if let Err(e) = src.read_exact(raw_bytes) {
        return Err(SpError::ReadFailed(e));
    }

    #[cfg(feature = "verbose")]
    ::log::debug!("  read({STATIC_SIZE})");

    ctx.cursor += STATIC_SIZE;

    Ok(())
}
