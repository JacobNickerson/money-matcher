use crate::moldudp64_core::types::*;
use std::collections::HashMap;
pub struct SessionTable {
    pub sessions: HashMap<SessionID, SequenceNumber>,
    pub current_session: SessionID,
}

impl Default for SessionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionTable {
    #[inline(always)]
    pub fn new() -> SessionTable {
        let current_session = SessionTable::make_session_id(1);

        let mut table = SessionTable {
            sessions: HashMap::new(),
            current_session,
        };

        table.add_session(current_session, 0_u64.to_be_bytes());

        table
    }

    #[inline(always)]
    pub fn generate_session_id(&mut self) -> SessionID {
        Self::make_session_id(self.sessions.len() + 1)
    }

    #[inline(always)]
    pub fn make_session_id(index: usize) -> SessionID {
        let s = format!("MM{:08}", index);
        let mut session_id = [b' '; 10];

        session_id.copy_from_slice(s.as_bytes());
        session_id
    }

    #[inline(always)]
    pub fn add_session(&mut self, session_id: SessionID, sequence_number: SequenceNumber) {
        self.sessions.insert(session_id, sequence_number);
        self.current_session = session_id;
    }

    #[inline(always)]
    pub fn next_sequence(&mut self, session_id: SessionID) -> SequenceNumber {
        let sequence = self.sessions.get_mut(&session_id).expect("Unknown Session");

        let mut cur_u64 = u64::from_be_bytes(*sequence);
        cur_u64 += 1;
        *sequence = cur_u64.to_be_bytes();

        *sequence
    }

    #[inline(always)]
    pub fn remove_session(&mut self, session_id: &SessionID) {
        self.sessions.remove(session_id);
    }

    #[inline(always)]
    pub fn get_current_session(&self) -> SessionID {
        self.current_session
    }
}
