/// Dream state machine with five distinct consciousness states.
use std::fmt;

/// The possible states of the dreaming consciousness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DreamState {
    Awake,
    LightSleep,
    DeepSleep,
    Rem,
    Lucid,
}

impl fmt::Display for DreamState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DreamState::Awake => write!(f, "AWAKE"),
            DreamState::LightSleep => write!(f, "LIGHT_SLEEP"),
            DreamState::DeepSleep => write!(f, "DEEP_SLEEP"),
            DreamState::Rem => write!(f, "REM"),
            DreamState::Lucid => write!(f, "LUCID"),
        }
    }
}

impl DreamState {
    /// Returns true if this state is any form of sleep.
    pub fn is_sleeping(&self) -> bool {
        matches!(self, DreamState::LightSleep | DreamState::DeepSleep | DreamState::Rem | DreamState::Lucid)
    }

    /// Returns true if the state involves dreaming.
    pub fn is_dreaming(&self) -> bool {
        matches!(self, DreamState::Rem | DreamState::Lucid)
    }

    /// Depth of sleep from 0 (awake) to 4 (deep sleep).
    pub fn depth(&self) -> u8 {
        match self {
            DreamState::Awake => 0,
            DreamState::LightSleep => 1,
            DreamState::DeepSleep => 3,
            DreamState::Rem => 2,
            DreamState::Lucid => 2,
        }
    }

    /// Returns all valid transitions from this state.
    pub fn valid_transitions(&self) -> Vec<DreamState> {
        match self {
            DreamState::Awake => vec![DreamState::LightSleep],
            DreamState::LightSleep => vec![DreamState::Awake, DreamState::DeepSleep, DreamState::Rem],
            DreamState::DeepSleep => vec![DreamState::LightSleep, DreamState::Rem],
            DreamState::Rem => vec![DreamState::LightSleep, DreamState::Awake, DreamState::Lucid],
            DreamState::Lucid => vec![DreamState::Rem, DreamState::Awake, DreamState::LightSleep],
        }
    }

    /// Check if transitioning to target state is valid.
    pub fn can_transition_to(&self, target: &DreamState) -> bool {
        if self == target {
            return true; // self-transitions are allowed (no-op)
        }
        self.valid_transitions().contains(target)
    }
}

/// A record of a state transition event.
#[derive(Clone, Debug)]
pub struct TransitionEvent {
    pub from: DreamState,
    pub to: DreamState,
    pub tick: u64,
}

/// The dream state machine that manages consciousness transitions.
pub struct DreamStateMachine {
    pub state: DreamState,
    pub tick: u64,
    pub history: Vec<TransitionEvent>,
}

impl DreamStateMachine {
    pub fn new() -> Self {
        Self {
            state: DreamState::Awake,
            tick: 0,
            history: Vec::new(),
        }
    }

    /// Advance time by one tick.
    pub fn advance(&mut self) {
        self.tick += 1;
    }

    /// Attempt to transition to a new dream state.
    pub fn transition(&mut self, target: DreamState) -> Result<(), String> {
        if self.state.can_transition_to(&target) {
            let from = self.state;
            self.history.push(TransitionEvent {
                from,
                to: target,
                tick: self.tick,
            });
            self.state = target;
            Ok(())
        } else {
            Err(format!(
                "Cannot transition from {} to {}",
                self.state, target
            ))
        }
    }

    /// Get the total number of transitions that occurred.
    pub fn transition_count(&self) -> usize {
        self.history.len()
    }

    /// Count how many ticks were spent in a given state.
    pub fn ticks_in_state(&self, state: DreamState) -> u64 {
        let mut count = 0u64;
        let mut current = DreamState::Awake;
        let mut last_change_tick = 0u64;

        for event in &self.history {
            if current == state {
                count += event.tick - last_change_tick;
            }
            current = event.to;
            last_change_tick = event.tick;
        }
        // Account for time spent in current state after last transition
        if current == state {
            count += self.tick - last_change_tick;
        }
        count
    }

    /// Return a reference to the transition history.
    pub fn history(&self) -> &[TransitionEvent] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dream_state_display() {
        assert_eq!(DreamState::Awake.to_string(), "AWAKE");
        assert_eq!(DreamState::Lucid.to_string(), "LUCID");
    }

    #[test]
    fn test_is_sleeping() {
        assert!(!DreamState::Awake.is_sleeping());
        assert!(DreamState::LightSleep.is_sleeping());
        assert!(DreamState::DeepSleep.is_sleeping());
        assert!(DreamState::Rem.is_sleeping());
        assert!(DreamState::Lucid.is_sleeping());
    }

    #[test]
    fn test_is_dreaming() {
        assert!(!DreamState::Awake.is_dreaming());
        assert!(!DreamState::LightSleep.is_dreaming());
        assert!(!DreamState::DeepSleep.is_dreaming());
        assert!(DreamState::Rem.is_dreaming());
        assert!(DreamState::Lucid.is_dreaming());
    }

    #[test]
    fn test_depth_ordering() {
        assert!(DreamState::Awake.depth() < DreamState::LightSleep.depth());
        assert!(DreamState::DeepSleep.depth() > DreamState::Rem.depth());
    }

    #[test]
    fn test_valid_transitions() {
        let awake_transitions = DreamState::Awake.valid_transitions();
        assert!(awake_transitions.contains(&DreamState::LightSleep));
        assert!(!awake_transitions.contains(&DreamState::DeepSleep));
    }

    #[test]
    fn test_state_machine_new() {
        let sm = DreamStateMachine::new();
        assert_eq!(sm.state, DreamState::Awake);
        assert_eq!(sm.tick, 0);
        assert!(sm.history.is_empty());
    }

    #[test]
    fn test_valid_transition() {
        let mut sm = DreamStateMachine::new();
        assert!(sm.transition(DreamState::LightSleep).is_ok());
        assert_eq!(sm.state, DreamState::LightSleep);
        assert_eq!(sm.transition_count(), 1);
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = DreamStateMachine::new();
        assert!(sm.transition(DreamState::DeepSleep).is_err());
        assert_eq!(sm.state, DreamState::Awake);
    }

    #[test]
    fn test_full_sleep_cycle() {
        let mut sm = DreamStateMachine::new();
        sm.advance();
        sm.transition(DreamState::LightSleep).unwrap();
        sm.advance();
        sm.transition(DreamState::DeepSleep).unwrap();
        sm.advance();
        sm.transition(DreamState::Rem).unwrap();
        sm.advance();
        sm.transition(DreamState::Lucid).unwrap();
        sm.advance();
        sm.transition(DreamState::Awake).unwrap();
        assert_eq!(sm.transition_count(), 5);
        assert_eq!(sm.state, DreamState::Awake);
    }

    #[test]
    fn test_ticks_in_state() {
        let mut sm = DreamStateMachine::new();
        sm.advance(); // tick 1
        sm.advance(); // tick 2
        sm.advance(); // tick 3
        sm.transition(DreamState::LightSleep).unwrap(); // at tick 3
        sm.advance(); // tick 4
        sm.advance(); // tick 5
        // Awake from 0..3 = 3 ticks
        assert_eq!(sm.ticks_in_state(DreamState::Awake), 3);
    }
}
