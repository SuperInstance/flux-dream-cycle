/// Creative association engine: link seemingly unrelated concepts.

use std::collections::HashMap;

/// An association link between two concepts.
#[derive(Clone, Debug)]
pub struct Association {
    pub concept_a: String,
    pub concept_b: String,
    pub strength: f64,      // 0.0 (weak) to 1.0 (strong)
    pub co_occurrences: u32, // how many times they appeared together
    pub created_at: u64,
}

/// A concept node in the association graph.
#[derive(Clone, Debug)]
pub struct Concept {
    pub name: String,
    pub category: String,
    pub associations: Vec<String>, // names of associated concepts
    pub activation: f64,           // current activation level (spreading activation)
}

/// Result of an association query.
#[derive(Clone, Debug)]
pub struct AssociationResult {
    pub concept: String,
    pub strength: f64,
    pub path: Vec<String>,
}

/// The creative association engine that builds and queries concept relationships.
pub struct CreativeEngine {
    concepts: HashMap<String, Concept>,
    associations: Vec<Association>,
    association_threshold: f64,
}

impl CreativeEngine {
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            associations: Vec::new(),
            association_threshold: 0.1,
        }
    }

    /// Register a new concept.
    pub fn add_concept(&mut self, name: &str, category: &str) {
        self.concepts.entry(name.to_string()).or_insert_with(|| Concept {
            name: name.to_string(),
            category: category.to_string(),
            associations: Vec::new(),
            activation: 0.0,
        });
    }

    /// Create an association between two concepts.
    pub fn associate(&mut self, a: &str, b: &str, tick: u64) -> Result<(), String> {
        if !self.concepts.contains_key(a) {
            return Err(format!("Concept '{}' not found", a));
        }
        if !self.concepts.contains_key(b) {
            return Err(format!("Concept '{}' not found", b));
        }
        if a == b {
            return Err("Cannot associate a concept with itself".to_string());
        }

        // Check if association already exists
        if let Some(existing) = self.associations.iter_mut().find(|assoc| {
            (assoc.concept_a == a && assoc.concept_b == b) || (assoc.concept_a == b && assoc.concept_b == a)
        }) {
            existing.co_occurrences += 1;
            existing.strength = (existing.strength + 0.1).min(1.0);
        } else {
            self.associations.push(Association {
                concept_a: a.to_string(),
                concept_b: b.to_string(),
                strength: 0.2,
                co_occurrences: 1,
                created_at: tick,
            });
        }

        // Update concept association lists
        if let Some(concept) = self.concepts.get_mut(a) {
            if !concept.associations.contains(&b.to_string()) {
                concept.associations.push(b.to_string());
            }
        }
        if let Some(concept) = self.concepts.get_mut(b) {
            if !concept.associations.contains(&a.to_string()) {
                concept.associations.push(a.to_string());
            }
        }

        Ok(())
    }

    /// Find concepts associated with a given concept.
    pub fn find_associations(&self, concept: &str) -> Vec<AssociationResult> {
        let mut results = Vec::new();
        for assoc in &self.associations {
            if assoc.concept_a == concept {
                results.push(AssociationResult {
                    concept: assoc.concept_b.clone(),
                    strength: assoc.strength,
                    path: vec![concept.to_string(), assoc.concept_b.clone()],
                });
            } else if assoc.concept_b == concept {
                results.push(AssociationResult {
                    concept: assoc.concept_a.clone(),
                    strength: assoc.strength,
                    path: vec![concept.to_string(), assoc.concept_a.clone()],
                });
            }
        }
        results.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Find creative bridges: paths connecting two seemingly unrelated concepts.
    pub fn find_bridge(&self, a: &str, b: &str) -> Option<Vec<String>> {
        if a == b {
            return Some(vec![a.to_string()]);
        }

        // BFS to find shortest path
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((a.to_string(), vec![a.to_string()]));
        visited.insert(a.to_string());

        while let Some((current, path)) = queue.pop_front() {
            let neighbors = self.concepts.get(&current)
                .map(|c| c.associations.clone())
                .unwrap_or_default();

            for neighbor in neighbors {
                if neighbor == b {
                    let mut result = path.clone();
                    result.push(neighbor);
                    return Some(result);
                }
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    let mut new_path = path.clone();
                    new_path.push(neighbor.clone());
                    queue.push_back((neighbor, new_path));
                }
            }
        }
        None
    }

    /// Spread activation from a concept to its neighbors (up to given depth).
    pub fn spread_activation(&mut self, concept: &str, initial_activation: f64, depth: usize) {
        if !self.concepts.contains_key(concept) {
            return;
        }

        // Reset all activations
        for c in self.concepts.values_mut() {
            c.activation = 0.0;
        }

        let mut current_wave = vec![concept.to_string()];
        if let Some(c) = self.concepts.get_mut(concept) {
            c.activation = initial_activation;
        }

        for _ in 0..depth {
            let mut next_wave = Vec::new();
            for current in &current_wave {
                let current_activation = self.concepts.get(current).map(|c| c.activation).unwrap_or(0.0);
                let neighbors = self.concepts.get(current)
                    .map(|c| c.associations.clone())
                    .unwrap_or_default();

                for neighbor in neighbors {
                    let decay = 0.5; // activation decays by half each hop
                    let new_activation = current_activation * decay;
                    if let Some(n) = self.concepts.get_mut(&neighbor) {
                        n.activation = n.activation.max(new_activation);
                    }
                    next_wave.push(neighbor);
                }
            }
            current_wave = next_wave;
        }
    }

    /// Get most activated concepts (sorted by activation level).
    pub fn most_activated(&self, n: usize) -> Vec<(&str, f64)> {
        let mut results: Vec<(&str, f64)> = self.concepts.iter()
            .filter(|(_, c)| c.activation > 0.0)
            .map(|(name, c)| (name.as_str(), c.activation))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().take(n).collect()
    }

    /// Get number of concepts and associations.
    pub fn stats(&self) -> (usize, usize) {
        (self.concepts.len(), self.associations.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_engine() -> CreativeEngine {
        let mut e = CreativeEngine::new();
        e.add_concept("sleep", "biology");
        e.add_concept("dreams", "psychology");
        e.add_concept("ocean", "nature");
        e.add_concept("moon", "astronomy");
        e.add_concept("tide", "nature");
        e.associate("sleep", "dreams", 0).unwrap();
        e.associate("moon", "tide", 0).unwrap();
        e.associate("ocean", "tide", 0).unwrap();
        e.associate("dreams", "moon", 0).unwrap();
        e
    }

    #[test]
    fn test_add_concept() {
        let mut e = CreativeEngine::new();
        e.add_concept("test", "cat");
        assert!(e.concepts.contains_key("test"));
    }

    #[test]
    fn test_associate() {
        let e = setup_engine();
        assert_eq!(e.stats(), (5, 4));
    }

    #[test]
    fn test_associate_same_concept_fails() {
        let mut e = CreativeEngine::new();
        e.add_concept("x", "cat");
        assert!(e.associate("x", "x", 0).is_err());
    }

    #[test]
    fn test_associate_missing_concept() {
        let mut e = CreativeEngine::new();
        e.add_concept("a", "cat");
        assert!(e.associate("a", "missing", 0).is_err());
    }

    #[test]
    fn test_find_associations() {
        let e = setup_engine();
        let results = e.find_associations("sleep");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].concept, "dreams");
    }

    #[test]
    fn test_find_bridge() {
        let e = setup_engine();
        let bridge = e.find_bridge("sleep", "ocean").unwrap();
        assert!(bridge.len() >= 3);
        assert_eq!(bridge[0], "sleep");
        assert_eq!(bridge[bridge.len() - 1], "ocean");
    }

    #[test]
    fn test_spread_activation() {
        let mut e = setup_engine();
        e.spread_activation("sleep", 1.0, 2);
        let activated = e.most_activated(10);
        assert!(activated.len() >= 2);
        assert_eq!(activated[0].0, "sleep");
    }

    #[test]
    fn test_repeated_association_strengthen() {
        let mut e = CreativeEngine::new();
        e.add_concept("a", "cat");
        e.add_concept("b", "cat");
        e.associate("a", "b", 0).unwrap();
        e.associate("a", "b", 1).unwrap();
        let results = e.find_associations("a");
        assert!(results[0].strength > 0.2);
    }
}
