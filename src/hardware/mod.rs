use std::{fmt::Debug, rc::Rc};

use crate::{
    physical_layer::cable::Cable,
    utils::mac_address::{MacAddress, MacAddressGenerator},
};

pub trait Node: Debug {
    fn get_mac(&self) -> &MacAddress;

    fn add_connection(&mut self, cable: Rc<Cable>);

    fn get_connections(&self) -> &Vec<Rc<Cable>>;

    fn receive_byte(&self, byte: u8);

    fn send_byte_mac(
        src: &MacAddress,
        dest: &MacAddress,
        cable: &mut Cable,
        data: Vec<u8>,
    ) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        cable.send_data(src, dest, data)
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
    connections: Vec<Rc<Cable>>,
    is_edge_router: bool,
}

impl Node for Router {
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

    #[allow(dead_code)]
    fn receive_byte(&self, _byte: u8) {
        todo!()
    }
}

impl Router {
    pub fn new(is_edge_router: bool, mac_address_gen: &mut MacAddressGenerator) -> Self {
        let mac = mac_address_gen.gen_addr();

        Self {
            mac,
            connections: Vec::new(),
            is_edge_router,
        }
    }

    pub fn is_edge_router(&self) -> bool {
        self.is_edge_router
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    mac: MacAddress,
    connections: Vec<Rc<Cable>>,
}

impl User {
    pub fn new(mac_address_gen: &mut MacAddressGenerator) -> Self {
        let mac = mac_address_gen.gen_addr();

        Self {
            mac,
            connections: Vec::new(),
        }
    }
}

impl Node for User {
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

    #[allow(dead_code)]
    fn receive_byte(&self, _byte: u8) {
        todo!()
    }
}
