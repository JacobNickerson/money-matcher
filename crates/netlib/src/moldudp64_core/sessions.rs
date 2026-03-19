use crate::moldudp64_core::types::*;
use std::collections::HashMap;
pub struct SessionTable {
    pub sessions: HashMap<SessionID, SequenceNumber>,
    pub current_session: SessionID,
    pub id_prefix: String, // "MM_L#"
}

impl Default for SessionTable {
    fn default() -> Self {
        Self::new("MM_L0".to_string())
    }
}

impl SessionTable {
    #[inline(always)]
    pub fn new(id_prefix: String) -> SessionTable {
        let current_session = SessionTable::make_session_id(1, &id_prefix);

        let mut table = SessionTable {
            sessions: HashMap::new(),
            current_session,
            id_prefix,
        };

        table.add_session(current_session, 0_u64.to_be_bytes());

        table
    }

    #[inline(always)]
    pub fn generate_session_id(&mut self) -> SessionID {
        Self::make_session_id(self.sessions.len() + 1, &self.id_prefix)
    }

    #[inline(always)]
    pub fn make_session_id(index: usize, id_prefix: &String) -> SessionID {
        let s = format!("{}_{:04}", id_prefix, index);
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
        assert_eq!(*st.sessions.get(&s1).expect("err"), 0_u64.to_be_bytes());
    }

    #[test]
    fn test_make_session_id_values() {
        let id_prefix = "MM_L0".to_string();
        let s1 = SessionTable::make_session_id(1, &id_prefix);
        let s12 = SessionTable::make_session_id(12, &id_prefix);
        let s123 = SessionTable::make_session_id(123, &id_prefix);

        assert_eq!(&s1, b"MM00000001");
        assert_eq!(&s12, b"MM00000012");
        assert_eq!(&s123, b"MM00000123");
    }

    #[test]
    fn test_add_session_updates_current() {
        let id_prefix = "MM_L0".to_string();
        let mut st = SessionTable::new(id_prefix);

        let s2 = SessionTable::make_session_id(2, &st.id_prefix);
        let n1 = 12_u64.to_be_bytes();

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

        let n1 = st.next_sequence(s1);
        let n2 = st.next_sequence(s1);

        assert_eq!(u64::from_be_bytes(n1), 1);
        assert_eq!(u64::from_be_bytes(n2), 2);
    }

    #[test]
    fn test_next_sequence_multiple_sessions() {
        let id_prefix = "MM_L0".to_string();
        let mut st: SessionTable = SessionTable::new(id_prefix);

        let s1 = st.get_current_session();
        let s2 = SessionTable::make_session_id(2, &st.id_prefix);

        st.add_session(s2, 1_u64.to_be_bytes());

        let a1 = st.next_sequence(s1);
        let b1 = st.next_sequence(s2);

        assert_eq!(u64::from_be_bytes(a1), 1);
        assert_eq!(u64::from_be_bytes(b1), 2);
    }
}
