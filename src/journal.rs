/// Dream journal: log experiences for later analysis.

use std::collections::HashMap;

/// The vividness/quality level of a dream entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vividness {
    Faint = 0,
    Moderate = 1,
    Vivid = 2,
    HyperReal = 3,
}

/// Tags that can be applied to journal entries.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DreamTag {
    Nightmare,
    Recurring,
    Lucid,
    Prophetic,
    Absurd,
    Emotional,
    Flying,
    Falling,
    Pursuit,
    Transformation,
}

impl std::fmt::Display for DreamTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DreamTag::Nightmare => write!(f, "nightmare"),
            DreamTag::Recurring => write!(f, "recurring"),
            DreamTag::Lucid => write!(f, "lucid"),
            DreamTag::Prophetic => write!(f, "prophetic"),
            DreamTag::Absurd => write!(f, "absurd"),
            DreamTag::Emotional => write!(f, "emotional"),
            DreamTag::Flying => write!(f, "flying"),
            DreamTag::Falling => write!(f, "falling"),
            DreamTag::Pursuit => write!(f, "pursuit"),
            DreamTag::Transformation => write!(f, "transformation"),
        }
    }
}

/// A single journal entry recording a dream experience.
#[derive(Clone, Debug)]
pub struct JournalEntry {
    pub id: u32,
    pub tick: u64,
    pub description: String,
    pub vividness: Vividness,
    pub emotional_intensity: f64, // 0.0 (calm) to 1.0 (intense)
    pub tags: Vec<DreamTag>,
    pub keywords: Vec<String>,
    pub analysis: Option<String>,  // post-waking analysis
}

/// A summary of journal statistics.
#[derive(Clone, Debug, Default)]
pub struct JournalSummary {
    pub total_entries: usize,
    pub most_common_tag: Option<DreamTag>,
    pub most_common_keyword: Option<String>,
    pub average_vividness: f64,
    pub average_emotional_intensity: f64,
    pub nightmare_count: usize,
    pub lucid_count: usize,
}

/// The dream journal that records and analyzes dream experiences.
pub struct DreamJournal {
    entries: Vec<JournalEntry>,
    next_id: u32,
}

impl DreamJournal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    /// Record a new dream entry.
    pub fn record(
        &mut self,
        tick: u64,
        description: &str,
        vividness: Vividness,
        emotional_intensity: f64,
        tags: Vec<DreamTag>,
        keywords: Vec<String>,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(JournalEntry {
            id,
            tick,
            description: description.to_string(),
            vividness,
            emotional_intensity: emotional_intensity.clamp(0.0, 1.0),
            tags,
            keywords,
            analysis: None,
        });
        id
    }

    /// Add a post-waking analysis to an entry.
    pub fn add_analysis(&mut self, id: u32, analysis: &str) -> Result<(), String> {
        let entry = self.entries.iter_mut().find(|e| e.id == id)
            .ok_or("Entry not found")?;
        entry.analysis = Some(analysis.to_string());
        Ok(())
    }

    /// Get an entry by ID.
    pub fn get(&self, id: u32) -> Option<&JournalEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get all entries.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Search entries by keyword.
    pub fn search_by_keyword(&self, keyword: &str) -> Vec<&JournalEntry> {
        self.entries.iter()
            .filter(|e| e.keywords.iter().any(|k| k.eq_ignore_ascii_case(keyword)) || e.description.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    /// Filter entries by tag.
    pub fn filter_by_tag(&self, tag: &DreamTag) -> Vec<&JournalEntry> {
        self.entries.iter()
            .filter(|e| e.tags.contains(tag))
            .collect()
    }

    /// Get entries within a tick range.
    pub fn entries_in_range(&self, start: u64, end: u64) -> Vec<&JournalEntry> {
        self.entries.iter()
            .filter(|e| e.tick >= start && e.tick <= end)
            .collect()
    }

    /// Generate a summary of the journal.
    pub fn summarize(&self) -> JournalSummary {
        if self.entries.is_empty() {
            return JournalSummary::default();
        }

        let total = self.entries.len();

        // Count tag frequencies
        let mut tag_counts: HashMap<&DreamTag, usize> = HashMap::new();
        for entry in &self.entries {
            for tag in &entry.tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }
        let most_common_tag = tag_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(tag, _)| tag.clone());

        // Count keyword frequencies
        let mut kw_counts: HashMap<&str, usize> = HashMap::new();
        for entry in &self.entries {
            for kw in &entry.keywords {
                *kw_counts.entry(kw.as_str()).or_insert(0) += 1;
            }
        }
        let most_common_keyword = kw_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(kw, _)| kw.to_string());

        let avg_vividness = self.entries.iter().map(|e| e.vividness as u32).sum::<u32>() as f64 / total as f64;
        let avg_emotional = self.entries.iter().map(|e| e.emotional_intensity).sum::<f64>() / total as f64;
        let nightmare_count = self.entries.iter().filter(|e| e.tags.contains(&DreamTag::Nightmare)).count();
        let lucid_count = self.entries.iter().filter(|e| e.tags.contains(&DreamTag::Lucid)).count();

        JournalSummary {
            total_entries: total,
            most_common_tag,
            most_common_keyword,
            average_vividness: avg_vividness,
            average_emotional_intensity: avg_emotional,
            nightmare_count,
            lucid_count,
        }
    }

    /// Get the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_entry() {
        let mut j = DreamJournal::new();
        let id = j.record(10, "Flying over mountains", Vividness::Vivid, 0.7, vec![DreamTag::Flying], vec!["mountains".to_string()]);
        assert_eq!(id, 1);
        assert_eq!(j.len(), 1);
    }

    #[test]
    fn test_add_analysis() {
        let mut j = DreamJournal::new();
        let id = j.record(10, "A dream", Vividness::Moderate, 0.5, vec![], vec![]);
        j.add_analysis(id, "Represents freedom").unwrap();
        assert_eq!(j.get(id).unwrap().analysis.as_deref(), Some("Represents freedom"));
    }

    #[test]
    fn test_search_by_keyword() {
        let mut j = DreamJournal::new();
        j.record(1, "Ocean waves at night", Vividness::Vivid, 0.6, vec![], vec!["ocean".to_string(), "night".to_string()]);
        j.record(2, "Mountain climbing", Vividness::Faint, 0.3, vec![], vec!["mountain".to_string()]);
        let results = j.search_by_keyword("ocean");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut j = DreamJournal::new();
        j.record(1, "Scary monster", Vividness::HyperReal, 0.9, vec![DreamTag::Nightmare], vec![]);
        j.record(2, "Nice walk", Vividness::Faint, 0.1, vec![DreamTag::Flying], vec![]);
        j.record(3, "Another scary thing", Vividness::Vivid, 0.8, vec![DreamTag::Nightmare], vec![]);
        let nightmares = j.filter_by_tag(&DreamTag::Nightmare);
        assert_eq!(nightmares.len(), 2);
    }

    #[test]
    fn test_entries_in_range() {
        let mut j = DreamJournal::new();
        j.record(5, "Dream 1", Vividness::Moderate, 0.5, vec![], vec![]);
        j.record(10, "Dream 2", Vividness::Vivid, 0.7, vec![], vec![]);
        j.record(15, "Dream 3", Vividness::Faint, 0.2, vec![], vec![]);
        let in_range = j.entries_in_range(6, 14);
        assert_eq!(in_range.len(), 1);
        assert_eq!(in_range[0].id, 2);
    }

    #[test]
    fn test_summarize() {
        let mut j = DreamJournal::new();
        j.record(1, "Nightmare 1", Vividness::HyperReal, 0.9, vec![DreamTag::Nightmare], vec!["dark".to_string()]);
        j.record(2, "Nightmare 2", Vividness::Vivid, 0.8, vec![DreamTag::Nightmare, DreamTag::Recurring], vec!["dark".to_string(), "shadow".to_string()]);
        j.record(3, "Lucid dream", Vividness::HyperReal, 0.7, vec![DreamTag::Lucid], vec!["control".to_string()]);
        let summary = j.summarize();
        assert_eq!(summary.total_entries, 3);
        assert_eq!(summary.nightmare_count, 2);
        assert_eq!(summary.lucid_count, 1);
        assert_eq!(summary.most_common_keyword.as_deref(), Some("dark"));
    }

    #[test]
    fn test_summarize_empty() {
        let j = DreamJournal::new();
        let summary = j.summarize();
        assert_eq!(summary.total_entries, 0);
    }

    #[test]
    fn test_vividness_clamp() {
        let mut j = DreamJournal::new();
        j.record(1, "test", Vividness::Moderate, 1.5, vec![], vec![]);
        assert_eq!(j.get(1).unwrap().emotional_intensity, 1.0);
        j.record(2, "test2", Vividness::Moderate, -0.5, vec![], vec![]);
        assert_eq!(j.get(2).unwrap().emotional_intensity, 0.0);
    }
}
