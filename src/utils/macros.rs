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

macro_rules! bit_string_from {
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

pub(crate) use append_type;
pub(crate) use bit_into_type;
pub(crate) use bit_string_from;
pub(crate) use bit_try_from;
pub(crate) use get_type;
pub(crate) use set_type;
