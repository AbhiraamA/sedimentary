//! Turns what happened (numbers from `rock`) into what the rock says
//! (lines from `lines`). Remembers recent lines so it never says the same
//! thing twice in a row.

use std::collections::VecDeque;

use crate::achievements::Achievement;
use crate::clock::{DAY, HOUR, duration_words};
use crate::dice::Dice;
use crate::journal::EntryKind;
use crate::lines::{self, Pool, Voice};
use crate::rock::{Action, Direction, Mood, NEGLECT_SECS, OfflineReport, Outcome, Rock};

/// How many recent lines to avoid.
const MEMORY: usize = 24;
/// Chance of a sarcastic tail, by sarcasm level.
const BARB_CHANCE: [f64; 4] = [0.0, 0.12, 0.3, 0.55];
/// Below this, coming back doesn't warrant a speech.
const WELCOME_BACK_AFTER: i64 = 6 * HOUR;
/// Chance the rock comments on its own movement.
const MOVE_COMMENT_CHANCE: f64 = 0.6;
/// Chance a repeated action draws a remark about the repetition itself.
const REPEAT_COMMENT_CHANCE: f64 = 0.5;

#[derive(Clone, Debug, PartialEq)]
pub struct Reaction {
    /// What the rock says about the action.
    pub speech: String,
    /// Movement, rare events and discoveries that happened alongside it.
    pub events: Vec<(EntryKind, String)>,
}

#[derive(Debug, Default)]
pub struct Narrator {
    recent: VecDeque<&'static str>,
}

fn action_pool(voice: &Voice, action: Action) -> Pool {
    match action {
        Action::Feed => voice.feed,
        Action::Water => voice.water,
        Action::Polish => voice.polish,
        Action::Talk => voice.talk,
        Action::Walk => voice.walk,
        Action::Stare => voice.stare,
        Action::Rename => voice.rename,
    }
}

/// Fill the `{name}` placeholder.
pub fn fill(template: &str, rock: &Rock) -> String {
    template.replace("{name}", &rock.name)
}

impl Narrator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pick a line from `pool`, avoiding anything said recently. With a pool
    /// of two or more lines, the same line is never returned twice in a row.
    pub fn pick(&mut self, pool: Pool, dice: &mut Dice) -> &'static str {
        if pool.is_empty() {
            return "";
        }
        let fresh: Vec<&'static str> = pool
            .iter()
            .copied()
            .filter(|line| !self.recent.contains(line))
            .collect();
        let line = if fresh.is_empty() {
            let last = self.recent.iter().rev().find(|r| pool.contains(r)).copied();
            let others: Vec<&'static str> = pool
                .iter()
                .copied()
                .filter(|line| Some(*line) != last)
                .collect();
            if others.is_empty() {
                pool[0]
            } else {
                others[dice.below(others.len())]
            }
        } else {
            fresh[dice.below(fresh.len())]
        };
        self.recent.push_back(line);
        while self.recent.len() > MEMORY {
            self.recent.pop_front();
        }
        line
    }

    fn say(&mut self, pool: Pool, rock: &Rock, dice: &mut Dice) -> String {
        let line = self.pick(pool, dice);
        fill(line, rock)
    }

    /// The rock's response to an action. `mood` is measured *before* the
    /// action, so neglect still counts when you finally show up.
    pub fn react(
        &mut self,
        rock: &Rock,
        outcome: &Outcome,
        mood: Mood,
        dice: &mut Dice,
    ) -> Reaction {
        let voice = lines::voice(rock.personality);
        let pool = if outcome.overdone {
            match outcome.action {
                Action::Water => voice.overwater,
                Action::Polish => voice.overpolish,
                other => action_pool(voice, other),
            }
        } else if outcome.repeated && dice.chance(REPEAT_COMMENT_CHANCE) {
            voice.repeat
        } else {
            action_pool(voice, outcome.action)
        };

        let mut speech = self.say(pool, rock, dice);
        if mood.neglect_secs >= NEGLECT_SECS {
            let opener = self.say(voice.neglected, rock, dice);
            speech = format!("{opener} {speech}");
        }
        let barb_chance = BARB_CHANCE[usize::from(mood.sarcasm.min(3))];
        if dice.chance(barb_chance) {
            let barb = self.say(voice.barbs, rock, dice);
            speech = format!("{speech} {barb}");
        }

        Reaction {
            speech,
            events: self.events(
                rock,
                outcome.moved,
                outcome.rare_event,
                &outcome.unlocked,
                dice,
            ),
        }
    }

    /// Narrate movement, rare events and discoveries.
    pub fn events(
        &mut self,
        rock: &Rock,
        moved: Option<Direction>,
        rare_event: bool,
        unlocked: &[Achievement],
        dice: &mut Dice,
    ) -> Vec<(EntryKind, String)> {
        let voice = lines::voice(rock.personality);
        let mut events = Vec::new();
        if let Some(direction) = moved {
            let total = match rock.counters.moves {
                1 => "1 character".to_string(),
                n => format!("{n} characters"),
            };
            let line = self
                .say(lines::MOVEMENT, rock, dice)
                .replace("{dir}", direction.word())
                .replace("{total}", &total);
            events.push((EntryKind::Movement, line));
            if dice.chance(MOVE_COMMENT_CHANCE) {
                events.push((EntryKind::Remark, self.say(voice.moved, rock, dice)));
            }
        }
        if rare_event {
            events.push((EntryKind::Rare, self.say(lines::RARE_EVENTS, rock, dice)));
        }
        for achievement in unlocked {
            events.push((
                EntryKind::Discovery,
                format!(
                    "Discovery: {}. {}",
                    achievement.title(),
                    achievement.description()
                ),
            ));
        }
        events
    }

    /// What the rock is thinking about right now. Neglected stats get a
    /// louder voice.
    pub fn thought(&mut self, rock: &Rock, dice: &mut Dice) -> String {
        let voice = lines::voice(rock.personality);
        let s = &rock.stats;
        let mut pressing: Vec<Pool> = Vec::new();
        if s.moisture < 15.0 {
            pressing.push(voice.dry);
        }
        if s.moisture > 85.0 {
            pressing.push(voice.soggy);
        }
        if s.polish < 20.0 {
            pressing.push(voice.grimy);
        }
        if s.boredom > 70.0 {
            pressing.push(voice.bored);
        }
        if s.disappointment > 70.0 {
            pressing.push(voice.disappointed);
        }
        let pool = if !pressing.is_empty() && dice.chance(0.6) {
            pressing[dice.below(pressing.len())]
        } else {
            voice.idle
        };
        self.say(pool, rock, dice)
    }

    /// Something to say when the game reopens, if it's been a while (or if
    /// the clock did something strange).
    pub fn welcome_back(
        &mut self,
        rock: &Rock,
        report: &OfflineReport,
        dice: &mut Dice,
    ) -> Option<String> {
        if report.clock_went_backwards {
            return Some(self.say(lines::CLOCK_SKEW, rock, dice));
        }
        let voice = lines::voice(rock.personality);
        let pool = match report.away_secs {
            s if s < WELCOME_BACK_AFTER => return None,
            s if s < 2 * DAY => voice.back_short,
            s if s < 7 * DAY => voice.back_medium,
            _ => voice.back_long,
        };
        let away = duration_words(report.away_secs);
        let line = self.say(pool, rock, dice).replace("{away}", &away);
        Some(capitalize(&line))
    }

    pub fn greeting(&mut self, rock: &Rock, dice: &mut Dice) -> String {
        self.say(lines::voice(rock.personality).greeting, rock, dice)
    }

    pub fn farewell(&mut self, rock: &Rock, dice: &mut Dice) -> String {
        self.say(lines::voice(rock.personality).farewell, rock, dice)
    }

    pub fn name_idea(&mut self, dice: &mut Dice) -> &'static str {
        self.pick(lines::NAME_IDEAS, dice)
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rock::{Personality, Stat};

    const T0: i64 = 1_750_000_000;

    fn rock(personality: Personality) -> Rock {
        Rock::new("Gerald", personality, T0, &mut Dice::seeded(1))
    }

    #[test]
    fn never_repeats_a_line_twice_in_a_row() {
        let mut narrator = Narrator::new();
        let mut dice = Dice::seeded(11);
        for pool in [
            lines::JUDGMENTAL.feed,
            lines::JUDGMENTAL.dry,
            lines::MOVEMENT,
        ] {
            let mut last = "";
            for _ in 0..500 {
                let line = narrator.pick(pool, &mut dice);
                assert_ne!(line, last);
                last = line;
            }
        }
    }

    #[test]
    fn consecutive_reactions_differ_for_every_personality_and_action() {
        let mut dice = Dice::seeded(21);
        for personality in Personality::ALL {
            let mut narrator = Narrator::new();
            let mut r = rock(personality);
            for action in Action::ALL {
                let mut last = String::new();
                for i in 0..40 {
                    let mood = r.mood(T0 + i);
                    let outcome = r.perform(action, T0 + i, &mut dice);
                    let reaction = narrator.react(&r, &outcome, mood, &mut dice);
                    assert!(!reaction.speech.is_empty());
                    assert!(!reaction.speech.contains('{'), "{}", reaction.speech);
                    assert_ne!(reaction.speech, last);
                    last = reaction.speech;
                }
            }
        }
    }

    #[test]
    fn pick_spreads_across_the_pool() {
        let mut narrator = Narrator::new();
        let mut dice = Dice::seeded(2);
        let pool = lines::EXISTENTIAL.idle;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..pool.len() {
            seen.insert(narrator.pick(pool, &mut dice));
        }
        assert_eq!(seen.len(), pool.len(), "should cycle before repeating");
    }

    #[test]
    fn templates_are_filled() {
        let mut narrator = Narrator::new();
        let mut dice = Dice::seeded(3);
        let mut r = rock(Personality::SuspiciouslySupportive);
        r.counters.moves = 4;
        for _ in 0..30 {
            let events = narrator.events(
                &r,
                Some(Direction::Left),
                true,
                &[Achievement::RareEarth],
                &mut dice,
            );
            for (_, text) in events {
                assert!(!text.contains('{'), "{text}");
            }
        }
        let report = OfflineReport {
            away_secs: 31 * DAY,
            simulated_secs: 2 * DAY,
            ..OfflineReport::default()
        };
        let line = narrator.welcome_back(&r, &report, &mut dice).unwrap();
        assert!(
            line.contains("31 days") || line.contains("31 Days"),
            "{line}"
        );
        assert!(!line.contains('{'));
    }

    #[test]
    fn short_absences_get_no_speech_but_clock_skew_does() {
        let mut narrator = Narrator::new();
        let mut dice = Dice::seeded(4);
        let r = rock(Personality::Existential);
        let brief = OfflineReport {
            away_secs: HOUR,
            ..OfflineReport::default()
        };
        assert!(narrator.welcome_back(&r, &brief, &mut dice).is_none());
        let skew = OfflineReport {
            clock_went_backwards: true,
            ..OfflineReport::default()
        };
        assert!(narrator.welcome_back(&r, &skew, &mut dice).is_some());
    }

    #[test]
    fn overwatering_uses_the_overwater_pool() {
        let mut narrator = Narrator::new();
        let mut dice = Dice::seeded(5);
        let mut r = rock(Personality::Judgmental);
        r.stats.set(Stat::Moisture, 99.0);
        let mood = Mood {
            sarcasm: 0,
            neglect_secs: 0,
        };
        let outcome = r.apply_action(Action::Water, T0);
        let reaction = narrator.react(&r, &outcome, mood, &mut dice);
        assert!(
            lines::JUDGMENTAL
                .overwater
                .iter()
                .any(|l| reaction.speech.starts_with(l))
        );
    }

    #[test]
    fn neglect_adds_an_opener() {
        let mut narrator = Narrator::new();
        let mut dice = Dice::seeded(6);
        let mut r = rock(Personality::Judgmental);
        let mood = Mood {
            sarcasm: 0,
            neglect_secs: 5 * HOUR,
        };
        let outcome = r.apply_action(Action::Stare, T0);
        let reaction = narrator.react(&r, &outcome, mood, &mut dice);
        assert!(
            lines::JUDGMENTAL
                .neglected
                .iter()
                .any(|l| reaction.speech.starts_with(l))
        );
    }
}
