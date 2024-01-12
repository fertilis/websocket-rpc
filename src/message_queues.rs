use crate::agent_id::AgentId;
use crate::message::Message;
use std::collections::{HashMap, VecDeque};

pub struct MessageQueues {
    queues: HashMap<AgentId, VecDeque<Message>>,
}

impl MessageQueues {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    pub fn push(&mut self, message: Message) {
        let dst_agent_id: AgentId = message.header.dst_agent_id;
        if !self.queues.contains_key(&dst_agent_id) {
            self.queues.insert(dst_agent_id, VecDeque::new());
        }
        let queue = self.queues.get_mut(&dst_agent_id).unwrap();
        queue.push_back(message);
    }

    pub fn pop(&mut self, agent_id: AgentId) -> Option<Message> {
        if !self.queues.contains_key(&agent_id) {
            return None;
        }
        let queue = self.queues.get_mut(&agent_id).unwrap();
        queue.pop_front()
    }
}
