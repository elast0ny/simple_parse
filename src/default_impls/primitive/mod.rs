/** Implements simple_parse traits on primitive types such as :
 * u8, u16, i8, i16, etc...
 * float, bool,
 * Atomic*, NonZero*
*/
mod read;
mod write;
pub use read::*;
pub use write::*;
