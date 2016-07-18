use std::{ ops, fmt };
use util::grisu2;
use util::write::write;

#[derive(Copy, Clone, Debug)]
pub struct Number {
    positive: bool,
    mantissa: u64,
    exponent: i16,
}

impl Number {
    #[inline]
    pub fn from_parts(positive: bool, mantissa: u64, exponent: i16) -> Self {
        Number {
            positive: positive,
            mantissa: mantissa,
            exponent: exponent,
        }
    }

    #[inline]
    pub fn as_parts(&self) -> (bool, u64, i16) {
        (self.positive, self.mantissa, self.exponent)
    }

    #[inline]
    pub fn is_sign_positive(&self) -> bool {
        self.positive
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.mantissa == 0
    }
}

impl PartialEq for Number {
    #[inline]
    fn eq(&self, other: &Number) -> bool {
        if self.is_zero() && other.is_zero() {
            return true;
        }

        if self.positive != other.positive {
            return false;
        }

        let e_diff = self.exponent - other.exponent;

        if e_diff == 0 {
            return self.mantissa == other.mantissa;
        } else if e_diff > 0 {
            // TODO: use cached powers
            self.mantissa
                .wrapping_mul(10u64.pow(e_diff as u32)) == other.mantissa
        } else {
            // TODO: use cached powers
            self.mantissa == other.mantissa
                                  .wrapping_mul(10u64.pow(-e_diff as u32))
        }

    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = Vec::new();

        let (positive, mantissa, exponent) = self.as_parts();
        write(&mut buf, positive, mantissa, exponent).unwrap();
        f.write_str(&unsafe { String::from_utf8_unchecked(buf) })
    }
}

fn exponent_to_power_f64(e: i16) -> f64 {
    static POS_POWERS: [f64; 23] = [
          1.0,    1e1,    1e2,    1e3,    1e4,    1e5,    1e6,    1e7,
          1e8,    1e9,   1e10,   1e11,   1e12,   1e13,   1e14,   1e15,
         1e16,   1e17,   1e18,   1e19,   1e20,   1e21,   1e22
    ];

    static NEG_POWERS: [f64; 23] = [
          1.0,   1e-1,   1e-2,   1e-3,   1e-4,   1e-5,   1e-6,   1e-7,
         1e-8,   1e-9,  1e-10,  1e-11,  1e-12,  1e-13,  1e-14,  1e-15,
        1e-16,  1e-17,  1e-18,  1e-19,  1e-20,  1e-21,  1e-22
    ];

    let index = e.abs() as usize;

    if index < 23 {
        if e < 0 {
            NEG_POWERS[index]
        } else {
            POS_POWERS[index]
        }
    } else {
        // powf is more accurate
        10f64.powf(e as f64)
    }
}

fn exponent_to_power_f32(e: i16) -> f32 {
    static POS_POWERS: [f32; 16] = [
          1.0,    1e1,    1e2,    1e3,    1e4,    1e5,    1e6,    1e7,
          1e8,    1e9,   1e10,   1e11,   1e12,   1e13,   1e14,   1e15
    ];

    static NEG_POWERS: [f32; 16] = [
          1.0,   1e-1,   1e-2,   1e-3,   1e-4,   1e-5,   1e-6,   1e-7,
         1e-8,   1e-9,  1e-10,  1e-11,  1e-12,  1e-13,  1e-14,  1e-15
    ];

    let index = e.abs() as usize;

    if index < 16 {
        if e < 0 {
            NEG_POWERS[index]
        } else {
            POS_POWERS[index]
        }
    } else {
        // powf is more accurate
        10f32.powf(e as f32)
    }
}

impl From<Number> for f64 {
    fn from(num: Number) -> f64 {
        let mut n = num.mantissa as f64;
        let mut e = num.exponent;

        if e < -308 {
            n *= exponent_to_power_f64(e + 308);
            e = -308;
        }

        let f = n * exponent_to_power_f64(e);
        if num.positive { f } else { -f }
    }
}

impl From<Number> for f32 {
    fn from(num: Number) -> f32 {
        let mut n = num.mantissa as f32;
        let mut e = num.exponent;

        if e < -127 {
            n *= exponent_to_power_f32(e + 127);
            e = -127;
        }

        let f = n * exponent_to_power_f32(e);
        if num.positive { f } else { -f }
    }
}

impl From<f64> for Number {
    fn from(float: f64) -> Number {
        if !float.is_sign_positive() {
            let (mantissa, exponent) = grisu2::convert(-float);

            Number::from_parts(false, mantissa, exponent)
        } else {
            let (mantissa, exponent) = grisu2::convert(float);

            Number::from_parts(true, mantissa, exponent)
        }
    }
}

impl From<f32> for Number {
    fn from(float: f32) -> Number {
        if !float.is_sign_positive() {
            let (mantissa, exponent) = grisu2::convert(-float as f64);

            Number::from_parts(false, mantissa, exponent)
        } else {
            let (mantissa, exponent) = grisu2::convert(float as f64);

            Number::from_parts(true, mantissa, exponent)
        }
    }
}

impl PartialEq<f64> for Number {
    fn eq(&self, other: &f64) -> bool {
        f64::from(*self) == *other
    }
}

impl PartialEq<f32> for Number {
    fn eq(&self, other: &f32) -> bool {
        f32::from(*self) == *other
    }
}

impl PartialEq<Number> for f64 {
    fn eq(&self, other: &Number) -> bool {
        f64::from(*other) == *self
    }
}

impl PartialEq<Number> for f32 {
    fn eq(&self, other: &Number) -> bool {
        f32::from(*other) == *self
    }
}

macro_rules! impl_unsigned {
    ($( $t:ty ),*) => ($(
        impl From<$t> for Number {
            fn from(num: $t) -> Number {
                Number {
                    positive: true,
                    mantissa: num as u64,
                    exponent: 0,
                }
            }
        }

        impl_integer!($t);
    )*)
}


macro_rules! impl_signed {
    ($( $t:ty ),*) => ($(
        impl From<$t> for Number {
            fn from(num: $t) -> Number {
                if num < 0 {
                    Number {
                        positive: false,
                        mantissa: -num as u64,
                        exponent: 0,
                    }
                } else {
                    Number {
                        positive: true,
                        mantissa: num as u64,
                        exponent: 0,
                    }
                }
            }
        }

        impl_integer!($t);
    )*)
}


macro_rules! impl_integer {
    ($t:ty) => {
        impl From<Number> for $t {
            fn from(num: Number) -> $t {
                let (positive, mantissa, exponent) = num.as_parts();

                if exponent <= 0 {
                    if positive {
                        mantissa as $t
                    } else {
                        -(mantissa as i64) as $t
                    }
                } else {
                    // This may overflow, which is fine
                    if positive {
                        (mantissa * 10u64.pow(exponent as u32)) as $t
                    } else {
                        (-(mantissa as i64) * 10i64.pow(exponent as u32)) as $t
                    }
                }
            }
        }

        impl PartialEq<$t> for Number {
            fn eq(&self, other: &$t) -> bool {
                *self == Number::from(*other)
            }
        }

        impl PartialEq<Number> for $t {
            fn eq(&self, other: &Number) -> bool {
                Number::from(*self) == *other
            }
        }
    }
}

impl_signed!(isize, i8, i16, i32, i64);
impl_unsigned!(usize, u8, u16, u32, u64);

impl ops::Neg for Number {
    type Output = Number;

    #[inline]
    fn neg(self) -> Number {
        Number {
            positive: !self.positive,
            mantissa: self.mantissa,
            exponent: self.exponent,
        }
    }
}

impl ops::Mul for Number {
    type Output = Number;

    #[inline]
    fn mul(self, other: Number) -> Number {
        Number {
            positive: self.positive && other.positive,
            mantissa: self.mantissa * other.mantissa,
            exponent: self.exponent + other.exponent,
        }
    }
}

impl ops::MulAssign for Number {
    #[inline]
    fn mul_assign(&mut self, other: Number) {
        *self = *self * other;
    }
}
