use derive_more::{Add, AddAssign, SubAssign};
use num_traits::{Num, PrimInt, Signed, Zero};
use std::{error::Error, fmt::Debug, fmt::Display, str::FromStr};
use thiserror::Error;

#[derive(Debug, Add, AddAssign, SubAssign, PartialEq, PartialOrd, Copy, Clone)]
pub struct DecimalFixedPoint<T: PrimInt + Signed, const N: u32>(T);

impl<T: PrimInt + Signed, const N: u32> DecimalFixedPoint<T, N> {
    fn exponent() -> T {
        T::from(10).unwrap().pow(N)
    }
}

impl<T: PrimInt + Signed, const N: u32> Zero for DecimalFixedPoint<T, N> {
    fn zero() -> Self {
        Self(T::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T: PrimInt + Signed + Display, const N: u32> Display for DecimalFixedPoint<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}.{:0>precision$}",
            self.0.is_negative().then(|| "-").unwrap_or(""),
            (self.0 / Self::exponent()).abs(),
            (self.0 % Self::exponent()).abs(),
            precision = N as usize
        )
    }
}

impl<T: PrimInt + Signed + Debug, const N: u32> FromStr for DecimalFixedPoint<T, N>
where
    <T as num_traits::Num>::FromStrRadixErr: Debug + Error + 'static,
{
    type Err = BigFixedPointParseError<T>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BigFixedPointParseError::*;
        let mut parts = s.split('.');
        let integer_part = parts.next().ok_or_else(|| InvalidFormat)?;
        let decimal_part = parts.next().unwrap_or_default();
        if parts.next().is_some() {
            return Err(InvalidFormat);
        }
        let requested_precision = decimal_part.len();
        if requested_precision > N as usize {
            return Err(PrecisionExceeded {
                precision: N as usize,
                requested_precision,
            });
        }
        let mut string_rep = integer_part.to_owned();
        string_rep.push_str(decimal_part);
        string_rep.push_str(&"0".repeat(N as usize - requested_precision));
        T::from_str_radix(&string_rep, 10)
            .map_err(InvalidNumberFormat)
            .map(|inner| Self(inner))
    }
}

#[derive(Error, Debug)]
pub enum BigFixedPointParseError<T: Num + Debug>
where
    <T as num_traits::Num>::FromStrRadixErr: Debug + Error + 'static,
{
    #[error("exceeded precision while parsing: supported precision {precision}, parsed precision {requested_precision}")]
    PrecisionExceeded {
        precision: usize,
        requested_precision: usize,
    },
    #[error("invalid BigFixedPoint format")]
    InvalidFormat,
    #[error("invaid number format")]
    InvalidNumberFormat(#[source] T::FromStrRadixErr),
}

#[cfg(test)]
mod tests {
    use super::*;

    type Bits128Precision4 = DecimalFixedPoint<i128, 4>;
    //type Bits64Precision4 = DecimalFixedPoint<i64, 4>;

    #[test]
    fn from_to_string() {
        let num = Bits128Precision4::from_str("792281625142643375935.0335").unwrap();
        assert_eq!("792281625142643375935.0335", num.to_string());
    }

    #[test]
    fn negative_from_string() {
        let num = Bits128Precision4::from_str("-792281625142643375935.0335").unwrap();
        assert_eq!(
            "-792281625142643375935.0335",
            num.to_string(),
            "testing negative {:?}",
            num
        );
    }

    #[test]
    fn small_from_string() {
        let num = Bits128Precision4::from_str("-0.23").unwrap();
        assert_eq!(
            "-0.2300",
            num.to_string(),
            "testing small negative {:?}",
            num
        );
    }
}
