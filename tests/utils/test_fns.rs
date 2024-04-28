use std::{rc::Rc, sync::mpsc::Receiver, time::Duration};

use network_sim::{
    corruption_type::Corruption, mac_address::MacAddressGenerator, physical_layer::cable::Cable,
};

use super::test_structs::TestUser;

pub fn bits_flipped<'a>(data1: &'a [u8], data2: &'a [u8]) -> u32 {
    assert_eq!(data1.len(), data2.len());

    let mut difference: u32 = 0;
    for i in 0..data1.len() {
        difference += (data1[i] ^ data2[i]).count_ones();
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
    Receiver<u8>,
    Rc<TestUser>,
    Receiver<u8>,
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
