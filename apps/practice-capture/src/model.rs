use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub display_id: Option<u32>,
    pub include_practice_context: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display_id: None,
            include_practice_context: true,
        }
    }
}

/// Reject work from a stopped session, including captures already in flight.
#[derive(Default, Debug)]
pub struct Session {
    pub running: bool,
    pub busy: bool,
    pub generation: u64,
    key_down: bool,
    last_capture: Option<Instant>,
}

impl Session {
    pub fn start(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.running = true;
        self.busy = false;
        self.key_down = false;
        self.last_capture = None;
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.busy = false;
        self.key_down = false;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn pressed(&mut self, now: Instant) -> bool {
        if self.key_down {
            return false;
        }
        self.key_down = true;
        if !self.running || self.busy {
            return false;
        }
        if self
            .last_capture
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(500))
        {
            return false;
        }
        self.last_capture = Some(now);
        self.busy = true;
        true
    }

    pub fn released(&mut self) {
        self.key_down = false;
    }

    pub fn accepts(&self, generation: u64) -> bool {
        self.running && self.generation == generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_invalidates_in_flight_captures_and_restart_does_not_revive_them() {
        let mut session = Session::default();
        session.start();
        let generation = session.generation;
        assert!(session.pressed(Instant::now()));
        session.stop();
        assert!(!session.accepts(generation));
        session.start();
        assert!(!session.accepts(generation));
    }

    #[test]
    fn held_key_and_busy_state_do_not_create_multiple_captures() {
        let mut session = Session::default();
        session.start();
        let now = Instant::now();
        assert!(session.pressed(now));
        assert!(!session.pressed(now + Duration::from_secs(2)));
        session.released();
        assert!(!session.pressed(now + Duration::from_secs(3)));
        session.released();
        session.busy = false;
        assert!(session.pressed(now + Duration::from_secs(4)));
    }

    #[test]
    fn inactive_shortcut_and_rapid_repress_do_not_trigger() {
        let mut session = Session::default();
        let now = Instant::now();
        assert!(!session.pressed(now));
        session.start();
        assert!(session.pressed(now));
        session.released();
        session.busy = false;
        assert!(!session.pressed(now + Duration::from_millis(100)));
        session.released();
        assert!(session.pressed(now + Duration::from_secs(1)));
    }
}
