use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};

const FIXED_DIGITS: u32 = 10;
pub const FIXED_ONE: i128 = i128::pow(10, FIXED_DIGITS);
pub const FIXED_ONE_U64: u64 = u64::pow(10, FIXED_DIGITS);

#[derive(PartialEq, Eq, Clone, Copy, PartialOrd)]
pub struct Fixed(pub i128);

impl Debug for Fixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fixed")
            .field("value", &(self.0))
            .field("value_as_f64", &(self.0 as f64 / FIXED_ONE as f64))
            .finish()
    }
}

impl Fixed {
    const ONE: Fixed = Fixed(FIXED_ONE);
    fn fractional(&self) -> i128 {
        self.0 - (self.0 / FIXED_ONE) * FIXED_ONE
    }

    fn mul_precision() -> i128 {
        // sqrt of fixed one
        100000
    }

    fn max_fixed_divisor() -> i128 {
        FIXED_ONE * FIXED_ONE
    }

    fn max_fixed_div() -> i128 {
        i128::MAX / FIXED_ONE
    }

    fn reciprocal(self) -> Self {
        assert!(self.0 != 0);
        let z = (FIXED_ONE * FIXED_ONE) / self.0; // Can't overflow
        Self(z)
    }
}

impl From<u64> for Fixed {
    fn from(value: u64) -> Self {
        Fixed(value.into())
    }
}

impl Add for Fixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let z = i128_add(self.0, rhs.0);
        Self(z)
    }
}

impl Sub for Fixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let z = self + -rhs; // self + (-rhs)
        Self(z.0)
    }
}

impl Mul for Fixed {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.0 == 0 || rhs.0 == 0 {
            return Self(0);
        }

        if self.0 == FIXED_ONE {
            return Self(rhs.0);
        }

        if rhs.0 == FIXED_ONE {
            return Self(self.0);
        }

        // Separate into integer and fractional parts
        // x = x1 + x2, y = y1 + y2
        let x1 = self.0 / FIXED_ONE;
        let mut x2 = self.fractional();
        let y1 = rhs.0 / FIXED_ONE;
        let mut y2 = rhs.fractional();

        // (x1 + x2) * (y1 + y2) = (x1 * y1) + (x1 * y2) + (x2 * y1) + (x2 * y2)
        let mut x1y1 = x1 * y1;
        if x1 != 0 {
            assert!(x1y1 / x1 == y1); // Overflow x1y1
        }

        // x1y1 needs to be multiplied back by fixed1
        // solium-disable-next-line mixedcase
        let fixed_x1y1 = x1y1 * FIXED_ONE;
        if x1y1 != 0 {
            assert!(fixed_x1y1 / x1y1 == FIXED_ONE); // Overflow x1y1 * fixed1
        }
        x1y1 = fixed_x1y1;

        let x2y1 = x2 * y1;
        if x2 != 0 {
            assert!(x2y1 / x2 == y1); // Overflow x2y1
        }

        let x1y2 = x1 * y2;
        if x1 != 0 {
            assert!(x1y2 / x1 == y2); // Overflow x1y2
        }

        x2 /= Fixed::mul_precision();
        y2 /= Fixed::mul_precision();
        let x2y2 = x2 * y2;
        if x2 != 0 {
            assert!(x2y2 / x2 == y2); // Overflow x2y2
        }

        // result = fixed1() * x1 * y1 + x1 * y2 + x2 * y1 + x2 * y2 / fixed1();
        let mut result = x1y1;
        result = i128_add(result, x2y1);
        result = i128_add(result, x1y2);
        result = i128_add(result, x2y2);
        Self(result)
    }
}

impl Div for Fixed {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == FIXED_ONE {
            return self;
        }
        if self.0 == rhs.0 {
            return Fixed::ONE;
        }

        assert!(rhs.0 != 0);
        assert!(rhs.0 <= Fixed::max_fixed_divisor());
        self * rhs.reciprocal()
    }
}

impl Neg for Fixed {
    type Output = Fixed;

    fn neg(self) -> Self::Output {
        Fixed(-self.0)
    }
}

fn i128_add(x: i128, y: i128) -> i128 {
    let z = x + y;

    if x > 0 && y > 0 {
        assert!(z > x && z > y);
    }
    if x < 0 && y < 0 {
        assert!(z < x && z < y);
    }

    z
}

#[cfg(test)]
mod tests {
    use crate::fixed::{FIXED_DIGITS, FIXED_ONE};

    use super::Fixed;

    #[test]
    fn multiply() {
        let zero = Fixed(0);
        let one = Fixed::ONE;
        let two = Fixed(2 * FIXED_ONE);
        let half = Fixed(FIXED_ONE / 2);

        assert_eq!(zero * zero, zero);
        assert_eq!(two * -two, Fixed(-4 * FIXED_ONE));
        assert_eq!(two * (two + half), Fixed(5 * FIXED_ONE));
        assert_eq!(two * -(two + half), Fixed(-5 * FIXED_ONE));
        assert_eq!(half * -half, Fixed(-FIXED_ONE / 4));
        assert_eq!(
            Fixed(one.0 / Fixed::mul_precision()) * Fixed(one.0 * Fixed::mul_precision()),
            one
        );
    }

    #[test]
    fn div() {
        let x = Fixed(1099991000000);

        assert_eq!(Fixed::ONE, x / x);

        //* Test divide(maxFixedDiv(),1) = maxFixedDiv()*(10^digits())
        let x = Fixed(Fixed::max_fixed_div());
        let y = Fixed(1);
        assert_eq!(
            x / y,
            Fixed(Fixed::max_fixed_div() * 10_i128.pow(FIXED_DIGITS)),
        );

        let x = Fixed(Fixed::max_fixed_divisor());
        assert_eq!(x / x, Fixed::ONE);
    }

    #[test]
    #[should_panic]
    fn fail_to_divide_by_zero() {
        assert_eq!(Fixed::ONE, Fixed(0));
    }

    #[test]
    #[should_panic]
    fn fail_to_divide_by_greater_number_than_max_fixed_divisor() {
        assert_eq!(Fixed(Fixed::max_fixed_div() + 1), Fixed(1));
    }
}
