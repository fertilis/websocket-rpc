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
