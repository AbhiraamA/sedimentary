//! The pet rock itself: stats, actions, drift, movement and discoveries.
//!
//! Nothing in here produces text or touches the disk. Every function takes
//! the current time and a [`Dice`] explicitly so behaviour is reproducible.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::achievements::Achievement;
use crate::clock::{DAY, HOUR};
use crate::dice::Dice;

pub const STAT_MIN: f64 = 0.0;
pub const STAT_MAX: f64 = 100.0;
pub const NAME_MAX_CHARS: usize = 24;
/// How far (in characters) the rock may wander from where it started.
pub const POSITION_LIMIT: i32 = 12;
/// Time away is simulated for at most this long. A month away feels like two days.
pub const OFFLINE_CAP_SECS: i64 = 48 * HOUR;
/// Longest gap a single in-session tick will simulate (e.g. after a laptop sleeps).
pub const MAX_TICK_SECS: f64 = 300.0;
/// Extra disappointment for doing the same thing twice in a row.
pub const REPEAT_PENALTY: f64 = 3.0;
pub const OVERWATER_AT: f64 = 85.0;
pub const OVERPOLISH_AT: f64 = 90.0;
pub const MOVE_CHANCE_PER_ACTION: f64 = 0.05;
pub const RARE_CHANCE_PER_ACTION: f64 = 0.03;
pub const MOVE_CHANCE_PER_IDLE_ROLL: f64 = 0.025;
pub const RARE_CHANCE_PER_IDLE_ROLL: f64 = 0.006;
/// Away this long and the rock opens with a remark about being ignored.
pub const NEGLECT_SECS: i64 = 3 * HOUR;

const DRIFT_STEP_SECS: f64 = 600.0;

// Drift rates, per hour, and the soft limits drift alone can never cross.
// Actions can push stats all the way to 0 or 100; time alone cannot.
pub const MOISTURE_DRIFT: f64 = -3.0;
pub const POLISH_DRIFT: f64 = -1.0;
pub const BOREDOM_DRIFT: f64 = 6.0;
pub const DISAPPOINTMENT_RISE: f64 = 2.5;
pub const DISAPPOINTMENT_EASE: f64 = -1.0;
pub const BORED_THRESHOLD: f64 = 50.0;
pub const DRIFT_FLOOR: f64 = 10.0;
pub const DRIFT_CEILING: f64 = 85.0;
const DISAPPOINTMENT_EASE_FLOOR: f64 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Personality {
    Judgmental,
    Existential,
    SuspiciouslySupportive,
}

impl Personality {
    pub const ALL: [Personality; 3] = [
        Personality::Judgmental,
        Personality::Existential,
        Personality::SuspiciouslySupportive,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Personality::Judgmental => "Judgmental",
            Personality::Existential => "Existential",
            Personality::SuspiciouslySupportive => "Suspiciously Supportive",
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            Personality::Judgmental => "Has standards. You do not meet them.",
            Personality::Existential => "Has been staring into the void since the Precambrian.",
            Personality::SuspiciouslySupportive => "Believes in you. Has been taking notes.",
        }
    }

    pub fn sample(self) -> &'static str {
        match self {
            Personality::Judgmental => "I've seen better owners in the fossil record.",
            Personality::Existential => "Sisyphus had a boulder. Did the boulder have a Sisyphus?",
            Personality::SuspiciouslySupportive => "You're doing amazing! No one suspects a thing!",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Formation {
    Igneous,
    #[default]
    Sedimentary,
    Metamorphic,
}

impl Formation {
    pub const ALL: [Formation; 3] = [
        Formation::Igneous,
        Formation::Sedimentary,
        Formation::Metamorphic,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Formation::Igneous => "igneous",
            Formation::Sedimentary => "sedimentary",
            Formation::Metamorphic => "metamorphic",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stat {
    Moisture,
    Polish,
    Boredom,
    Disappointment,
}

impl Stat {
    pub const ALL: [Stat; 4] = [
        Stat::Moisture,
        Stat::Polish,
        Stat::Boredom,
        Stat::Disappointment,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stat::Moisture => "Moisture",
            Stat::Polish => "Polish",
            Stat::Boredom => "Boredom",
            Stat::Disappointment => "Disappointment",
        }
    }

    /// A word for how things are going on this axis.
    pub fn describe(self, value: f64) -> &'static str {
        let tiers: [&'static str; 5] = match self {
            Stat::Moisture => [
                "desiccated",
                "arid",
                "pleasantly damp",
                "moist (sorry)",
                "waterlogged",
            ],
            Stat::Polish => ["grimy", "matte", "presentable", "lustrous", "blinding"],
            Stat::Boredom => [
                "engaged, allegedly",
                "idle",
                "restless",
                "catatonic",
                "contemplating erosion",
            ],
            Stat::Disappointment => [
                "tolerant",
                "unimpressed",
                "disappointed",
                "profoundly let down",
                "drafting a memoir about you",
            ],
        };
        let index = match value {
            v if v < 15.0 => 0,
            v if v < 35.0 => 1,
            v if v < 60.0 => 2,
            v if v < 85.0 => 3,
            _ => 4,
        };
        tiers[index]
    }

    fn default_value(self) -> f64 {
        match self {
            Stat::Moisture => 50.0,
            Stat::Polish => 40.0,
            Stat::Boredom => 35.0,
            Stat::Disappointment => 30.0,
        }
    }
}

/// Clamp into the legal stat range. NaN becomes the minimum.
pub fn clamp_stat(value: f64) -> f64 {
    if value.is_nan() {
        STAT_MIN
    } else {
        value.clamp(STAT_MIN, STAT_MAX)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub moisture: f64,
    pub polish: f64,
    pub boredom: f64,
    pub disappointment: f64,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            moisture: Stat::Moisture.default_value(),
            polish: Stat::Polish.default_value(),
            boredom: Stat::Boredom.default_value(),
            disappointment: Stat::Disappointment.default_value(),
        }
    }
}

impl Stats {
    pub fn get(&self, stat: Stat) -> f64 {
        match stat {
            Stat::Moisture => self.moisture,
            Stat::Polish => self.polish,
            Stat::Boredom => self.boredom,
            Stat::Disappointment => self.disappointment,
        }
    }

    fn slot(&mut self, stat: Stat) -> &mut f64 {
        match stat {
            Stat::Moisture => &mut self.moisture,
            Stat::Polish => &mut self.polish,
            Stat::Boredom => &mut self.boredom,
            Stat::Disappointment => &mut self.disappointment,
        }
    }

    pub fn set(&mut self, stat: Stat, value: f64) {
        *self.slot(stat) = clamp_stat(value);
    }

    /// Add `delta` (clamped) and return the change that actually happened.
    pub fn add(&mut self, stat: Stat, delta: f64) -> f64 {
        let before = self.get(stat);
        self.set(stat, before + delta);
        self.get(stat) - before
    }

    /// Repair anything a hand-edited save could contain.
    pub fn sanitize(&mut self) {
        for stat in Stat::ALL {
            let value = self.get(stat);
            let repaired = if value.is_finite() {
                clamp_stat(value)
            } else {
                stat.default_value()
            };
            *self.slot(stat) = repaired;
        }
    }

    /// Net per-stat change from `before` to `self`, skipping anything below 0.05.
    pub fn changes_since(&self, before: &Stats) -> Vec<(Stat, f64)> {
        Stat::ALL
            .into_iter()
            .map(|s| (s, self.get(s) - before.get(s)))
            .filter(|(_, d)| d.abs() >= 0.05)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Feed,
    Water,
    Polish,
    Talk,
    Walk,
    Stare,
    Rename,
}

impl Action {
    pub const ALL: [Action; 7] = [
        Action::Feed,
        Action::Water,
        Action::Polish,
        Action::Talk,
        Action::Walk,
        Action::Stare,
        Action::Rename,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Action::Feed => "Feed",
            Action::Water => "Water",
            Action::Polish => "Polish",
            Action::Talk => "Talk",
            Action::Walk => "Take for a walk",
            Action::Stare => "Stare",
            Action::Rename => "Rename",
        }
    }

    pub fn key(self) -> char {
        match self {
            Action::Feed => 'f',
            Action::Water => 'w',
            Action::Polish => 'p',
            Action::Talk => 't',
            Action::Walk => 'k',
            Action::Stare => 's',
            Action::Rename => 'r',
        }
    }

    /// The base effect of each action. The help screen renders this table,
    /// so what players read is exactly what the game does.
    pub fn effects(self) -> &'static [(Stat, f64)] {
        match self {
            Action::Feed => &[
                (Stat::Boredom, -5.0),
                (Stat::Disappointment, 6.0),
                (Stat::Polish, -3.0),
            ],
            Action::Water => &[
                (Stat::Moisture, 25.0),
                (Stat::Polish, 3.0),
                (Stat::Boredom, -3.0),
            ],
            Action::Polish => &[
                (Stat::Polish, 20.0),
                (Stat::Disappointment, -6.0),
                (Stat::Boredom, -4.0),
                (Stat::Moisture, -2.0),
            ],
            Action::Talk => &[(Stat::Boredom, -15.0), (Stat::Disappointment, 3.0)],
            Action::Walk => &[
                (Stat::Boredom, -25.0),
                (Stat::Disappointment, -4.0),
                (Stat::Moisture, -6.0),
                (Stat::Polish, -6.0),
            ],
            Action::Stare => &[(Stat::Boredom, -10.0), (Stat::Disappointment, -5.0)],
            Action::Rename => &[(Stat::Boredom, -6.0), (Stat::Disappointment, 4.0)],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    pub fn word(self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Counters {
    pub feeds: u32,
    pub waters: u32,
    pub polishes: u32,
    pub talks: u32,
    pub walks: u32,
    pub stares: u32,
    pub renames: u32,
    /// Lifetime displacement, in characters.
    pub moves: u32,
    pub rare_events: u32,
    /// How many times in a row the last action has been repeated.
    pub streak: u32,
}

impl Counters {
    pub fn of(&self, action: Action) -> u32 {
        match action {
            Action::Feed => self.feeds,
            Action::Water => self.waters,
            Action::Polish => self.polishes,
            Action::Talk => self.talks,
            Action::Walk => self.walks,
            Action::Stare => self.stares,
            Action::Rename => self.renames,
        }
    }

    fn bump(&mut self, action: Action) {
        let slot = match action {
            Action::Feed => &mut self.feeds,
            Action::Water => &mut self.waters,
            Action::Polish => &mut self.polishes,
            Action::Talk => &mut self.talks,
            Action::Walk => &mut self.walks,
            Action::Stare => &mut self.stares,
            Action::Rename => &mut self.renames,
        };
        *slot = slot.saturating_add(1);
    }

    pub fn total_interactions(&self) -> u32 {
        Action::ALL.iter().map(|a| self.of(*a)).sum()
    }
}

/// What an action did, in numbers. `responses` turns this into words.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub action: Action,
    /// Same action as last time.
    pub repeated: bool,
    /// Watering a soaked rock or polishing a gleaming one.
    pub overdone: bool,
    /// Net change per stat after clamping, including any rare-event effects.
    pub changes: Vec<(Stat, f64)>,
    pub moved: Option<Direction>,
    pub rare_event: bool,
    pub unlocked: Vec<Achievement>,
}

/// Something that happened while nobody was pressing keys.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IdleOutcome {
    pub moved: Option<Direction>,
    pub rare_event: bool,
    pub unlocked: Vec<Achievement>,
}

impl IdleOutcome {
    pub fn is_empty(&self) -> bool {
        self.moved.is_none() && !self.rare_event && self.unlocked.is_empty()
    }
}

/// What happened while the game was closed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OfflineReport {
    /// Real time since the last save (0 if the clock went backwards).
    pub away_secs: i64,
    /// Time actually simulated: never more than [`OFFLINE_CAP_SECS`].
    pub simulated_secs: i64,
    pub clock_went_backwards: bool,
    pub changes: Vec<(Stat, f64)>,
    pub unlocked: Vec<Achievement>,
}

/// How sarcastic the rock is right now, plus how long it's been ignored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mood {
    /// 0 (mildly unimpressed) to 3 (geologically unimpressed).
    pub sarcasm: u8,
    pub neglect_secs: i64,
}

impl Mood {
    pub fn label(self) -> &'static str {
        match self.sarcasm {
            0 => "mildly unimpressed",
            1 => "unimpressed",
            2 => "deeply unimpressed",
            _ => "geologically unimpressed",
        }
    }
}

/// Trim, collapse whitespace, drop control characters and cap the length.
/// Returns `None` if nothing printable is left.
pub fn sanitize_name(raw: &str) -> Option<String> {
    let printable: String = raw.chars().filter(|c| !c.is_control()).collect();
    let collapsed = printable.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        None
    } else {
        Some(
            collapsed
                .chars()
                .take(NAME_MAX_CHARS)
                .collect::<String>()
                .trim_end()
                .to_string(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rock {
    pub name: String,
    pub personality: Personality,
    #[serde(default)]
    pub formation: Formation,
    /// How old the rock claims to be, in millions of years.
    #[serde(default)]
    pub age_myr: u32,
    /// When you adopted it (Unix seconds).
    pub born_at: i64,
    /// Last time the simulation was brought up to date.
    pub last_seen: i64,
    /// Last time you actually did something with it.
    #[serde(default)]
    pub last_interaction: i64,
    #[serde(default)]
    pub stats: Stats,
    /// Horizontal offset from where it started, in characters.
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub counters: Counters,
    #[serde(default)]
    pub last_action: Option<Action>,
    /// Discovery id -> when it was unlocked.
    #[serde(default)]
    pub achievements: BTreeMap<String, i64>,
}

impl Rock {
    pub fn new(name: &str, personality: Personality, now: i64, dice: &mut Dice) -> Self {
        let formation = Formation::ALL[dice.below(Formation::ALL.len())];
        Self {
            name: sanitize_name(name).unwrap_or_else(|| "Rock".to_string()),
            personality,
            formation,
            age_myr: dice.between(40, 3_900),
            born_at: now,
            last_seen: now,
            last_interaction: now,
            stats: Stats::default(),
            position: 0,
            counters: Counters::default(),
            last_action: None,
            achievements: BTreeMap::new(),
        }
    }

    /// Repair a rock loaded from disk so the rest of the game can trust it.
    pub fn sanitize(&mut self, now: i64) {
        self.stats.sanitize();
        self.name = sanitize_name(&self.name).unwrap_or_else(|| "Rock".to_string());
        self.position = self.position.clamp(-POSITION_LIMIT, POSITION_LIMIT);
        if self.age_myr == 0 {
            self.age_myr = 300;
        }
        if self.last_seen <= 0 {
            self.last_seen = now;
        }
        if self.born_at <= 0 {
            self.born_at = self.last_seen;
        }
        if self.last_interaction <= 0 {
            self.last_interaction = self.last_seen;
        }
        self.achievements
            .retain(|id, _| Achievement::from_id(id).is_some());
    }

    pub fn age_secs(&self, now: i64) -> i64 {
        (now - self.born_at).max(0)
    }

    pub fn neglect_secs(&self, now: i64) -> i64 {
        (now - self.last_interaction).max(0)
    }

    /// Sarcasm rises with disappointment, boredom and plain neglect.
    pub fn mood(&self, now: i64) -> Mood {
        let neglect_secs = self.neglect_secs(now);
        let neglect_bonus = (neglect_secs as f64 / HOUR as f64 * 2.0).min(25.0);
        let score = self.stats.disappointment * 0.6 + self.stats.boredom * 0.4 + neglect_bonus;
        let sarcasm = match score {
            s if s < 30.0 => 0,
            s if s < 50.0 => 1,
            s if s < 70.0 => 2,
            _ => 3,
        };
        Mood {
            sarcasm,
            neglect_secs,
        }
    }

    pub fn has(&self, achievement: Achievement) -> bool {
        self.achievements.contains_key(achievement.id())
    }

    /// Unlock a discovery. Returns true only the first time.
    pub fn unlock(&mut self, achievement: Achievement, now: i64) -> bool {
        if self.has(achievement) {
            return false;
        }
        self.achievements.insert(achievement.id().to_string(), now);
        true
    }

    /// Apply an action's effects with no randomness. Used by [`Rock::perform`].
    pub fn apply_action(&mut self, action: Action, now: i64) -> Outcome {
        let before = self.stats.clone();
        let repeated = self.last_action == Some(action);
        let overdone = match action {
            Action::Water => self.stats.moisture >= OVERWATER_AT,
            Action::Polish => self.stats.polish >= OVERPOLISH_AT,
            _ => false,
        };

        for &(stat, amount) in action.effects() {
            // Novelty wears off: repeats only relieve half as much boredom.
            let amount = if repeated && stat == Stat::Boredom && amount < 0.0 {
                amount * 0.5
            } else {
                amount
            };
            self.stats.add(stat, amount);
        }
        if repeated {
            self.stats.add(Stat::Disappointment, REPEAT_PENALTY);
        }
        if overdone {
            match action {
                Action::Water => {
                    self.stats.add(Stat::Disappointment, 5.0);
                }
                Action::Polish => {
                    self.stats.add(Stat::Boredom, 6.0);
                    self.stats.add(Stat::Disappointment, 2.0);
                }
                _ => {}
            }
        }

        self.counters.bump(action);
        self.counters.streak = if repeated {
            self.counters.streak.saturating_add(1)
        } else {
            1
        };
        self.last_action = Some(action);
        self.last_interaction = now;
        self.last_seen = self.last_seen.max(now);

        Outcome {
            action,
            repeated,
            overdone,
            changes: self.stats.changes_since(&before),
            moved: None,
            rare_event: false,
            unlocked: Vec::new(),
        }
    }

    /// Do something to the rock. It may respond by moving or by doing
    /// something remarkable, then any newly earned discoveries are unlocked.
    pub fn perform(&mut self, action: Action, now: i64, dice: &mut Dice) -> Outcome {
        let before = self.stats.clone();
        let mut outcome = self.apply_action(action, now);
        if dice.chance(MOVE_CHANCE_PER_ACTION) {
            outcome.moved = Some(self.shift(dice));
        }
        if dice.chance(RARE_CHANCE_PER_ACTION) {
            self.rare_event();
            outcome.rare_event = true;
        }
        outcome.changes = self.stats.changes_since(&before);
        outcome.unlocked = self.check_achievements(now);
        outcome
    }

    /// Rename, then count it as an action. Returns `None` if the new name is
    /// empty after cleaning or identical to the current one.
    pub fn rename(&mut self, raw: &str, now: i64, dice: &mut Dice) -> Option<Outcome> {
        let name = sanitize_name(raw)?;
        if name == self.name {
            return None;
        }
        self.name = name;
        Some(self.perform(Action::Rename, now, dice))
    }

    /// Chances for things to happen while the player just watches.
    pub fn idle_roll(&mut self, now: i64, dice: &mut Dice) -> IdleOutcome {
        let mut outcome = IdleOutcome::default();
        if dice.chance(MOVE_CHANCE_PER_IDLE_ROLL) {
            outcome.moved = Some(self.shift(dice));
        }
        if dice.chance(RARE_CHANCE_PER_IDLE_ROLL) {
            self.rare_event();
            outcome.rare_event = true;
        }
        outcome.unlocked = self.check_achievements(now);
        outcome
    }

    /// Move exactly one character, bouncing off the edges of the habitat.
    pub fn shift(&mut self, dice: &mut Dice) -> Direction {
        let direction = if self.position >= POSITION_LIMIT {
            Direction::Left
        } else if self.position <= -POSITION_LIMIT {
            Direction::Right
        } else if dice.coin() {
            Direction::Left
        } else {
            Direction::Right
        };
        self.position += match direction {
            Direction::Left => -1,
            Direction::Right => 1,
        };
        self.counters.moves = self.counters.moves.saturating_add(1);
        direction
    }

    /// Rare events are exciting, even for a rock.
    fn rare_event(&mut self) {
        self.counters.rare_events = self.counters.rare_events.saturating_add(1);
        self.stats.add(Stat::Boredom, -15.0);
        self.stats.add(Stat::Disappointment, -5.0);
    }

    /// Let `secs` of time pass. Moisture and polish fade, boredom grows, and
    /// a bored rock slowly grows more disappointed. Drift never pushes past
    /// the soft limits, so time alone can't wreck a rock.
    pub fn drift(&mut self, secs: f64) {
        if secs.is_nan() || secs <= 0.0 {
            return;
        }
        let mut remaining = secs;
        while remaining > 0.0 {
            let step = remaining.min(DRIFT_STEP_SECS);
            self.drift_step(step / HOUR as f64);
            remaining -= step;
        }
    }

    fn drift_step(&mut self, hours: f64) {
        self.drift_toward(Stat::Moisture, MOISTURE_DRIFT * hours, DRIFT_FLOOR);
        self.drift_toward(Stat::Polish, POLISH_DRIFT * hours, DRIFT_FLOOR);
        self.drift_toward(Stat::Boredom, BOREDOM_DRIFT * hours, DRIFT_CEILING);
        if self.stats.boredom > BORED_THRESHOLD {
            self.drift_toward(
                Stat::Disappointment,
                DISAPPOINTMENT_RISE * hours,
                DRIFT_CEILING,
            );
        } else {
            self.drift_toward(
                Stat::Disappointment,
                DISAPPOINTMENT_EASE * hours,
                DISAPPOINTMENT_EASE_FLOOR,
            );
        }
    }

    /// Move `stat` by `delta` but never past `bound`. A stat already beyond
    /// the bound (thanks to an action) is left alone rather than dragged back.
    fn drift_toward(&mut self, stat: Stat, delta: f64, bound: f64) {
        let value = self.stats.get(stat);
        if delta > 0.0 {
            if value < bound {
                self.stats.set(stat, (value + delta).min(bound));
            }
        } else if delta < 0.0 && value > bound {
            self.stats.set(stat, (value + delta).max(bound));
        }
    }

    /// Advance the simulation during play.
    pub fn tick(&mut self, elapsed_secs: f64, now: i64) {
        self.drift(elapsed_secs.min(MAX_TICK_SECS));
        self.last_seen = now;
    }

    /// Bring the rock up to date after the game was closed. Time away is
    /// capped, and a clock that went backwards simply costs nothing.
    pub fn catch_up(&mut self, now: i64) -> OfflineReport {
        let before = self.stats.clone();
        let mut report = OfflineReport::default();

        if now < self.last_seen {
            report.clock_went_backwards = true;
            self.last_seen = now;
            self.last_interaction = self.last_interaction.min(now);
            if self.unlock(Achievement::TemporalAnomaly, now) {
                report.unlocked.push(Achievement::TemporalAnomaly);
            }
            report.unlocked.extend(self.check_achievements(now));
            return report;
        }

        report.away_secs = now - self.last_seen;
        report.simulated_secs = report.away_secs.min(OFFLINE_CAP_SECS);
        self.drift(report.simulated_secs as f64);
        self.last_seen = now;

        if report.away_secs >= 7 * DAY && self.unlock(Achievement::ProdigalOwner, now) {
            report.unlocked.push(Achievement::ProdigalOwner);
        }
        report.unlocked.extend(self.check_achievements(now));
        report.changes = self.stats.changes_since(&before);
        report
    }

    /// Unlock every discovery whose condition now holds.
    pub fn check_achievements(&mut self, now: i64) -> Vec<Achievement> {
        let c = &self.counters;
        let s = &self.stats;
        let candidates = [
            (Achievement::ContinentalDrift, c.moves >= 1),
            (Achievement::PlateTectonics, c.moves >= 5),
            (Achievement::CommitmentToTheBit, c.feeds >= 10),
            (Achievement::FloodPlain, s.moisture >= STAT_MAX),
            (Achievement::MirrorFinish, s.polish >= STAT_MAX),
            (
                Achievement::Unblinking,
                self.last_action == Some(Action::Stare) && c.streak >= 5,
            ),
            (Achievement::IdentityCrisis, c.renames >= 3),
            (Achievement::RareEarth, c.rare_events >= 1),
            (Achievement::RockBottom, s.disappointment >= STAT_MAX),
            (Achievement::ZenGarden, s.boredom <= STAT_MIN),
            (Achievement::GlacialErratic, c.walks >= 10),
            (Achievement::DeepTime, self.age_secs(now) >= 30 * DAY),
        ];
        let mut unlocked = Vec::new();
        for (achievement, earned) in candidates {
            if earned && self.unlock(achievement, now) {
                unlocked.push(achievement);
            }
        }
        unlocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: i64 = 1_750_000_000;

    fn rock() -> Rock {
        Rock::new("Gerald", Personality::Judgmental, T0, &mut Dice::seeded(1))
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn assert_in_bounds(stats: &Stats) {
        for stat in Stat::ALL {
            let v = stats.get(stat);
            assert!(
                (STAT_MIN..=STAT_MAX).contains(&v),
                "{stat:?} out of range: {v}"
            );
        }
    }

    #[test]
    fn stats_clamp_at_both_ends() {
        let mut stats = Stats::default();
        assert!(approx(stats.add(Stat::Moisture, 500.0), 50.0));
        assert!(approx(stats.moisture, STAT_MAX));
        assert!(approx(stats.add(Stat::Polish, -500.0), -40.0));
        assert!(approx(stats.polish, STAT_MIN));
        stats.set(Stat::Boredom, f64::NAN);
        assert!(approx(stats.boredom, STAT_MIN));
        stats.set(Stat::Disappointment, f64::INFINITY);
        assert!(approx(stats.disappointment, STAT_MAX));
        assert_in_bounds(&stats);
    }

    #[test]
    fn sanitize_repairs_hand_edited_values() {
        let mut stats = Stats {
            moisture: f64::NAN,
            polish: -40.0,
            boredom: 9_000.0,
            disappointment: f64::NEG_INFINITY,
        };
        stats.sanitize();
        assert_in_bounds(&stats);
        assert!(approx(stats.moisture, 50.0));
        assert!(approx(stats.polish, 0.0));
        assert!(approx(stats.boredom, 100.0));
        assert!(approx(stats.disappointment, 30.0));
    }

    #[test]
    fn every_action_applies_its_documented_effects() {
        for action in Action::ALL {
            let mut r = rock();
            let before = r.stats.clone();
            let outcome = r.apply_action(action, T0 + 10);
            assert!(!outcome.repeated && !outcome.overdone);
            for stat in Stat::ALL {
                let expected: f64 = action
                    .effects()
                    .iter()
                    .filter(|(s, _)| *s == stat)
                    .map(|(_, d)| d)
                    .sum();
                let actual = r.stats.get(stat) - before.get(stat);
                assert!(
                    approx(actual, expected),
                    "{action:?} changed {stat:?} by {actual}, expected {expected}"
                );
            }
            assert_eq!(r.counters.of(action), 1);
            assert_eq!(r.last_interaction, T0 + 10);
        }
    }

    #[test]
    fn repeating_yourself_is_penalised() {
        let mut r = rock();
        r.apply_action(Action::Talk, T0);
        let before = r.stats.clone();
        let outcome = r.apply_action(Action::Talk, T0 + 1);
        assert!(outcome.repeated);
        assert!(approx(r.stats.boredom - before.boredom, -7.5));
        assert!(approx(
            r.stats.disappointment - before.disappointment,
            3.0 + REPEAT_PENALTY
        ));
        assert_eq!(r.counters.streak, 2);
        r.apply_action(Action::Stare, T0 + 2);
        assert_eq!(r.counters.streak, 1);
    }

    #[test]
    fn overwatering_and_overpolishing_backfire() {
        let mut r = rock();
        r.stats.set(Stat::Moisture, 90.0);
        let before = r.stats.disappointment;
        let outcome = r.apply_action(Action::Water, T0);
        assert!(outcome.overdone);
        assert!(approx(r.stats.moisture, STAT_MAX));
        assert!(approx(r.stats.disappointment - before, 5.0));

        let mut r = rock();
        r.stats.set(Stat::Polish, 95.0);
        let before = r.stats.clone();
        assert!(r.apply_action(Action::Polish, T0).overdone);
        assert!(approx(r.stats.boredom - before.boredom, -4.0 + 6.0));
    }

    #[test]
    fn hammering_actions_never_escapes_bounds() {
        let mut dice = Dice::seeded(99);
        let mut r = rock();
        for i in 0..2_000 {
            let action = Action::ALL[dice.below(Action::ALL.len())];
            r.perform(action, T0 + i, &mut dice);
            assert_in_bounds(&r.stats);
            assert!(r.position.abs() <= POSITION_LIMIT);
        }
    }

    #[test]
    fn outcome_reports_net_clamped_changes() {
        let mut r = rock();
        r.stats.set(Stat::Moisture, 95.0);
        let outcome = r.apply_action(Action::Water, T0);
        let moisture = outcome
            .changes
            .iter()
            .find(|(s, _)| *s == Stat::Moisture)
            .map(|(_, d)| *d)
            .unwrap();
        assert!(approx(moisture, 5.0));
    }

    #[test]
    fn shift_moves_one_character_and_bounces_off_edges() {
        let mut dice = Dice::seeded(3);
        let mut r = rock();
        r.position = POSITION_LIMIT;
        assert_eq!(r.shift(&mut dice), Direction::Left);
        assert_eq!(r.position, POSITION_LIMIT - 1);
        r.position = -POSITION_LIMIT;
        assert_eq!(r.shift(&mut dice), Direction::Right);
        assert_eq!(r.counters.moves, 2);
    }

    #[test]
    fn drift_is_slow_and_directional() {
        let mut r = rock();
        let before = r.stats.clone();
        r.drift(HOUR as f64);
        assert!(approx(r.stats.moisture, before.moisture + MOISTURE_DRIFT));
        assert!(approx(r.stats.polish, before.polish + POLISH_DRIFT));
        assert!(approx(r.stats.boredom, before.boredom + BOREDOM_DRIFT));
        // Not bored yet, so disappointment eases a little.
        assert!(r.stats.disappointment < before.disappointment);
        r.drift(-50.0);
        r.drift(f64::NAN);
        assert_in_bounds(&r.stats);
    }

    #[test]
    fn drift_respects_soft_limits_but_leaves_extremes_alone() {
        let mut r = rock();
        r.drift(1_000.0 * HOUR as f64);
        assert!(approx(r.stats.moisture, DRIFT_FLOOR));
        assert!(approx(r.stats.polish, DRIFT_FLOOR));
        assert!(approx(r.stats.boredom, DRIFT_CEILING));
        assert!(approx(r.stats.disappointment, DRIFT_CEILING));

        // A stat pushed past the soft limit by an action isn't dragged back.
        r.stats.set(Stat::Boredom, 97.0);
        r.stats.set(Stat::Moisture, 2.0);
        r.drift(10.0 * HOUR as f64);
        assert!(approx(r.stats.boredom, 97.0));
        assert!(approx(r.stats.moisture, 2.0));
    }

    #[test]
    fn offline_progression_is_capped() {
        let mut month = rock();
        let mut two_days = rock();
        let report = month.catch_up(T0 + 30 * DAY);
        two_days.catch_up(T0 + OFFLINE_CAP_SECS);

        assert_eq!(report.away_secs, 30 * DAY);
        assert_eq!(report.simulated_secs, OFFLINE_CAP_SECS);
        assert_eq!(month.stats, two_days.stats);
        assert_eq!(month.last_seen, T0 + 30 * DAY);
        assert!(report.unlocked.contains(&Achievement::ProdigalOwner));
        assert!(report.unlocked.contains(&Achievement::DeepTime));

        // Still playable: nothing pinned at an extreme by neglect alone.
        assert!(month.stats.moisture >= DRIFT_FLOOR);
        assert!(month.stats.polish >= DRIFT_FLOOR);
        assert!(month.stats.boredom <= DRIFT_CEILING);
        assert!(month.stats.disappointment <= DRIFT_CEILING);
        assert_in_bounds(&month.stats);
    }

    #[test]
    fn short_absence_drifts_proportionally() {
        let mut r = rock();
        let before = r.stats.clone();
        let report = r.catch_up(T0 + 2 * HOUR);
        assert_eq!(report.simulated_secs, 2 * HOUR);
        assert!(approx(
            r.stats.boredom,
            before.boredom + 2.0 * BOREDOM_DRIFT
        ));
        assert!(!report.changes.is_empty());
    }

    #[test]
    fn clock_going_backwards_costs_nothing() {
        let mut r = rock();
        r.last_seen = T0 + 10 * DAY;
        let before = r.stats.clone();
        let report = r.catch_up(T0);
        assert!(report.clock_went_backwards);
        assert_eq!(report.simulated_secs, 0);
        assert_eq!(r.stats, before);
        assert_eq!(r.last_seen, T0);
        assert!(r.has(Achievement::TemporalAnomaly));

        // Age and neglect never go negative.
        r.born_at = T0 + DAY;
        assert_eq!(r.age_secs(T0), 0);
        r.last_interaction = T0 + DAY;
        assert_eq!(r.neglect_secs(T0), 0);
    }

    #[test]
    fn neglect_makes_it_more_sarcastic() {
        let r = rock();
        let fresh = r.mood(T0).sarcasm;
        let ignored = r.mood(T0 + 12 * HOUR).sarcasm;
        assert!(ignored > fresh, "{fresh} -> {ignored}");
    }

    #[test]
    fn feeding_ten_times_unlocks_a_discovery() {
        let mut dice = Dice::seeded(5);
        let mut r = rock();
        let mut seen = false;
        for i in 0..10 {
            let outcome = r.perform(Action::Feed, T0 + i, &mut dice);
            seen |= outcome.unlocked.contains(&Achievement::CommitmentToTheBit);
        }
        assert!(seen);
        assert!(!r.unlock(Achievement::CommitmentToTheBit, T0));
    }

    #[test]
    fn rename_cleans_names_and_rejects_empty_ones() {
        let mut dice = Dice::seeded(8);
        let mut r = rock();
        assert!(r.rename("   \t ", T0, &mut dice).is_none());
        assert!(r.rename("Gerald", T0, &mut dice).is_none());
        let outcome = r
            .rename("  Sir   Reginald\u{7} Gneiss ", T0, &mut dice)
            .unwrap();
        assert_eq!(outcome.action, Action::Rename);
        assert_eq!(r.name, "Sir Reginald Gneiss");
        let long = "x".repeat(100);
        r.rename(&long, T0, &mut dice).unwrap();
        assert_eq!(r.name.chars().count(), NAME_MAX_CHARS);
    }

    #[test]
    fn sanitize_fixes_broken_rocks() {
        let mut r = rock();
        r.name = "\n\n".into();
        r.position = 400;
        r.age_myr = 0;
        r.last_interaction = 0;
        r.achievements.insert("made_up".into(), T0);
        r.sanitize(T0);
        assert_eq!(r.name, "Rock");
        assert_eq!(r.position, POSITION_LIMIT);
        assert!(r.age_myr > 0);
        assert_eq!(r.last_interaction, r.last_seen);
        assert!(r.achievements.is_empty());
    }
}
