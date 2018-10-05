extern crate rust_decimal;
extern crate widestring;
extern crate winapi;

mod array;
mod bstr;
mod ptr;
mod types;
mod variant;

pub use array::{SafeArray, SafeArrayElement};
pub use ptr::{Ptr};
pub use types::{Currency, Date, DecWrapper,Int, SCode, UInt, VariantBool};
pub use variant::{Variant, VariantType, VtEmpty, VtNull};