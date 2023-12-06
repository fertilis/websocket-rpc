use crate::message_header::MessageHeader;

#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub body: Vec<u8>,
}
