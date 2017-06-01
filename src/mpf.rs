use libc::{c_double, c_int, c_long, c_ulong, c_void,c_char};
use std::mem::uninitialized;
use std::cmp;
use std::cmp::Ordering::{self, Greater, Less, Equal};
use std::ops::{Div, DivAssign, Rem, RemAssign, Mul, MulAssign, Add, AddAssign, Sub, SubAssign, Neg};
use std::ffi::CString;
use std::string::String;
use std::error::Error;
use std::fmt;
use std::i32;
use super::mpz::mp_bitcnt_t;
use super::mpz::{Mpz, mpz_srcptr};
use super::mpq::{Mpq, mpq_srcptr};
use super::sign::Sign;
use num_traits::{Zero, One, Num, Signed};

type mp_exp_t = c_long;

#[repr(C)]
pub struct mpf_struct {
    _mp_prec: c_int,
    _mp_size: c_int,
    _mp_exp: mp_exp_t,
    _mp_d: *mut c_void
}

pub type mpf_srcptr = *const mpf_struct;
pub type mpf_ptr = *mut mpf_struct;

#[link(name = "gmp")]
extern "C" {
    fn __gmpf_init2(x: mpf_ptr, prec: mp_bitcnt_t);
    fn __gmpf_init_set(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_clear(x: mpf_ptr);
    fn __gmpf_get_prec(op: mpf_srcptr) -> mp_bitcnt_t;
    fn __gmpf_set_prec(rop: mpf_ptr, prec: mp_bitcnt_t);
    fn __gmpf_set(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_set_z(rop: mpf_ptr, op: mpz_srcptr);
    fn __gmpf_set_q(rop: mpf_ptr, op: mpq_srcptr);

    fn __gmpf_set_str(rop: mpf_ptr, str: *const c_char, base: c_int) -> c_int;
    fn __gmpf_set_si(rop: mpf_ptr, op: c_long);
    fn __gmpf_get_str(str: *const c_char, expptr: *const mp_exp_t, base: i32, n_digits: i32, op: mpf_ptr) -> *mut c_char;

    fn __gmpf_cmp(op1: mpf_srcptr, op2: mpf_srcptr) -> c_int;
    fn __gmpf_cmp_d(op1: mpf_srcptr, op2: c_double) -> c_int;
    fn __gmpf_cmp_ui(op1: mpf_srcptr, op2: c_ulong) -> c_int;
    fn __gmpf_cmp_si(op1: mpf_srcptr, op2: c_long) -> c_int;
    fn __gmpf_reldiff(rop: mpf_ptr, op1: mpf_srcptr, op2: mpf_srcptr);
    fn __gmpf_add(rop: mpf_ptr, op1: mpf_srcptr, op2: mpf_srcptr);
    fn __gmpf_sub(rop: mpf_ptr, op1: mpf_srcptr, op2: mpf_srcptr);
    fn __gmpf_mul(rop: mpf_ptr, op1: mpf_srcptr, op2: mpf_srcptr);
    fn __gmpf_div(rop: mpf_ptr, op1: mpf_srcptr, op2: mpf_srcptr);
    fn __gmpf_neg(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_abs(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_ceil(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_floor(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_trunc(rop: mpf_ptr, op: mpf_srcptr);
    fn __gmpf_sqrt(rop: mpf_ptr, op: mpf_srcptr);
}

pub struct Mpf {
    mpf: mpf_struct,
}

unsafe impl Send for Mpf { }
unsafe impl Sync for Mpf { }

impl Drop for Mpf {
    fn drop(&mut self) { unsafe { __gmpf_clear(&mut self.mpf) } }
}

impl Mpf {
    pub unsafe fn inner(&self) -> mpf_srcptr {
        &self.mpf
    }

    pub unsafe fn inner_mut(&mut self) -> mpf_ptr {
        &mut self.mpf
    }

    pub fn zero() -> Mpf { Mpf::new(32) }

    pub fn new(precision: usize) -> Mpf {
        unsafe {
            let mut mpf = uninitialized();
            __gmpf_init2(&mut mpf, precision as c_ulong);
            Mpf { mpf: mpf }
        }
    }

    pub fn set(&mut self, other: &Mpf) {
        unsafe { __gmpf_set(&mut self.mpf, &other.mpf) }
    }

    pub fn set_z(&mut self, other: &Mpz) {
        unsafe { __gmpf_set_z(&mut self.mpf, other.inner()) }
    }

    pub fn set_q(&mut self, other: &Mpq) {
        unsafe { __gmpf_set_q(&mut self.mpf, other.inner()) }
    }

    pub fn get_prec(&self) -> usize {
        unsafe { __gmpf_get_prec(&self.mpf) as usize }
    }

    pub fn set_prec(&mut self, precision: usize) {
        unsafe { __gmpf_set_prec(&mut self.mpf, precision as c_ulong) }
    }

    pub fn set_from_str(&mut self, string: &str, base: i32) -> Result<(), ParseMpfError> {
        let c_str = CString::new(string).map_err(|_| ParseMpfError { _priv: () })?;
        unsafe {
            let r = __gmpf_set_str(&mut self.mpf, c_str.as_ptr(), base as c_int);
            if r == 0 {
                Ok(())
            } else {
                Err(ParseMpfError { _priv: () })
            }
        }
    }

    pub fn set_from_si(&mut self, int: i64){
        unsafe{
            __gmpf_set_si(&mut self.mpf,int as c_long);
        }
    }

    pub fn get_str(&mut self, n_digits: i32, base: i32, exp: &mut c_long) -> String{
        let c_str = CString::new("").unwrap();
        let out;
        unsafe{
            out = CString::from_raw(__gmpf_get_str(c_str.into_raw(), exp, base, n_digits, &mut self.mpf));
        }
        out.to_str().unwrap().to_string()
    }

    pub fn abs(&self) -> Mpf {
        unsafe {
            let mut res = Mpf::new(self.get_prec());
            __gmpf_abs(&mut res.mpf, &self.mpf);
            res
        }
    }

    pub fn ceil(&self) -> Mpf {
        unsafe {
            let mut res = Mpf::new(self.get_prec());
            __gmpf_ceil(&mut res.mpf, &self.mpf);
            res
        }
    }

    pub fn floor(&self) -> Mpf {
        unsafe {
            let mut res = Mpf::new(self.get_prec());
            __gmpf_floor(&mut res.mpf, &self.mpf);
            res
        }
    }

    pub fn trunc(&self) -> Mpf {
        unsafe {
            let mut res = Mpf::new(self.get_prec());
            __gmpf_trunc(&mut res.mpf, &self.mpf);
            res
        }
    }

    pub fn reldiff(&self, other: &Mpf) -> Mpf {
        unsafe {
            let mut res = Mpf::new(cmp::max(self.get_prec(), other.get_prec()));
            __gmpf_reldiff(&mut res.mpf, &self.mpf, &other.mpf);
            res
        }
    }

    pub fn sqrt(self) -> Mpf {
        let mut retval:Mpf;
        unsafe {
            retval = Mpf::new(__gmpf_get_prec(&self.mpf) as usize);
            retval.set_from_si(0);
            if __gmpf_cmp_si(&self.mpf, 0) > 0 {
                __gmpf_sqrt(&mut retval.mpf, &self.mpf);
            } else {
                panic!("Square root of negative/zero");
            }
        }
        retval
    }

    pub fn sign(&self) -> Sign {
        let size = self.mpf._mp_size;
        if size == 0 {
            Sign::Zero
        } else if size > 0 {
            Sign::Positive
        } else {
            Sign::Negative
        }
    }
}

#[derive(Debug)]
pub struct ParseMpfError {
    _priv: ()
}

impl fmt::Display for ParseMpfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for ParseMpfError {
    fn description(&self) -> &'static str {
        "invalid rational number"
    }

    fn cause(&self) -> Option<&'static Error> {
        None
    }
}

impl Clone for Mpf {
    fn clone(&self) -> Mpf {
        unsafe {
            let mut mpf = uninitialized();
            __gmpf_init_set(&mut mpf, &self.mpf);
            Mpf { mpf: mpf }
        }
    }
}

impl Eq for Mpf { }
impl PartialEq for Mpf {
    fn eq(&self, other: &Mpf) -> bool {
        unsafe { __gmpf_cmp(&self.mpf, &other.mpf) == 0 }
    }
}

impl Ord for Mpf {
    fn cmp(&self, other: &Mpf) -> Ordering {
        let cmp = unsafe { __gmpf_cmp(&self.mpf, &other.mpf) };
        if cmp == 0 {
            Equal
        } else if cmp > 0 {
            Greater
        } else {
            Less
        }
    }
}

impl PartialOrd for Mpf {
    fn partial_cmp(&self, other: &Mpf) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

macro_rules! div_guard {
    (Div, $is_zero: expr) => {
        if $is_zero {
            panic!("divide by zero")
        }
    };
    ($tr: ident, $what: expr) => {}
}

macro_rules! impl_oper {
    ($tr: ident, $meth: ident, $tr_assign: ident, $meth_assign: ident, $fun: ident) => {
        impl<'a> $tr<Mpf> for &'a Mpf {
            type Output = Mpf;
            #[inline]
            fn $meth(self, other: Mpf) -> Mpf {
                self.$meth(&other)
            }
        }

        impl<'a> $tr<&'a Mpf> for Mpf {
            type Output = Mpf;
            #[inline]
            fn $meth(mut self, other: &Mpf) -> Mpf {
                self.$meth_assign(other);
                self
            }
        }

        impl $tr<Mpf> for Mpf {
            type Output = Mpf;
            #[inline]
            fn $meth(self, other: Mpf) -> Mpf {
                self.$meth(&other)
            }
        }

        impl<'a, 'b> $tr<&'a Mpf> for &'b Mpf {
            type Output = Mpf;
            fn $meth(self, other: &Mpf) -> Mpf {
                unsafe {
                    div_guard!($tr, __gmpf_cmp_ui(&other.mpf, 0) == 0);
                    let mut res = Mpf::new(cmp::max(self.get_prec(), other.get_prec()));
                    $fun(&mut res.mpf, &self.mpf, &other.mpf);
                    res
                }
            }
        }

        impl<'a> $tr_assign<Mpf> for Mpf {
            #[inline]
            fn $meth_assign(&mut self, other: Mpf) {
                self.$meth_assign(&other)
            }
        }

        impl<'a> $tr_assign<&'a Mpf> for Mpf {
            #[inline]
            fn $meth_assign(&mut self, other: &Mpf) {
                unsafe {
                    div_guard!($tr, __gmpf_cmp_ui(&other.mpf, 0) == 0);
                    $fun(&mut self.mpf, &self.mpf, &other.mpf)
                }
            }
        }
    }
}

impl_oper!(Add, add, AddAssign, add_assign, __gmpf_add);
impl_oper!(Sub, sub, SubAssign, sub_assign, __gmpf_sub);
impl_oper!(Mul, mul, MulAssign, mul_assign, __gmpf_mul);
impl_oper!(Div, div, DivAssign, div_assign, __gmpf_div);


impl Rem<Mpf> for Mpf {
    type Output = Mpf;
    #[inline]
    fn rem(self, other: Mpf) -> Mpf {
        self.rem(&other)
    }
}

impl<'a> Rem<&'a Mpf> for Mpf {
    type Output = Mpf;
    #[inline]
    fn rem(mut self, other: &Mpf) -> Mpf {
        self.rem_assign(other);
        self
    }
}

impl<'a> Rem<Mpf> for &'a Mpf {
    type Output = Mpf;
    #[inline]
    fn rem(self, other: Mpf) -> Mpf {
        self.rem(&other)
    }
}

impl<'a, 'b> Rem<&'a Mpf> for &'b Mpf {
    type Output = Mpf;
    #[inline]
    fn rem(self, other: &Mpf) -> Mpf {
        self.clone().rem(other)
    }
}

impl<'a> RemAssign<Mpf> for Mpf {
    #[inline]
    fn rem_assign(&mut self, other: Mpf) {
        self.rem_assign(&other)
    }
}

impl<'a> RemAssign<&'a Mpf> for Mpf {
    fn rem_assign(&mut self, other: &Mpf) {
        *self -= other * (&*self / other).floor();
    }
}


impl<'b> Neg for &'b Mpf {
    type Output = Mpf;
    fn neg(self) -> Mpf {
        unsafe {
            let mut res = Mpf::new(self.get_prec());
            __gmpf_neg(&mut res.mpf, &self.mpf);
            res
        }
    }
}

impl Neg for Mpf {
    type Output = Mpf;
    #[inline]
    fn neg(mut self) -> Mpf {
        unsafe {
            __gmpf_neg(&mut self.mpf, &self.mpf);
            self
        }
    }
}

impl Zero for Mpf {
    #[inline]
    fn zero() -> Mpf {
        Mpf::zero()
    }

    #[inline]
    fn is_zero(&self) -> bool {
        unsafe {
            __gmpf_cmp_ui(&self.mpf, 0) == 0
        }
    }
}

impl One for Mpf {
    #[inline]
    fn one() -> Mpf {
        let mut res = Mpf::new(32);
        res.set_from_si(1);
        res
    }
}

impl Num for Mpf {
    type FromStrRadixErr = ParseMpfError;
    fn from_str_radix(str: &str, radix: u32) -> Result<Mpf, ParseMpfError> {
        assert!(radix <= i32::MAX as u32);
        let mut res = Mpf::new(32);
        res.set_from_str(str, radix as i32)?;
        Ok(res)
    }
}

impl Signed for Mpf {
    fn abs(&self) -> Mpf {
        self.abs()
    }

    fn abs_sub(&self, other: &Mpf) -> Mpf {
        let mut res = self - other;
        unsafe {
            __gmpf_abs(&mut res.mpf, &res.mpf);
        }
        res
    }

    fn signum(&self) -> Mpf {
        let mut res = Mpf::new(self.get_prec());
        res.set_from_si(unsafe { __gmpf_cmp_ui(&self.mpf, 0) } as i64);
        res
    }

    fn is_positive(&self) -> bool {
        unsafe { __gmpf_cmp_ui(&self.mpf, 0) > 0 }
    }

    fn is_negative(&self) -> bool {
        unsafe { __gmpf_cmp_ui(&self.mpf, 0) < 0 }
    }
}
