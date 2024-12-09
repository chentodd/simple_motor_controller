#![cfg_attr(not(feature = "std"), no_std)]

/// Define the id of the packet that communicates between firmware and tools
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum MessageId {
    /// The default id, this means nothing
    #[default]
    NoId = 0x00,
    /// For the packet that sends target velocity to firmware
    CommandVelId = 0x10,
}

/// Polynomial for `CRC8` calculation
pub const CRC_POLYNOMIAL: u8 = 0x07;

/// Define the size of `length` field in packet.
pub const LENGTH_TYPE_IN_BYTES: usize = 4;

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
