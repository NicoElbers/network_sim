macro_rules! bit_into_type {
    ($t:ty) => {
        impl Into<$t> for Bit {
            #![allow(clippy::from_over_into)]
            fn into(self) -> $t {
                match self {
                    Self::On => 1,
                    Self::Off => 0,
                }
            }
        }
    };
}

macro_rules! bit_try_from {
    ($t:ty) => {
        impl TryFrom<$t> for Bit {
            type Error = String;

            fn try_from(value: $t) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(Bit::Off),
                    1 => Ok(Bit::On),
                    value => Err(format!("Cannot represent {} as a single bit", value)),
                }
            }
        }
    };
}

macro_rules! append_type {
    ($t:ty) => {
        ::paste::paste! {
            pub fn [<append_ $t>](&mut self, data: $t) {
                    let bit_size = <$t>::BITS as usize;

                    let mask: $t = 0b1 << (bit_size - 1);

                    for idx in 0..bit_size {
                    let mask = mask >> idx;

                    let masked = data & mask;

                    assert!(masked.count_ones() == 1 || masked.count_ones() == 0);

                    if masked.count_ones() == 1 {
                        self.append_bit(Bit::On);
                    } else {
                        self.append_bit(Bit::Off)
                    }
                }
            }
        }
    };
}

macro_rules! insert_type {
    ($t:ty) => {
        ::paste::paste! {

            pub fn [<insert_ $t>](&mut self, index: usize, data: $t) {
                assert!(index < self.bit_vec.len());

                self.bit_vec.reserve(<$t>::BITS as usize);

                let to_add = BitString::from(data);

                self.bit_vec.splice(index..index, to_add);
            }

            pub fn [<prepend_ $t>](&mut self, data: $t) {
                self.[<insert_ $t>](0, data);
            }
        }
    };
}

macro_rules! get_type {
    ($t:ty) => {
        ::paste::paste! {
            pub fn [<get_ $t>](&self, index: usize) -> $t {
                let bit_size = <$t>::BITS as usize;

                let mut output: $t = 0;

                for idx in 0..bit_size {
                    if index + idx >= self.bit_vec.len() {
                        break;
                    }

                    let bit = self.get_bit(index + idx);

                    if *bit == Bit::On {
                        let mask: $t = 0b1 << (bit_size - idx - 1);
                        output |= mask;
                    }
                }

                output
            }

            pub fn [<get_exact_ $t>](&self, index: usize) -> anyhow::Result<$t> {
                let bit_size = <$t>::BITS as usize;

                ensure!(
                    index + bit_size <= self.bit_vec.len(),
                    "Unable to get bits until index {} because length is {}",
                    index + bit_size,
                    self.bit_vec.len()
                );

                Ok(self.[<get_ $t>](index))
            }
        }
    };
}

macro_rules! set_type {
    ($t:ty) => {
        ::paste::paste! {
            pub fn [<set_ $t>] (&mut self, index: usize, data: $t) {
                let bit_size = <$t>::BITS;
                let mask = 0b1;

                self.bit_vec
                    .iter_mut()
                    .skip(index)
                    .take(bit_size as usize)
                    .enumerate()
                    .for_each(|(idx, bit)| {
                        let (shifted_data, _) = data.overflowing_shr(bit_size - idx as u32);

                        *bit = Bit::try_from(shifted_data & mask)
                            .expect("This is ensured to work because we mask with 0b1");
                    })
            }

            pub fn [<set_exact_ $t>] (&mut self, index: usize, data: $t) -> anyhow::Result<()> {
                let bit_size = <$t>::BITS;

                ensure!(
                    index + bit_size as usize <= self.bit_vec.len(),
                    "Trying to set up to index {}, but bit_string only {} bits",
                    index + bit_size as usize,
                    index
                );

                self.[<set_ $t>](index, data);
                Ok(())
            }
        }
    };
}

macro_rules! bit_string_from_val {
    ($t:ty) => {
        impl From<$t> for BitString {
            fn from(data: $t) -> Self {
                let bit_size = <$t>::BITS as usize;

                let mut bit_string = BitString::with_capacity(bit_size);

                let mask: $t = 0b1 << (bit_size - 1);

                for idx in 0..bit_size {
                    let mask = mask >> idx;

                    let masked = data & mask;

                    assert!(masked.count_ones() == 1 || masked.count_ones() == 0);

                    if masked.count_ones() == 1 {
                        bit_string.append_bit(Bit::On);
                    } else {
                        bit_string.append_bit(Bit::Off)
                    }
                }

                bit_string
            }
        }
    };
}

macro_rules! bit_string_from_vec {
    ($t:ty) => {
        impl From<Vec<$t>> for BitString {
            fn from(data: Vec<$t>) -> Self {
                let bit_size = <$t>::BITS as usize;

                let mut bit_string = BitString::with_capacity(bit_size * data.len());

                for el in data {
                    let mask: $t = 0b1 << (bit_size - 1);

                    for idx in 0..bit_size {
                        let mask = mask >> idx;

                        let masked = el & mask;

                        assert!(masked.count_ones() == 1 || masked.count_ones() == 0);

                        if masked.count_ones() == 1 {
                            bit_string.append_bit(Bit::On);
                        } else {
                            bit_string.append_bit(Bit::Off)
                        }
                    }
                }

                bit_string
            }
        }
        ::paste::paste! {
            impl From<&[$t]> for BitString {
                fn from(bytes: &[$t]) -> Self {
                    let bit_size = <$t>::BITS;
                    let mut bs = BitString::with_capacity(bytes.len() * bit_size as usize);

                    for byte in bytes {
                    bs.[<append_ $t>](*byte);
                    }

                    bs
                }
            }
        }
        ::paste::paste! {
            impl<const N: usize> From<[$t;N]> for BitString {
                fn from(bytes: [$t; N]) -> Self {
                    let bit_size = <$t>::BITS;
                    let mut bs = BitString::with_capacity(N * bit_size as usize);

                    for byte in bytes {
                    bs.[<append_ $t>](byte);
                    }

                    bs
                }
            }
        }

        impl From<&Vec<$t>> for BitString {
            fn from(value: &Vec<$t>) -> Self {
                value.as_slice().into()
            }
        }
    };
}

macro_rules! bit_string_as_vec {
    ($t:ty) => {
        paste::paste! {
            pub fn [<try_as_vec_exact_ $t>](&self) -> anyhow::Result<Vec<$t>> {
                ensure!(
                    self.len() % <$t>::BITS as usize == 0,
                    "Length of the bitstring was not a multiple of bit size"
                );

                Ok(self.[<as_vec_with_padding_ $t>]())
            }

            pub fn [<as_vec_exact_ $t>](&self) -> Vec<$t> {
                assert!(self.len() % <$t>::BITS as usize == 0,
                    "The bitstring is not a multiple of bit size, and can thus not be neatly made into bytes. Consider adding {} bits of padding",
                    self.len() % <$t>::BITS as usize
                );

                self.[<as_vec_with_padding_ $t>]()
            }

            pub fn [<as_vec_with_padding_ $t>](&self) -> Vec<$t> {
                let chunk_iter = self.bit_vec.chunks_exact(<$t>::BITS as usize);
                let mut byte_vec: Vec<$t> = Vec::new();
                let bit_size = <$t>::BITS as usize;

                let remainder = chunk_iter.remainder();
                for chunk in chunk_iter {
                    let mut byte: $t = 0;

                    for (idx, bit) in chunk.iter().enumerate() {
                        byte |= (*bit as $t) << (bit_size -1 - idx);
                    }

                    byte_vec.push(byte);
                }

                // This implicitly pads the last byte with zeroes
                if !remainder.is_empty() {
                    let mut byte: $t = 0;
                    for(idx,bit)in remainder.iter().enumerate(){
                        byte|= (*bit as $t)<<idx;
                    }

                    byte_vec.push(byte);
                }

                byte_vec
            }
        }
    };
}

pub(crate) use append_type;
pub(crate) use bit_into_type;
pub(crate) use bit_string_as_vec;
pub(crate) use bit_string_from_val;
pub(crate) use bit_string_from_vec;
pub(crate) use bit_try_from;
pub(crate) use get_type;
pub(crate) use insert_type;
pub(crate) use set_type;
