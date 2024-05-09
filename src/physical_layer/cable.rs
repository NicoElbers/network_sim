use std::{
    rc::Rc,
    sync::{mpsc::Sender, Arc},
    thread::sleep,
    time::Duration,
};

use anyhow::bail;

use crate::{
    bit::Bit,
    bit_string::BitString,
    hardware::Node,
    utils::{corruption_type::Corruption, mac_address::MacAddress},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CableContext {
    pub bit: Bit,
    pub source_port: u16,
    pub target_port: u16,
}

#[derive(Debug)]
pub struct Cable {
    node1_mac: MacAddress,
    node2_mac: MacAddress,
    node1_transmitter: Arc<Sender<CableContext>>,
    node2_transmitter: Arc<Sender<CableContext>>,
    latency: Duration,
    corruption_type: Corruption,
    time_between_bits: Duration,
}

impl Eq for Cable {}

impl PartialEq for Cable {
    fn eq(&self, other: &Self) -> bool {
        self.node1_mac == other.node1_mac
            && self.node2_mac == other.node2_mac
            && self.latency == other.latency
            && self.corruption_type == other.corruption_type
    }
}

impl Cable {
    pub fn new(
        node1: Rc<dyn Node>,
        node2: Rc<dyn Node>,
        latency: Duration,
        corruption_type: Corruption,
        throughput_ms: u32,
    ) -> Self {
        let time_between_bytes = Duration::from_millis(1) / throughput_ms;
        let time_between_bits = time_between_bytes / 8;

        let node1_mac = *node1.get_mac();
        let node2_mac = *node2.get_mac();

        let node1_transmitter = node1.get_transmitter();
        let node2_transmitter = node2.get_transmitter();

        Self {
            node1_mac,
            node2_mac,
            node1_transmitter,
            node2_transmitter,
            latency,
            corruption_type,
            time_between_bits,
        }
    }

    pub fn send_bits(
        &mut self,
        source_mac: MacAddress,
        source_port: u16,
        target_port: u16,
        mut data: BitString,
    ) -> anyhow::Result<()> {
        let dest = if self.node1_mac == source_mac {
            self.node2_transmitter.clone()
        } else if self.node2_mac == source_mac {
            self.node1_transmitter.clone()
        } else {
            bail!("Cable does not connect these nodes")
        };

        sleep(self.latency);

        self.corruption_type.corrupt_borrow(&mut data);

        for bit in data {
            dest.send(CableContext {
                bit,
                source_port,
                target_port,
            })?;
            sleep(self.time_between_bits);
        }

        Ok(())
    }
}
