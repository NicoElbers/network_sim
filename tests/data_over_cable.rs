#[path = "utils/mod.rs"]
mod test_utils;

use crate::test_utils::test_fns::{bits_flipped, create_cable};

pub use std::time::Duration;
use std::time::Instant;

use network_sim::{corruption_type::Corruption, hardware::Node, rand::XorShift};

const ASCII_TEST_MSG: &[u8] = "Hello world!".as_bytes();
const BIN_TEST_MSG: [u8; 5] = [0x69, 0x7, 0x9, 0x2, 0x42];

#[test]
fn send_data_clean() -> anyhow::Result<()> {
    let corruption = Corruption::None;
    let (mut cable, usr1, rx1, usr2, rx2) = create_cable(Duration::ZERO, corruption, 100);

    let data = ASCII_TEST_MSG;

    cable.send_data(
        usr1.as_ref().get_mac(),
        usr2.as_ref().get_mac(),
        data.to_vec(),
    )?;

    // No pending values
    assert!(rx1.try_iter().count() == 0);

    let recv_data = rx2.try_iter().collect::<Vec<u8>>();

    // Pending hello world
    assert!(recv_data.as_slice() == ASCII_TEST_MSG);

    Ok(())
}

#[test]
fn send_data_one_flip() -> anyhow::Result<()> {
    let rand = XorShift::new(0);
    let corruption = Corruption::OneBitFlip(rand);

    let (mut cable, usr1, rx1, usr2, rx2) = create_cable(Duration::ZERO, corruption, 100);

    let data = ASCII_TEST_MSG;

    cable.send_data(
        usr1.as_ref().get_mac(),
        usr2.as_ref().get_mac(),
        data.to_vec(),
    )?;

    // No pending values
    assert!(rx1.try_iter().count() == 0);

    let recv_data = rx2.try_iter().collect::<Vec<u8>>();

    assert!(bits_flipped(data, recv_data.as_slice()) == 1);

    Ok(())
}

#[test]
fn correct_latency() -> anyhow::Result<()> {
    let corruption = Corruption::None;

    let latency_ms = 10;
    let latency = Duration::from_millis(latency_ms);

    let data = ASCII_TEST_MSG;

    let (mut cable, usr1, _rx1, usr2, rx2) = create_cable(latency, corruption, 100);

    let start = Instant::now();
    cable.send_data(
        usr1.as_ref().get_mac(),
        usr2.as_ref().get_mac(),
        data.to_vec(),
    )?;

    for _ in 0..data.len() {
        assert!(rx2.recv_timeout(Duration::from_millis(100)).is_ok());
    }

    assert!(start.elapsed() >= latency);

    Ok(())
}

#[test]
fn correct_throughput() -> anyhow::Result<()> {
    let corruption = Corruption::None;

    let latency_ms = 0;
    let latency = Duration::from_millis(latency_ms);

    let throughput_per_ms = 1;
    let time_per_byte = Duration::from_millis(1) / throughput_per_ms;

    let data = ASCII_TEST_MSG;

    let (mut cable, usr1, _rx1, usr2, rx2) = create_cable(latency, corruption, throughput_per_ms);

    let start = Instant::now();
    cable.send_data(
        usr1.as_ref().get_mac(),
        usr2.as_ref().get_mac(),
        data.to_vec(),
    )?;

    for _ in 0..data.len() {
        assert!(rx2.recv_timeout(Duration::from_millis(100)).is_ok());
    }

    assert!(start.elapsed() >= time_per_byte * data.len().try_into().unwrap());

    Ok(())
}
