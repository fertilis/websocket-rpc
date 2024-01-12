use anyhow::anyhow;

use crate::agent_id::AgentId;
use crate::request_id::RequestId;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MessageHeader {
    pub src_agent_id: AgentId,
    pub dst_agent_id: AgentId,
    pub request_id: RequestId,
}

impl MessageHeader {
    pub const SIZE: usize = std::mem::size_of::<MessageHeader>();
}

impl TryFrom<&[u8]> for MessageHeader {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(anyhow!("Buffer should equal to MessageHeader::SIZE"));
        }
        let header: &MessageHeader =
            unsafe { &*(bytes[..Self::SIZE].as_ptr() as *const MessageHeader) };
        Ok(header.clone())
    }
}
