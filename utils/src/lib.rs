#![cfg_attr(not(feature = "std"), no_std)]

//! A crate that handles the packet between host and embedded board
//! The packet is formatted as follows, unit: u8
//! [MESSAGE_ID:1] [LEN:4] [PROTO_PACKET:X] [CRC:1]
//!
//! MESSAGE_ID:
//! should be the same with enum `MessageId`
//!
//! LEN:
//! the length of current packet which is:
//! - size_of(MESSAGE_ID), 1 bytes +
//! - size_of(LEN), 4 bytes +
//! - size_of(PROTO_PACKET), X bytes +
//! - size_of(CRC), 1 bytes
//! = X + 6 bytes
//!
//! PROTO_PACKET:
//! the serialized bytes sent by sender
//!
//! CRC:
//! the check code of the packect

use micropb::{MessageDecode, PbDecoder};

#[cfg(all(not(feature = "std"), feature = "debug"))]
use defmt::println;

/// Polynomial for `CRC8` calculation
pub const CRC_POLYNOMIAL: u8 = 0x07;

/// Define the size of `length` field in packet.
pub const LENGTH_TYPE_IN_BYTES: usize = 4;

/// Define the id of the packet that communicates between firmware and tools
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum MessageId {
    /// The default id, this means nothing
    #[default]
    NoId = 0x00,
    /// The packet that is sent by host, received by firmware
    CommandRx = 0x10,
    /// The packet that is sent by firmware, received by host
    CommandTx = 0x11,
}

impl MessageId {
    fn from_u8(val: u8) -> MessageId {
        match val {
            0x10 => MessageId::CommandRx,
            0x11 => MessageId::CommandTx,
            _ => MessageId::NoId,
        }
    }
}

/// Helper that calculates CRC8 using [CRC_POLYNOMIAL]
///
/// # Arguments
///
/// * `stream`: calculate CRC8 for input bytes array
pub fn calculate_crc(stream: &[u8]) -> u8 {
    let mut crc: u8 = 0xFF;
    for &x in stream.iter() {
        crc ^= x;
        for _ in 0..8 {
            if ((crc >> 7) & 0b01) != 0 {
                crc = (crc << 1) ^ CRC_POLYNOMIAL;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}

pub struct PacketEncoder<const N: usize> {
    buffer: [u8; N],
}

/// Helper that encodes proto message
///
/// # Arguments
///
/// * `message_id`: the message id that will be set in the packet
/// * `proto_message`: the proto message from [PbEncoder](micropb::PbEncoder)
///
impl<const N: usize> PacketEncoder<N> {
    pub fn new(buffer: [u8; N]) -> Self {
        Self { buffer }
    }

    pub fn create_packet(&mut self, message_id: MessageId, proto_message: &[u8]) -> &[u8] {
        self.buffer.fill(0);

        self.buffer[0] = message_id as u8;

        // message_id + length bytes + proto_message length + CRC
        let length = (1 + LENGTH_TYPE_IN_BYTES + proto_message.len() + 1) as u32;
        self.buffer[1..=LENGTH_TYPE_IN_BYTES].copy_from_slice(&length.to_le_bytes());

        self.buffer[LENGTH_TYPE_IN_BYTES + 1..=(LENGTH_TYPE_IN_BYTES + proto_message.len())]
            .copy_from_slice(proto_message);

        let length = length as usize;
        self.buffer[length - 1] = calculate_crc(&self.buffer[0..=length - 2]);

        &self.buffer[0..length]
    }
}

#[derive(Default)]
pub struct PacketDecoder {
    len: u32,
}

impl PacketDecoder {
    pub fn new() -> Self {
        Self {
            len: u32::MAX,
        }
    }

    pub fn get_valid_packet_index(&mut self, stream: &[u8]) -> Option<usize> {
        if stream.len() >= size_of::<MessageId>() + LENGTH_TYPE_IN_BYTES {
            for i in 0..stream.len() {
                // Check [MESSAGE_ID:1]
                if MessageId::from_u8(stream[i]) == MessageId::NoId {
                    continue;
                }
    
                #[cfg(feature = "debug")]
                println!("is_packet_valid: {:?}", &stream[1..=4]);

                // Check [LEN:4]
                let i = i + 1;
                if i + 4 < stream.len() {
                    self.len = u32::from_le_bytes(stream[i..i + 4].try_into().unwrap());
                    if self.len == 0 {
                        continue;
                    }
                }

                // Check CRC
                let i = i - 1;
                let n = self.len as usize;
                if i + n - 1 < stream.len() {
                    let actual_crc = stream[i + n - 1];
                    let expected_crc = calculate_crc(&stream[i..=i + n - 2]);

                    if actual_crc == expected_crc {
                        return Some(i);
                    }
                }
            }
        }

        None
    }

    pub fn parse_proto_message(&mut self, stream: &[u8], packet: &mut impl MessageDecode) -> bool {
        let n = self.len as usize;
        let header_len = size_of::<MessageId>() + LENGTH_TYPE_IN_BYTES;
        let stream = &stream[header_len..=n - 2];

        let mut decoder = PbDecoder::new(stream);
        match packet.decode(&mut decoder, stream.len()) {
            Ok(_) => {
                return true;
            }
            Err(_e) => {
                #[cfg(feature = "debug")]
                println!("proto packet debug error");
            }
        }

        false
    }
}
