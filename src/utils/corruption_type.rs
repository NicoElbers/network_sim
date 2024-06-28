use crate::{bit::Bit, bit_string::BitString};

use super::rand::XorShift;

const ENUM_VARIANTS: usize = 6;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Corruption {
    None,
    Random(XorShift),
    RandomCorruption(XorShift),
    OneBitFlip(XorShift),
    MultiBitFlipOdd(XorShift, u8),
    MultiBitFlipEven(XorShift, u8),
    BurstFlip(XorShift),
    //ByteLoss,
}

impl Corruption {
    #[must_use]
    pub fn corrupt(mut self, data: BitString) -> BitString {
        self.corrupt_borrow(data)
    }

    pub fn corrupt_borrow(&mut self, data: BitString) -> BitString {
        assert!(!data.is_empty());

        match self {
            Self::None => Self::no_corruption(data),
            Self::OneBitFlip(ref mut rand) => Self::one_bit_flip(rand, data),
            Self::MultiBitFlipEven(ref mut rand, chance) => {
                Self::multi_bit_flip_even(rand, *chance, data)
            }
            Self::MultiBitFlipOdd(ref mut rand, chance) => {
                Self::multi_bit_flip_odd(rand, *chance, data)
            }
            Self::BurstFlip(ref mut rand) => Self::burst_flip(rand, data),
            Self::Random(rand) => Self::random(rand, data),
            Self::RandomCorruption(rand) => Self::random_corruption(rand, data),
        }
    }

    const fn no_corruption(data: BitString) -> BitString {
        data
    }

    fn one_bit_flip(rand: &mut XorShift, mut data: BitString) -> BitString {
        let idx = (rand.next_int() % data.len() as u128) as usize;
        data.flip_bit(idx);
        data
    }

    /// This function is restricted to flipping at most one bit per byte. It will
    /// only flip 2 bits in a byte if only one byte is provided.
    ///
    /// The chance is per bit pair. A chance of 0 ensures that the data is unchanged.
    fn multi_bit_flip_even(rand: &mut XorShift, chance: u8, mut data: BitString) -> BitString {
        assert!(chance <= 100);

        if chance == 0 {
            return data;
        }

        let count_ones_before = (&data).into_iter().filter(|bit| **bit == Bit::On).count();

        for bit in &mut data {
            let event = (rand.next_int() % 100) as u8;

            if event > chance {
                continue;
            }

            bit.flip();
        }

        let count_ones_after = (&data).into_iter().filter(|bit| **bit == Bit::On).count();

        // If the number of ones before and after differ by a value divisible by 2,
        // we have an even amount of flips. Otherwise we flip again.
        if count_ones_before.abs_diff(count_ones_after) % 2 != 0 {
            Self::one_bit_flip(rand, data)
        } else {
            data
        }
    }

    /// This function is restricted to flipping at most one bit per byte.
    fn multi_bit_flip_odd(rand: &mut XorShift, chance: u8, data: BitString) -> BitString {
        assert!(chance <= 100);

        if chance == 0 {
            return data;
        }

        // Corrupt the data an even amount of times, then once more
        let data = Self::multi_bit_flip_even(rand, chance, data);
        Self::one_bit_flip(rand, data)
    }

    /// Flips 8 bits in order in the bitstring
    fn burst_flip(rand: &mut XorShift, mut data: BitString) -> BitString {
        let len = rand.next_int_bound(
            usize::min(data.len() / 2, 4) as u128,
            usize::min(data.len() / 2, 16) as u128,
        ) as usize;
        let idx = (rand.next_int() % (data.len() - len) as u128) as usize;

        data.flip_bits(idx, len);

        data
    }

    fn random(rand: &mut XorShift, data: BitString) -> BitString {
        let mut rand = rand.copy_reset();

        // between 0 and 100, we exclude 101
        let chance = (rand.next_int() % 101) as u8;

        // Random is a variant we ignore, so -1
        let idx = rand.next_int() % (ENUM_VARIANTS - 1) as u128;

        match idx {
            0 => Self::corrupt(Self::None, data),
            1 => Self::corrupt(Self::OneBitFlip(rand), data),
            2 => Self::corrupt(Self::MultiBitFlipOdd(rand, chance), data),
            3 => Self::corrupt(Self::MultiBitFlipEven(rand, chance), data),
            4 => Self::corrupt(Self::BurstFlip(rand), data),
            _ => unreachable!(),
        }
    }

    fn random_corruption(rand: &mut XorShift, data: BitString) -> BitString {
        let mut rand = rand.copy_reset();

        // between 0 and 100, we exclude 101
        let chance = (rand.next_int() % 101) as u8;

        // Random is a variant we ignore, so -1
        let idx = rand.next_int() % (ENUM_VARIANTS - 2) as u128;

        match idx {
            0 => Self::corrupt(Self::OneBitFlip(rand), data),
            1 => Self::corrupt(Self::MultiBitFlipOdd(rand, chance), data),
            2 => Self::corrupt(Self::MultiBitFlipEven(rand, chance), data),
            3 => Self::corrupt(Self::BurstFlip(rand), data),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{bit_string::BitString, utils::rand::XorShift};

    use super::Corruption;

    const RANDOM_TEST_CYCLES: usize = 100usize;
    const DEFAULT_DATA: u8 = 0b0011_1010;

    fn bits_flipped(left: &BitString, right: &BitString) -> u32 {
        assert_eq!(
            left.len(),
            right.len(),
            "the length of the bitstrings is not equal. Left is {} and right is {}",
            left.len(),
            right.len()
        );

        let mut difference: u32 = 0;
        for i in 0..left.len() {
            difference += (left[i] ^ right[i]) as u32;
        }

        difference
    }

    fn get_data(data: u8) -> BitString {
        let mut bs = BitString::new();
        bs.append_u8(data);
        bs
    }

    fn get_data_default() -> BitString {
        get_data(DEFAULT_DATA)
    }

    #[test]
    fn test_no_flip() {
        let data = get_data_default();
        let data_copy = data.clone();

        let data = Corruption::no_corruption(data);

        assert_eq!(bits_flipped(&data, &data_copy), 0);
    }

    #[test]
    fn test_bit_flip() {
        let mut rand = XorShift::new(69);
        let data = get_data_default();
        let data_copy = data.clone();

        let data = Corruption::one_bit_flip(&mut rand, data);

        assert!(bits_flipped(&data, &data_copy) == 1);
    }

    #[test]
    fn test_mutli_bit_flip_even_byte() {
        let mut rand = XorShift::new(69);
        let data = get_data_default();
        let data_copy = data.clone();

        let data = Corruption::multi_bit_flip_even(&mut rand, 100, data);

        assert_eq!(bits_flipped(&data, &data_copy) % 2, 0);
    }

    #[test]
    fn test_mutli_bit_flip_odd_byte() {
        let mut rand = XorShift::new(69);
        let data = get_data_default();
        let data_copy = data.clone();

        let data = Corruption::multi_bit_flip_odd(&mut rand, 100, data);

        assert_ne!(bits_flipped(&data, &data_copy) % 2, 0);
    }

    #[test]
    fn test_burst_flip() {
        let mut rand = XorShift::new(69);
        let data = get_data_default();
        let data_copy = data.clone();

        let data = Corruption::burst_flip(&mut rand, data);

        assert!(bits_flipped(&data, &data_copy) >= 4);
        assert!(bits_flipped(&data, &data_copy) <= 8);
    }

    #[test]
    fn test_burst_flip_short() {
        let mut rand = XorShift::new(69);
        let mut data = BitString::new();
        data.append_u8(0b0011_0011);

        let data_copy = data.clone();

        let data = Corruption::burst_flip(&mut rand, data);

        assert!(bits_flipped(&data, &data_copy) >= 4);
        assert!(bits_flipped(&data, &data_copy) <= 8);
    }

    // --- Make sure the panics work as intended ---
    const fn get_data_empty() -> BitString {
        BitString::new()
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn none_assert_panics() {
        Corruption::corrupt(Corruption::None, get_data_empty());
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn one_bit_flip_assert_panics() {
        Corruption::corrupt(Corruption::OneBitFlip(XorShift::new(0)), get_data_empty());
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn multi_bit_flip_even_assert_panics_on_no_data() {
        Corruption::corrupt(
            Corruption::MultiBitFlipEven(XorShift::new(0), 69),
            get_data_empty(),
        );
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn multi_bit_flip_even_assert_panics_on_impossible_chance() {
        Corruption::corrupt(
            Corruption::MultiBitFlipEven(XorShift::new(0), 128),
            get_data_default(),
        );
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn multi_bit_flip_odd_assert_panics_on_no_data() {
        Corruption::corrupt(
            Corruption::MultiBitFlipOdd(XorShift::new(0), 69),
            get_data_empty(),
        );
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn multi_bit_flip_odd_assert_panics_on_impossible_chance() {
        Corruption::corrupt(
            Corruption::MultiBitFlipOdd(XorShift::new(0), 128),
            get_data_default(),
        );
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn burst_flip_assert_panic() {
        Corruption::corrupt(Corruption::BurstFlip(XorShift::new(0)), get_data_empty());
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn assert_random_panics_on_no_data() {
        Corruption::corrupt(Corruption::Random(XorShift::new(0)), get_data_empty());
    }

    #[test]
    #[allow(clippy::should_panic_without_expect)]
    #[should_panic]
    fn assert_random_corrupt_panics_on_no_data() {
        let _ = Corruption::corrupt(
            Corruption::RandomCorruption(XorShift::new(0)),
            get_data_empty(),
        );
    }

    #[test]
    fn assert_random_does_not_panic_on_use() {
        let mut seed_gen = XorShift::new(0);

        for _ in 0..RANDOM_TEST_CYCLES {
            let rand1 = XorShift::new(seed_gen.next_int());
            let rand2 = XorShift::new(seed_gen.next_int());
            Corruption::corrupt(Corruption::Random(rand1), get_data_default());
            Corruption::corrupt(Corruption::RandomCorruption(rand2), get_data_default());
        }
    }
}
