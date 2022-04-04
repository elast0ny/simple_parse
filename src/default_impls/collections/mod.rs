/** Implements simple_parse traits on primitive types such as :
 * String, CString
 * Option<T>, Vec<T>, etc...
*/
mod read;
mod write;
pub use read::*;
pub use write::*;
