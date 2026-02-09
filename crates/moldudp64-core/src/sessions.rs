use crate::SessionTable;
use crate::types::{SequenceNumber, SessionID};
use std::collections::HashMap;

impl SessionTable {
    pub fn new() -> Self {
        SessionTable {
            sessions: HashMap::new(),
        }
    }

    pub fn generate_session_id(&mut self) -> SessionID {
        let s = format!("MM{:08}", self.sessions.len() + 1);
        let mut session_id = [b' '; 10];

        session_id.copy_from_slice(s.as_bytes());
        session_id
    }

    pub fn add_session(&mut self, session_id: SessionID, sequence_number: SequenceNumber) {
        self.sessions.insert(session_id, sequence_number);
    }

    pub fn get_sequence_number(&self, session_id: SessionID) -> Option<&SequenceNumber> {
        self.sessions.get(&session_id)
    }

    pub fn remove_session(&mut self, session_id: &SessionID) {
        self.sessions.remove(session_id);
    }
}
