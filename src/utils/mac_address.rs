use super::rand::XorShift;

#[derive(Debug, PartialEq, Eq)]
pub struct MacAddress {
    addr: [u8; 6],
}

impl MacAddress {
    pub fn new(addr: [u8; 6]) -> Self {
        Self { addr }
    }
}

pub struct MacAddressGenerator {
    rand: XorShift,
}

impl MacAddressGenerator {
    pub fn new(seed: u128) -> Self {
        let rand = XorShift::new(seed);
        Self { rand }
    }

    pub fn gen_addr(&mut self) -> MacAddress {
        let mut num = self.rand.next_int();
        let mask: u128 = u8::MAX.into(); // All ones
        let mut addr: [u8; 6] = [0, 0, 0, 0, 0, 0];

        (0..6).for_each(|i| {
            addr[i] = (num & mask) as u8;
            num >>= 8;
        });

        MacAddress::new(addr)
    }
}
