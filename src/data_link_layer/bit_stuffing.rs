use crate::{bit::Bit, bit_string::BitString};

pub const FLAG_SEQUECE: u8 = 0b0111_1110u8;

pub fn prepare_bits(data: BitString) -> BitString {
    let bs = stuff_bits(data);
    surround_flags(bs)
}

fn unstuff_bits(mut data: BitString) -> BitString {
    let mut count = 0;
    let mut remove_places = Vec::new();

    //                              I hate this notation
    for (idx, bit) in (&data).into_iter().enumerate() {
        if count == 5 {
            remove_places.push(idx);
        }

        match bit {
            Bit::On => count += 1,
            Bit::Off => count = 0,
        }
    }

    for &idx in remove_places.iter().rev() {
        data.remove_bit(idx);
    }

    data
}

fn stuff_bits(mut data: BitString) -> BitString {
    let mut count = 0;
    let mut insert_places = Vec::new();

    //                              I hate this notation
    for (idx, bit) in (&data).into_iter().enumerate() {
        if count == 5 {
            insert_places.push(idx);
            count = 0;
        }

        match bit {
            Bit::Off => count = 0,
            Bit::On => count += 1,
        }
    }

    for idx in insert_places.iter().rev() {
        data.insert_bit(*idx, Bit::Off);
    }

    data
}

fn surround_flags(mut data: BitString) -> BitString {
    data.prepend_u8(FLAG_SEQUECE);
    data.append_u8(FLAG_SEQUECE);
    data
}

#[cfg(test)]
mod test {
    use crate::{bit_string::BitString, bitstring, data_link_layer::bit_stuffing::FLAG_SEQUECE};

    use super::{stuff_bits, surround_flags, unstuff_bits};

    #[test]
    fn surround_flags_test() {
        let bs = BitString::from(0b0011_1100u8);

        let bs = surround_flags(bs);

        assert_eq!(bs.get_u8(0), FLAG_SEQUECE);
        assert_eq!(bs.get_u8(bs.len() - 8), FLAG_SEQUECE);
    }

    #[test]
    fn unstuff_bits_test() {
        let expected = bitstring!(0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,);
        let bs = bitstring![0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0,];

        let bs = unstuff_bits(bs);

        assert_eq!(expected, bs);
    }

    #[test]
    fn stuff_bits_test() {
        let bs = bitstring![0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,];

        let bs = stuff_bits(bs);
        let expected = bitstring![0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0,];

        assert_eq!(expected, bs);
    }

    #[cfg(feature = "fuzz")]
    mod fuzz {
        use crate::data_link_layer::bit_stuffing::stuff_bits;
        use crate::data_link_layer::bit_stuffing::unstuff_bits;

        use crate::{bit_string::BitString, rand::XorShift};
        fn generate_random_data<const N: usize>(seed: u128) -> [u8; N] {
            let mut rand = XorShift::new(seed);
            let mut data: [u8; N] = [0; N];

            #[allow(clippy::cast_possible_truncation)]
            data.iter_mut()
                .for_each(|el| *el = (rand.next_int() & u8::MAX as u128) as u8);

            data
        }

        #[test]
        fn stuffing_fuzz() {
            for seed in 0..=1024 {
                let bs = BitString::from(generate_random_data::<125>(seed));
                let bs_clone = bs.clone();

                let stuffed = stuff_bits(bs);
                let unstuffed = unstuff_bits(stuffed.clone());

                assert_eq!(bs_clone, unstuffed, "Failed with seed {seed}");
            }
        }
    }
}
