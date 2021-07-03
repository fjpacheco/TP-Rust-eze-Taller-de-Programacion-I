use std::sync::Mutex;
use std::sync::Arc;
use crate::tcp_protocol::client_atributes::client_fields::ClientFields;
use std::collections::HashSet;
use std::mem;

use crate::tcp_protocol::runnables_map::RunnablesMap;

#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum Status {
    Executor,
    Subscriber,
    Monitor,
    Dead,
}

impl Status {
    pub fn replace(&mut self, new_status: Status) -> Status {
        mem::replace(self, new_status)
    }

    pub fn update_map(&self) -> Option<RunnablesMap<Arc<Mutex<ClientFields>>>> {
        match self {
            Self::Executor => Some(RunnablesMap::<Arc<Mutex<ClientFields>>>::executor()),
            Self::Subscriber => Some(RunnablesMap::<Arc<Mutex<ClientFields>>>::subscriber()),
            _ => None,
        }
    }

}
