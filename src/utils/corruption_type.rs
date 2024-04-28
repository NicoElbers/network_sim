use super::rand::XorShift;

const ENUM_VARIANTS: usize = 6;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Corruption {
    None,
    Random(XorShift),
    OneBitFlip(XorShift),
    MultiBitFlipOdd(XorShift, u8),
    MultiBitFlipEven(XorShift, u8),
    BurstFlip(XorShift),
    //ByteLoss,
}

impl Corruption {
    pub fn corrupt(mut self, data: &mut [u8]) -> &[u8] {
        self.corrupt_borrow(data)
    }

    pub fn corrupt_borrow<'a, 'b>(&'a mut self, data: &'b mut [u8]) -> &'b [u8]
    where
        'b: 'a,
    {
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
        }
    }

    fn no_corruption(data: &mut [u8]) -> &mut [u8] {
        assert!(!data.is_empty());

        data
    }

    fn one_bit_flip<'a>(rand: &mut XorShift, data: &'a mut [u8]) -> &'a mut [u8] {
        assert!(!data.is_empty());

        let idx = (rand.next_int() % data.len() as u128) as usize;
        let bit = rand.next_int() % 8u128;
        let mask = 0x1u8 << bit;

        data[idx] ^= mask;

        data
    }

    /// This function is restricted to flipping at most one bit per byte. It will
    /// only flip 2 bits in a byte if only one byte is provided.
    ///
    /// The chance is per bit pair. A chance of 0 ensures that the data is unchanged.
    fn multi_bit_flip_even<'a>(
        rand: &mut XorShift,
        chance: u8,
        data: &'a mut [u8],
    ) -> &'a mut [u8] {
        assert!(chance <= 100);
        assert!(!data.is_empty());

        if chance == 0 {
            return data;
        }

        let count_ones_before = data.iter().map(|byte| byte.count_ones()).sum::<u32>();

        for byte in data.iter_mut() {
            let event = (rand.next_int() % 100) as u8;

            if event > chance {
                continue;
            }

            let bit = rand.next_int() % 8u128;
            let mask = 0x1u8 << bit;

            *byte ^= mask;
        }

        let count_ones_after = data.iter().map(|byte| byte.count_ones()).sum::<u32>();

        // If the number of ones before and after differ by a value divisible by 2,
        // we have an even amount of flips. Otherwise we flip again.
        if count_ones_before.abs_diff(count_ones_after) % 2 != 0 {
            Self::one_bit_flip(rand, data);
        }

        data
    }

    /// This function is restricted to flipping at most one bit per byte.
    fn multi_bit_flip_odd<'a>(rand: &mut XorShift, chance: u8, data: &'a mut [u8]) -> &'a mut [u8] {
        assert!(chance <= 100);
        assert!(!data.is_empty());

        if chance == 0 {
            return data;
        }

        // Corrupt the data an even amount of times, then once more
        let data = Self::multi_bit_flip_even(rand, chance, data);
        Self::one_bit_flip(rand, data)
    }

    /// Because I'm a lazy fuck this'll just flip a byte and is thus always be
    /// byte aligned. I don't care
    fn burst_flip<'a>(rand: &mut XorShift, data: &'a mut [u8]) -> &'a [u8] {
        assert!(!data.is_empty());

        let idx = (rand.next_int() % data.len() as u128) as usize;
        let mask = u8::MAX;

        data[idx] ^= mask;

        data
    }

    fn random<'a>(rand: &mut XorShift, data: &'a mut [u8]) -> &'a [u8] {
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
}

#[cfg(test)]
mod test {
    use crate::utils::rand::XorShift;

    use super::Corruption;

    const RANDOM_TEST_CYCLES: usize = 100usize;

    fn bits_flipped<'a>(data1: &'a [u8], data2: &'a [u8]) -> u32 {
        assert_eq!(data1.len(), data2.len());

        let mut difference: u32 = 0;
        for i in 0..data1.len() {
            difference += (data1[i] ^ data2[i]).count_ones();
        }

        difference
    }

    #[test]
    fn test_no_flip() {
        let mut data = [15u8];
        let data_copy = data;

        Corruption::no_corruption(&mut data);

        assert!(bits_flipped(&data, &data_copy) == 0);
    }

    #[test]
    fn test_bit_flip() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8];
        let data_copy = data;

        Corruption::one_bit_flip(&mut rand, &mut data);

        assert!(bits_flipped(&data, &data_copy) == 1);
    }

    #[test]
    fn test_mutli_bit_flip_even_single_byte() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8];
        let data_copy = data;

        Corruption::multi_bit_flip_even(&mut rand, 100, &mut data);

        assert!(bits_flipped(&data, &data_copy) % 2 == 0);
    }

    #[test]
    fn test_mutli_bit_flip_even_multi_byte() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8, 128u8, 0u8];
        let data_copy = data;

        Corruption::multi_bit_flip_even(&mut rand, 100, &mut data);

        assert!(bits_flipped(&data, &data_copy) % 2 == 0);
    }

    #[test]
    fn test_mutli_bit_flip_odd_single_byte() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8];
        let data_copy = data;

        Corruption::multi_bit_flip_odd(&mut rand, 100, &mut data);

        assert!(bits_flipped(&data, &data_copy) % 2 != 0);
    }

    #[test]
    fn test_mutli_bit_flip_odd_multi_byte() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8, 128u8, 0u8];
        let data_copy = data;

        Corruption::multi_bit_flip_odd(&mut rand, 100, &mut data);

        assert!(bits_flipped(&data, &data_copy) % 2 != 0);
    }

    #[test]
    fn test_burst_flip_single_byte() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8];
        let data_copy = data;

        Corruption::burst_flip(&mut rand, &mut data);

        assert!(bits_flipped(&data, &data_copy) == 8);
    }

    #[test]
    fn test_burst_flip_multi_byte() {
        let mut rand = XorShift::new(69);
        let mut data = [15u8, 128u8, 0u8];
        let data_copy = data;

        Corruption::burst_flip(&mut rand, &mut data);

        assert!(bits_flipped(&data, &data_copy) == 8);
    }

    // Make sure the panics work as intended
    #[test]
    #[should_panic]
    fn none_assert_panics() {
        Corruption::corrupt(Corruption::None, &mut []);
    }

    #[test]
    #[should_panic]
    fn one_bit_flip_assert_panics() {
        Corruption::corrupt(Corruption::OneBitFlip(XorShift::new(0)), &mut []);
    }

    #[test]
    #[should_panic]
    fn multi_bit_flip_even_assert_panics_on_no_data() {
        Corruption::corrupt(Corruption::MultiBitFlipEven(XorShift::new(0), 69), &mut []);
    }

    #[test]
    #[should_panic]
    fn multi_bit_flip_even_assert_panics_on_impossible_chance() {
        Corruption::corrupt(
            Corruption::MultiBitFlipEven(XorShift::new(0), 128),
            &mut [0],
        );
    }

    #[test]
    #[should_panic]
    fn multi_bit_flip_odd_assert_panics_on_no_data() {
        Corruption::corrupt(Corruption::MultiBitFlipOdd(XorShift::new(0), 69), &mut []);
    }

    #[test]
    #[should_panic]
    fn multi_bit_flip_odd_assert_panics_on_impossible_chance() {
        Corruption::corrupt(Corruption::MultiBitFlipOdd(XorShift::new(0), 128), &mut [0]);
    }

    #[test]
    #[should_panic]
    fn burst_flip_assert_panic() {
        Corruption::corrupt(Corruption::BurstFlip(XorShift::new(0)), &mut []);
    }

    #[test]
    #[should_panic]
    fn assert_random_panics_on_no_data() {
        Corruption::corrupt(Corruption::Random(XorShift::new(0)), &mut []);
    }

    #[test]
    fn assert_random_does_not_panic_on_use() {
        let mut seed_gen = XorShift::new(0);

        for _ in 0..RANDOM_TEST_CYCLES {
            let rand = XorShift::new(seed_gen.next_int());
            Corruption::corrupt(Corruption::Random(rand), &mut [0]);
        }
    }
}
