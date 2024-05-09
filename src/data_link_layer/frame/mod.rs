use crate::bit_string::BitString;

pub mod tcp;
pub mod udp;

pub trait Frame<T> {
    fn setup_frames(data: BitString, builder: T) -> Vec<Self>
    where
        Self: Sized;

    fn as_bit_string(&self) -> &BitString;
}
