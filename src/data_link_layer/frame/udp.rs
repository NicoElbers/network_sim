use crate::bit_string::BitString;

use super::Frame;

#[allow(dead_code)]
pub struct UDPBuilder {}

#[derive(Debug)]
pub struct UDPFrame {}

#[allow(dead_code)]
impl UDPFrame {
    pub fn new(_data: &BitString) -> Self {
        unimplemented!("Implement UDP mfer");
    }
}

// TODO: Do this implementation
#[allow(dead_code)]
impl Frame<Self> for UDPFrame {
    fn setup_frames(_data: BitString, _builder: Self) -> Vec<Self> {
        unimplemented!("Implement UDP mfer");
    }

    fn as_bit_string(&self) -> &BitString {
        unimplemented!("Implement UDP mfer");
    }
}
