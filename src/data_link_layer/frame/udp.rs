use crate::bit_string::BitString;

use super::Frame;

#[derive(Debug)]
pub struct UDPFrame {
    bs: BitString,
}

impl UDPFrame {
    pub fn new(data: BitString) -> Self {
        unimplemented!("Implement UDP mfer");
        Self { bs: data }
    }
}

// TODO: Do this implementation
impl Frame for UDPFrame {
    fn setup_frames(data: BitString) -> Vec<Self> {
        unimplemented!("Implement UDP mfer");
        vec![Self { bs: data }]
    }

    fn to_bit_string(&self) -> BitString {
        unimplemented!("Implement UDP mfer");
        self.bs.clone()
    }
}
