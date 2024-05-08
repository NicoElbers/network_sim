use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use network_sim::{
    bit::Bit,
    hardware::Node,
    mac_address::{MacAddress, MacAddressGenerator},
    physical_layer::cable::Cable,
};

#[derive(Debug)]
pub(crate) struct TestUser {
    mac: MacAddress,
    connections: Vec<Arc<Cable>>,
    byte_sender: Sender<Bit>,
}

impl TestUser {
    pub fn new(mac_address_gen: &mut MacAddressGenerator) -> (Self, Receiver<Bit>) {
        let mac = mac_address_gen.gen_addr();

        let (tx, rx) = channel::<Bit>();

        let usr = Self {
            mac,
            connections: Vec::new(),
            byte_sender: tx,
        };

        (usr, rx)
    }
}

impl Node for TestUser {
    fn add_connection(&mut self, cable: Arc<Cable>) {
        if self.connections.contains(&cable) {
            return;
        }
        self.connections.push(cable.clone());
    }

    fn get_transmitter(&self) -> Arc<Sender<network_sim::physical_layer::cable::CableContext>> {
        unimplemented!()
    }

    fn get_connections(&self) -> &Vec<Arc<Cable>> {
        &self.connections
    }

    fn get_mac(&self) -> &MacAddress {
        &self.mac
    }

    fn receive_bit(&self, bit: Bit) {
        self.byte_sender.send(bit).unwrap()
    }
}
