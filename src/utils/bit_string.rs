use core::slice;
use std::{
    ops::{Shl, ShlAssign, Shr, ShrAssign},
    vec::IntoIter,
};

use anyhow::ensure;

use crate::{
    bit::Bit,
    macros::{append_type, bit_string_from, get_type, set_type},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitString {
    bit_vec: Vec<Bit>,
}

impl BitString {
    pub fn new() -> Self {
        Self {
            bit_vec: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bit_vec: Vec::with_capacity(capacity),
        }
    }

    pub fn with_zeroes(amount: usize) -> Self {
        let mut bit_string = BitString::with_capacity(amount);
        bit_string.append_zeroes(amount);

        bit_string
    }

    pub fn with_ones(amount: usize) -> Self {
        let mut bit_string = BitString::with_capacity(amount);
        bit_string.append_ones(amount);

        bit_string
    }

    append_type!(u8);
    append_type!(u16);
    append_type!(u32);
    append_type!(u64);
    append_type!(u128);

    get_type!(u8);
    get_type!(u16);
    get_type!(u32);
    get_type!(u64);
    get_type!(u128);

    set_type!(u8);
    set_type!(u16);
    set_type!(u32);
    set_type!(u64);
    set_type!(u128);

    pub fn set_bit(&mut self, index: usize, bit: Bit) {
        assert!(index < self.bit_vec.len());

        *self.get_bit_mut(index) = bit;
    }

    pub fn flip_bits(&mut self, index: usize, length: usize) {
        assert!(length > 0);
        assert!(index < self.bit_vec.len());

        self.bit_vec
            .iter_mut()
            .skip(index)
            .take(length)
            .for_each(|bit| *bit ^= Bit::On);
    }

    pub fn flip_bits_exact(&mut self, index: usize, length: usize) -> anyhow::Result<()> {
        let bit_size = u8::BITS as usize;

        ensure!(
            index + bit_size <= self.bit_vec.len(),
            "Unable to get bits until index {} because length is {}",
            index + bit_size,
            self.bit_vec.len()
        );

        self.flip_bits(index, length);
        Ok(())
    }

    pub fn flip_bit(&mut self, index: usize) {
        self.flip_bits(index, 1);
    }

    pub fn append_bit(&mut self, bit: Bit) {
        self.bit_vec.push(bit);
    }

    pub fn append_bits(&mut self, bits: &[Bit]) {
        self.bit_vec = [self.bit_vec.as_slice(), bits].concat();
    }

    pub fn append_zeroes(&mut self, amount: usize) {
        let new_len = self.bit_vec.len() + amount;
        self.bit_vec.resize(new_len, Bit::Off);
    }

    pub fn append_ones(&mut self, amount: usize) {
        let new_len = self.bit_vec.len() + amount;
        self.bit_vec.resize(new_len, Bit::On);
    }

    pub fn as_bit_slice(&self) -> &[Bit] {
        &self.bit_vec
    }

    pub fn as_bit_slice_mut(&mut self) -> &mut [Bit] {
        &mut self.bit_vec
    }

    pub fn checked_get_bit(&self, index: usize) -> Option<&Bit> {
        self.bit_vec.get(index)
    }

    pub fn get_bit(&self, index: usize) -> &Bit {
        &self.bit_vec[index]
    }

    pub fn get_bit_mut(&mut self, index: usize) -> &mut Bit {
        &mut self.bit_vec[index]
    }

    pub fn get_inner(&self) -> &Vec<Bit> {
        &self.bit_vec
    }

    pub fn get_inner_mut(&mut self) -> &mut Vec<Bit> {
        &mut self.bit_vec
    }

    pub fn stringify(&self) -> String {
        let mut string = String::new();

        for bit in &self.bit_vec {
            string += bit.stringify();
        }

        string
    }
}

impl IntoIterator for BitString {
    type Item = Bit;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.bit_vec.into_iter()
    }
}

impl<'a> IntoIterator for &'a BitString {
    type Item = &'a Bit;
    type IntoIter = slice::Iter<'a, Bit>;

    fn into_iter(self) -> Self::IntoIter {
        self.bit_vec.iter()
    }
}

impl<'a> IntoIterator for &'a mut BitString {
    type Item = &'a mut Bit;
    type IntoIter = slice::IterMut<'a, Bit>;

    fn into_iter(self) -> Self::IntoIter {
        self.bit_vec.iter_mut()
    }
}

impl ShrAssign<usize> for BitString {
    fn shr_assign(&mut self, amount: usize) {
        if amount >= self.bit_vec.len() {
            *self = Self::with_zeroes(amount);
        }

        let mut clone = BitString::with_capacity(self.bit_vec.len());

        println!("{}", clone.stringify());

        // prepend amount zeroes
        clone.append_zeroes(amount);

        // remove last amount elements
        let old_elements = self.bit_vec.len() - amount;
        self.bit_vec
            .iter()
            .take(old_elements)
            .copied()
            .for_each(|bit| clone.append_bit(bit));

        println!("{}", clone.stringify());

        *self = clone;
    }
}

impl Shr<usize> for BitString {
    type Output = BitString;

    fn shr(self, amount: usize) -> Self::Output {
        let mut clone = self.clone();
        clone >>= amount;
        clone
    }
}

impl ShlAssign<usize> for BitString {
    fn shl_assign(&mut self, amount: usize) {
        if amount >= self.bit_vec.len() {
            *self = Self::with_zeroes(amount);
        }

        // We remove the first amount elements and put them in a new bit_vec
        let new_bit_vec = self
            .bit_vec
            .iter()
            .skip(amount)
            .copied()
            .collect::<Vec<Bit>>();

        // Assign the new bitvec and append zeroes equal to the amount fo elements
        // we removed earlier
        self.bit_vec = new_bit_vec;
        self.append_zeroes(amount)
    }
}

impl Shl<usize> for BitString {
    type Output = Self;

    fn shl(self, amount: usize) -> Self::Output {
        let mut clone = self.clone();
        clone <<= amount;
        clone
    }
}

impl Default for BitString {
    fn default() -> Self {
        Self::new()
    }
}

bit_string_from!(u8);
bit_string_from!(u16);
bit_string_from!(u32);
bit_string_from!(u64);
bit_string_from!(u128);

#[cfg(test)]
mod test {
    use super::{Bit, BitString};

    const BYTE: u8 = 0b1100_0011;
    const BIT_ON: Bit = Bit::On;
    const BIT_OFF: Bit = Bit::Off;
    const U128: u128 = 0x1;

    #[test]
    fn shift_right() {
        let mut bit_string = BitString::from(0b1010_1010u8);

        bit_string >>= 1;

        println!("{}", bit_string.stringify());

        assert_eq!(bit_string.get_u8(0), 0b0101_0101u8);
    }

    #[test]
    fn shift_right_overflow() {
        let mut bit_string = BitString::from(0b1010_1010u8);

        bit_string >>= 10;

        assert_eq!(bit_string.get_u8(0), 0b0000_0000u8);
    }

    #[test]
    fn shift_left() {
        let mut bit_string = BitString::from(0b1010_1010u8);

        bit_string <<= 1;

        assert_eq!(bit_string.get_u8(0), 0b0101_0100u8);
    }

    #[test]
    fn shift_left_overflow() {
        let mut bit_string = BitString::from(0b1010_1010u8);

        bit_string <<= 10;

        assert_eq!(bit_string.get_u8(0), 0b0000_0000u8);
    }

    #[test]
    fn append_and_get() {
        let mut bit_string = BitString::new();

        bit_string.append_u128(U128);

        let got = bit_string.get_u128(0);

        let mut new_bit_string = BitString::new();

        new_bit_string.append_u128(got);

        println!("{}", bit_string.stringify());
        println!("{}", new_bit_string.stringify());
        assert_eq!(bit_string.get_u128(0), U128);
    }

    #[test]
    fn assert_equals() {
        let mut bit_string1 = BitString::new();
        let mut bit_string2 = BitString::new();

        bit_string1.append_u128(U128);
        bit_string2.append_u128(U128);

        assert_eq!(bit_string1, bit_string2);
    }

    #[test]
    fn append_byte() {
        let mut bit_string = BitString::new();

        bit_string.append_u8(BYTE);

        assert_eq!(bit_string.get_u8(0), BYTE)
    }

    #[test]
    fn append_u8s() {
        let mut bit_string = BitString::new();

        bit_string.append_u8(BYTE);
        bit_string.append_u8(!BYTE);

        assert_eq!(bit_string.get_u8(0), BYTE);
        assert_eq!(bit_string.get_u8(8), !BYTE);
    }

    #[test]
    fn append_bit_on() {
        let mut bit_string = BitString::new();

        bit_string.append_bit(BIT_ON);

        assert_eq!(bit_string.get_bit(0), &Bit::On)
    }

    #[test]
    fn append_bit_off() {
        let mut bit_string = BitString::new();

        bit_string.append_bit(BIT_OFF);

        assert_eq!(bit_string.get_bit(0), &Bit::Off)
    }

    #[test]
    fn append_bits() {
        let mut bit_string = BitString::new();

        bit_string.append_bit(BIT_ON);
        bit_string.append_bit(BIT_OFF);
        bit_string.append_bit(BIT_ON);

        assert_eq!(bit_string.get_bit(0), &Bit::On);
        assert_eq!(bit_string.get_bit(1), &Bit::Off);
        assert_eq!(bit_string.get_bit(2), &Bit::On);
    }

    #[test]
    fn append_u8_then_bits() {
        let mut bit_string = BitString::new();

        bit_string.append_u8(BYTE);

        bit_string.append_bit(BIT_ON);
        bit_string.append_bit(BIT_OFF);
        bit_string.append_bit(BIT_ON);

        assert_eq!(bit_string.get_u8(0), BYTE);
        assert_eq!(bit_string.get_u8(8), 0b1010_0000);
    }

    #[test]
    fn append_bits_then_byte() {
        let mut bit_string = BitString::new();

        bit_string.append_bit(BIT_ON);
        bit_string.append_bit(BIT_OFF);
        bit_string.append_bit(BIT_ON);
        bit_string.append_bit(BIT_OFF);

        bit_string.append_u8(BYTE);

        assert_eq!(bit_string.get_u16(0), 0b1010_1100_0011_0000);
    }

    #[test]
    fn set_u8() {
        let mut bit_string = BitString::new();

        bit_string.append_u8(0b0000_0000u8);

        bit_string.set_u8(0, 0b1111_1111u8);

        assert_eq!(bit_string.get_u8(0), 0b1111_1111u8);
    }

    #[test]
    fn set_exact_u8() {
        let mut bit_string = BitString::new();

        bit_string.append_u8(0b0000_0000u8);

        assert!(bit_string.set_exact_u8(0, 0b1111_1111u8).is_ok());
        assert!(bit_string.set_exact_u8(1, 0b1111_1111u8).is_err());

        assert_eq!(bit_string.get_u8(0), 0b1111_1111u8);
    }
}
