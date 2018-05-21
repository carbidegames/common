use {
    byteorder::{WriteBytesExt, LittleEndian, ByteOrder},
    num_traits::{ToPrimitive, FromPrimitive},
};

#[derive(PartialEq, Debug)]
pub struct Header {
    pub class: PacketClass,
}

impl Header {
    pub const START_OFFSET: usize = 5;

    pub fn extract(mut data: Vec<u8>, protocol_id: u32) -> Option<(Self, Vec<u8>)> {
        let start = data.len() - Self::START_OFFSET;

        // Verify the protocol ID, if it's not right, skip this packet
        let client_protocol_id = LittleEndian::read_u32(&data[start..start+4]);
        if client_protocol_id != protocol_id { return None }

        // The remaining byte is message class
        let class = PacketClass::from_u8(data[start+4])?;

        // Hide the header
        data.resize(start, 0);

        Some((Header {
            class,
        }, data))
    }

    pub fn write_to(&self, data: &mut Vec<u8>, protocol_id: u32) {
        // Append the protocol ID so the receiver can verify its validness.
        // It's appended at the end because we will know the length anyways, so our header doesn't
        // have to be at the start. This way we can avoid having to copy data to put the header at
        // the start.
        data.write_u32::<LittleEndian>(protocol_id).unwrap();

        data.push(self.class.to_u8().unwrap());
    }
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum PacketClass {
    Heartbeat,
    UnreliableMessage,
    SequencedMessage,
}

#[derive(Debug)]
pub struct SequencedHeader {
    pub packet_number: u16,
}

impl SequencedHeader {
    pub const START_OFFSET: usize = 2;

    pub fn extract(mut data: Vec<u8>) -> (Self, Vec<u8>) {
        let start = data.len() - Self::START_OFFSET;

        let packet_number = LittleEndian::read_u16(&data[start..start+2]);

        // Hide the header
        data.resize(start, 0);

        (SequencedHeader {
            packet_number,
        }, data)
    }

    pub fn write_to(&self, data: &mut Vec<u8>) {
        data.write_u16::<LittleEndian>(self.packet_number).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_serialization_works_two_ways() {
        let header = Header {
            class: PacketClass::Message,
        };
        let payload = vec![55; 32];
        let mut data = payload.clone();
        let protocol_id = 5;

        header.write_to(&mut data, protocol_id);
        let res = Header::extract(data.clone(), protocol_id);

        assert!(res.is_some());
        let (new_header, new_payload) = res.unwrap();
        assert_eq!(payload, new_payload);
        assert_eq!(header, new_header)
    }
}
