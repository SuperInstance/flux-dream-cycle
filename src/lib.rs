pub mod scheduler;
pub mod task;

#[cfg(test)]
mod tests {
    use super::scheduler::Scheduler;
    use super::task::{PENDING, RUNNING, COMPLETED, FAILED, CANCELLED, DEFERRED, CRITICAL, HIGH, NORMAL, LOW, BACKGROUND};

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
}
