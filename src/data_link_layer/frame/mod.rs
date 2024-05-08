use crate::bit_string::BitString;

pub mod tcp;
pub mod udp;

pub trait Frame {
    fn setup_frames(data: BitString) -> Vec<Self>
    where
        Self: Sized;

    fn to_bit_string(&self) -> BitString;
}
