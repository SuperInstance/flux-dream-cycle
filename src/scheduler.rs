use crate::task::{Task, TaskPriority, TaskState, PENDING, RUNNING, COMPLETED, FAILED, CANCELLED};

pub struct Scheduler {
    pub tasks: Vec<Task>,
    next_id: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler { tasks: Vec::new(), next_id: 1 }
    }

    pub fn add(&mut self, name: &str, priority: TaskPriority, deadline: u64, recurring: bool, interval: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let now = 0; // caller can update created if needed
        self.tasks.push(Task {
            id, name: name.to_string(), state: PENDING, priority,
            created: now, started: 0, deadline,
            retries_left: 3, recurring, recurr_interval: interval,
            last_completed: 0, run_count: 0,
        });
        id
    }

    pub fn find(&self, id: u32) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn next(&self) -> Option<&Task> {
        self.tasks.iter()
            .filter(|t| t.state == PENDING)
            .min_by_key(|t| t.priority)
    }

    pub fn start(&mut self, id: u32) -> Result<(), &str> {
        let task = self.tasks.iter_mut().find(|t| t.id == id).ok_or("task not found")?;
        if task.state != PENDING { return Err("task not pending"); }
        task.state = RUNNING;
        task.started = 0; // placeholder
        Ok(())
    }

    pub fn complete(&mut self, id: u32) -> Result<(), &str> {
        let task = self.tasks.iter_mut().find(|t| t.id == id).ok_or("task not found")?;
        if task.state != RUNNING { return Err("task not running"); }
        task.last_completed = 0;
        task.run_count += 1;
        if task.recurring {
            task.state = PENDING;
        } else {
            task.state = COMPLETED;
        }
        Ok(())
    }

    pub fn fail(&mut self, id: u32) -> Result<(), &str> {
        let task = self.tasks.iter_mut().find(|t| t.id == id).ok_or("task not found")?;
        if task.state != RUNNING { return Err("task not running"); }
        if task.retries_left > 0 {
            task.retries_left -= 1;
            task.state = PENDING;
        } else {
            task.state = FAILED;
        }
        Ok(())
    }

    pub fn cancel(&mut self, id: u32) -> Result<(), &str> {
        let task = self.tasks.iter_mut().find(|t| t.id == id).ok_or("task not found")?;
        if task.state == COMPLETED || task.state == CANCELLED { return Err("cannot cancel"); }
        task.state = CANCELLED;
        Ok(())
    }

    pub fn overdue(&self, now: u64) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.state != COMPLETED && t.state != CANCELLED && t.deadline > 0 && now > t.deadline).collect()
    }

    pub fn by_state(&self, state: TaskState) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.state == state).collect()
    }
}
