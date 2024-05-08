use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{bit_string::BitString, mac_address::MacAddress, physical_layer::cable::Cable};

use self::{
    bit_stuffing::prepare_bits,
    frame::{tcp::TCPFrame, Frame},
};

pub(crate) mod bit_stuffing;
pub(crate) mod frame;

pub struct DataLinkLayer<F: Frame> {
    frame_type: PhantomData<F>,
}

impl<F: Frame> Default for DataLinkLayer<F> {
    fn default() -> Self {
        Self {
            frame_type: PhantomData::<F>,
        }
    }
}

impl DataLinkLayer<TCPFrame> {
    pub fn new() -> Self {
        Self {
            frame_type: PhantomData::<TCPFrame>,
        }
    }

    pub fn data_link_layer(
        window_size: usize,
        source_mac: MacAddress,
        source_port: u16,
        target_port: u16,
        cable: Arc<Mutex<Cable>>,
        data: BitString,
    ) -> anyhow::Result<()> {
        let data: Vec<TCPFrame> = TCPFrame::setup_frames(data);

        Self::sliding_window(
            window_size,
            source_mac,
            source_port,
            target_port,
            cable,
            data,
        )
    }

    fn sliding_window(
        window_size: usize,
        source_mac: MacAddress,
        source_port: u16,
        target_port: u16,
        cable: Arc<Mutex<Cable>>,
        data: Vec<TCPFrame>,
    ) -> anyhow::Result<()> {
        let windows = data.windows(window_size);

        // TODO: Fix this implementation
        for window in windows {
            let data = window[0].to_bit_string();
            let data = prepare_bits(data);
            cable
                .lock()
                .expect("The cable should never panic")
                .send_bits(source_mac, source_port, target_port, data)?
        }

        Ok(())
    }
}
