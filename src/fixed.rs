use std::ops::{Add, Div, Mul, Sub};

const FIXED_DIGITS: u64 = 6;
pub const FIXED_ONE: u64 = u64::pow(10, FIXED_DIGITS as u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Fixed {
    ammount: u64,
}

impl Fixed {
    fn fractional(&self) -> u64 {
        return self.ammount - (self.ammount / FIXED_ONE) * FIXED_ONE;
    }

    fn mul_precision() -> u64 {
        // sqrt of fixed one
        1000
    }

    fn max_fixed_divisor() -> u64 {
        FIXED_ONE * FIXED_ONE
    }

    fn reciprocal(self) -> Self {
        assert!(self.ammount != 0);
        let ammount = (FIXED_ONE * FIXED_ONE) / self.ammount; // Can't overflow
        Self { ammount }
    }

    // Helper function to create new Fixed numbers
    pub fn new(number: f64) -> Self {
        let ammount = (number * FIXED_ONE as f64) as u64;
        Self { ammount }
    }
}

impl Add for Fixed {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let ammount = self.ammount + rhs.ammount;
        Self { ammount }
    }
}

impl Sub for Fixed {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        dbg!(self.ammount);
        dbg!(rhs.ammount);
        let ammount = self.ammount - rhs.ammount;
        Self { ammount }
    }
}

impl Mul for Fixed {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.ammount == 0 || rhs.ammount == 0 {
            return Self { ammount: 0 };
        }

        if self.ammount == FIXED_ONE {
            return Self {
                ammount: rhs.ammount,
            };
        }

        if rhs.ammount == FIXED_ONE {
            return Self {
                ammount: self.ammount,
            };
        }

        // Separate into integer and fractional parts
        // x = x1 + x2, y = y1 + y2
        let x1 = self.ammount / FIXED_ONE;
        let mut x2 = self.fractional();
        let y1 = rhs.ammount / FIXED_ONE;
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

        x2 = x2 / Fixed::mul_precision();
        y2 = y2 / Fixed::mul_precision();
        let x2y2 = x2 * y2;
        if x2 != 0 {
            assert!(x2y2 / x2 == y2); // Overflow x2y2
        }

        // result = fixed1() * x1 * y1 + x1 * y2 + x2 * y1 + x2 * y2 / fixed1();
        let mut result = x1y1;
        result = result + x2y1; // Add checks for overflow
        result = result + x1y2; // Add checks for overflow
        result = result + x2y2; // Add checks for overflow
        return Self { ammount: result };
    }
}

impl Div for Fixed {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.ammount == FIXED_ONE {
            return self;
        }

        assert!(rhs.ammount != 0);
        assert!(rhs.ammount <= Fixed::max_fixed_divisor());
        return self * rhs.reciprocal();
    }
}
