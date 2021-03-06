
//! # Types
//! Convenience wrapper types and conversion logic for the winapi-defined types: 
//!   * CY
//!   * DATE
//!   * DECIMAL
//! 
use std::fmt;

#[cfg(feature = "impl_tryfrom")]
use std::convert::{TryFrom};

#[cfg(feature = "impl_tryfrom")]
use std::num::{TryFromIntError};

use rust_decimal::Decimal;

use winapi::shared::wtypes::{CY, DECIMAL, DECIMAL_NEG, VARIANT_BOOL, VARIANT_TRUE};

/// Pseudo-`From` trait because of orphan rules
trait Conversion<T> {
    fn convert(val: T) -> Self;
}

impl<T> Conversion<T> for T where T: From<T>{
    fn convert(val: T) -> Self {
        T::from(val)
    }
}

macro_rules! wrapper_conv_impl {
    ($inner:ident, $wrapper:ident) => {
        impl From<$inner> for $wrapper {
            fn from(i: $inner) -> Self {
                $wrapper(i)
            }
        }

        impl<'f> From<&'f $inner> for $wrapper {
            fn from(i: &$inner) -> Self {
                $wrapper(*i)
            }
        }

        impl<'f> From<&'f mut $inner> for $wrapper {
            fn from(i: &mut $inner) -> Self {
                $wrapper(*i)
            }
        }

        impl From<$wrapper> for $inner {
            fn from(o: $wrapper) -> Self {
                o.0
            }
        }

        impl<'f> From<&'f $wrapper> for $inner {
            fn from(o: &$wrapper) -> Self {
                o.0
            }
        }

        impl<'f> From<&'f mut $wrapper> for $inner {
            fn from(o: &mut $wrapper) -> Self {
                o.0
            }
        }

        impl Conversion<$inner> for $wrapper {
            fn convert(val: $inner) -> Self {
                $wrapper::from(val)
            }
        }

        impl Conversion<$wrapper> for $inner {
            fn convert(val: $wrapper) -> Self {
                $inner::from(val)
            }
        }

        impl<'c> Conversion<&'c $inner> for $wrapper {
            fn convert(val: &$inner) -> Self {
                $wrapper::from(val)
            }
        }

        impl<'c> Conversion<&'c $wrapper> for $inner {
            fn convert(val: &$wrapper) -> Self {
                $inner::from(val)
            }
        }

        impl<'c> Conversion<&'c mut $inner> for $wrapper {
            fn convert(val: &mut $inner) -> Self {
                $wrapper::from(val)
            }
        }

        impl<'c> Conversion<&'c mut $wrapper> for $inner {
            fn convert(val: &mut $wrapper) -> Self {
                $inner::from(val)
            }
        }
    };
}

macro_rules! conversions_impl {
    ($inner:ident, $wrapper:ident) => {
        impl Conversion<$inner> for $wrapper {
            fn convert(val: $inner) -> Self {
                $wrapper::from(val)
            }
        }

        impl Conversion<$wrapper> for $inner {
            fn convert(val: $wrapper) -> Self {
                $inner::from(val)
            }
        }

        impl<'c> Conversion<&'c $inner> for $wrapper {
            fn convert(val: &$inner) -> Self {
                $wrapper::from(val)
            }
        }

        impl<'c> Conversion<&'c $wrapper> for $inner {
            fn convert(val: &$wrapper) -> Self {
                $inner::from(val)
            }
        }

        impl<'c> Conversion<&'c mut $inner> for $wrapper {
            fn convert(val: &mut $inner) -> Self {
                $wrapper::from(val)
            }
        }

        impl<'c> Conversion<&'c mut $wrapper> for $inner {
            fn convert(val: &mut $wrapper) -> Self {
                $inner::from(val)
            }
        }
    };
}


/// Helper type for the OLE/COM+ type CY
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq,  Hash, PartialOrd, PartialEq)]
pub struct Currency(i64);

impl From<CY> for Currency {
    fn from(cy: CY) -> Currency {
        Currency(cy.int64)
    }
}
impl<'c> From<&'c CY> for Currency {
    fn from(cy: &CY) -> Currency {
        Currency(cy.int64)
    }
}
impl<'c> From<&'c mut CY> for Currency {
    fn from(cy: &mut CY) -> Currency {
        Currency(cy.int64)
    }
}

impl From<Currency> for CY {
    fn from(cy: Currency) -> CY {
        CY {int64: cy.0}
    }
}
impl<'c> From<&'c Currency> for CY {
    fn from(cy: &Currency) -> CY {
        CY {int64: cy.0}
    }
}
impl<'c> From<&'c mut Currency> for CY {
    fn from(cy: &mut Currency) -> CY {
        CY {int64: cy.0}
    }
}

impl AsRef<i64> for Currency {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}
wrapper_conv_impl!(i64, Currency);
conversions_impl!(Currency, CY);

/// Helper type for the OLE/COM+ type DATE
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct Date(f64); //DATE <--> F64

impl AsRef<f64> for Date {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

wrapper_conv_impl!(f64, Date);

/// Helper type for the OLE/COM+ type DECIMAL
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DecWrapper(Decimal);

impl DecWrapper {
    /// wraps a `Decimal` from rust_decimal
    pub fn new(dec: Decimal) -> DecWrapper {
        DecWrapper(dec)
    }

    /// Get access to the internal value, consuming it in the process
    pub fn unwrap(self) -> Decimal {
        self.0
    }

    /// Get borrow of internal value
    pub fn borrow(&self) -> &Decimal {
        &self.0
    }

    /// Get mutable borrow of internal value
    pub fn borrow_mut(&mut self) -> &mut Decimal {
        &mut self.0
    }

    fn build_c_decimal(dec: Decimal) -> DECIMAL {
        let scale = dec.scale() as u8;
        let sign = if dec.is_sign_positive() {0} else {DECIMAL_NEG};
        let serial = dec.serialize();
        let lo: u64 = (serial[4]  as u64)        + 
                    ((serial[5]  as u64) << 8)  + 
                    ((serial[6]  as u64) << 16) + 
                    ((serial[7]  as u64) << 24) + 
                    ((serial[8]  as u64) << 32) +
                    ((serial[9]  as u64) << 40) +
                    ((serial[10] as u64) << 48) + 
                    ((serial[11] as u64) << 56);
        let hi: u32 = (serial[12] as u32)        +
                    ((serial[13] as u32) << 8)  +
                    ((serial[14] as u32) << 16) +
                    ((serial[15] as u32) << 24);
        DECIMAL {
            wReserved: 0, 
            scale: scale, 
            sign: sign, 
            Hi32: hi, 
            Lo64: lo
        }
    }

    fn build_rust_decimal(dec: DECIMAL) -> Decimal {
        let sign = if dec.sign == DECIMAL_NEG {true} else {false};
        Decimal::from_parts((dec.Lo64 & 0xFFFFFFFF) as u32, 
                            ((dec.Lo64 >> 32) & 0xFFFFFFFF) as u32, 
                            dec.Hi32, 
                            sign,
                            dec.scale as u32 ) 
    }

}
//Conversions between triad of types:
//                    DECIMAL    |   DecWrapper   |   Decimal
//   DECIMAL    |      N/A       | owned, &, &mut | orphan rules
//   DecWrapper | owned, &, &mut |     N/A        | owned, &, &mut 
//   Decimal    | orphan rules   | owned, &, &mut |     N/A
//
// Ophan rules mean that I can't apply traits from other crates
// to types that come from still other traits. 

//DECIMAL to DecWrapper conversions
impl From<DECIMAL> for DecWrapper {
    fn from(d: DECIMAL) -> DecWrapper {
        DecWrapper(DecWrapper::build_rust_decimal(d))
    }
}
impl<'d> From<&'d DECIMAL> for DecWrapper {
    fn from(d: &DECIMAL) -> DecWrapper {
        DecWrapper(DecWrapper::build_rust_decimal(d.clone()))
    }
}
impl<'d> From<&'d mut DECIMAL> for DecWrapper {
    fn from(d: &mut DECIMAL) -> DecWrapper {
        DecWrapper(DecWrapper::build_rust_decimal(d.clone()))
    }
}

//DecWrapper to DECIMAL conversions
impl From<DecWrapper> for DECIMAL {
    fn from(d: DecWrapper) -> DECIMAL {
        DecWrapper::build_c_decimal(d.0)
    }
}
impl<'d> From<&'d DecWrapper> for DECIMAL {
    fn from(d: &DecWrapper) -> DECIMAL {
        DecWrapper::build_c_decimal(d.0)
    }
}
impl<'d> From<&'d mut DecWrapper> for DECIMAL {
    fn from(d: & mut DecWrapper) -> DECIMAL {
        DecWrapper::build_c_decimal(d.0)
    }
}

//DecWrapper to Decimal conversions
impl From<DecWrapper> for Decimal {
    fn from(dw: DecWrapper) -> Decimal {
        dw.0
    }
}
impl<'w> From<&'w DecWrapper> for Decimal {
    fn from(dw: &DecWrapper) -> Decimal {
        dw.0
    }
}
impl<'w> From<&'w mut DecWrapper> for Decimal {
    fn from(dw: &mut DecWrapper) -> Decimal {
        dw.0
    }
}

//Decimal to DecWrapper conversions
impl From<Decimal> for DecWrapper {
    fn from(dec: Decimal) -> DecWrapper {
        DecWrapper(dec)
    }
}
impl<'d> From<&'d Decimal> for DecWrapper {
    fn from(dec: &Decimal) -> DecWrapper {
        DecWrapper(dec.clone())
    }
}
impl<'d> From<&'d mut Decimal> for DecWrapper {
    fn from(dec: &mut Decimal) -> DecWrapper {
        DecWrapper(dec.clone())
    }
}

impl AsRef<Decimal> for DecWrapper {
    fn as_ref(&self) -> &Decimal {
        &self.0
    }
}
conversions_impl!(Decimal, DecWrapper);
conversions_impl!(DecWrapper, DECIMAL);

/// Helper type for the OLE/COM+ type VARIANT_BOOL
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantBool(bool);

impl From<VariantBool> for VARIANT_BOOL {
    fn from(vb: VariantBool) -> VARIANT_BOOL {
        if vb.0 {VARIANT_TRUE} else {0}
    }
}
impl<'v> From<&'v VariantBool> for VARIANT_BOOL {
    fn from(vb: &VariantBool) -> VARIANT_BOOL {
        if vb.0 {VARIANT_TRUE} else {0}
    }
}
impl<'v> From<&'v mut VariantBool> for VARIANT_BOOL {
    fn from(vb: &mut VariantBool) -> VARIANT_BOOL {
        if vb.0 {VARIANT_TRUE} else {0}
    }
}

impl From<VARIANT_BOOL> for VariantBool {
    fn from(vb: VARIANT_BOOL) -> VariantBool {
        VariantBool(vb < 0) 
    }
}
impl<'v> From<&'v VARIANT_BOOL> for VariantBool {
    fn from(vb: &VARIANT_BOOL) -> VariantBool {
        VariantBool(*vb < 0) 
    }
}
impl<'v> From<&'v mut VARIANT_BOOL> for VariantBool {
    fn from(vb: &mut VARIANT_BOOL) -> VariantBool {
        VariantBool(*vb < 0) 
    }
}

impl From<bool> for VariantBool {
    fn from(b: bool) -> Self {
        VariantBool(b)
    }
}

impl<'b> From<&'b bool> for VariantBool {
    fn from(b: &bool) -> Self {
        VariantBool(*b)
    }
}

impl<'b> From<&'b mut bool> for VariantBool {
    fn from(b: &mut bool) -> Self {
        VariantBool(*b)
    }
}

impl From<VariantBool> for bool {
    fn from(b: VariantBool) -> Self {
        b.0
    }
}
impl<'v> From<&'v VariantBool> for bool {
    fn from(b: &VariantBool) -> Self {
        b.0
    }
}
impl<'v> From<&'v mut VariantBool> for bool {
    fn from(b: &mut VariantBool) -> Self {
        b.0
    }
}

impl AsRef<bool> for VariantBool {
    fn as_ref(&self) -> &bool {
        &self.0
    }
}

conversions_impl!(bool, VariantBool);
conversions_impl!(VariantBool, VARIANT_BOOL);

/// Helper type for the OLE/COM+ type INT
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Int(i32);

impl AsRef<i32> for Int {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl fmt::UpperHex for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

impl fmt::LowerHex for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::Octal for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:o}", self.0)
    }
}

impl fmt::Binary for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

wrapper_conv_impl!(i32, Int);

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<i64> for Int {
    type Error = TryFromIntError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Int(i32::try_from(value)?))
    }
}

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<i128> for Int {
    type Error = TryFromIntError;
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        Ok(Int(i32::try_from(value)?))
    }
}

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<i16> for Int {
    type Error = TryFromIntError;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Ok(Int(value as i32))
    }
}

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<i8> for Int {
    type Error = TryFromIntError;
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Ok(Int(value as i32))
    }
}

/// Helper type for the OLE/COM+ type UINT
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UInt(u32);

impl AsRef<u32> for UInt {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl fmt::UpperHex for UInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

impl fmt::LowerHex for UInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::Octal for UInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:o}", self.0)
    }
}

impl fmt::Binary for UInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.0)
    }
}


wrapper_conv_impl!(u32, UInt);

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<u64> for UInt {
    type Error = TryFromIntError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(UInt(u32::try_from(value)?))
    }
}

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<u128> for UInt {
    type Error = TryFromIntError;
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Ok(UInt(u32::try_from(value)?))
    }
}

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<u16> for UInt {
    type Error = TryFromIntError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(UInt(value as u32))
    }
}

#[cfg(feature = "impl_tryfrom")]
impl TryFrom<u8> for UInt {
    type Error = TryFromIntError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(UInt(value as u32))
    }
}

/// Helper type for the OLE/COM+ type SCODE
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SCode(i32);

impl AsRef<i32> for SCode {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl fmt::UpperHex for SCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0X{:X}", self.0)
    }
}

impl fmt::LowerHex for SCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl fmt::Octal for SCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:o}", self.0)
    }
}

impl fmt::Binary for SCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

wrapper_conv_impl!(i32, SCode);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c_decimal() {
        let d = Decimal::new(0xFFFFFFFFFFFF, 0);
        let d = d * Decimal::new(0xFFFFFFFF, 0);
        assert_eq!(d.is_sign_positive(), true);
        assert_eq!(format!("{}", d), "1208925819333149903028225" );
        
        let c = DecWrapper::build_c_decimal(d);
        //println!("({}, {}, {}, {})", c.Hi32, c.Lo64, c.scale, c.sign);
        //println!("{:?}", d.serialize());
        assert_eq!(c.Hi32, 65535);
        assert_eq!(c.Lo64, 18446462594437873665); 
        assert_eq!(c.scale, 0);
        assert_eq!(c.sign, 0);
    }

    #[test]
    fn rust_decimal_from() {
        let d = DECIMAL {
            wReserved: 0, 
            scale: 0, 
            sign: 0, 
            Hi32: 65535, 
            Lo64: 18446462594437873665
        };
        let new_d = DecWrapper::build_rust_decimal(d);
        //println!("{:?}", new_d.serialize());
       // assert_eq!(new_d.is_sign_positive(), true);
        assert_eq!(format!("{}", new_d), "1208925819333149903028225"  );
    }

    #[test]
    fn variant_bool() {
        let vb = VariantBool::from(true);
        let pvb = VARIANT_BOOL::from(vb);
        assert_eq!(VARIANT_TRUE, pvb);

        let vb = VariantBool::from(false);
        let pvb = VARIANT_BOOL::from(vb);
        assert_ne!(VARIANT_TRUE, pvb);
    }

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Currency>();
        assert_send::<Date>();
        assert_send::<DecWrapper>();
        assert_send::<Int>();
        assert_send::<SCode>();
        assert_send::<UInt>();
        assert_send::<VariantBool>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Currency>();
        assert_sync::<Date>();
        assert_sync::<DecWrapper>();
        assert_sync::<Int>();
        assert_sync::<SCode>();
        assert_sync::<UInt>();
        assert_sync::<VariantBool>();
    }

    #[cfg(feature = "impl_tryfrom")]
    #[cfg_attr(feature = "impl_tryfrom", test)]
    fn test_tryfrom() {
        let v = Int::try_from(999999999999999i64);
        assert!(v.is_err());
    }
}
