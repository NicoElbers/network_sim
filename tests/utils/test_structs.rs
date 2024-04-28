use std::{
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use network_sim::{
    hardware::Node,
    mac_address::{MacAddress, MacAddressGenerator},
    physical_layer::cable::Cable,
};

#[derive(Debug)]
pub(crate) struct TestUser {
    mac: MacAddress,
    connections: Vec<Rc<Cable>>,
    byte_sender: Sender<u8>,
}

impl TestUser {
    pub fn new(mac_address_gen: &mut MacAddressGenerator) -> (Self, Receiver<u8>) {
        let mac = mac_address_gen.gen_addr();

        let (tx, rx) = channel::<u8>();

        let usr = Self {
            mac,
            connections: Vec::new(),
            byte_sender: tx,
        };

        (usr, rx)
    }
}

impl Node for TestUser {
    fn add_connection(&mut self, cable: Rc<Cable>) {
        if self.connections.contains(&cable) {
            return;
        }
        self.connections.push(cable.clone());
    }

    fn get_connections(&self) -> &Vec<Rc<Cable>> {
        &self.connections
    }

    fn get_mac(&self) -> &MacAddress {
        &self.mac
    }

    fn receive_byte(&self, byte: u8) {
        self.byte_sender.send(byte).unwrap()
    }
}
