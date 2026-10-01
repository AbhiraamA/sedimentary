//! Field notes: a capped log of recent interactions and events.

use serde::{Deserialize, Serialize};

use crate::rock::Action;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    /// Something you did, and what the rock said about it.
    Action,
    /// The rock moved one character. History was made.
    Movement,
    /// Something improbable happened.
    Rare,
    /// A hidden discovery was unlocked.
    Discovery,
    /// The rock said something unprompted.
    Remark,
    /// Bookkeeping: adoption, time away, recovered saves.
    System,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub at: i64,
    pub kind: EntryKind,
    #[serde(default)]
    pub action: Option<Action>,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Journal {
    entries: Vec<Entry>,
}

impl Journal {
    pub const CAPACITY: usize = 200;

    pub fn push(&mut self, at: i64, kind: EntryKind, text: impl Into<String>) {
        self.push_entry(Entry {
            at,
            kind,
            action: None,
            text: text.into(),
        });
    }

    pub fn push_action(&mut self, at: i64, action: Action, text: impl Into<String>) {
        self.push_entry(Entry {
            at,
            kind: EntryKind::Action,
            action: Some(action),
            text: text.into(),
        });
    }

    fn push_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
        self.trim();
    }

    fn trim(&mut self) {
        if self.entries.len() > Self::CAPACITY {
            let excess = self.entries.len() - Self::CAPACITY;
            self.entries.drain(..excess);
        }
    }

    /// Drop anything beyond capacity (e.g. from a hand-edited save).
    pub fn sanitize(&mut self) {
        self.entries.retain(|e| !e.text.trim().is_empty());
        self.trim();
    }

    /// Newest first.
    pub fn recent(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().rev()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_most_recent_entries() {
        let mut journal = Journal::default();
        for i in 0..(Journal::CAPACITY as i64 + 50) {
            journal.push(i, EntryKind::System, format!("entry {i}"));
        }
        assert_eq!(journal.len(), Journal::CAPACITY);
        let newest = journal.recent().next().unwrap();
        assert_eq!(newest.text, format!("entry {}", Journal::CAPACITY + 49));
    }
}
