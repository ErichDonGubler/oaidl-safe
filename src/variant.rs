//! Variant conversions
//! 
//! This module contains the trait [`VariantExt`] and the types [`Variant`], [`VtEmpty`], [`VtNull`].
//! 
//! It implements [`VariantExt`] for many built in types to enable conversions to VARIANT.  
//! 
//! [`VariantExt`]: trait.VariantExt.html
//! [`Variant`]: struct.Variant.html
//! [`VtEmpty`]: struct.VtEmpty.html
//! [`VtNull`]: struct.VtNull.html

use std::marker::PhantomData;
use std::mem;
use std::ptr::null_mut;

use winapi::ctypes::c_void;
use winapi::shared::wtypes::{
    BSTR, 
    CY, 
    DATE, 
    DECIMAL,
    VARIANT_BOOL,
    VT_ARRAY, 
    VT_BSTR, 
    VT_BOOL,
    VT_BYREF, 
    VT_CY,
    VT_DATE,
    VT_DECIMAL, 
    VT_DISPATCH,
    VT_EMPTY, 
    VT_ERROR, 
    VT_I1, 
    VT_I2, 
    VT_I4,
    VT_I8,
    VT_INT,  
    VT_NULL,
    VT_R4, 
    VT_R8, 
    //VT_RECORD,
    VT_UI1,
    VT_UI2,
    VT_UI4,
    VT_UI8,  
    VT_UINT, 
    VT_UNKNOWN, 
    VT_VARIANT, 
};
use winapi::shared::wtypesbase::SCODE;
use winapi::um::oaidl::{IDispatch, SAFEARRAY, __tagVARIANT, VARIANT, VARIANT_n3, VARIANT_n1};
use winapi::um::oleauto::VariantClear;
use winapi::um::unknwnbase::IUnknown;

use super::array::SafeArrayElement;
use super::bstr::U16String;
use super::errors::{ElementError,  FromVariantError, IntoVariantError};
use super::ptr::Ptr;
use super::types::{ Currency, Date, DecWrapper, Int, SCode, TryConvert, UInt, VariantBool };

const VT_PUI1:      u32 = VT_BYREF | VT_UI1;
const VT_PI2:       u32 = VT_BYREF | VT_I2;
const VT_PI4:       u32 = VT_BYREF | VT_I4;
const VT_PI8:       u32 = VT_BYREF | VT_I8;
const VT_PUI8:      u32 = VT_BYREF | VT_UI8;
const VT_PR4:       u32 = VT_BYREF | VT_R4;
const VT_PR8:       u32 = VT_BYREF | VT_R8;
const VT_PBOOL:     u32 = VT_BYREF | VT_BOOL;
const VT_PERROR:    u32 = VT_BYREF | VT_ERROR;
const VT_PCY:       u32 = VT_BYREF | VT_CY;
const VT_PDATE:     u32 = VT_BYREF | VT_DATE;
const VT_PBSTR:     u32 = VT_BYREF | VT_BSTR;
const VT_PUNKNOWN:  u32 = VT_BYREF | VT_UNKNOWN;
const VT_PDISPATCH: u32 = VT_BYREF | VT_DISPATCH;
const VT_PARRAY:    u32 = VT_BYREF | VT_ARRAY;
const VT_PDECIMAL:  u32 = VT_BYREF | VT_DECIMAL;
const VT_PI1:       u32 = VT_BYREF | VT_I1;
const VT_PUI2:      u32 = VT_BYREF | VT_UI2;
const VT_PUI4:      u32 = VT_BYREF | VT_UI4;
const VT_PINT:      u32 = VT_BYREF | VT_INT;
const VT_PUINT:     u32 = VT_BYREF | VT_UINT;

mod private {
    use super::*;

    #[doc(hidden)]
    pub trait Sealed {}

    #[doc(hidden)]
    pub trait VariantAccess: Sized {
        #[doc(hidden)]
        const VTYPE: u32;

        #[doc(hidden)]
        type Field;

        #[doc(hidden)]
        fn from_var(n1: &VARIANT_n1, n3: &VARIANT_n3) -> Self::Field;
        
        #[doc(hidden)]
        fn into_var(inner: Self::Field, n1: &mut VARIANT_n1, n3: &mut VARIANT_n3);
    }

    macro_rules! impl_conversions {
        (@impl <$($life:lifetime)*> $t:ty, $f:ty, $vtype:ident, $member:ident, $member_mut:ident) => {
            impl $(<$life>)* VariantAccess for $t {
                const VTYPE: u32 = $vtype;
                type Field = $f;
                fn from_var(_n1: &VARIANT_n1, n3: &VARIANT_n3) -> Self::Field {
                    unsafe {*n3.$member()}
                }
                
                fn into_var(inner: Self::Field, _n1: &mut VARIANT_n1, n3: &mut VARIANT_n3) {
                    unsafe {
                        let n_ptr = n3.$member_mut();
                        *n_ptr = inner;
                    }
                }
            }
        };
        ( < $($tl:lifetime,)* $tn:ident : $tb:ident > $t:ty, $field:ty, $vtype:ident, $member:ident, $member_mut:ident ) => {
            impl<$($tl,)* $tn> VariantAccess for $t 
            where
                $tn: $tb
            {
                const VTYPE: u32 = $vtype;
                type Field = $field;

                fn from_var(_n1: &VARIANT_n1, n3: &VARIANT_n3) -> Self::Field {
                    unsafe {*n3.$member()}
                }

                fn into_var(inner: Self::Field, _n1: &mut VARIANT_n1, n3: &mut VARIANT_n3) {
                    unsafe {
                        let n_ptr = n3.$member_mut();
                        *n_ptr = inner;
                    }
                }
            }
        };
        (Ptr<$field:ty>, $vtype:ident, $member:ident, $member_mut:ident) => {
            impl VariantAccess for Ptr<$field>{
                const VTYPE: u32 = $vtype;
                type Field = Ptr<$field>;

                fn from_var(_n1: &VARIANT_n1, n3: &VARIANT_n3) -> Self::Field {
                    unsafe {Ptr::with_checked(*n3.$member()).unwrap()}
                }
                
                fn into_var(inner: Self::Field, _n1: &mut VARIANT_n1, n3: &mut VARIANT_n3) {
                    unsafe {
                        let n_ptr = n3.$member_mut();
                        *n_ptr = inner.as_ptr();
                    }
                }
            }
        };
        (Box<$t:ty>, $vtype:ident, $member:ident, $member_mut:ident) => {
            impl_conversions!(@impl <> Box<$t>, *mut $t, $vtype, $member, $member_mut);

            impl TryConvert<Box<$t>, IntoVariantError> for *mut $t {
                fn try_convert(b: Box<$t>) -> Result<Self, IntoVariantError> {
                    Ok(Box::into_raw(b))
                }
            }

            impl TryConvert<*mut $t, FromVariantError> for Box<$t> {
                fn try_convert(ptr: *mut $t) -> Result<Self, FromVariantError> {
                    if ptr.is_null() { return Err(FromVariantError::VariantPtrNull)}
                    Ok(Box::new(unsafe{*ptr}))
                }
            }
        };
        ($interm:ty => $f:ty, $vtype:ident, $member:ident, $member_mut:ident) => {
            impl_conversions!(@impl <> $interm, $f, $vtype, $member, $member_mut);
            impl_conversions!(@impl <'s> &'s mut $interm, $f, $vtype, $member, $member_mut);
        };
        
        ($t:ty, $vtype:ident, $member:ident, $member_mut:ident) => {
            impl_conversions!(@impl <> $t, $t, $vtype, $member, $member_mut);
            impl_conversions!(@impl <'s> &'s $t, $t, $vtype, $member, $member_mut);
            impl_conversions!(@impl <'s> &'s mut $t, $t, $vtype, $member, $member_mut);
        };
        
    }
    //direct conversions: 
    //  i64, i32, u8, i16, f32, f64, i8, u16, u32, u64
    // conversions with an intermediary needed:
    //  bool, String
    // boxed types: 
    //  i64, i32, u8, i16, f32, f64, i8, u16, u32, u64
    //  bool, String
    //  SCode, Currency, Date, *mut IUnknown, *mut IDispatch
    //  Decimal (DecWrapper)
    // convenience types: 
    //  SCode, Currency, Date, *mut IUnknown, *mut IDispatch
    //  Decimal (DecWrapper)
    impl_conversions!(i64, VT_I8, llVal, llVal_mut);
    impl_conversions!(i32, VT_I4, lVal, lVal_mut);
    impl_conversions!(u8, VT_UI1,  bVal, bVal_mut);
    impl_conversions!(i16, VT_I2, iVal, iVal_mut);
    impl_conversions!(f32, VT_R4, fltVal, fltVal_mut);
    impl_conversions!(f64, VT_R8, dblVal, dblVal_mut);
    impl_conversions!(VariantBool => VARIANT_BOOL, VT_BOOL, boolVal, boolVal_mut);
    impl_conversions!(SCode => SCODE, VT_ERROR, scode, scode_mut);
    impl_conversions!(Currency => CY, VT_CY, cyVal, cyVal_mut);
    impl_conversions!(Date => DATE, VT_DATE, date, date_mut);
    impl_conversions!(U16String => BSTR, VT_BSTR, bstrVal, bstrVal_mut);
    impl_conversions!(Ptr<IUnknown>, VT_UNKNOWN, punkVal, punkVal_mut);
    impl_conversions!(Ptr<IDispatch>, VT_DISPATCH,  pdispVal, pdispVal_mut);
    impl_conversions!(< S : SafeArrayElement> Vec<S>, *mut SAFEARRAY, VT_ARRAY, parray, parray_mut);
    impl_conversions!(<'s, S: SafeArrayElement> &'s [S], *mut SAFEARRAY, VT_ARRAY, parray, parray_mut);
    impl_conversions!(Box<VariantBool> => *mut VARIANT_BOOL, VT_PBOOL, pboolVal, pboolVal_mut);
    impl_conversions!(Box<u8>,  VT_PUI1, pbVal,   pbVal_mut);
    impl_conversions!(Box<i16>, VT_PI2,  piVal,   piVal_mut);
    impl_conversions!(Box<i32>, VT_PI4,  plVal,   plVal_mut);
    impl_conversions!(Box<i64>, VT_PI8,  pllVal,  pllVal_mut);
    impl_conversions!(Box<f32>, VT_PR4,  pfltVal, pfltVal_mut);
    impl_conversions!(Box<f64>, VT_PR8,  pdblVal, pdblVal_mut);
    impl_conversions!(Box<SCode> => *mut SCODE, VT_PERROR, pscode, pscode_mut);
    impl_conversions!(Box<Currency> => *mut CY, VT_PCY, pcyVal, pcyVal_mut);
    impl_conversions!(Box<Date> => *mut DATE, VT_PDATE, pdate, pdate_mut);
    impl_conversions!(Box<U16String> => *mut BSTR, VT_PBSTR, pbstrVal, pbstrVal_mut);
    impl_conversions!(Box<Ptr<IUnknown>> => *mut *mut IUnknown,  VT_PUNKNOWN, ppunkVal, ppunkVal_mut);
    impl_conversions!(Box<Ptr<IDispatch>> => *mut *mut IDispatch, VT_PDISPATCH, ppdispVal, ppdispVal_mut);
    impl_conversions!(< S : SafeArrayElement> Box<Vec<S>>, *mut *mut SAFEARRAY, VT_PARRAY, pparray, pparray_mut);
    impl_conversions!(<'s, S: SafeArrayElement> Box<&'s [S]>, *mut *mut SAFEARRAY, VT_PARRAY, pparray, pparray_mut);
    impl<D, T> VariantAccess for Variant<D, T>
    where
        D: VariantExt<T>
    {
        const VTYPE: u32 = VT_VARIANT;
        type Field = Ptr<VARIANT>;
        fn from_var(_n1: &VARIANT_n1, n3: &VARIANT_n3) -> Self::Field {
            unsafe {Ptr::with_checked(*n3.pvarVal()).unwrap()}
        }
        
        fn into_var(inner: Self::Field, _n1: &mut VARIANT_n1, n3: &mut VARIANT_n3) {            
            unsafe {
                let n_ptr = n3.pvarVal_mut();
                *n_ptr = inner.as_ptr();
            }
        }
    }
    impl_conversions!(Ptr<c_void>, VT_BYREF, byref, byref_mut); 
    impl_conversions!(i8, VT_I1, cVal, cVal_mut);
    impl_conversions!(u16, VT_UI2, uiVal, uiVal_mut);
    impl_conversions!(u32, VT_UI4, ulVal, ulVal_mut);
    impl_conversions!(u64, VT_UI8, ullVal, ullVal_mut);
    impl_conversions!(Int => i32, VT_INT, intVal, intVal_mut);
    impl_conversions!(UInt => u32, VT_UINT, uintVal, uintVal_mut);
    impl_conversions!(Box<DecWrapper> => *mut DECIMAL, VT_PDECIMAL, pdecVal, pdecVal_mut);
    impl_conversions!(Box<i8>,  VT_PI1,  pcVal, pcVal_mut);
    impl_conversions!(Box<u16>, VT_PUI2, puiVal, puiVal_mut);
    impl_conversions!(Box<u32>, VT_PUI4, pulVal, pulVal_mut);
    impl_conversions!(Box<u64>, VT_PUI8, pullVal, pullVal_mut);
    impl_conversions!(Box<Int> => *mut i32, VT_PINT, pintVal, pintVal_mut);
    impl_conversions!(Box<UInt> => *mut u32, VT_PUINT, puintVal, puintVal_mut);
    impl VariantAccess for DecWrapper {
        const VTYPE: u32 = VT_DECIMAL;
        type Field = DECIMAL;
        fn from_var(n1: &VARIANT_n1, _n3: &VARIANT_n3) -> Self::Field {
            unsafe {*n1.decVal()}
        }
        
        fn into_var(inner: Self::Field, n1: &mut VARIANT_n1, _n3: &mut VARIANT_n3) {
            unsafe {
                let n_ptr = n1.decVal_mut();
                *n_ptr = inner;
            }
        }
    }

    impl<'s> VariantAccess for &'s DecWrapper {
        const VTYPE: u32 = VT_DECIMAL;
        type Field = DECIMAL;
        fn from_var(n1: &VARIANT_n1, _n3: &VARIANT_n3) -> Self::Field {
            unsafe {*n1.decVal()}
        }
        
        fn into_var(inner: Self::Field, n1: &mut VARIANT_n1, _n3: &mut VARIANT_n3) {
            unsafe {
                let n_ptr = n1.decVal_mut();
                *n_ptr = inner;
            }
        }
    }

    impl<'s> VariantAccess for &'s mut DecWrapper {
        const VTYPE: u32 = VT_DECIMAL;
        type Field = DECIMAL;
        fn from_var(n1: &VARIANT_n1, _n3: &VARIANT_n3) -> Self::Field {
            unsafe {*n1.decVal()}
        }
        
        fn into_var(inner: Self::Field, n1: &mut VARIANT_n1, _n3: &mut VARIANT_n3) {
            unsafe {
                let n_ptr = n1.decVal_mut();
                *n_ptr = inner;
            }
        }
    }

    impl VariantAccess for VtEmpty {
        const VTYPE: u32 = VT_EMPTY;
        type Field = ();
        fn from_var(_n1: &VARIANT_n1, _n3: &VARIANT_n3) -> Self::Field {
            ()
        }
        fn into_var(_inner: Self::Field, _n1: &mut VARIANT_n1, _n3: &mut VARIANT_n3) 
        {}
    }

    impl VariantAccess for VtNull {
        const VTYPE: u32 = VT_NULL;
        type Field = ();
        fn from_var(_n1: &VARIANT_n1, _n3: &VARIANT_n3) -> Self::Field {
            ()
        }
        fn into_var(_inner: Self::Field, _n1: &mut VARIANT_n1, _n3: &mut VARIANT_n3) 
        {}
    }
    
}

/// Container for variant-compatible types. Wrap them with this 
/// so that the output VARIANT structure has vt == VT_VARIANT
/// and the data is a *mut VARIANT. 
/// 
/// ### Example
/// ```
/// extern crate oaidl;
/// 
/// use oaidl::{ConversionError, Variant, VariantExt};
/// 
/// fn main() -> Result<(), ConversionError> {
///     let val = 1337u16;
///     let val = Variant::wrap(val);
///     // convert into a Ptr<VARIANT> as per usual. 
///     Ok(())
/// }
/// ```
/// 
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Variant<D, T> 
where 
    D: VariantExt<T>
{
    inner: D, 
    _marker: PhantomData<T>
}

impl<D, T> Variant<D, T> 
where
    D: VariantExt<T> 
{
    /// Associated method to wrap a VariantExt compatible type `D` into a Variant
    pub fn wrap(d: D) -> Self {
        Variant{
            inner: d, 
            _marker: PhantomData
        }
    }

    /// Consumes self to return the inner data of type `D` where `D: VariantExt<T>`
    pub fn unwrap(self) -> D {
        self.inner
    }
}

impl<D, T> TryConvert<Ptr<VARIANT>, FromVariantError> for Variant<D, T> 
where
    D: VariantExt<T>
{
    /// Converts a [`Ptr<VARIANT>`] to a [`Variant<D, T>`] where D: [`VariantExt<T>`]
    fn try_convert(ptr: Ptr<VARIANT>) -> Result<Self, FromVariantError> {
        Ok(Variant::wrap(VariantExt::<T>::from_variant(ptr)?))
    }
}

impl<D, T> TryConvert<Variant<D, T>, IntoVariantError> for Ptr<VARIANT> 
where
    D: VariantExt<T>
{
    /// Converts a  [`Variant<D, T>`] to a [`Ptr<VARIANT>`] where D: [`VariantExt<T>`]
    /// This converts the value *inside* Variant into a Ptr<VARIANT> which is then stuffed
    /// inside a containing variant by the caller of the method.
    fn try_convert(v: Variant<D, T>) -> Result<Self, IntoVariantError> {
        let v = v.unwrap();
        Ok(VariantExt::<T>::into_variant(v)?)
    }
}

impl<D, T> TryConvert<Variant<D, T>, ElementError> for Ptr<VARIANT> 
where
    D: VariantExt<T>
{
    /// Converts a  [`Variant<D, T>`] to a [`Ptr<VARIANT>`] where D: [`VariantExt<T>`]
    /// This converts the value *inside* Variant into a Ptr<VARIANT> which is then stuffed
    /// inside a containing variant by the caller of the method.
    fn try_convert(v: Variant<D, T>) -> Result<Self, ElementError> {
        let v = v.unwrap();
        Ok(VariantExt::<T>::into_variant(v)?)
    }
}

impl<D, T> TryConvert<Ptr<VARIANT> , ElementError> for Variant<D, T> 
where
    D: VariantExt<T>
{
    /// Converts a [`Ptr<VARIANT>`] to a [`Variant<D, T>`] where D: [`VariantExt<T>`]
    fn try_convert(ptr: Ptr<VARIANT> ) -> Result<Self, ElementError> {
        Ok(Variant::wrap(VariantExt::<T>::from_variant(ptr)?))
    }
}

// Ensures the allocated memory is cleared correctly
struct VariantDestructor {
    inner: *mut VARIANT, 
    _marker: PhantomData<VARIANT>
}

impl VariantDestructor {
    fn new(p: *mut VARIANT) -> VariantDestructor {
        VariantDestructor {
            inner: p, 
            _marker: PhantomData
        }
    }
}

impl Drop for VariantDestructor {
    fn drop(&mut self) {
        if self.inner.is_null() {
            return;
        }
        unsafe { VariantClear(self.inner)};
        unsafe { mem::drop(*self.inner);}
        self.inner = null_mut();
    }
}

/// Trait implemented to convert the type into a VARIANT.
/// Do not implement this yourself without care. 
pub trait VariantExt<B>: Sized { 
    /// VARTYPE constant value for the type
    const VARTYPE: u32;

    /// Call this associated function on a [`Ptr<VARIANT>`] to obtain a value `T`
    fn from_variant(var: Ptr<VARIANT>) -> Result<Self, FromVariantError>;  

    /// Convert a value of type `T` into a [`Ptr<VARIANT>`]
    fn into_variant(value: Self) -> Result<Ptr<VARIANT>, IntoVariantError>;
}

/// Blanket implementation where TryConvert implementations exist between OutTy<==>InTy 
/// and a private trait is implemented on OutTy. 
impl<OutTy, InTy> VariantExt<InTy> for OutTy
where
    OutTy: TryConvert<InTy, FromVariantError> + self::private::VariantAccess<Field=InTy>,
    InTy: TryConvert<OutTy, IntoVariantError>
{
    const VARTYPE: u32 = OutTy::VTYPE;
    fn from_variant(pvar: Ptr<VARIANT>) -> Result<Self, FromVariantError> {
        let var = pvar.as_ptr();
        let _var_d = VariantDestructor::new(var);
        let mut n1 = unsafe {(*var).n1};
        let n3 = unsafe {n1.n2_mut().n3};
        let inner = OutTy::from_var(&n1, &n3);
        Ok(<OutTy as TryConvert<InTy, FromVariantError>>::try_convert(inner)?)
    }

    fn into_variant(value: OutTy) -> Result<Ptr<VARIANT>, IntoVariantError> {
        let mut n3: VARIANT_n3 = unsafe {mem::zeroed()};
        let mut n1: VARIANT_n1 = unsafe {mem::zeroed()};
        OutTy::into_var(InTy::try_convert(value)?, &mut n1, &mut n3);
        if OutTy::VARTYPE != VT_DECIMAL {
            let tv = __tagVARIANT { vt: OutTy::VARTYPE as u16, 
                            wReserved1: 0, 
                            wReserved2: 0, 
                            wReserved3: 0, 
                            n3: n3};
            unsafe {
                let n_ptr = n1.n2_mut();
                *n_ptr = tv;
            };
        }
        
        let var = Box::new(VARIANT{ n1: n1 });
        Ok(Ptr::with_checked(Box::into_raw(var)).unwrap())
    }
}

/// Helper type for VT_EMPTY variants
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VtEmpty;

impl TryConvert<(), FromVariantError> for VtEmpty {
    fn try_convert(_e: ()) -> Result<Self, FromVariantError> {
        Ok(VtEmpty)
    }
}

impl TryConvert<VtEmpty, IntoVariantError> for () {
    fn try_convert(_e: VtEmpty) -> Result<Self, IntoVariantError> {
        Ok(())
    }
}

/// Helper type for VT_NULL variants
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VtNull;

impl TryConvert<(), FromVariantError> for VtNull {
    fn try_convert(_e: ()) -> Result<Self, FromVariantError> {
        Ok(VtNull)
    }
}

impl TryConvert<VtNull, IntoVariantError> for () {
    fn try_convert(_e: VtNull) -> Result<Self, IntoVariantError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_decimal::Decimal;
    
    macro_rules! validate_variant {
        (@impl $t:ty, $val:expr, $vt:expr) => {
            let v = $val;
            let var: Ptr<VARIANT> = VariantExt::<_>::into_variant(v.clone()).unwrap();
            assert!(!var.as_ptr().is_null());
            unsafe {
                let pvar = var.as_ptr();
                let n1 = (*pvar).n1;
                let tv: &__tagVARIANT = n1.n2();
                assert_eq!(tv.vt as u32, $vt);
            };
            let var: $t = VariantExt::<_>::from_variant(var).unwrap();
            assert_eq!(v, var);
        };
        (Box<$t:ty>, $val:expr, $vt:expr) => {
            validate_variant!(@impl Box<$t>, $val, $vt);
        };
        ($b:ident, $val:expr, $vt:expr) => {
            validate_variant!(@impl $b, $val, $vt);
        };
    }
    #[test]
    fn test_i64() {
        validate_variant!(i64, 1337i64, VT_I8);
    }
    #[test]
    fn test_i32() {
        validate_variant!(i32, 1337i32, VT_I4);
    }
    #[test]
    fn test_u8() {
        validate_variant!(u8,  137u8, VT_UI1);
    }
        #[test]
    fn test_i16() {
        validate_variant!(i16, 1337i16, VT_I2);
    }
    #[test]
    fn test_f32() {
        validate_variant!(f32, 1337.9f32, VT_R4);
    }
    #[test]
    fn test_f64() {
        validate_variant!(f64, 1337.9f64, VT_R8);
    }
    #[test]
    fn test_bool_t() {
        validate_variant!(VariantBool, VariantBool::from(true), VT_BOOL);
    }
    #[test]
    fn test_bool_f() {
        validate_variant!(VariantBool, VariantBool::from(false), VT_BOOL);
    }
    #[test]
    fn test_scode() {
        validate_variant!(SCode, SCode::from(137), VT_ERROR);
    }
    #[test]
    fn test_cy() {
        validate_variant!(Currency, Currency::from(137), VT_CY);
    }
    #[test]
    fn test_date() {
        validate_variant!(Date, Date::from(137.7), VT_DATE);
    }
    #[test]
    fn test_str() {
        validate_variant!(U16String, U16String::from_str("testing abc1267 ?Ťũřǐꝥꞔ"), VT_BSTR);
    }
    #[test]
    fn test_box_u8() {
        type Bu8 = Box<u8>;
        validate_variant!(Bu8, Box::new(139), VT_PUI1);
    }
    #[test]
    fn test_box_i16() {
        type Bi16 = Box<i16>;
        validate_variant!(Bi16, Box::new(139), VT_PI2);
    }
    #[test]
    fn test_box_i32() {
        type Bi32 = Box<i32>;
        validate_variant!(Bi32, Box::new(139), VT_PI4);
    }
    #[test]
    fn test_box_i64() {
        type Bi64 = Box<i64>;
        validate_variant!(Bi64, Box::new(139), VT_PI8);
    }
    #[test]
    fn test_box_f32() {
        type Bf32 = Box<f32>;
        validate_variant!(Bf32, Box::new(1337.9f32), VT_PR4);
    }
    #[test]
    fn test_box_f64() {
        validate_variant!(Box<f64>, Box::new(1337.9f64), VT_PR8);
    }
    #[test]
    fn test_box_bool() {
        type BVb = Box<VariantBool>;
        validate_variant!(BVb, Box::new(VariantBool::from(true)), VT_PBOOL);
    }
    #[test]
    fn test_box_scode() {
        type BSCode = Box<SCode>;
        validate_variant!(BSCode, Box::new(SCode::from(-50)), VT_PERROR);
    }
    #[test]
    fn test_box_cy() {
        type BCy = Box<Currency>;
        validate_variant!(BCy, Box::new(Currency::from(137)), VT_PCY);
    }
    #[test]
    fn test_box_date() {
        type BDate = Box<Date>;
        validate_variant!(BDate, Box::new(Date::from(-10.333f64)), VT_PDATE);
    }
    #[test]
    fn test_i8() {
        validate_variant!(i8, -119i8, VT_I1);
    }
    #[test]
    fn test_u16() {
        validate_variant!(u16, 119u16, VT_UI2);
    }
    #[test]
    fn test_u32() {
        validate_variant!(u32, 11976u32, VT_UI4);
    }
    #[test]
    fn test_u64() {
        validate_variant!(u64, 11976u64, VT_UI8);
    }
    #[test]
    fn int_wrapper() {
        validate_variant!(Int, Int::from(13875), VT_INT);
    }
    #[test]
    fn uint_wrapper() {
        validate_variant!(UInt, UInt::from(13875), VT_UINT);
    }
    #[test]
    fn test_box_i8() {
        type Bi8 = Box<i8>;
        validate_variant!(Bi8, Box::new(-119i8), VT_PI1);
    }
    #[test]
    fn test_box_u16() {
        type Bu16 = Box<u16>;
        validate_variant!(Bu16, Box::new(119u16), VT_PUI2);
    }
    #[test]
    fn test_box_u32() {
        type Bu32 = Box<u32>;
        validate_variant!(Bu32, Box::new(11976u32), VT_PUI4);
    }
    #[test]
    fn test_box_u64() {
        validate_variant!(Box<u64>, Box::new(11976u64), VT_PUI8);
    }
    #[test]
    fn decimal() {
        validate_variant!(DecWrapper, DecWrapper::from(Decimal::new(2,2)), 0);
    }
    #[test]
    fn variant() {
        let b = 156u8;
        let c = Variant::wrap(b);
        let v = VariantExt::<_>::into_variant(c).unwrap();
        let v: Variant<u8, u8>  = VariantExt::<_>::from_variant(v).unwrap();
        let d = v.unwrap();
        assert_eq!(d, b);
    }
    #[test]
    fn empty() {
        validate_variant!(VtEmpty, VtEmpty, VT_EMPTY);
    }
    #[test]
    fn null() {
        validate_variant!(VtNull, VtNull, VT_NULL);
    }
    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Variant<i64, i64>>();
    }
    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Variant<i64, i64>>();
    }
}