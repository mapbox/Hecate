use protobuf;
use protobuf::core::Message;
use protobuf::error::ProtobufError;
use protobuf::stream::CodedOutputStream;
use std::io::{BufReader, Read, Write};
use crate::mvt::proto;

pub trait Encode {
    fn to_writer(&self, out: &mut dyn Write) -> Result<(), ProtobufError>;
    fn to_bytes(&self) -> Result<Vec<u8>, ProtobufError>;
}

pub trait Decode {
    fn from_reader(input: &mut dyn Read) -> Result<Self, ProtobufError> where Self: Sized;
    fn from_bytes(bytes: &Vec<u8>) -> Result<Self, ProtobufError> where Self: Sized;
}

impl Encode for proto::Tile {
    fn to_writer(&self, mut out: &mut dyn Write) -> Result<(), ProtobufError> {
        let mut os = CodedOutputStream::new(&mut out);
        let _ = self.write_to(&mut os);
        os.flush()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, ProtobufError> {
        let mut bytes = Vec::<u8>::new();
        {
            let mut os = CodedOutputStream::vec(&mut bytes);
            let _ = self.write_to(&mut os);
        }

        Ok(bytes)
    }
}

impl Decode for proto::Tile {
    fn from_reader(input: &mut dyn Read) -> Result<Self, ProtobufError> {
        let mut reader = BufReader::new(input);
        protobuf::parse_from_reader(&mut reader)
    }

    fn from_bytes(bytes: &Vec<u8>) -> Result<Self, ProtobufError> {
        protobuf::parse_from_bytes(bytes)
    }
}
