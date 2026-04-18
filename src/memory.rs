/// Memory consolidation simulation: replay and strengthen important memories during sleep.

use std::collections::HashMap;

/// Importance level of a memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Importance {
    Trivial = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// A single memory unit that can be consolidated.
#[derive(Clone, Debug)]
pub struct Memory {
    pub id: u32,
    pub content: String,
    pub importance: Importance,
    pub strength: f64,       // 0.0 (forgotten) to 1.0 (perfectly remembered)
    pub access_count: u32,    // how many times this memory has been accessed
    pub last_accessed: u64,   // tick of last access
    pub created_at: u64,      // tick of creation
    pub replay_count: u32,    // how many times replayed during consolidation
}

/// A record of a consolidation pass.
#[derive(Clone, Debug)]
pub struct ConsolidationRecord {
    pub tick: u64,
    pub memories_replayed: usize,
    pub memories_strengthened: usize,
    pub memories_forgotten: usize,
}

/// The memory consolidation engine that replays and strengthens memories during sleep.
pub struct MemoryConsolidator {
    pub memories: HashMap<u32, Memory>,
    next_id: u32,
    pub consolidation_log: Vec<ConsolidationRecord>,
    decay_rate: f64,          // how much strength decays per tick
    replay_boost: f64,        // how much strength increases per replay
    forgetting_threshold: f64, // strength below which a memory is forgotten
}

impl MemoryConsolidator {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            next_id: 1,
            consolidation_log: Vec::new(),
            decay_rate: 0.01,
            replay_boost: 0.15,
            forgetting_threshold: 0.05,
        }
    }

    /// Create a new memory and return its ID.
    pub fn create_memory(&mut self, content: &str, importance: Importance, tick: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let initial_strength = match importance {
            Importance::Trivial => 0.2,
            Importance::Low => 0.4,
            Importance::Medium => 0.6,
            Importance::High => 0.8,
            Importance::Critical => 1.0,
        };
        self.memories.insert(id, Memory {
            id,
            content: content.to_string(),
            importance,
            strength: initial_strength,
            access_count: 0,
            last_accessed: tick,
            created_at: tick,
            replay_count: 0,
        });
        id
    }

    /// Access a memory, boosting its access count and last_accessed time.
    pub fn access(&mut self, id: u32, tick: u64) -> Option<&Memory> {
        if let Some(mem) = self.memories.get_mut(&id) {
            mem.access_count += 1;
            mem.last_accessed = tick;
            Some(mem)
        } else {
            None
        }
    }

    /// Apply natural decay to all memories for one tick.
    pub fn decay(&mut self) {
        for mem in self.memories.values_mut() {
            mem.strength = (mem.strength - self.decay_rate).max(0.0);
        }
    }

    /// Consolidate memories: replay important ones to strengthen them,
    /// decay others, and forget those below threshold.
    /// Higher importance memories are replayed first.
    pub fn consolidate(&mut self, tick: u64) -> ConsolidationRecord {
        let mut memories_replayed = 0usize;
        let mut memories_strengthened = 0usize;
        let mut memories_forgotten = 0usize;

        // Sort memories by importance (descending), then by strength (ascending, weakest first)
        let mut sorted: Vec<u32> = self.memories.keys().copied().collect();
        sorted.sort_by(|&a, &b| {
            let ma = &self.memories[&a];
            let mb = &self.memories[&b];
            mb.importance.cmp(&ma.importance)
                .then_with(|| ma.strength.partial_cmp(&mb.strength).unwrap_or(std::cmp::Ordering::Equal))
        });

        for id in &sorted {
            if let Some(mem) = self.memories.get_mut(id) {
                // More important memories get replayed
                if mem.importance >= Importance::Medium {
                    mem.strength = (mem.strength + self.replay_boost).min(1.0);
                    mem.replay_count += 1;
                    memories_replayed += 1;
                    if mem.strength > 0.5 {
                        memories_strengthened += 1;
                    }
                }

                // Apply decay
                mem.strength = (mem.strength - self.decay_rate).max(0.0);

                // Forget weak memories
                if mem.strength < self.forgetting_threshold && mem.importance < Importance::High {
                    memories_forgotten += 1;
                }
            }
        }

        // Remove forgotten memories
        let to_remove: Vec<u32> = self.memories.iter()
            .filter(|(_, mem)| mem.strength < self.forgetting_threshold && mem.importance < Importance::High)
            .map(|(&id, _)| id)
            .collect();
        for id in to_remove {
            self.memories.remove(&id);
        }

        let record = ConsolidationRecord {
            tick,
            memories_replayed,
            memories_strengthened,
            memories_forgotten,
        };
        self.consolidation_log.push(record.clone());
        record
    }

    /// Get a memory by ID.
    pub fn get(&self, id: u32) -> Option<&Memory> {
        self.memories.get(&id)
    }

    /// Get all memories sorted by strength (strongest first).
    pub fn strongest_memories(&self, n: usize) -> Vec<&Memory> {
        let mut sorted: Vec<&Memory> = self.memories.values().collect();
        sorted.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(n).collect()
    }

    /// Get total number of active memories.
    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memory() {
        let mut mc = MemoryConsolidator::new();
        let id = mc.create_memory("test memory", Importance::High, 0);
        assert_eq!(id, 1);
        let mem = mc.get(id).unwrap();
        assert_eq!(mem.content, "test memory");
        assert_eq!(mem.importance, Importance::High);
    }

    #[test]
    fn test_initial_strength_by_importance() {
        let mut mc = MemoryConsolidator::new();
        let id_crit = mc.create_memory("critical", Importance::Critical, 0);
        let id_trivial = mc.create_memory("trivial", Importance::Trivial, 0);
        assert_eq!(mc.get(id_crit).unwrap().strength, 1.0);
        assert_eq!(mc.get(id_trivial).unwrap().strength, 0.2);
    }

    #[test]
    fn test_access_memory() {
        let mut mc = MemoryConsolidator::new();
        let id = mc.create_memory("accessed", Importance::Medium, 10);
        mc.access(id, 20);
        assert_eq!(mc.get(id).unwrap().access_count, 1);
        assert_eq!(mc.get(id).unwrap().last_accessed, 20);
    }

    #[test]
    fn test_consolidate_strengthen() {
        let mut mc = MemoryConsolidator::new();
        let id = mc.create_memory("important", Importance::High, 0);
        let record = mc.consolidate(1);
        assert!(record.memories_replayed > 0);
        // After consolidation, strength should increase
        let mem = mc.get(id).unwrap();
        assert!(mem.replay_count > 0);
    }

    #[test]
    fn test_consolidate_forget_weak() {
        let mut mc = MemoryConsolidator::new();
        let id = mc.create_memory("weak", Importance::Trivial, 0);
        // Manually weaken it
        mc.memories.get_mut(&id).unwrap().strength = 0.03;
        mc.consolidate(1);
        assert!(mc.get(id).is_none());
    }

    #[test]
    fn test_strongest_memories() {
        let mut mc = MemoryConsolidator::new();
        mc.create_memory("strong", Importance::Critical, 0);
        mc.create_memory("weak", Importance::Trivial, 0);
        let strongest = mc.strongest_memories(1);
        assert_eq!(strongest[0].content, "strong");
    }

    #[test]
    fn test_memory_count() {
        let mut mc = MemoryConsolidator::new();
        assert_eq!(mc.memory_count(), 0);
        mc.create_memory("a", Importance::Medium, 0);
        mc.create_memory("b", Importance::Low, 0);
        assert_eq!(mc.memory_count(), 2);
    }
}
