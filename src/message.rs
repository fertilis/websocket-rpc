use crate::agent_id::AgentId;
use anyhow::anyhow;

use crate::message_header::MessageHeader;

#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub body: Vec<u8>,
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.header.into();
        bytes.extend(self.body);
        bytes
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < MessageHeader::SIZE {
            return Err(anyhow!("Message too short"));
        }
        let header = MessageHeader::try_from(bytes)?;
        let body = bytes[MessageHeader::SIZE..].to_vec();
        Ok(Self { header, body })
    }
}

impl Message {
    pub fn is_handshake(&self) -> bool {
        self.header.dst_agent_id == AgentId(0) // && self.body.len() == 0
    }
}
