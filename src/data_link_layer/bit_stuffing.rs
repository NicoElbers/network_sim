use crate::{bit::Bit, bit_string::BitString};

pub const FLAG_SEQUECE: u8 = 0b0111_1110u8;

pub fn prepare_bits(data: BitString) -> BitString {
    let bs = stuff_bits(data);
    surround_flags(bs)
}

fn decode_bits(mut data: BitString) -> BitString {
    let mut count = 0;
    let mut remove_places = Vec::new();

    //                              I hate this notation
    for (idx, bit) in (&data).into_iter().enumerate() {
        match bit {
            Bit::On => count += 1,
            Bit::Off => {
                if count == 5 {
                    remove_places.push(idx);
                }

                count = 0;
            }
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
        match bit {
            Bit::Off => count = 0,
            Bit::On => count += 1,
        }

        if count == 6 {
            insert_places.push(idx);
            count = 0;
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
    use crate::{bit::Bit, bit_string::BitString, data_link_layer::bit_stuffing::FLAG_SEQUECE};

    use super::{decode_bits, stuff_bits, surround_flags};

    #[test]
    fn surround_flags_test() {
        let bs = BitString::from(0b0011_1100u8);

        let bs = surround_flags(bs);

        assert_eq!(bs.get_u8(0), FLAG_SEQUECE);
        assert_eq!(bs.get_u8(bs.len() - 8), FLAG_SEQUECE);
    }

    #[test]
    fn unstuff_bits_test() {
        let expected = BitString::from([
            Bit::Off,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::Off,
        ]);
        let bs = BitString::from([
            Bit::Off,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::Off,
        ]);

        let bs = decode_bits(bs);

        assert_eq!(expected, bs);
    }

    #[test]
    fn stuff_bits_test() {
        let bs = BitString::from([
            Bit::Off,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::Off,
        ]);

        let bs = stuff_bits(bs);
        let expected = BitString::from([
            Bit::Off,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::On,
            Bit::Off,
            Bit::Off,
        ]);

        println!("{:?}", bs);
        assert_eq!(expected, bs);
    }
}
