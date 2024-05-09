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
    sequence_num: u32,
    ack_num: u32,
    data_offset: u8,
    flag_byte: u8,
    window_size: Option<u16>,
    urgent_pointer: u16,
    options: [u32; 10],
}

impl TCPFrameBuilder {
    pub fn new() -> Self {
        Self {
            source_port: None,
            target_port: None,
            sequence_num: 0,
            ack_num: 0,
            data_offset: 5,
            flag_byte: 0,
            window_size: None,
            urgent_pointer: 0,
            options: [0; 10],
        }
    }

    pub fn build_all(mut self, data_points: &[BitString]) -> Vec<TCPFrame> {
        assert!(self.source_port.is_some());
        assert!(self.target_port.is_some());
        assert!(self.window_size.is_some());

        assert!(
            data_points.len() < u32::MAX as usize,
            "Cannot support data transfers of more than {} packets",
            u32::MAX
        );

        let mut res_vec = Vec::new();

        for (idx, data) in data_points.iter().enumerate() {
            let data = data.clone();
            self.sequence_num = idx as u32;
            res_vec.push(self.build(data));
        }

        res_vec
    }

    fn build(&self, data: BitString) -> TCPFrame {
        assert!(self.source_port.is_some());
        assert!(self.target_port.is_some());
        assert!(self.window_size.is_some());

        // All these values have been asserted to be present
        let source_port = self.source_port.unwrap();
        let target_port = self.target_port.unwrap();
        let window_size = self.window_size.unwrap();

        let sequence_num = self.sequence_num;
        let ack_num = self.ack_num;
        let data_offset = self.data_offset;
        let flag_byte = self.flag_byte;
        let urgent_pointer = self.urgent_pointer;
        let options = self.options;

        let mut output_bitstring = BitString::with_capacity(data_offset as usize * 32 + data.len());

        output_bitstring.append_u16(source_port);
        output_bitstring.append_u16(target_port);
        output_bitstring.append_u32(sequence_num);
        output_bitstring.append_u32(ack_num);
        output_bitstring.append_u8(data_offset << 4); // We must shift this because we don't
                                                      // have a u4
        output_bitstring.append_u8(flag_byte);
        output_bitstring.append_u16(window_size);
        // Checksum defaults to zero
        output_bitstring.append_u16(0);
        output_bitstring.append_u16(urgent_pointer);

        if let Some(words) = data_offset.checked_sub(5) {
            for i in 0..words {
                output_bitstring.append_u32(options[i as usize]);
            }
        }

        output_bitstring.append_bits(data.as_bit_slice());

        // pad with zeros
        for _ in 0..(output_bitstring.len() % 16) {
            output_bitstring.append_bit(Bit::Off);
        }

        assert!(
            output_bitstring.len() % 16 == 0,
            "The full bitstring wasn't padded correctly"
        );

        // -- Find checksum --
        let vec = output_bitstring.as_vec_exact_u16();
        let mut sum: u32 = vec.iter().map(|&x| x as u32).sum();

        while sum > 0xFFFF {
            sum = (sum >> 16) + (sum & 0xFFFF);
        }

        let checksum: u16 = !(sum as u16);

        output_bitstring.set_u16(128, checksum);
        assert_eq!(
            BitString::from(output_bitstring.get_u16(128)),
            BitString::from(checksum),
            "AAAAAAaa"
        );

        assert!(output_bitstring.len() % 16 == 0);

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
            output_bitstring,
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

    // pub fn set_sequence_num(self, sequence_num: u32) -> Self {
    //     Self {
    //         sequence_num: Some(sequence_num),
    //         ..self
    //     }
    // }

    pub fn set_ack_num(self, ack_num: u32) -> Self {
        Self { ack_num, ..self }
    }

    pub fn set_data_offset(self, data_offset: u8) -> Self {
        assert!(data_offset <= 0b0000_1111u8);
        Self {
            data_offset,
            ..self
        }
    }

    pub fn set_flags(self, flag: u8) -> Self {
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

    pub fn set_options(self, options: [u32; 10]) -> Self {
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
    options: [u32; 10],

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

        builder.build_all(&bundled_data)
    }

    fn as_bit_string(&self) -> &BitString {
        &self.output_bitstring
    }
}

#[cfg(test)]
mod test {
    use crate::bit_string::BitString;

    use super::{TCPFrame, TCPFrameBuilder};

    // Given
    const SOURCE_PORT: u16 = 0b1111_1111_1111_1111u16;
    const TARGET_PORT: u16 = 0b0000_0000_0000_0000u16;
    const ACK_NUM: u32 = 0b1111_0000_1111_0000_1111_0000_1111_0000u32;
    const DATA_OFFSET: u8 = 0b0000_1111u8;
    const FLAG: u8 = 0b0101_0101u8;
    const WINDOW_SIZE: u16 = 0b0011_1100_0011_1100u16;
    const URGENT_POINTER: u16 = 0b1100_0011_1100_0011u16;
    const OPTIONS: [u32; 10] = [1, 0, 1, 0, 1, 0, 1, 0, 1, 0];

    // Assumed
    const SEQUENCE_NUM1: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000u32;
    const SEQUENCE_NUM2: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001u32;

    // Hand calculated
    const CHECKSUM1: u16 = 0b0010_1101_1100_0011;
    const CHECKSUM2: u16 = 0b0010_1101_1100_0010;

    // Datapoints
    const DATA: [u128; 2] = [0b10110010101110100100101001011011011010010010100101101011101010101001010100101010110111010101010010101001010101110101010010101010u128,
                             0b10011010100101110100100101010010101010110101001010100101111101010101001010101001010100101011001010101101010110010011001100001101u128];

    fn headers() -> Vec<TCPFrame> {
        let data_points = [BitString::new(), BitString::new()];

        let builder = TCPFrameBuilder::new()
            .set_source_port(SOURCE_PORT)
            .set_target_port(TARGET_PORT)
            .set_ack_num(ACK_NUM)
            .set_data_offset(DATA_OFFSET)
            .set_flags(FLAG)
            .set_window_size(WINDOW_SIZE)
            .set_urgent_pointer(URGENT_POINTER)
            .set_options(OPTIONS);

        builder.build_all(&data_points)
    }

    fn headers_with_data(data_points: &[BitString]) -> Vec<TCPFrame> {
        let builder = TCPFrameBuilder::new()
            .set_source_port(SOURCE_PORT)
            .set_target_port(TARGET_PORT)
            .set_ack_num(ACK_NUM)
            .set_data_offset(DATA_OFFSET)
            .set_flags(FLAG)
            .set_window_size(WINDOW_SIZE)
            .set_urgent_pointer(URGENT_POINTER)
            .set_options(OPTIONS);

        builder.build_all(data_points)
    }

    #[test]
    fn basic_header() {
        let headers = headers();

        assert_eq!(headers.len(), 2);

        let header1 = &headers[0];
        assert_eq!(header1.source_port, SOURCE_PORT, "Failed at source_port");
        assert_eq!(header1.target_port, TARGET_PORT, "Failed at target_port");
        assert_eq!(
            header1.sequence_num, SEQUENCE_NUM1,
            "Failed at sequence_num1"
        );
        assert_eq!(header1.ack_num, ACK_NUM, "Failed at ack_num");
        assert_eq!(header1.data_offset, DATA_OFFSET, "Failed at data_offset");
        assert_eq!(header1.flag_byte, FLAG, "Failed at flag");
        assert_eq!(header1.window_size, WINDOW_SIZE, "Failed at window_size");
        assert_eq!(header1.checksum, CHECKSUM1, "Failed at checksum1");
        assert_eq!(
            header1.urgent_pointer, URGENT_POINTER,
            "Failed at urgent_pointer"
        );
        assert_eq!(header1.options[0], OPTIONS[0], "Failed at options[0]");
        assert_eq!(header1.options[1], OPTIONS[1], "Failed at options[1]");
        assert_eq!(header1.options[2], OPTIONS[2], "Failed at options[2]");
        assert_eq!(header1.options[3], OPTIONS[3], "Failed at options[3]");
        assert_eq!(header1.options[4], OPTIONS[4], "Failed at options[4]");
        assert_eq!(header1.options[5], OPTIONS[5], "Failed at options[5]");
        assert_eq!(header1.options[6], OPTIONS[6], "Failed at options[6]");
        assert_eq!(header1.options[7], OPTIONS[7], "Failed at options[7]");
        assert_eq!(header1.options[8], OPTIONS[8], "Failed at options[8]");
        assert_eq!(header1.options[9], OPTIONS[9], "Failed at options[9]");

        let header2 = &headers[1];
        assert_eq!(header2.source_port, SOURCE_PORT, "Failed at source_port");
        assert_eq!(header2.target_port, TARGET_PORT, "Failed at target_port");
        assert_eq!(
            header2.sequence_num, SEQUENCE_NUM2,
            "Failed at sequence_num2"
        );
        assert_eq!(header2.ack_num, ACK_NUM, "Failed at ack_num");
        assert_eq!(header2.data_offset, DATA_OFFSET, "Failed at data_offset");
        assert_eq!(header2.flag_byte, FLAG, "Failed at flag");
        assert_eq!(header2.window_size, WINDOW_SIZE, "Failed at window_size");
        assert_eq!(header2.checksum, CHECKSUM2, "Failed at checksum2");
        assert_eq!(
            header2.urgent_pointer, URGENT_POINTER,
            "Failed at urgent_pointer"
        );
        assert_eq!(header2.options[0], OPTIONS[0], "Failed at options[0]");
        assert_eq!(header2.options[1], OPTIONS[1], "Failed at options[1]");
        assert_eq!(header2.options[2], OPTIONS[2], "Failed at options[2]");
        assert_eq!(header2.options[3], OPTIONS[3], "Failed at options[3]");
        assert_eq!(header2.options[4], OPTIONS[4], "Failed at options[4]");
        assert_eq!(header2.options[5], OPTIONS[5], "Failed at options[5]");
        assert_eq!(header2.options[6], OPTIONS[6], "Failed at options[6]");
        assert_eq!(header2.options[7], OPTIONS[7], "Failed at options[7]");
        assert_eq!(header2.options[8], OPTIONS[8], "Failed at options[8]");
        assert_eq!(header2.options[9], OPTIONS[9], "Failed at options[9]");
    }

    #[test]
    fn basic_header_from_bitstring() {
        let headers = headers();

        assert_eq!(headers.len(), 2);

        let header1_bs = &headers[0].output_bitstring;
        let header1 = &headers[0];

        assert_eq!(header1_bs.get_u16(0), SOURCE_PORT, "Failed at source_port");
        assert_eq!(header1_bs.get_u16(16), TARGET_PORT, "Failed at target_port");
        assert_eq!(
            header1_bs.get_u32(32),
            SEQUENCE_NUM1,
            "Failed at sequence_num1"
        );
        assert_eq!(header1_bs.get_u32(64), ACK_NUM, "Failed at ack_num");
        assert_eq!(
            header1_bs.get_u8(96),
            DATA_OFFSET << 4,
            "Failed at data_offset"
        );
        assert_eq!(header1_bs.get_u8(104), FLAG, "Failed at flag");
        assert_eq!(
            header1_bs.get_u16(112),
            WINDOW_SIZE,
            "Failed at window_size"
        );
        assert_eq!(
            BitString::from(header1_bs.get_u16(128)),
            BitString::from(header1.checksum),
            "Failed at checksum1"
        );
        assert_eq!(
            header1_bs.get_u16(144),
            URGENT_POINTER,
            "Failed at urgent_pointer"
        );
        assert_eq!(header1_bs.get_u32(160), OPTIONS[0], "Failed at options[0]");
        assert_eq!(header1_bs.get_u32(192), OPTIONS[1], "Failed at options[1]");
        assert_eq!(header1_bs.get_u32(224), OPTIONS[2], "Failed at options[2]");
        assert_eq!(header1_bs.get_u32(256), OPTIONS[3], "Failed at options[3]");
        assert_eq!(header1_bs.get_u32(288), OPTIONS[4], "Failed at options[4]");
        assert_eq!(header1_bs.get_u32(320), OPTIONS[5], "Failed at options[5]");
        assert_eq!(header1_bs.get_u32(352), OPTIONS[6], "Failed at options[6]");
        assert_eq!(header1_bs.get_u32(384), OPTIONS[7], "Failed at options[7]");
        assert_eq!(header1_bs.get_u32(416), OPTIONS[8], "Failed at options[8]");
        assert_eq!(header1_bs.get_u32(448), OPTIONS[9], "Failed at options[9]");

        let header2_bs = &headers[1].output_bitstring;
        let header2 = &headers[1];
        assert_eq!(header2_bs.get_u16(0), SOURCE_PORT, "Failed at source_port");
        assert_eq!(header2_bs.get_u16(16), TARGET_PORT, "Failed at target_port");
        assert_eq!(
            header2_bs.get_u32(32),
            SEQUENCE_NUM2,
            "Failed at sequence_num2"
        );
        assert_eq!(header2_bs.get_u32(64), ACK_NUM, "Failed at ack_num");
        assert_eq!(
            header2_bs.get_u8(96),
            DATA_OFFSET << 4,
            "Failed at data_offset"
        );
        assert_eq!(header2_bs.get_u8(104), FLAG, "Failed at flag");
        assert_eq!(
            header2_bs.get_u16(112),
            WINDOW_SIZE,
            "Failed at window_size"
        );
        assert_eq!(
            BitString::from(header2_bs.get_u16(128)),
            BitString::from(header2.checksum),
            "Failed at checksum2"
        );
        assert_eq!(
            header2_bs.get_u16(144),
            URGENT_POINTER,
            "Failed at urgent_pointer"
        );
        assert_eq!(header2_bs.get_u32(160), OPTIONS[0], "Failed at options[0]");
        assert_eq!(header2_bs.get_u32(192), OPTIONS[1], "Failed at options[1]");
        assert_eq!(header2_bs.get_u32(224), OPTIONS[2], "Failed at options[2]");
        assert_eq!(header2_bs.get_u32(256), OPTIONS[3], "Failed at options[3]");
        assert_eq!(header2_bs.get_u32(288), OPTIONS[4], "Failed at options[4]");
        assert_eq!(header2_bs.get_u32(320), OPTIONS[5], "Failed at options[5]");
        assert_eq!(header2_bs.get_u32(352), OPTIONS[6], "Failed at options[6]");
        assert_eq!(header2_bs.get_u32(384), OPTIONS[7], "Failed at options[7]");
        assert_eq!(header2_bs.get_u32(416), OPTIONS[8], "Failed at options[8]");
        assert_eq!(header2_bs.get_u32(448), OPTIONS[9], "Failed at options[9]");
    }

    #[test]
    #[should_panic]
    fn too_large_data_offset() {
        TCPFrameBuilder::new().set_data_offset(0b0001_0000u8); // 16
    }

    #[test]
    fn correct_checksum_placement() {
        // Given
        let source_port = 0b0000_0000_0000_0000u16;
        let target_port = 0b0000_0000_0000_0000u16;
        let ack_num = 0b0000_0000_0000_0000_0000_0000_0000_0000u32;
        let data_offset = 0b0000_0000u8;
        let flag = 0b0000_0000u8;
        let window_size = 0b0000_0000_0000_0000u16;
        let urgent_pointer = 0b0000_0000_0000_0000u16;
        let options = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        // Assumed
        let _sequence_num1 = 0b0000_0000_0000_0000_0000_0000_0000_0000u32;

        // Hand calculated
        let checksum = 0b1111_1111_1111_1111;

        // Empty datapoints
        let data_points = [BitString::new()];

        let builder = TCPFrameBuilder::new()
            .set_source_port(source_port)
            .set_target_port(target_port)
            .set_ack_num(ack_num)
            .set_data_offset(data_offset)
            .set_flags(flag)
            .set_window_size(window_size)
            .set_urgent_pointer(urgent_pointer)
            .set_options(options);

        let headers = builder.build_all(&data_points);
        let header_bs = headers[0].output_bitstring.clone();
        let header = &headers[0];

        println!("{header_bs}");
        assert_eq!(header_bs.get_u16(128 - 16), 0);
        assert_eq!(header_bs.get_u16(128), checksum);
        assert_eq!(header_bs.get_u16(128 + 16), 0);
        assert_eq!(header.checksum, checksum);
    }

    #[test]
    fn with_data() {
        let frames = headers_with_data(&[BitString::from(DATA)]);
        let frame = &frames[0];
        let frame_bs = &frames[0].output_bitstring;

        assert_eq!(
            frame_bs.get_u16(0),
            frame.source_port,
            "Failed at source_port"
        );
        assert_eq!(
            frame_bs.get_u16(16),
            frame.target_port,
            "Failed at target_port"
        );
        assert_eq!(
            frame_bs.get_u32(32),
            frame.sequence_num,
            "Failed at sequence_num1"
        );
        assert_eq!(frame_bs.get_u32(64), frame.ack_num, "Failed at ack_num");
        assert_eq!(
            frame_bs.get_u8(96) >> 4,
            frame.data_offset,
            "Failed at data_offset"
        );
        assert_eq!(frame_bs.get_u8(104), frame.flag_byte, "Failed at flag");
        assert_eq!(
            frame_bs.get_u16(112),
            frame.window_size,
            "Failed at window_size"
        );
        assert_eq!(
            BitString::from(frame_bs.get_u16(128)),
            BitString::from(frame.checksum),
            "Failed at checksum"
        );
        assert_eq!(
            frame_bs.get_u16(144),
            URGENT_POINTER,
            "Failed at urgent_pointer"
        );
        assert_eq!(
            frame_bs.get_u32(160),
            frame.options[0],
            "Failed at options[0]"
        );
        assert_eq!(
            frame_bs.get_u32(192),
            frame.options[1],
            "Failed at options[1]"
        );
        assert_eq!(
            frame_bs.get_u32(224),
            frame.options[2],
            "Failed at options[2]"
        );
        assert_eq!(
            frame_bs.get_u32(256),
            frame.options[3],
            "Failed at options[3]"
        );
        assert_eq!(
            frame_bs.get_u32(288),
            frame.options[4],
            "Failed at options[4]"
        );
        assert_eq!(
            frame_bs.get_u32(320),
            frame.options[5],
            "Failed at options[5]"
        );
        assert_eq!(
            frame_bs.get_u32(352),
            frame.options[6],
            "Failed at options[6]"
        );
        assert_eq!(
            frame_bs.get_u32(384),
            frame.options[7],
            "Failed at options[7]"
        );
        assert_eq!(
            frame_bs.get_u32(416),
            frame.options[8],
            "Failed at options[8]"
        );
        assert_eq!(
            frame_bs.get_u32(448),
            frame.options[9],
            "Failed at options[9]"
        );
    }
}
