# flux-dream-cycle

> Priority-based task scheduler with recurring tasks, retry logic, and deadline tracking for FLUX agents.

## What This Is

`flux-dream-cycle` is a Rust crate implementing a **task scheduler** — it manages tasks with 6 priority levels (Critical, High, Normal, Low, Background, Deferred), supports recurring tasks with intervals, automatic retry on failure, deadline-based overdue detection, and state tracking.

## Role in the FLUX Ecosystem

Agents in the FLUX fleet juggle multiple responsibilities. `flux-dream-cycle` is their task orchestration brain:

- **`flux-navigate`** tasks (move to waypoint) are scheduled as High priority
- **`flux-evolve`** cycles run as recurring Normal tasks
- **`flux-social`** group coordination happens through scheduled tasks
- **`flux-trust`** periodic decay calls are scheduled as Background tasks
- **`flux-simulator`** simulates multi-step workflows via task sequences

The name "dream cycle" reflects the FLUX philosophy: agents alternate between active execution and reflective consolidation, much like dreaming consolidates waking experience.

## Key Features

| Feature | Description |
|---------|-------------|
| **6 Priority Levels** | CRITICAL > HIGH > NORMAL > LOW > BACKGROUND > DEFERRED |
| **Recurring Tasks** | Auto-reset to PENDING on completion with configurable intervals |
| **Retry Logic** | 3 automatic retries before marking FAILED |
| **Deadline Tracking** | `overdue(now)` lists tasks past their deadline |
| **State Machine** | PENDING → RUNNING → COMPLETED/FAILED/CANCELLED |
| **State Filtering** | `by_state(RUNNING)` queries tasks in any state |

## Quick Start

```rust
use flux_dream_cycle::{Scheduler, NORMAL, HIGH, CRITICAL};

let mut scheduler = Scheduler::new();

// Add tasks with priority and deadline
let nav_task = scheduler.add("navigate_to_base", HIGH, 500, false, 0);
let pulse = scheduler.add("heartbeat", NORMAL, 0, true, 60); // recurring every 60 ticks
let critical = scheduler.add("respond_to_fence", CRITICAL, 10, false, 0);

// Pick next task (highest priority, PENDING)
if let Some(task) = scheduler.next() {
    println!("Next: {} (priority {:?})", task.name, task.priority);
    scheduler.start(task.id).unwrap();
    // ... execute ...
    scheduler.complete(task.id).unwrap(); // or scheduler.fail(task.id).unwrap()
}

// Check overdue tasks
let overdue = scheduler.overdue(1000);
for task in overdue {
    println!("OVERDUE: {}", task.name);
}

// Filter by state
let running = scheduler.by_state(RUNNING);
```

## Building & Testing

```bash
cargo build
cargo test
```

## Related Fleet Repos

- [`flux-navigate`](https://github.com/SuperInstance/flux-navigate) — Pathfinding tasks
- [`flux-evolve`](https://github.com/SuperInstance/flux-evolve) — Recurring evolution cycles
- [`flux-social`](https://github.com/SuperInstance/flux-social) — Group coordination tasks
- [`flux-trust`](https://github.com/SuperInstance/flux-trust) — Periodic trust decay
- [`flux-simulator`](https://github.com/SuperInstance/flux-simulator) — Task-based fleet simulation

## License

Part of the [SuperInstance](https://github.com/SuperInstance) FLUX fleet.
