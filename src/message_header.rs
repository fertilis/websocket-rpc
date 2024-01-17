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

impl Into<Vec<u8>> for MessageHeader {
    fn into(self) -> Vec<u8> {
        unsafe {
            std::slice::from_raw_parts(
                &self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
        .to_vec()
    }
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
