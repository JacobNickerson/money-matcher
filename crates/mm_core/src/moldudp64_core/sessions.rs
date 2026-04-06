use crate::moldudp64_core::types::*;
use std::collections::HashMap;

/// A registry for managing MoldUDP64 session identifiers and their corresponding sequence numbers.
pub struct SessionTable {
    pub sessions: HashMap<SessionID, SequenceNumber>,
    pub current_session: SessionID,
    pub current_sequence_number: SequenceNumber,
    pub id_prefix: String,
}

impl Default for SessionTable {
    /// Creates a default session table using "MM_L0" as the standard identifier prefix.
    fn default() -> Self {
        Self::new("MM_L0".to_string())
    }
}

impl SessionTable {
    /// Initializes a new session table with an starting session based on the provided prefix.
    #[inline(always)]
    pub fn new(id_prefix: String) -> SessionTable {
        let current_session = SessionTable::make_session_id(1, &id_prefix);

        let mut table = SessionTable {
            sessions: HashMap::new(),
            current_session,
            current_sequence_number: 0,
            id_prefix,
        };

        table.add_session(current_session, 0_u64);

        table
    }

    /// Generates a new unique session identifier based on the current number of active sessions.
    #[inline(always)]
    pub fn generate_session_id(&mut self) -> SessionID {
        Self::make_session_id(self.sessions.len() + 1, &self.id_prefix)
    }

    /// Formats an index and prefix into a fixed-length 10-byte MoldUDP64 session identifier.
    #[inline(always)]
    pub fn make_session_id(index: usize, id_prefix: &String) -> SessionID {
        let s = format!("{}_{:04}", id_prefix, index);
        let mut session_id = [b' '; 10];

        session_id.copy_from_slice(s.as_bytes());
        session_id
    }

    /// Registers a new session in the table and updates the current session ID.
    #[inline(always)]
    pub fn add_session(&mut self, session_id: SessionID, sequence_number: SequenceNumber) {
        let sequence = self
            .sessions
            .get_mut(&self.current_session)
            .expect("Unknown Session");
        *sequence = self.current_sequence_number;

        self.sessions.insert(session_id, sequence_number);

        self.current_sequence_number = sequence_number;
        self.current_session = session_id;
    }

    /// Increments and returns the next sequence number for the  session.
    #[inline(always)]
    pub fn next_sequence(&mut self) -> u64 {
        self.current_sequence_number += 1;

        if self.current_sequence_number >= u64::MAX {
            let next_id = self.generate_session_id();
            self.add_session(next_id, 1u64);
        }

        self.current_sequence_number
    }

    /// Removes a session entry from the registry.
    #[inline(always)]
    pub fn remove_session(&mut self, session_id: &SessionID) {
        self.sessions.remove(session_id);
    }

    /// Returns the currently active session identifier.
    #[inline(always)]
    pub fn get_current_session(&self) -> SessionID {
        self.current_session
    }
    /// Returns the currently active session sequence number.
    #[inline(always)]
    pub fn get_current_sequence_number(&self) -> SequenceNumber {
        self.current_sequence_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_initial_state() {
        let id_prefix = "MM_L0".to_string();
        let st = SessionTable::new(id_prefix);

        let s1 = SessionTable::make_session_id(1, &st.id_prefix);

        assert_eq!(st.sessions.len(), 1);
        assert_eq!(st.get_current_session(), s1);
        assert_eq!(*st.sessions.get(&s1).expect("err"), 0_u64);
    }

    #[test]
    fn test_make_session_id_values() {
        let id_prefix = "MM_L0".to_string();
        let s1 = SessionTable::make_session_id(1, &id_prefix);
        let s12 = SessionTable::make_session_id(12, &id_prefix);
        let s123 = SessionTable::make_session_id(123, &id_prefix);

        assert_eq!(&s1, b"MM_L0_0001");
        assert_eq!(&s12, b"MM_L0_0012");
        assert_eq!(&s123, b"MM_L0_0123");
    }

    #[test]
    fn test_add_session_updates_current() {
        let id_prefix = "MM_L0".to_string();
        let mut st = SessionTable::new(id_prefix);

        let s2 = SessionTable::make_session_id(2, &st.id_prefix);
        let n1 = 12_u64;

        st.add_session(s2, n1);

        assert_eq!(st.sessions.len(), 2);
        assert_eq!(st.get_current_session(), s2);
        assert_eq!(*st.sessions.get(&s2).expect("err"), n1);
    }

    #[test]
    fn test_remove_session_removes_entry() {
        let id_prefix = "MM_L0".to_string();
        let mut st = SessionTable::new(id_prefix);
        let s1 = st.get_current_session();

        st.remove_session(&s1);

        assert!(st.sessions.is_empty());
    }

    #[test]
    fn test_next_sequence_single_session() {
        let id_prefix = "MM_L0".to_string();
        let mut st = SessionTable::new(id_prefix);
        let s1 = st.get_current_session();

        let n1 = st.next_sequence();
        let n2 = st.next_sequence();

        assert_eq!(n1, 1);
        assert_eq!(n2, 2);
    }
}
