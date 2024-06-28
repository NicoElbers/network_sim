pub(crate) mod bit_stuffing;
pub(crate) mod crc;
pub(crate) mod frame;

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{bit_string::BitString, mac_address::MacAddress, physical_layer::cable::Cable};

use self::{
    bit_stuffing::prepare_bits,
    frame::{
        tcp::{TCPFrame, TCPFrameBuilder},
        Frame,
    },
};

pub struct DataLinkLayer<B, F: Frame<B>> {
    frame_type: PhantomData<F>,
    builder_type: PhantomData<B>,
}

impl<B, F: Frame<B>> Default for DataLinkLayer<B, F> {
    fn default() -> Self {
        Self {
            frame_type: PhantomData::<F>,
            builder_type: PhantomData::<B>,
        }
    }
}

impl DataLinkLayer<TCPFrameBuilder, TCPFrame> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            frame_type: PhantomData::<TCPFrame>,
            builder_type: PhantomData::<TCPFrameBuilder>,
        }
    }

    pub fn send_bits(
        window_size: u16,
        source_mac: MacAddress,
        source_port: u16,
        target_port: u16,
        cable: &Arc<Mutex<Cable>>,
        data: BitString,
    ) -> anyhow::Result<()> {
        let tcp_builder = TCPFrameBuilder::new()
            .set_source_port(source_port)
            .set_target_port(target_port)
            .set_window_size(window_size);

        let data: Vec<TCPFrame> = TCPFrame::setup_frames(data, tcp_builder);

        Self::sliding_window(
            window_size,
            source_mac,
            source_port,
            target_port,
            cable,
            &data,
        )
    }

    fn sliding_window(
        window_size: u16,
        source_mac: MacAddress,
        source_port: u16,
        target_port: u16,
        cable: &Arc<Mutex<Cable>>,
        data: &[TCPFrame],
    ) -> anyhow::Result<()> {
        let windows = data.windows(window_size.into());

        // TODO: Fix this implementation
        for window in windows {
            let data = window[0].as_bit_string().clone();
            let data = prepare_bits(data);
            cable
                .lock()
                .expect("The cable should never panic")
                .send_bits(source_mac, source_port, target_port, data)?;
        }

        Ok(())
    }
}
