use std::{
    fmt::Debug,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

use easy_threadpool::ThreadPool;

use crate::{
    bit_string::BitString,
    physical_layer::cable::{Cable, CableContext},
    utils::mac_address::{MacAddress, MacAddressGenerator},
};

pub trait Node: Debug {
    fn get_mac(&self) -> &MacAddress;

    fn get_transmitter(&self) -> Arc<Sender<CableContext>>;

    fn add_connection(&mut self, cable: Arc<Cable>);

    fn get_connections(&self) -> &Vec<Arc<Cable>>;

    fn send_bit_mac(
        source_mac: MacAddress,
        source_port: u16,
        target_port: u16,
        cable: &mut Cable,
        data: BitString,
    ) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        // cable.send_bits(src, dest, data)
        cable.send_bits(source_mac, source_port, target_port, data)
    }
}

impl PartialEq for dyn Node {
    fn eq(&self, other: &Self) -> bool {
        self.get_mac() == other.get_mac()
    }
}

impl Eq for dyn Node {}

#[derive(Debug)]
pub struct Router {
    mac: MacAddress,
    connections: Vec<Arc<Cable>>,
    receiver: Receiver<CableContext>,
    transmitter: Arc<Sender<CableContext>>,
    is_edge_router: bool,
    runtime: ThreadPool,
}

impl Node for Router {
    fn add_connection(&mut self, cable: Arc<Cable>) {
        if self.connections.contains(&cable) {
            return;
        }
        self.connections.push(cable);
    }

    fn get_connections(&self) -> &Vec<Arc<Cable>> {
        &self.connections
    }

    fn get_mac(&self) -> &MacAddress {
        &self.mac
    }

    fn get_transmitter(&self) -> Arc<Sender<CableContext>> {
        self.transmitter.clone()
    }
}

impl Router {
    pub fn new(
        is_edge_router: bool,
        mac_address_gen: &mut MacAddressGenerator,
        threadpool: ThreadPool,
    ) -> Self {
        let mac = mac_address_gen.gen_addr();

        let (tx, rx) = channel::<CableContext>();
        let transmitter = tx.into();

        Self {
            mac,
            transmitter,
            receiver: rx,
            connections: Vec::new(),
            is_edge_router,
            runtime: threadpool,
        }
    }

    #[must_use]
    pub const fn is_edge_router(&self) -> bool {
        self.is_edge_router
    }
}

#[derive(Debug)]
pub struct User {
    mac: MacAddress,
    connections: Vec<Arc<Cable>>,
    receiver: Receiver<CableContext>,
    transmitter: Arc<Sender<CableContext>>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.mac == other.mac && self.connections == other.connections
    }
}

impl User {
    pub fn new(mac_address_gen: &mut MacAddressGenerator) -> Self {
        let mac = mac_address_gen.gen_addr();

        let (tx, rx) = channel::<CableContext>();
        let transmitter = tx.into();

        Self {
            mac,
            connections: Vec::new(),
            transmitter,
            receiver: rx,
        }
    }
}

impl Node for User {
    fn add_connection(&mut self, cable: Arc<Cable>) {
        if self.connections.contains(&cable) {
            return;
        }
        self.connections.push(cable);
    }

    fn get_connections(&self) -> &Vec<Arc<Cable>> {
        &self.connections
    }

    fn get_mac(&self) -> &MacAddress {
        &self.mac
    }

    fn get_transmitter(&self) -> Arc<Sender<CableContext>> {
        self.transmitter.clone()
    }
}
