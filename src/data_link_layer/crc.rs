use anyhow::ensure;

use crate::{bit::Bit, bit_string::BitString};

pub fn crc_add(generator: &BitString, mut data: BitString) -> BitString {
    assert!(!generator.is_empty(), "Generator cannot be empty");
    assert!(!data.is_empty(), "Unable to add a crc to no data");
    assert!(
        generator[0] == Bit::On,
        "Generator must start with a 1 or On bit"
    );

    data.append_zeroes(generator.len() - 1);

    let crc = binary_division(&data, generator);

    data.set_bits(data.len() - crc.len(), &crc);
    data
}

pub fn crc_check_and_remove(
    generator: &BitString,
    mut data: BitString,
) -> anyhow::Result<BitString> {
    ensure!(
        binary_division(&data, generator)
            .into_iter()
            .map(|bit| bit as u32)
            .sum::<u32>()
            == 0,
        "The message {data} is invalid for generator {generator}"
    );

    data.remove_last_len(generator.len() - 1);

    Ok(data)
}

fn binary_division(divident: &BitString, divisor: &BitString) -> BitString {
    if divident.len() < divisor.len() {
        println!("{} : {}", divisor.len(), divident.len());
        let len_to_add = divisor.len() - divident.len() - 1;

        let mut res: BitString = BitString::with_capacity(divident.len() - 1);
        res.append_zeroes(len_to_add);
        res.append_bits(divident.clone());

        debug_assert_eq!(res.len(), divisor.len() - 1, "Incorrect return length");
        return res;
    }

    let mut res = divident.clone();
    res.reverse();

    let mut div = divisor.clone();
    div.reverse();

    let len_diff = divident.len() - divisor.len();

    for xor_index in (0..=len_diff).rev() {
        let last = res.get_last().expect("crc should never be empty");

        if *last == Bit::On {
            res.xor_assign_on_index(&div, xor_index);
        }
        res.remove_last();
    }

    // Undo the reversal
    res.reverse();

    debug_assert_eq!(res.len(), divisor.len() - 1, "Incorrect return length");
    res
}

#[cfg(test)]
mod test {

    use crate::bit::Bit;
    use crate::bit_string::{bitstring, BitString};
    use crate::corruption_type::Corruption;
    use crate::data_link_layer::crc::{binary_division, crc_add, crc_check_and_remove};
    use crate::rand::XorShift;

    #[test]
    fn simple_check() {
        let data = bitstring!(1, 1, 0, 1, 0, 0);
        let generator = bitstring!(1, 0, 0);

        assert!(crc_check_and_remove(&generator, data).is_ok());
    }

    #[test]
    #[should_panic]
    fn incorrect_generator() {
        let data = bitstring!(1, 0, 1);
        let generator = bitstring!(0, 1);

        crc_add(&generator, data);
    }

    #[test]
    fn small_data() {
        let data = bitstring![0, 1];
        let generator = bitstring![1, 0, 0, 0];

        assert_eq!(crc_add(&generator, data), bitstring![0, 1, 0, 0, 0]);
    }

    #[test]
    fn simple_case() {
        let data = bitstring!(1, 0, 1, 1, 0);
        let gen = bitstring![1, 0, 0];

        let full = crc_add(&gen, data);

        assert_eq!(full, bitstring!(1, 0, 1, 1, 0, 0, 0));
    }

    fn gen_data(min_len: u128, max_len: u128, seed: u128) -> BitString {
        let mut rand = XorShift::new(seed);

        // HACK: This is lazy for testing
        let len = rand.next_int_bound(min_len, max_len) as usize;

        let mut bs = BitString::with_capacity(len);
        for _ in 0..len {
            match rand.next_int() % 2 {
                0 => bs.append_bit(Bit::Off),
                1 => bs.append_bit(Bit::On),
                _ => unreachable!(),
            }
        }

        bs
    }

    #[test]
    fn small_gen() {
        let data = bitstring!(0, 1, 1, 0);
        let gen = bitstring!(1, 0);

        let expected = bitstring!(0, 1, 1, 0, 0);

        let with_crc = crc_add(&gen, data);
        assert_eq!(expected, with_crc);

        assert!(crc_check_and_remove(&gen, with_crc).is_ok());
    }

    #[test]
    fn equal_len() {
        let data = bitstring!(1, 0, 1);
        let gen = bitstring!(1, 0, 0);

        let expected = bitstring!(1, 0, 1, 0, 0);

        let made = crc_add(&gen, data);

        assert_eq!(expected, made);
    }

    fn check_crc(bs: BitString, gen: BitString, expected: BitString) {
        assert_eq!(
            expected,
            binary_division(&bs, &gen),
            "CRC from data {bs}, gen {gen} is not {expected}"
        );
    }

    #[test]
    fn test_make_crc() {
        check_crc(bitstring!(0, 1, 1, 0), bitstring!(1, 1), bitstring!(0));
        check_crc(
            bitstring!(1, 0, 1, 1),
            bitstring!(1, 0, 1),
            bitstring!(0, 1),
        );
    }

    fn break_crc(corruption: &mut Corruption, generator: &BitString, valid_crc: BitString) -> bool {
        // println!("---");
        // println!("corrupt: {corruption:?}");
        // println!("gen    : {generator}");
        // println!("valid  : {valid_crc}");

        let invalid_crc = corruption.corrupt_borrow(valid_crc);

        // println!("invalid: {invalid_crc}");

        crc_check_and_remove(generator, invalid_crc).is_err()
    }

    #[test]
    fn crc_fuzz() {
        let percentage_expected = 0.98;

        let data_min = 1;
        let max_data_len = 100;
        let gen_len = 10;
        let cycles = 1_000;
        let mut correctly_detected_errors: u32 = 0;

        let mut rand = XorShift::new(113241324);
        let mut corruption = Corruption::RandomCorruption(rand.copy_reset());
        for seed in 1..=cycles {
            let data = gen_data(data_min, max_data_len, seed);
            let mut gen = gen_data(gen_len, gen_len, seed << 3);
            gen.prepend_bit(Bit::On);

            // println!("----");
            // println!("Data              : {data}");
            // println!("Gen               : {gen}");

            let data_clone = data.clone();

            let data_with_crc = crc_add(&gen, data_clone);
            // println!("Data with crc     : {data_with_crc}");

            if break_crc(&mut corruption, &gen, data_with_crc.clone()) {
                correctly_detected_errors += 1;
            }

            let data_received = crc_check_and_remove(&gen, data_with_crc);

            assert!(
                data_received.is_ok(),
                "CRC was thought to be incorrect on received data"
            );

            let data_received = data_received.unwrap();
            // println!("Data data_received: {}", data_received);

            assert_eq!(
                data, data_received,
                "Data send and received is not the same"
            );
        }

        assert!(
            correctly_detected_errors as f64 >= percentage_expected * cycles as f64,
            "Expected a detection rate of {percentage_expected} but detected {}",
            correctly_detected_errors as f64 / cycles as f64
        );
    }
}
