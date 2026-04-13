pub mod circadian;
pub mod creative;
pub mod journal;
pub mod memory;
pub mod scheduler;
pub mod state_machine;
pub mod task;
pub mod transition;

pub use circadian::CircadianRhythm;
pub use creative::CreativeEngine;
pub use journal::{DreamJournal, DreamTag, Vividness};
pub use memory::{MemoryConsolidator, Importance};
pub use state_machine::{DreamState, DreamStateMachine};
pub use transition::{TransitionManager, TransitionProtocol};

#[cfg(test)]
mod tests {
    use super::scheduler::Scheduler;
    use super::task::{PENDING, RUNNING, COMPLETED, FAILED, CANCELLED, CRITICAL, HIGH, NORMAL, LOW};
    use super::state_machine::{DreamState, DreamStateMachine};
    use super::memory::{MemoryConsolidator, Importance};
    use super::creative::CreativeEngine;
    use super::journal::{DreamJournal, DreamTag, Vividness};
    use super::circadian::CircadianRhythm;
    use super::transition::TransitionManager;

    // ─── Task scheduler tests (original) ───

    #[test]
    fn test_new_scheduler_empty() {
        let s = Scheduler::new();
        assert!(s.tasks.is_empty());
    }

    #[test]
    fn test_add_returns_id() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        assert_eq!(id, 1);
        let id2 = s.add("task2", HIGH, 200, false, 0);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_find_existing() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        let t = s.find(id).unwrap();
        assert_eq!(t.name, "task1");
    }

    #[test]
    fn test_find_missing() {
        let s = Scheduler::new();
        assert!(s.find(999).is_none());
    }

    #[test]
    fn test_next_picks_highest_priority() {
        let mut s = Scheduler::new();
        s.add("low", LOW, 100, false, 0);
        s.add("crit", CRITICAL, 200, false, 0);
        s.add("high", HIGH, 300, false, 0);
        let n = s.next().unwrap();
        assert_eq!(n.name, "crit");
    }

    #[test]
    fn test_next_ignores_non_pending() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.start(id).unwrap();
        assert!(s.next().is_none());
    }

    #[test]
    fn test_start_success() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        assert!(s.start(id).is_ok());
        assert_eq!(s.find(id).unwrap().state, RUNNING);
    }

    #[test]
    fn test_start_not_found() {
        let mut s = Scheduler::new();
        assert!(s.start(999).is_err());
    }

    #[test]
    fn test_start_not_pending() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.start(id).unwrap();
        assert!(s.start(id).is_err());
    }

    #[test]
    fn test_complete_non_recurring() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.start(id).unwrap();
        s.complete(id).unwrap();
        assert_eq!(s.find(id).unwrap().state, COMPLETED);
        assert_eq!(s.find(id).unwrap().run_count, 1);
    }

    #[test]
    fn test_complete_recurring_resets() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, true, 50);
        s.start(id).unwrap();
        s.complete(id).unwrap();
        assert_eq!(s.find(id).unwrap().state, PENDING);
        assert_eq!(s.find(id).unwrap().run_count, 1);
    }

    #[test]
    fn test_complete_not_running() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        assert!(s.complete(id).is_err());
    }

    #[test]
    fn test_fail_with_retries() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.start(id).unwrap();
        s.fail(id).unwrap();
        assert_eq!(s.find(id).unwrap().state, PENDING);
        assert_eq!(s.find(id).unwrap().retries_left, 2);
    }

    #[test]
    fn test_fail_no_retries() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.tasks[0].retries_left = 0;
        s.start(id).unwrap();
        s.fail(id).unwrap();
        assert_eq!(s.find(id).unwrap().state, FAILED);
    }

    #[test]
    fn test_cancel_pending() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.cancel(id).unwrap();
        assert_eq!(s.find(id).unwrap().state, CANCELLED);
    }

    #[test]
    fn test_cancel_completed_fails() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.start(id).unwrap();
        s.complete(id).unwrap();
        assert!(s.cancel(id).is_err());
    }

    #[test]
    fn test_overdue() {
        let mut s = Scheduler::new();
        s.add("overdue_task", NORMAL, 50, false, 0);
        let od = s.overdue(100);
        assert_eq!(od.len(), 1);
    }

    #[test]
    fn test_by_state() {
        let mut s = Scheduler::new();
        let id = s.add("task1", NORMAL, 100, false, 0);
        s.start(id).unwrap();
        let running = s.by_state(RUNNING);
        assert_eq!(running.len(), 1);
        let pending = s.by_state(PENDING);
        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn test_add_task_fields() {
        let mut s = Scheduler::new();
        let id = s.add("fields", HIGH, 999, true, 60);
        let t = s.find(id).unwrap();
        assert_eq!(t.priority, HIGH);
        assert!(t.recurring);
        assert_eq!(t.recurr_interval, 60);
        assert_eq!(t.retries_left, 3);
        assert_eq!(t.run_count, 0);
    }

    #[test]
    fn test_next_empty() {
        let s = Scheduler::new();
        assert!(s.next().is_none());
    }

    // ─── Integration tests ───

    #[test]
    fn test_full_dream_cycle_integration() {
        let mut sm = DreamStateMachine::new();
        let mut mc = MemoryConsolidator::new();
        let mut journal = DreamJournal::new();
        let mut engine = CreativeEngine::new();
        let circadian = CircadianRhythm::standard();
        let mut tm = TransitionManager::new();

        // Set up concepts for creative engine
        engine.add_concept("task_done", "work");
        engine.add_concept("satisfaction", "emotion");
        engine.add_concept("rest", "biology");
        engine.associate("task_done", "satisfaction", 0).unwrap();
        engine.associate("satisfaction", "rest", 0).unwrap();

        // Create a memory before sleep
        let mem_id = mc.create_memory("completed important project", Importance::Critical, 0);
        mc.access(mem_id, 1);

        // Transition to sleep
        tm.initiate(DreamState::Awake, DreamState::LightSleep, 3).unwrap();
        sm.advance();
        sm.transition(DreamState::LightSleep).unwrap();

        // Deepen sleep
        sm.advance();
        sm.transition(DreamState::DeepSleep).unwrap();

        // Consolidate memories during deep sleep
        mc.consolidate(2);

        // Enter REM
        sm.advance();
        sm.transition(DreamState::Rem).unwrap();

        // Log a dream during REM
        journal.record(3, "Sailing on a sea of completed tasks", Vividness::Vivid, 0.8,
            vec![DreamTag::Lucid], vec!["sailing".to_string(), "tasks".to_string()]);

        // Check circadian recommendation
        let rec = circadian.target_state_at(2.0); // 2 AM
        assert_eq!(rec, DreamState::DeepSleep);

        // Wake up
        sm.advance();
        sm.transition(DreamState::LightSleep).unwrap();
        sm.advance();
        sm.transition(DreamState::Awake).unwrap();

        // Verify state
        assert_eq!(sm.state, DreamState::Awake);
        assert!(mc.get(mem_id).is_some());
        assert_eq!(journal.len(), 1);

        // Use creative engine for insight
        let bridge = engine.find_bridge("task_done", "rest");
        assert!(bridge.is_some());
    }

    #[test]
    fn test_sleep_cycle_with_circadian_tracking() {
        let mut sm = DreamStateMachine::new();
        let circadian = CircadianRhythm::standard();

        // Simulate a full day-night cycle at 2-hour intervals
        let hours: Vec<f64> = (0..24).map(|h| h as f64).collect();
        let mut transitions = 0;

        for hour in hours {
            let target = circadian.target_state_at(hour);
            if sm.state != target && sm.state.can_transition_to(&target) {
                sm.advance();
                sm.transition(target).unwrap();
                transitions += 1;
            }
        }

        // Should have made several transitions over 24 hours
        assert!(transitions > 3);
    }

    #[test]
    fn test_memory_consolidation_during_sleep_cycle() {
        let mut mc = MemoryConsolidator::new();

        // Create memories of varying importance
        mc.create_memory("critical insight", Importance::Critical, 0);
        mc.create_memory("moderate observation", Importance::Medium, 0);
        mc.create_memory("trivial detail", Importance::Trivial, 0);

        let initial_count = mc.memory_count();

        // Run multiple consolidation passes (simulating a full night)
        for tick in 1..=5 {
            mc.consolidate(tick);
        }

        // Critical memory should still exist
        assert!(mc.memory_count() > 0);
        assert!(mc.memory_count() <= initial_count); // Some may have been forgotten
    }

    #[test]
    fn test_journal_analysis_workflow() {
        let mut journal = DreamJournal::new();

        // Record several dreams
        journal.record(1, "Falling into darkness", Vividness::HyperReal, 0.95,
            vec![DreamTag::Nightmare, DreamTag::Falling], vec!["darkness".to_string()]);
        journal.record(2, "Flying over the city", Vividness::Vivid, 0.7,
            vec![DreamTag::Flying, DreamTag::Lucid], vec!["city".to_string(), "flying".to_string()]);
        journal.record(3, "Being chased through a forest", Vividness::Vivid, 0.85,
            vec![DreamTag::Nightmare, DreamTag::Pursuit], vec!["forest".to_string(), "chased".to_string()]);
        journal.record(4, "Turning into a bird", Vividness::HyperReal, 0.8,
            vec![DreamTag::Transformation, DreamTag::Flying], vec!["bird".to_string()]);

        // Add analyses
        journal.add_analysis(1, "Anxiety about losing control").unwrap();
        journal.add_analysis(2, "Desire for freedom").unwrap();

        // Summarize
        let summary = journal.summarize();
        assert_eq!(summary.total_entries, 4);
        assert_eq!(summary.nightmare_count, 2);

        // Search
        let flying = journal.filter_by_tag(&DreamTag::Flying);
        assert_eq!(flying.len(), 2);
    }

    #[test]
    fn test_creative_insight_from_sleep() {
        let mut engine = CreativeEngine::new();

        // Build a concept network
        engine.add_concept("bug", "programming");
        engine.add_concept("feature", "programming");
        engine.add_concept("butterfly", "nature");
        engine.add_concept("metamorphosis", "nature");
        engine.add_concept("refactor", "programming");

        engine.associate("bug", "feature", 0).unwrap();
        engine.associate("butterfly", "metamorphosis", 0).unwrap();
        engine.associate("bug", "butterfly", 0).unwrap();
        engine.associate("metamorphosis", "refactor", 0).unwrap();

        // Spread activation from "bug"
        engine.spread_activation("bug", 1.0, 3);
        let _activated = engine.most_activated(5);

        // Should reach "refactor" through creative bridge
        let bridge = engine.find_bridge("bug", "refactor");
        assert!(bridge.is_some());
        assert!(bridge.unwrap().len() >= 3);
    }
}
