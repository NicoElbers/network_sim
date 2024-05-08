use std::{rc::Rc, sync::mpsc::Receiver, time::Duration};

use network_sim::{
    bit::Bit, bit_string::BitString, corruption_type::Corruption, mac_address::MacAddressGenerator,
    physical_layer::cable::Cable,
};

use super::test_structs::TestUser;

pub fn bits_flipped_slice_bit_vec(slice: &[u8], vec: &Vec<Bit>) -> u32 {
    let slice_bs: BitString = slice.into();
    let vec_bs: BitString = vec.into();

    let mut difference: u32 = 0;
    for i in 0..slice_bs.len() {
        difference += (slice_bs[i] ^ vec_bs[i]) as u32;
    }

    difference
}

pub fn create_cable(
    latency: Duration,
    corruption_type: Corruption,
    throughput_ms: u32,
) -> (
    Cable,
    Rc<TestUser>,
    Receiver<Bit>,
    Rc<TestUser>,
    Receiver<Bit>,
) {
    let mut mac_gen = MacAddressGenerator::new(6969);

    let (node1, rx1) = TestUser::new(&mut mac_gen);
    let (node2, rx2) = TestUser::new(&mut mac_gen);

    let node1 = Rc::new(node1);
    let node2 = Rc::new(node2);

    let cable = Cable::new(
        node1.clone(),
        node2.clone(),
        latency,
        corruption_type,
        throughput_ms,
    );

    (cable, node1, rx1, node2, rx2)
}

pub fn equals_bit_vec_and_byte_slice(vec: Vec<Bit>, slice: &[u8]) -> bool {
    let recv_bs: BitString = vec.as_slice().into();
    let test_msg_bs: BitString = slice.into();

    recv_bs == test_msg_bs
}
