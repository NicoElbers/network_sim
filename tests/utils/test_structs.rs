use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use network_sim::{
    hardware::Node,
    mac_address::{MacAddress, MacAddressGenerator},
    physical_layer::cable::{Cable, CableContext},
};

#[derive(Debug)]
pub struct TestUser {
    mac: MacAddress,
    connections: Vec<Arc<Cable>>,
    receiver: Receiver<CableContext>,
    sender: Arc<Sender<CableContext>>,
}

impl TestUser {
    pub fn new(mac_address_gen: &mut MacAddressGenerator) -> Self {
        let mac = mac_address_gen.gen_addr();

        let (tx, rx) = channel::<CableContext>();
        let sender = Arc::new(tx);
        let receiver = rx;

        Self {
            mac,
            connections: Vec::new(),
            sender,
            receiver,
        }
    }
}

impl TestUser {
    pub const fn get_receiver(&self) -> &Receiver<CableContext> {
        &self.receiver
    }
}

impl Node for TestUser {
    fn add_connection(&mut self, cable: Arc<Cable>) {
        if self.connections.contains(&cable) {
            return;
        }
        self.connections.push(cable);
    }

    fn get_transmitter(&self) -> Arc<Sender<CableContext>> {
        self.sender.clone()
    }

    fn get_connections(&self) -> &Vec<Arc<Cable>> {
        &self.connections
    }

    fn get_mac(&self) -> &MacAddress {
        &self.mac
    }
}
