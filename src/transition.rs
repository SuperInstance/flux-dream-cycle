/// Sleep/wake transition protocols: manage smooth transitions between consciousness states.

use crate::state_machine::DreamState;
use crate::circadian::CircadianRhythm;

/// Status of a sleep/wake transition.
#[derive(Clone, Debug, PartialEq)]
pub enum TransitionStatus {
    Pending,
    InProgress,
    Complete,
    Failed(String),
}

/// A sleep/wake transition protocol.
#[derive(Clone, Debug)]
pub struct TransitionProtocol {
    pub from: DreamState,
    pub to: DreamState,
    pub name: String,
    pub steps: Vec<String>,
    pub current_step: usize,
    pub status: TransitionStatus,
    pub duration_ticks: u64,
    pub elapsed_ticks: u64,
}

impl TransitionProtocol {
    pub fn new(name: &str, from: DreamState, to: DreamState, duration_ticks: u64) -> Self {
        let steps = match (from, to) {
            (DreamState::Awake, DreamState::LightSleep) => vec![
                "Dim sensory input".to_string(),
                "Relax muscle tone".to_string(),
                "Slow breathing rhythm".to_string(),
                "Reduce heart rate".to_string(),
                "Enter hypnagogic state".to_string(),
            ],
            (DreamState::LightSleep, DreamState::DeepSleep) => vec![
                "Suppress external stimuli".to_string(),
                "Deepen muscle relaxation".to_string(),
                "Lower core temperature".to_string(),
                "Activate slow-wave activity".to_string(),
            ],
            (DreamState::DeepSleep, DreamState::Rem) => vec![
                "Reactivate cortical circuits".to_string(),
                "Initiate rapid eye movement".to_string(),
                "Begin dream narrative".to_string(),
                "Disable motor output".to_string(),
            ],
            (DreamState::Rem, DreamState::Awake) => vec![
                "Reduce dream intensity".to_string(),
                "Restore motor function".to_string(),
                "Re-engage sensory processing".to_string(),
                "Re-establish conscious awareness".to_string(),
                "Calibrate temporal sense".to_string(),
            ],
            (DreamState::Rem, DreamState::Lucid) => vec![
                "Recognize dream state".to_string(),
                "Assert conscious control".to_string(),
                "Stabilize dream environment".to_string(),
                "Engage metacognition".to_string(),
            ],
            (_, _) => vec!["Execute transition".to_string()],
        };

        Self {
            from,
            to,
            name: name.to_string(),
            steps,
            current_step: 0,
            status: TransitionStatus::Pending,
            duration_ticks,
            elapsed_ticks: 0,
        }
    }

    /// Start the transition protocol.
    pub fn start(&mut self) {
        self.status = TransitionStatus::InProgress;
        self.current_step = 0;
        self.elapsed_ticks = 0;
    }

    /// Advance one tick of the transition.
    pub fn tick(&mut self) -> Option<&str> {
        if self.status != TransitionStatus::InProgress {
            return None;
        }
        self.elapsed_ticks += 1;

        // Check if complete first
        if self.elapsed_ticks >= self.duration_ticks {
            self.status = TransitionStatus::Complete;
        }

        // Determine which step we're on based on progress
        let progress = (self.elapsed_ticks as f64 / self.duration_ticks as f64).min(1.0);
        let step_index = ((progress * self.steps.len() as f64) as usize).min(self.steps.len() - 1);

        if step_index != self.current_step {
            self.current_step = step_index;
            return Some(&self.steps[step_index]);
        }

        None
    }

    /// Get current step description.
    pub fn current_step_description(&self) -> Option<&str> {
        if self.current_step < self.steps.len() {
            Some(&self.steps[self.current_step])
        } else {
            None
        }
    }

    /// Progress as a fraction (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        if self.duration_ticks == 0 {
            return 1.0;
        }
        (self.elapsed_ticks as f64 / self.duration_ticks as f64).min(1.0)
    }
}

/// The sleep/wake transition manager that orchestrates transitions.
pub struct TransitionManager {
    pub current_protocol: Option<TransitionProtocol>,
    pub completed_transitions: Vec<TransitionProtocol>,
    circadian: CircadianRhythm,
}

impl TransitionManager {
    pub fn new() -> Self {
        Self {
            current_protocol: None,
            completed_transitions: Vec::new(),
            circadian: CircadianRhythm::standard(),
        }
    }

    /// Initiate a transition between two dream states.
    pub fn initiate(&mut self, from: DreamState, to: DreamState, duration: u64) -> Result<&TransitionProtocol, String> {
        let name = format!("{} -> {}", from, to);
        let mut protocol = TransitionProtocol::new(&name, from, to, duration);
        protocol.start();
        self.current_protocol = Some(protocol);
        Ok(self.current_protocol.as_ref().unwrap())
    }

    /// Process one tick of the current transition.
    pub fn process_tick(&mut self) -> Option<String> {
        let is_complete = self.current_protocol.as_ref()
            .map(|p| p.status == TransitionStatus::Complete)
            .unwrap_or(false);

        if is_complete {
            let completed = self.current_protocol.take().unwrap();
            self.completed_transitions.push(completed);
            return None;
        }

        if let Some(ref mut protocol) = self.current_protocol {
            return protocol.tick().map(|s| s.to_string());
        }
        None
    }

    /// Check if a transition is currently in progress.
    pub fn is_transitioning(&self) -> bool {
        self.current_protocol.is_some() && self.current_protocol.as_ref().unwrap().status == TransitionStatus::InProgress
    }

    /// Get progress of current transition.
    pub fn current_progress(&self) -> f64 {
        self.current_protocol.as_ref().map(|p| p.progress()).unwrap_or(1.0)
    }

    /// Determine the recommended target state based on circadian rhythm.
    pub fn recommended_state(&self, hour: f64) -> DreamState {
        self.circadian.target_state_at(hour)
    }

    /// Total completed transitions.
    pub fn total_completed(&self) -> usize {
        self.completed_transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_creation() {
        let p = TransitionProtocol::new("test", DreamState::Awake, DreamState::LightSleep, 10);
        assert_eq!(p.from, DreamState::Awake);
        assert_eq!(p.to, DreamState::LightSleep);
        assert_eq!(p.status, TransitionStatus::Pending);
        assert!(!p.steps.is_empty());
    }

    #[test]
    fn test_protocol_start() {
        let mut p = TransitionProtocol::new("test", DreamState::Awake, DreamState::LightSleep, 10);
        p.start();
        assert_eq!(p.status, TransitionStatus::InProgress);
        assert_eq!(p.current_step, 0);
    }

    #[test]
    fn test_protocol_ticks_to_completion() {
        let mut p = TransitionProtocol::new("test", DreamState::Awake, DreamState::LightSleep, 5);
        p.start();
        for _ in 0..5 {
            p.tick();
        }
        assert_eq!(p.status, TransitionStatus::Complete);
    }

    #[test]
    fn test_protocol_progress() {
        let mut p = TransitionProtocol::new("test", DreamState::Rem, DreamState::Awake, 10);
        p.start();
        for _ in 0..5 {
            p.tick();
        }
        let prog = p.progress();
        assert!(prog >= 0.4 && prog <= 0.6);
    }

    #[test]
    fn test_protocol_current_step() {
        let p = TransitionProtocol::new("test", DreamState::Awake, DreamState::LightSleep, 10);
        assert!(p.current_step_description().is_some());
    }

    #[test]
    fn test_manager_initiate() {
        let mut m = TransitionManager::new();
        let result = m.initiate(DreamState::Awake, DreamState::LightSleep, 10);
        assert!(result.is_ok());
        assert!(m.is_transitioning());
    }

    #[test]
    fn test_manager_process_tick() {
        let mut m = TransitionManager::new();
        m.initiate(DreamState::Rem, DreamState::Awake, 3);
        m.process_tick();
        assert!(m.is_transitioning());
    }

    #[test]
    fn test_manager_completion() {
        let mut m = TransitionManager::new();
        m.initiate(DreamState::Awake, DreamState::LightSleep, 2);
        m.process_tick();
        m.process_tick();
        m.process_tick(); // triggers completion
        assert!(!m.is_transitioning());
        assert_eq!(m.total_completed(), 1);
    }

    #[test]
    fn test_manager_recommended_state() {
        let m = TransitionManager::new();
        let state = m.recommended_state(10.0);
        assert_eq!(state, DreamState::Awake);
        let night_state = m.recommended_state(1.0);
        assert_eq!(night_state, DreamState::DeepSleep);
    }
}
