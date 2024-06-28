use super::rand::XorShift;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress {
    addr: [u8; 6],
}

impl MacAddress {
    #[must_use]
    pub const fn new(addr: [u8; 6]) -> Self {
        Self { addr }
    }
}

pub struct MacAddressGenerator {
    rand: XorShift,
}

impl MacAddressGenerator {
    #[must_use]
    pub const fn new(seed: u128) -> Self {
        let rand = XorShift::new(seed);
        Self { rand }
    }

    pub fn gen_addr(&mut self) -> MacAddress {
        let mut num = self.rand.next_int();
        let mask: u128 = u8::MAX.into(); // All ones
        let mut addr: [u8; 6] = [0, 0, 0, 0, 0, 0];

        (0..6u8).for_each(|i| {
            addr[usize::from(i)] =
                u8::try_from(num & mask).expect("Mask should disallow values above u8::MAX");
            num >>= 8;
        });

        MacAddress::new(addr)
    }
}
