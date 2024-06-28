use std::{sync::Arc, time::Duration};

use network_sim::{
    bit::Bit,
    bit_string::BitString,
    corruption_type::Corruption,
    mac_address::MacAddressGenerator,
    physical_layer::cable::{Cable, CableContext},
};

use super::test_structs::TestUser;

pub fn bits_flipped_slice_bit_vec(slice: &[u8], vec: &[CableContext]) -> u32 {
    let slice_bs: BitString = slice.into();
    let vec_bs: BitString = vec
        .iter() //
        .map(|cc| cc.bit)
        .collect::<Vec<Bit>>()
        .into();

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
) -> (Cable, Arc<TestUser>, Arc<TestUser>) {
    let mut mac_gen = MacAddressGenerator::new(6969);

    let node1 = TestUser::new(&mut mac_gen);
    let node2 = TestUser::new(&mut mac_gen);

    let node1 = Arc::new(node1);
    let node2 = Arc::new(node2);

    let cable = Cable::new(&node1, &node2, latency, corruption_type, throughput_ms);

    (cable, node1, node2)
}

pub fn equals_bit_vec_and_byte_slice(vec: &[CableContext], slice: &[u8]) -> bool {
    let recv_bs: BitString = vec.iter().map(|cc| cc.bit).collect::<Vec<Bit>>().into();
    let test_msg_bs: BitString = slice.into();

    recv_bs == test_msg_bs
}
