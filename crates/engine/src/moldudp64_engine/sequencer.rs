use crate::moldudp64_engine::types::*;
use netlib::moldudp64_core::sessions::SessionTable;
use nexus_queue::spsc;

impl Sequencer {
    pub fn new(input: spsc::Consumer<Event>, output: spsc::Producer<SequencedEvent>) -> Self {
        Self {
            input,
            output,
            sequence_number: 1,
            session_table: SessionTable::new(),
        }
    }

    pub fn run(mut self) {
        loop {
            if let Some(event) = self.input.pop() {
                let sequenced_event = SequencedEvent {
                    payload: event.payload,
                    sequence_number: self.sequence_number,
                    session_id: self.session_table.get_current_session(),
                };

                self.sequence_number += 1;

                let _ = self.output.push(sequenced_event);
            } else {
                std::thread::yield_now();
            }
        }
    }
}
