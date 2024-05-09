use crate::bit_string::BitString;

use super::Frame;

#[allow(dead_code)]
pub struct UDPBuilder {}

#[derive(Debug)]
pub struct UDPFrame {
    bs: BitString,
}

#[allow(dead_code)]
impl UDPFrame {
    pub fn new(_data: BitString) -> Self {
        unimplemented!("Implement UDP mfer");
        Self { bs: _data }
    }
}

// TODO: Do this implementation
#[allow(dead_code)]
impl Frame<UDPFrame> for UDPFrame {
    fn setup_frames(_data: BitString, _builder: UDPFrame) -> Vec<Self> {
        unimplemented!("Implement UDP mfer");
        vec![Self { bs: _data }]
    }

    fn to_bit_string(&self) -> BitString {
        unimplemented!("Implement UDP mfer");
        self.bs.clone()
    }
}
