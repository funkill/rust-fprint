use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Finger {
    LeftThumb = 1,
    LeftIndex = 2,
    LeftMiddle = 3,
    LeftRing = 4,
    LeftLittle = 5,
    RightThumb = 6,
    RightIndex = 7,
    RightMiddle = 8,
    RightRing = 9,
    RightLittle = 10,
}

impl fmt::Display for Finger {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let string = match self {
            Finger::LeftThumb => "LeftThumb",
            Finger::LeftIndex => "LeftIndex",
            Finger::LeftMiddle => "LeftMiddle",
            Finger::LeftRing => "LeftRing",
            Finger::LeftLittle => "LeftLittle",
            Finger::RightThumb => "RightThumb",
            Finger::RightIndex => "RightIndex",
            Finger::RightMiddle => "RightMiddle",
            Finger::RightRing => "RightRing",
            Finger::RightLittle => "RightLittle",
        };

        write!(f, "{}", string)
    }
}

impl TryFrom<fprint_sys::fp_finger> for Finger {
    type Error = crate::FPrintError;

    fn try_from(value: fprint_sys::fp_finger) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Finger::LeftThumb),
            2 => Ok(Finger::LeftIndex),
            3 => Ok(Finger::LeftMiddle),
            4 => Ok(Finger::LeftRing),
            5 => Ok(Finger::LeftLittle),
            6 => Ok(Finger::RightThumb),
            7 => Ok(Finger::RightIndex),
            8 => Ok(Finger::RightMiddle),
            9 => Ok(Finger::RightRing),
            10 => Ok(Finger::RightLittle),
            n => Err(crate::FPrintError::TryFromError(n)),
        }
    }
}

