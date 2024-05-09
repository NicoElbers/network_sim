use crate::{bit::Bit, bit_string::BitString};

use super::Frame;

const MAX_TCP_HEADER_LEN: usize = 60;
const MAX_TCP_DATA_LEN: usize = u16::MAX as usize - MAX_TCP_HEADER_LEN;

// Flags
#[allow(dead_code)]
pub const FIN: u8 = 0b1 << 0;
#[allow(dead_code)]
pub const SYN: u8 = 0b1 << 1;
#[allow(dead_code)]
pub const RST: u8 = 0b1 << 2;
#[allow(dead_code)]
pub const PSH: u8 = 0b1 << 3;
#[allow(dead_code)]
pub const ACK: u8 = 0b1 << 4;
#[allow(dead_code)]
pub const URG: u8 = 0b1 << 5;
#[allow(dead_code)]
pub const ECE: u8 = 0b1 << 6;
#[allow(dead_code)]
pub const CWR: u8 = 0b1 << 7;

#[derive(Debug, Clone)]
pub struct TCPFrameBuilder {
    // Header
    source_port: Option<u16>,
    target_port: Option<u16>,
    sequence_num: Option<u32>,
    ack_num: u32,
    data_offset: u8,
    flag_byte: u8,
    window_size: Option<u16>,
    urgent_pointer: u16,
    options: [u32; 9],

    // Data
    data: Option<BitString>,
}

impl TCPFrameBuilder {
    pub fn new() -> Self {
        Self {
            source_port: None,
            target_port: None,
            sequence_num: None,
            ack_num: 0,
            data_offset: 5,
            flag_byte: 0,
            window_size: None,
            urgent_pointer: 0,
            options: [0; 9],
            data: None,
        }
    }

    pub fn build_all(mut self, data_points: Vec<BitString>) -> Vec<TCPFrame> {
        assert!(self.source_port.is_some());
        assert!(self.target_port.is_some());
        assert!(self.window_size.is_some());

        assert!(
            data_points.len() < u32::MAX as usize,
            "Cannot support data transfers of more than {} packets",
            u32::MAX
        );

        self.data = None;

        let mut res_vec = Vec::new();

        for (idx, data) in data_points.iter().enumerate() {
            let data = data.clone();
            self.sequence_num = Some(idx as u32);
            res_vec.push(self.build(data));
        }

        res_vec
    }

    fn build(&self, data: BitString) -> TCPFrame {
        assert!(self.source_port.is_some());
        assert!(self.target_port.is_some());
        assert!(self.window_size.is_some());
        assert!(self.sequence_num.is_some());
        assert!(self.data.is_some());

        // All these values have been asserted to be present
        let source_port = self.source_port.unwrap();
        let target_port = self.target_port.unwrap();
        let window_size = self.window_size.unwrap();
        let sequence_num = self.sequence_num.unwrap();

        let data_offset = self.data_offset;
        let ack_num = self.ack_num;
        let flag_byte = self.flag_byte;
        let urgent_pointer = self.urgent_pointer;
        let options = self.options;

        let mut full_bitstring = BitString::with_capacity(data_offset as usize * 32 + data.len());

        full_bitstring.append_u16(source_port);
        full_bitstring.append_u16(target_port);
        full_bitstring.append_u32(sequence_num);
        full_bitstring.append_u32(ack_num);
        full_bitstring.append_u8(data_offset);
        full_bitstring.append_u8(flag_byte);
        full_bitstring.append_u16(window_size);
        // Checksum defaults to zero
        full_bitstring.append_u16(0);
        full_bitstring.append_u16(urgent_pointer);

        if let Some(words) = data_offset.checked_sub(5) {
            for i in 0..words {
                full_bitstring.append_u32(options[i as usize]);
            }
        }

        full_bitstring.append_bits(data.as_bit_slice());

        // pad with zeros
        for _ in 0..(full_bitstring.len() % 16) {
            full_bitstring.append_bit(Bit::Off);
        }

        assert!(
            full_bitstring.len() % 16 == 0,
            "The full bitstring wasn't padded correctly"
        );

        // -- Find checksum --
        let vec = full_bitstring.as_vec_exact_u16();
        let sum: u32 = vec.iter().map(|&x| x as u32).sum();

        let mut checksum = (sum >> 16) + (sum & 0xFFFF);
        while checksum > u16::MAX as u32 {
            checksum = (checksum >> 16) + (checksum & 0xFFFF);
        }

        let checksum: u16 = !(checksum as u16);

        full_bitstring.set_u16(128, checksum);

        TCPFrame {
            source_port,
            target_port,
            sequence_num,
            ack_num,
            data_offset,
            flag_byte,
            window_size,
            checksum,
            urgent_pointer,
            options,
            data,
            output_bitstring: full_bitstring,
        }
    }

    pub fn set_source_port(self, source_port: u16) -> Self {
        Self {
            source_port: Some(source_port),
            ..self
        }
    }

    pub fn set_target_port(self, target_port: u16) -> Self {
        Self {
            target_port: Some(target_port),
            ..self
        }
    }

    pub fn set_sequence_num(self, sequence_num: u32) -> Self {
        Self {
            sequence_num: Some(sequence_num),
            ..self
        }
    }

    pub fn set_ack_num(self, ack_num: u32) -> Self {
        Self { ack_num, ..self }
    }

    pub fn set_data_offset(self, data_offset: u8) -> Self {
        Self {
            data_offset,
            ..self
        }
    }

    pub fn set_flag(self, flag: u8) -> Self {
        let mut flag_byte = self.flag_byte;

        flag_byte |= flag;

        Self { flag_byte, ..self }
    }

    pub fn set_window_size(self, window_size: u16) -> Self {
        Self {
            window_size: Some(window_size),
            ..self
        }
    }

    pub fn set_urgent_pointer(self, urgent_pointer: u16) -> Self {
        Self {
            urgent_pointer,
            ..self
        }
    }

    pub fn set_options(self, options: [u32; 9]) -> Self {
        Self { options, ..self }
    }
}

impl Default for TCPFrameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TCPFrame {
    // Header
    source_port: u16,
    target_port: u16,
    sequence_num: u32,
    ack_num: u32,
    data_offset: u8,
    flag_byte: u8,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
    options: [u32; 9],

    // Data
    data: BitString,

    // Full bit_string, since it already had to be calculated for the checksum
    output_bitstring: BitString,
}

impl Frame<TCPFrameBuilder> for TCPFrame {
    fn setup_frames(data: BitString, builder: TCPFrameBuilder) -> Vec<Self> {
        let chunks = data.as_bit_slice().chunks(MAX_TCP_DATA_LEN);

        let mut bundled_data: Vec<BitString> = Vec::new();

        for chunk in chunks {
            let data_point = BitString::from(chunk);
            bundled_data.push(data_point);
        }

        builder.build_all(bundled_data)
    }

    fn to_bit_string(&self) -> BitString {
        self.output_bitstring.clone()
    }
}