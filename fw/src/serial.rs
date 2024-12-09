use core::{fmt::Debug, u32};
use defmt::debug;
use micropb::{MessageDecode, PbDecoder};
use utils::*;

#[repr(u8)]
#[derive(Default, Debug)]
enum PacketDecodeState {
    #[default]
    GetHeader,
    GetPacketBody,
    CheckCRC,
}

#[derive(Default)]
pub struct PacketDecoder {
    len: u32,
    message_id: MessageId,
    packet_decode_state: PacketDecodeState,
    good_packet: bool,
}

impl Debug for PacketDecoder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PacketDecoder")
            .field("len", &self.len)
            .field("message_id", &self.message_id)
            .field("packet_decode_state", &self.packet_decode_state)
            .field("good_packet", &self.good_packet)
            .finish()
    }
}

impl PacketDecoder {
    // The packet is formatted as follows, unit: u8
    // [MESSAGE_ID:1] [LEN:4] [PROTO_PACKET:X] [CRC:1]
    //
    // MESSAGE_ID:
    // should be the same with enum `MessageId`
    //
    // LEN:
    // the length of current packet which is:
    // - size_of(MESSAGE_ID), 1 bytes +
    // - size_of(LEN), 4 bytes +
    // - size_of(PROTO_PACKET), X bytes +
    // - size_of(CRC), 1 bytes
    // = X + 6 bytes
    //
    // PROTO_PACKET:
    // the serialized bytes sent by sender
    //
    // CRC:
    // the check code of the packect
    pub fn new() -> Self {
        Self {
            len: u32::MAX,
            ..Default::default()
        }
    }

    pub fn get_message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn polling(&mut self, stream: &[u8]) -> bool {
        match self.packet_decode_state {
            PacketDecodeState::GetHeader => self.get_header(stream),
            PacketDecodeState::GetPacketBody => self.get_packet_body(stream),
            PacketDecodeState::CheckCRC => self.check_crc(stream),
        }

        if !self.good_packet {
            self.reset_struct_data();
            debug!("Bad packet");
            debug!("{:?}", stream);
            debug!("{:?}", self);
        }

        true
    }

    pub fn parse_proto_message(&mut self, stream: &[u8], packet: &mut impl MessageDecode) -> bool {
        if self.good_packet {
            let header_len = size_of_val(&self.message_id) + size_of_val(&self.len);
            let stream = &stream[header_len..];

            let mut decoder = PbDecoder::new(stream);
            match packet.decode_len_delimited(&mut decoder) {
                Ok(_) => {
                    return true;
                }
                Err(e) => {
                    debug!("{:?}", e)
                }
            }
        }

        false
    }

    fn get_header(&mut self, stream: &[u8]) {
        // Get [MESSAGE_ID:1] and [LEN:4]
        if stream.len() >= size_of_val(&self.message_id) + size_of_val(&self.len) {
            if self.message_id == MessageId::NoId && self.len == u32::MAX {
                if let Some(first_ch) = stream.first() {
                    match first_ch {
                        0x00 => self.message_id = MessageId::NoId,
                        0x10 => self.message_id = MessageId::CommandVelId,
                        _ => (),
                    }
                }

                self.len = u32::from_le_bytes(stream[1..=4].try_into().unwrap());

                // change to next state
                self.packet_decode_state = PacketDecodeState::GetPacketBody;
            }

            debug!("serial, get_header: {}, {}", self.message_id, self.len);
        }
    }

    fn reset_struct_data(&mut self) {
        self.len = u32::MAX;
        self.message_id = MessageId::NoId;
        self.packet_decode_state = PacketDecodeState::GetHeader;
        self.good_packet = false;
    }

    fn get_packet_body(&mut self, stream: &[u8]) {
        if stream.len() >= (self.len as usize) {
            self.packet_decode_state = PacketDecodeState::CheckCRC;
            debug!("serial, get_packet_body: {:?}", stream);
        }
    }

    fn check_crc(&mut self, stream: &[u8]) {
        let n = stream.len();
        self.good_packet = stream[n - 1] == calculate_crc(&stream[0..=n-2]);
        debug!("serial, check_crc: {}", self.good_packet);
    }
}
