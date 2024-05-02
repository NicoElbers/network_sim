use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::macros::{bit_into_type, bit_try_from};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bit {
    On,
    Off,
}

impl Bit {
    pub fn stringify(&self) -> &str {
        match self {
            Bit::Off => "0",
            Bit::On => "1",
        }
    }
}

impl BitXor for Bit {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) => Bit::Off,
            (Bit::Off, Bit::On) => Bit::On,
            (Bit::On, Bit::Off) => Bit::On,
            (Bit::On, Bit::On) => Bit::Off,
        }
    }
}

impl BitXor for &Bit {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) => &Bit::Off,
            (Bit::Off, Bit::On) => &Bit::On,
            (Bit::On, Bit::Off) => &Bit::On,
            (Bit::On, Bit::On) => &Bit::Off,
        }
    }
}

impl BitXorAssign for Bit {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl BitXorAssign for &Bit {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl BitAnd for Bit {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) => Bit::Off,
            (Bit::Off, Bit::On) => Bit::Off,
            (Bit::On, Bit::Off) => Bit::Off,
            (Bit::On, Bit::On) => Bit::On,
        }
    }
}

impl BitAnd for &Bit {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) => &Bit::Off,
            (Bit::Off, Bit::On) => &Bit::Off,
            (Bit::On, Bit::Off) => &Bit::Off,
            (Bit::On, Bit::On) => &Bit::On,
        }
    }
}

impl BitAndAssign for Bit {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitAndAssign for &Bit {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for Bit {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) => Bit::Off,
            (Bit::Off, Bit::On) => Bit::On,
            (Bit::On, Bit::Off) => Bit::On,
            (Bit::On, Bit::On) => Bit::On,
        }
    }
}

impl BitOr for &Bit {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bit::Off, Bit::Off) => &Bit::Off,
            (Bit::Off, Bit::On) => &Bit::On,
            (Bit::On, Bit::Off) => &Bit::On,
            (Bit::On, Bit::On) => &Bit::On,
        }
    }
}

impl BitOrAssign for Bit {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitOrAssign for &Bit {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl Not for Bit {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Bit::Off => Bit::On,
            Bit::On => Bit::Off,
        }
    }
}

impl Not for &Bit {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Bit::Off => &Bit::On,
            Bit::On => &Bit::Off,
        }
    }
}

impl From<bool> for Bit {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Off,
            true => Self::On,
        }
    }
}

bit_into_type!(u8);
bit_into_type!(u16);
bit_into_type!(u32);
bit_into_type!(u64);
bit_into_type!(u128);

bit_try_from!(u8);
bit_try_from!(u16);
bit_try_from!(u32);
bit_try_from!(u64);
bit_try_from!(u128);
