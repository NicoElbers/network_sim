use std::{rc::Rc, thread::sleep, time::Duration};

use anyhow::bail;

use crate::{
    hardware::Node,
    utils::{corruption_type::Corruption, mac_address::MacAddress},
};

#[derive(Debug, Eq)]
pub struct Cable {
    node1: Rc<dyn Node>,
    node2: Rc<dyn Node>,
    latency: Duration,
    corruption_type: Corruption,
    time_between_bytes: Duration,
}

impl PartialEq for Cable {
    fn eq(&self, other: &Self) -> bool {
        self.node1.get_mac() == other.node1.get_mac()
            && self.node2.get_mac() == other.node2.get_mac()
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

        Self {
            node1,
            node2,
            latency,
            corruption_type,
            time_between_bytes,
        }
    }

    pub fn send_data(
        &mut self,
        src: &MacAddress,
        dest: &MacAddress,
        mut data: Vec<u8>,
    ) -> anyhow::Result<()> {
        if src == dest {
            bail!("Cannot send to self");
        }

        let dest = if self.node1.get_mac() == src && self.node2.get_mac() == dest {
            self.node2.clone()
        } else if self.node1.get_mac() == src && self.node2.get_mac() == dest {
            self.node1.clone()
        } else {
            bail!("Cable does not connect these nodes")
        };

        sleep(self.latency);

        let data = self.corruption_type.corrupt_borrow(data.as_mut_slice());

        for byte in data {
            dest.receive_byte(*byte);
            sleep(self.time_between_bytes);
        }

        Ok(())
    }
}
