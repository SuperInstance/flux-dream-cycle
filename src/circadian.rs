/// Circadian rhythm cycle management.

use crate::state_machine::DreamState;

/// One phase of the circadian rhythm.
#[derive(Clone, Debug)]
pub struct CircadianPhase {
    pub name: String,
    pub start_hour: f64,   // 0.0 - 24.0
    pub duration_hours: f64,
    pub target_state: DreamState,
    pub alertness: f64,     // 0.0 (groggy) to 1.0 (alert)
    pub creativity: f64,    // 0.0 to 1.0
}

/// A complete circadian cycle configuration.
pub struct CircadianRhythm {
    pub phases: Vec<CircadianPhase>,
    pub total_cycle_hours: f64,
}

impl CircadianRhythm {
    /// Create a standard human-like circadian rhythm.
    pub fn standard() -> Self {
        let phases = vec![
            CircadianPhase {
                name: "Deep Night".to_string(),
                start_hour: 0.0,
                duration_hours: 3.0,
                target_state: DreamState::DeepSleep,
                alertness: 0.05,
                creativity: 0.1,
            },
            CircadianPhase {
                name: "Late Sleep".to_string(),
                start_hour: 3.0,
                duration_hours: 2.0,
                target_state: DreamState::Rem,
                alertness: 0.1,
                creativity: 0.8,
            },
            CircadianPhase {
                name: "Dawn REM".to_string(),
                start_hour: 5.0,
                duration_hours: 2.0,
                target_state: DreamState::Lucid,
                alertness: 0.2,
                creativity: 0.9,
            },
            CircadianPhase {
                name: "Morning Wake".to_string(),
                start_hour: 7.0,
                duration_hours: 2.0,
                target_state: DreamState::LightSleep,
                alertness: 0.4,
                creativity: 0.5,
            },
            CircadianPhase {
                name: "Morning Peak".to_string(),
                start_hour: 9.0,
                duration_hours: 3.0,
                target_state: DreamState::Awake,
                alertness: 0.9,
                creativity: 0.7,
            },
            CircadianPhase {
                name: "Midday".to_string(),
                start_hour: 12.0,
                duration_hours: 2.0,
                target_state: DreamState::Awake,
                alertness: 0.75,
                creativity: 0.6,
            },
            CircadianPhase {
                name: "Afternoon Dip".to_string(),
                start_hour: 14.0,
                duration_hours: 2.0,
                target_state: DreamState::LightSleep,
                alertness: 0.4,
                creativity: 0.4,
            },
            CircadianPhase {
                name: "Late Afternoon".to_string(),
                start_hour: 16.0,
                duration_hours: 3.0,
                target_state: DreamState::Awake,
                alertness: 0.7,
                creativity: 0.65,
            },
            CircadianPhase {
                name: "Evening Wind Down".to_string(),
                start_hour: 19.0,
                duration_hours: 2.0,
                target_state: DreamState::Awake,
                alertness: 0.5,
                creativity: 0.7,
            },
            CircadianPhase {
                name: "Pre-Sleep".to_string(),
                start_hour: 21.0,
                duration_hours: 2.0,
                target_state: DreamState::LightSleep,
                alertness: 0.2,
                creativity: 0.5,
            },
            CircadianPhase {
                name: "Sleep Onset".to_string(),
                start_hour: 23.0,
                duration_hours: 1.0,
                target_state: DreamState::LightSleep,
                alertness: 0.1,
                creativity: 0.3,
            },
        ];
        Self {
            total_cycle_hours: 24.0,
            phases,
        }
    }

    /// Get the current phase based on a given hour.
    pub fn phase_at(&self, hour: f64) -> &CircadianPhase {
        let wrapped_hour = hour % self.total_cycle_hours;
        self.phases.iter()
            .find(|p| {
                let end = p.start_hour + p.duration_hours;
                if end <= self.total_cycle_hours {
                    wrapped_hour >= p.start_hour && wrapped_hour < end
                } else {
                    wrapped_hour >= p.start_hour || wrapped_hour < end - self.total_cycle_hours
                }
            })
            .unwrap_or(&self.phases[0])
    }

    /// Get alertness level at a given hour.
    pub fn alertness_at(&self, hour: f64) -> f64 {
        self.phase_at(hour).alertness
    }

    /// Get creativity level at a given hour.
    pub fn creativity_at(&self, hour: f64) -> f64 {
        self.phase_at(hour).creativity
    }

    /// Get target dream state at a given hour.
    pub fn target_state_at(&self, hour: f64) -> DreamState {
        self.phase_at(hour).target_state
    }

    /// Get the name of the current phase.
    pub fn phase_name_at(&self, hour: f64) -> &str {
        &self.phase_at(hour).name
    }

    /// Get all sleep hours in the cycle.
    pub fn sleep_hours(&self) -> Vec<f64> {
        self.phases.iter()
            .filter(|p| p.target_state != DreamState::Awake)
            .map(|p| p.duration_hours)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_creation() {
        let rhythm = CircadianRhythm::standard();
        assert_eq!(rhythm.total_cycle_hours, 24.0);
        assert!(!rhythm.phases.is_empty());
    }

    #[test]
    fn test_phase_at_midnight() {
        let rhythm = CircadianRhythm::standard();
        let phase = rhythm.phase_at(0.0);
        assert_eq!(phase.name, "Deep Night");
        assert_eq!(phase.target_state, DreamState::DeepSleep);
    }

    #[test]
    fn test_phase_at_morning() {
        let rhythm = CircadianRhythm::standard();
        let phase = rhythm.phase_at(10.0);
        assert_eq!(phase.name, "Morning Peak");
        assert_eq!(phase.target_state, DreamState::Awake);
    }

    #[test]
    fn test_alertness_at_peak() {
        let rhythm = CircadianRhythm::standard();
        let alertness = rhythm.alertness_at(10.0);
        assert!(alertness > 0.8);
    }

    #[test]
    fn test_alertness_at_night() {
        let rhythm = CircadianRhythm::standard();
        let alertness = rhythm.alertness_at(1.0);
        assert!(alertness < 0.2);
    }

    #[test]
    fn test_creativity_at_rem() {
        let rhythm = CircadianRhythm::standard();
        let creativity = rhythm.creativity_at(4.0);
        assert!(creativity > 0.7);
    }

    #[test]
    fn test_wrapped_hour() {
        let rhythm = CircadianRhythm::standard();
        // 25.0 should wrap to 1.0
        let phase = rhythm.phase_at(25.0);
        assert_eq!(phase.name, "Deep Night");
    }

    #[test]
    fn test_sleep_hours() {
        let rhythm = CircadianRhythm::standard();
        let sleep_h = rhythm.sleep_hours();
        let total_sleep: f64 = sleep_h.iter().sum();
        // Should be roughly 8-14 hours of sleep
        assert!(total_sleep > 6.0);
        assert!(total_sleep <= 16.0);
    }
}
