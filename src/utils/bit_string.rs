use core::slice;
use std::{
    fmt::Display,
    iter::once,
    ops::{Index, IndexMut, Shl, ShlAssign, Shr, ShrAssign},
    vec::{Drain, IntoIter},
};

use anyhow::ensure;

use crate::{
    bit::Bit,
    macros::{
        append_type, bit_string_as_vec, bit_string_from_val, bit_string_from_vec, get_type,
        insert_type, set_type,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitString {
    bit_vec: Vec<Bit>,
}

impl Display for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::with_capacity(self.len() + 11);
        str.push_str("BitString[");
        for bit in self.bit_vec.clone() {
            match bit {
                Bit::On => str.push('1'),
                Bit::Off => str.push('0'),
            }
        }
        str.push(']');

        write!(f, "{str}")
    }
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

    insert_type!(u8);
    insert_type!(u16);
    insert_type!(u32);
    insert_type!(u64);
    insert_type!(u128);

    pub fn remove_len(&mut self, index: usize, len: usize) -> Drain<Bit> {
        assert!(
            index + len <= self.len(),
            "Trying to remove index out of bounds"
        );

        self.bit_vec.drain(index..index + len)
    }

    pub fn remove_bit(&mut self, index: usize) -> Bit {
        assert!(index < self.len(), "Trying to remove index out of bounds");

        self.bit_vec.remove(index)
    }

    pub fn remove_last_len(&mut self, len: usize) -> Drain<Bit> {
        assert!(len <= self.len(), "Trying to remove index out of bounds");

        let index = self.len() - len;

        self.remove_len(index, len)
    }

    pub fn remove_last(&mut self) -> Option<Bit> {
        self.bit_vec.pop()
    }

    get_type!(u8);
    get_type!(u16);
    get_type!(u32);
    get_type!(u64);
    get_type!(u128);

    pub fn copy_len(&self, index: usize, len: usize) -> BitString {
        self.bit_vec
            .iter()
            .skip(index)
            .take(len)
            .copied()
            .collect::<BitString>()
    }

    set_type!(u8);
    set_type!(u16);
    set_type!(u32);
    set_type!(u64);
    set_type!(u128);

    pub fn set_bit(&mut self, index: usize, bit: Bit) {
        assert!(index < self.len(), "Trying to set index out of bounds");

        *self.get_bit_mut(index) = bit;
    }

    pub fn set_bits(&mut self, index: usize, bits: &BitString) {
        assert!(
            index + bits.len() <= self.len(),
            "Trying to set index out of bounds"
        );

        self.bit_vec
            .iter_mut()
            .skip(index)
            .take(bits.len())
            .enumerate()
            .for_each(|(idx, bit)| *bit = bits[idx]);
    }

    bit_string_as_vec!(u8);
    bit_string_as_vec!(u16);
    bit_string_as_vec!(u32);
    bit_string_as_vec!(u64);
    bit_string_as_vec!(u128);

    pub fn is_empty(&self) -> bool {
        self.bit_vec.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bit_vec.len()
    }

    pub fn flip_bits(&mut self, index: usize, length: usize) {
        assert!(index < self.len(), "Trying to flip index out of bounds");

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

    pub fn append_bits<T>(&mut self, bits: T)
    where
        T: Into<Vec<Bit>>,
    {
        self.bit_vec.append(&mut bits.into())
    }

    pub fn append_zeroes(&mut self, amount: usize) {
        let new_len = self.bit_vec.len() + amount;
        self.bit_vec.resize(new_len, Bit::Off);
    }

    pub fn append_ones(&mut self, amount: usize) {
        let new_len = self.bit_vec.len() + amount;
        self.bit_vec.resize(new_len, Bit::On);
    }

    pub fn insert_bit<T>(&mut self, index: usize, bit: T)
    where
        T: Into<Bit>,
    {
        assert!(
            index < self.bit_vec.len(),
            "Trying to insert index out of bounds"
        );
        self.bit_vec.reserve(1);

        self.bit_vec.splice(index..index, once(bit.into()));
    }

    pub fn prepend_bit(&mut self, bit: Bit) {
        if self.is_empty() {
            self.append_bit(bit)
        } else {
            self.insert_bit(0, bit)
        }
    }

    pub fn xor_on_index<'a, T>(&self, other: T, index: usize) -> BitString
    where
        T: Into<&'a BitString>,
    {
        let mut clone = self.clone();
        clone.xor_assign_on_index(other, index);
        clone
    }

    pub fn xor_assign_on_index<'a, T>(&mut self, other: T, index: usize)
    where
        T: Into<&'a BitString>,
    {
        let other: &BitString = Into::into(other);
        assert!(
            other.len() + index <= self.len(),
            "Trying to xor bitstring with len {} until index {}",
            self.len(),
            other.len() + index
        );

        self.bit_vec
            .iter_mut()
            .skip(index)
            .take(other.len())
            .enumerate()
            .for_each(|(idx, bit)| *bit ^= other[idx]);
    }

    pub fn reverse(&mut self) {
        self.bit_vec.reverse();
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

    pub fn get_last(&self) -> Option<&Bit> {
        self.bit_vec.last()
    }

    pub fn get_last_mut(&mut self) -> Option<&mut Bit> {
        self.bit_vec.last_mut()
    }

    pub fn as_vec(&self) -> &Vec<Bit> {
        &self.bit_vec
    }

    pub fn as_vec_mut(&mut self) -> &mut Vec<Bit> {
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

impl Index<usize> for BitString {
    type Output = Bit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bit_vec[index]
    }
}

impl IndexMut<usize> for BitString {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bit_vec[index]
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

bit_string_from_val!(u8);
bit_string_from_val!(u16);
bit_string_from_val!(u32);
bit_string_from_val!(u64);
bit_string_from_val!(u128);

bit_string_from_vec!(u8);
bit_string_from_vec!(u16);
bit_string_from_vec!(u32);
bit_string_from_vec!(u64);
bit_string_from_vec!(u128);

impl<const N: usize> From<[Bit; N]> for BitString {
    fn from(bits: [Bit; N]) -> Self {
        let mut bs = BitString::with_capacity(N);

        for bit in bits {
            bs.append_bit(bit)
        }

        bs
    }
}

impl From<&[Bit]> for BitString {
    fn from(bits: &[Bit]) -> Self {
        let mut bs = BitString::with_capacity(bits.len());

        for bit in bits {
            bs.append_bit(*bit)
        }

        bs
    }
}

impl From<BitString> for Vec<Bit> {
    fn from(value: BitString) -> Self {
        value.bit_vec
    }
}

impl<'a> From<&'a BitString> for &'a Vec<Bit> {
    fn from(value: &'a BitString) -> Self {
        &value.bit_vec
    }
}

impl From<Vec<Bit>> for BitString {
    fn from(value: Vec<Bit>) -> Self {
        value.as_slice().into()
    }
}

impl From<&Vec<Bit>> for BitString {
    fn from(value: &Vec<Bit>) -> Self {
        value.as_slice().into()
    }
}

impl FromIterator<Bit> for BitString {
    fn from_iter<T: IntoIterator<Item = Bit>>(iter: T) -> Self {
        BitString::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<'a> FromIterator<&'a Bit> for BitString {
    fn from_iter<T: IntoIterator<Item = &'a Bit>>(iter: T) -> Self {
        BitString::from(iter.into_iter().copied().collect::<Vec<_>>())
    }
}

impl<'a> From<Drain<'a, Bit>> for BitString {
    fn from(value: Drain<'a, Bit>) -> Self {
        BitString::from(value.into_iter().collect::<Vec<_>>())
    }
}

#[macro_export]
macro_rules! bitstring {
    () => {
        BitString::new()
    };

    ($($val:expr),* $(,)?) => {
        $crate::utils::bit_string::BitString::from(
            [$($crate::utils::bit::Bit::try_from($val as u8).unwrap()),*]
        )
    };
}

pub use bitstring;

#[cfg(test)]
mod test {
    use super::{Bit, BitString};

    const BYTE: u8 = 0b1100_0011;
    const BIT_ON: Bit = Bit::On;
    const BIT_OFF: Bit = Bit::Off;
    const U128: u128 = 0x1;

    #[test]
    fn test_macro() {
        let bit_string_1 = bitstring!(1, 0, 1, 0, 1, 0, 1, 0);
        let bit_string = BitString::from(0b1010_1010u8);

        assert_eq!(bit_string_1, bit_string);
    }

    #[test]
    #[should_panic]
    fn macro_fail_big() {
        let _ = bitstring!(2);
    }

    #[test]
    #[should_panic]
    fn macro_fail_max() {
        let _ = bitstring!(u128::MAX);
    }

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

    #[test]
    fn set_bit_long() {
        let mut bs = BitString::from([u32::MAX, u32::MAX, 0, u32::MAX]);

        bs.set_u32(64, u32::MAX);

        for bit in bs {
            assert_eq!(bit, Bit::On);
        }
    }

    #[test]
    fn as_vec() {
        let bs = BitString::from(0b1100_1010u8);

        let vec = bs.as_vec_exact_u8();

        assert_eq!(vec, vec![0b1100_1010u8])
    }

    #[test]
    fn as_vec_multi() {
        let bs = BitString::from(0b1100_0011_0011_1100u16);

        let vec = bs.as_vec_exact_u8();

        assert_eq!(vec, vec![0b1100_0011u8, 0b0011_1100u8]);
    }

    #[test]
    fn test_insert_u8() {
        let mut bs = BitString::from(0b1111_1111u8);

        bs.insert_u8(4, 0b0000_0000u8);

        let byte_vec = bs.as_vec_exact_u8();
        let expected = vec![0b1111_0000u8, 0b0000_1111u8];

        println!("{bs:?}");
        println!("stats: {}, ", bs.len());
        println!("Byte vec: {byte_vec:?}");
        println!("Expected: {expected:?}");

        assert_eq!(byte_vec, expected);
    }

    #[test]
    fn bs_equals_vec() {
        let bs = BitString::from(0b0011_1100_1101_0010u16);

        let vec = bs.as_vec_exact_u8();

        let bs_from_vec = BitString::from(vec);

        assert_eq!(bs, bs_from_vec);
    }

    #[test]
    fn bs_equals_vec_long() {
        let bs = BitString::from(0b0011_1100_1101_0010_0011_1010_0101_1100u32);

        let vec = bs.as_vec_exact_u8();

        let bs_from_vec = BitString::from(vec);

        assert_eq!(bs, bs_from_vec);
    }

    #[test]
    fn append_zero_test() {
        let mut bs = BitString::new();

        bs.append_u16(0);

        assert!(bs.len() == 16);

        bs.append_u16(0);

        assert!(bs.len() == 32);

        assert!(bs.get_u32(0) == 0);
    }

    #[test]
    fn append_zero_and_insert_test() {
        let mut bs = BitString::new();

        bs.append_u16(0);

        assert!(bs.len() == 16);

        bs.append_u16(0);

        assert!(bs.len() == 32);

        bs.append_u16(0);

        assert!(bs.len() == 48);

        bs.set_u16(16, u16::MAX);

        assert!(bs.len() == 48);

        assert!(bs.get_u16(16) == u16::MAX);
        assert!(bs.get_u16(0) == 0);
        assert!(bs.get_u16(32) == 0);
    }

    #[test]
    fn get_and_set1() {
        let mut bs = BitString::from(u64::MAX);

        const TEST_DATA: u16 = 0b1010_0000_0101_1111u16;

        assert_eq!(bs.get_u16(0), u16::MAX);
        assert_eq!(bs.get_u16(16), u16::MAX);
        assert_eq!(bs.get_u16(16), u16::MAX);
        bs.set_u16(16, TEST_DATA);
        assert_eq!(bs.get_u16(0), u16::MAX);
        assert_eq!(BitString::from(bs.get_u16(16)), BitString::from(TEST_DATA));
        assert_eq!(bs.get_u16(32), u16::MAX);
    }
    #[test]
    fn get_and_set2() {
        let mut bs = BitString::from(u64::MAX);

        const TEST_DATA: u16 = 0b1010_1010_1010_1010u16;

        assert_eq!(bs.get_u16(0), u16::MAX);
        assert_eq!(bs.get_u16(16), u16::MAX);
        assert_eq!(bs.get_u16(16), u16::MAX);
        bs.set_u16(16, TEST_DATA);
        assert_eq!(bs.get_u16(0), u16::MAX);
        assert_eq!(BitString::from(bs.get_u16(16)), BitString::from(TEST_DATA));
        assert_eq!(bs.get_u16(32), u16::MAX);
    }

    #[test]
    fn test_xor() {
        let bs = BitString::from(0b0000_1111_1111_0000u16);
        let other = BitString::from(0b1111_1111u8);

        println!("{}", BitString::from(0b1111_0000_1111_0000u16));
        println!("{}", bs.xor_on_index(&other, 0));
        assert_eq!(
            BitString::from(0b1111_0000_1111_0000u16),
            bs.xor_on_index(&other, 0),
            "Failed on start xor"
        );

        assert_eq!(
            BitString::from(0b0000_0000_0000_0000u16),
            bs.xor_on_index(&other, 4),
            "Failed on middle xor"
        );

        assert_eq!(
            BitString::from(0b0000_1111_0000_1111u16),
            bs.xor_on_index(&other, 8),
            "Failed on end xor"
        );
    }

    #[test]
    fn remove_last() {
        let mut bs = bitstring!(1, 1, 1, 1, 0, 0);

        let rem: BitString = bs.remove_last_len(2).collect();

        assert_eq!(2, rem.len());
        assert_eq!(bitstring!(0, 0), rem);
        assert_eq!(bitstring!(1, 1, 1, 1), bs);
    }

    #[test]
    fn remove_last_order() {
        let mut bs = bitstring!(1, 0);

        let rem: BitString = bs.remove_last_len(2).collect();

        assert_eq!(2, rem.len());
        assert!(bs.is_empty());

        assert_eq!(bitstring!(1, 0), rem);
    }

    #[test]
    fn test_set_bits() {
        let mut bs = bitstring!(0, 0);

        bs.set_bits(0, &bitstring!(1, 1));

        assert_eq!(bitstring!(1, 1), bs);

        bs = bitstring!(0, 0, 0);
        bs.set_bits(1, &bitstring!(1, 0));

        assert_eq!(bitstring!(0, 1, 0), bs);
    }
}
