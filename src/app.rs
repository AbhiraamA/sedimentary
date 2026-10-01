//! Screens, input and timing. Glues the rock, the narrator, the journal and
//! the save file together. Drawing lives in `ui`; nothing here touches the
//! terminal, and every method takes the current time so tests can control it.

use std::cell::Cell;
use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::clock::{HOUR, duration_words};
use crate::dice::Dice;
use crate::journal::{EntryKind, Journal};
use crate::responses::Narrator;
use crate::rock::{
    Action, Mood, NAME_MAX_CHARS, Outcome, Personality, REPEAT_PENALTY, Rock, Stat, sanitize_name,
};
use crate::save::{self, Loaded, SaveData};

/// How long the rock's face reacts after an action.
const EXPRESSION_SECS: i64 = 4;
/// How often the rock gets a chance to move or do something improbable.
const IDLE_ROLL_SECS: i64 = 15;
/// Background saves while stats drift. Actions save immediately.
const AUTOSAVE_SECS: i64 = 30;
const THOUGHT_SECS: (u32, u32) = (30, 50);
/// Shorter absences get no welcome-back speech.
const WELCOME_AFTER_SECS: i64 = 6 * HOUR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStep {
    Name,
    Personality,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    /// First launch, or after confirming New Rock.
    Setup {
        step: SetupStep,
        name: String,
        selected: usize,
        suggestion: String,
    },
    Main,
    Rename {
        input: String,
        suggestion: String,
    },
    ConfirmNew,
    Journal {
        scroll: usize,
    },
    Discoveries {
        scroll: usize,
    },
    Help {
        scroll: usize,
    },
}

/// The most recent thing worth reading: an action and its fallout, a
/// welcome back, or something that happened while you were watching.
#[derive(Clone, Debug, PartialEq)]
pub struct Banner {
    pub title: String,
    pub speech: String,
    pub changes: Vec<(Stat, f64)>,
    /// Rule explanations, e.g. why disappointment went up more than expected.
    pub notes: Vec<String>,
    pub events: Vec<(EntryKind, String)>,
    pub at: i64,
}

pub struct App {
    pub rock: Option<Rock>,
    pub journal: Journal,
    pub screen: Screen,
    pub thought: String,
    pub banner: Option<Banner>,
    /// One-off message about the app itself (saves, recovered files).
    pub notice: Option<String>,
    pub save_path: PathBuf,
    /// False if an unreadable save couldn't be moved aside: we never
    /// overwrite a file we couldn't back up.
    pub saving_enabled: bool,
    pub now: i64,
    /// Animation counter, advanced by the main loop.
    pub frame: u64,
    pub should_quit: bool,
    /// Largest useful scroll offset for the open overlay. Written by the UI,
    /// which is the only part that knows how tall things are.
    pub max_scroll: Cell<usize>,
    narrator: Narrator,
    dice: Dice,
    expression: Option<(Action, i64)>,
    last_tick: i64,
    next_thought: i64,
    next_idle_roll: i64,
    next_autosave: i64,
    dirty: bool,
    save_failing: bool,
    farewell: Option<String>,
    /// A rock set aside by New Rock, restored if setup is cancelled.
    previous: Option<(Rock, Journal)>,
    /// Journal entry to record once a rock exists (e.g. a recovered save).
    pending_note: Option<String>,
}

fn action_for_key(c: char) -> Option<Action> {
    if let Some(digit) = c.to_digit(10) {
        let index = usize::try_from(digit).ok()?.checked_sub(1)?;
        return Action::ALL.get(index).copied();
    }
    Action::ALL.into_iter().find(|a| a.key() == c)
}

fn action_title(action: Action, name: &str) -> String {
    match action {
        Action::Feed => format!("You fed {name}."),
        Action::Water => format!("You watered {name}."),
        Action::Polish => format!("You polished {name}."),
        Action::Talk => format!("You talked at {name}."),
        Action::Walk => format!("You took {name} for a walk."),
        Action::Stare => format!("You stared at {name}. {name} stared back."),
        Action::Rename => format!("Your rock is now called {name}."),
    }
}

/// Typing into a text field: plain characters only, capped in length.
fn edit_text(field: &mut String, key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Backspace => {
            field.pop();
            true
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                && !c.is_control() =>
        {
            if field.chars().count() < NAME_MAX_CHARS {
                field.push(c);
            }
            true
        }
        _ => false,
    }
}

impl App {
    fn blank(save_path: PathBuf, now: i64, dice: Dice) -> Self {
        Self {
            rock: None,
            journal: Journal::default(),
            screen: Screen::Main,
            thought: String::new(),
            banner: None,
            notice: None,
            save_path,
            saving_enabled: true,
            now,
            frame: 1,
            should_quit: false,
            max_scroll: Cell::new(0),
            narrator: Narrator::new(),
            dice,
            expression: None,
            last_tick: now,
            next_thought: now,
            next_idle_roll: now,
            next_autosave: now,
            dirty: false,
            save_failing: false,
            farewell: None,
            previous: None,
            pending_note: None,
        }
    }

    /// Open the save at `save_path`, recovering from anything odd in it.
    pub fn load(save_path: PathBuf, now: i64, dice: Dice) -> Self {
        let mut app = Self::blank(save_path, now, dice);
        match save::load(&app.save_path, now) {
            Loaded::Fresh => app.begin_setup(),
            Loaded::Existing(data) => app.resume(*data, now),
            Loaded::Recovered { backup, problem } => {
                let note = match &backup {
                    Some(path) => format!(
                        "The old save couldn't be used: {problem}. It was kept at {}.",
                        path.display()
                    ),
                    None => {
                        app.saving_enabled = false;
                        format!(
                            "The save couldn't be used ({problem}) and couldn't be moved aside, \
                             so this session won't save over it."
                        )
                    }
                };
                app.notice = Some(note.clone());
                app.pending_note = Some(note);
                app.begin_setup();
            }
        }
        app
    }

    fn begin_setup(&mut self) {
        let suggestion = self.narrator.name_idea(&mut self.dice).to_string();
        self.screen = Screen::Setup {
            step: SetupStep::Name,
            name: String::new(),
            selected: 0,
            suggestion,
        };
    }

    fn reset_timers(&mut self, now: i64) {
        self.last_tick = now;
        self.next_thought = now + i64::from(self.dice.between(THOUGHT_SECS.0, THOUGHT_SECS.1));
        self.next_idle_roll = now + IDLE_ROLL_SECS;
        self.next_autosave = now + AUTOSAVE_SECS;
    }

    fn resume(&mut self, data: SaveData, now: i64) {
        let SaveData {
            mut rock,
            mut journal,
            ..
        } = data;
        rock.sanitize(now);
        journal.sanitize();
        let report = rock.catch_up(now);
        let welcome = self.narrator.welcome_back(&rock, &report, &mut self.dice);
        let events = self
            .narrator
            .events(&rock, None, false, &report.unlocked, &mut self.dice);

        if report.clock_went_backwards {
            journal.push(
                now,
                EntryKind::System,
                "The clock went backwards since last time. No time was counted.",
            );
        } else if report.away_secs >= WELCOME_AFTER_SECS {
            journal.push(
                now,
                EntryKind::System,
                format!("You came back after {}.", duration_words(report.away_secs)),
            );
        }
        if let Some(speech) = &welcome {
            journal.push(now, EntryKind::Remark, speech.clone());
        }
        for (kind, text) in &events {
            journal.push(now, *kind, text.clone());
        }

        let mut notes = Vec::new();
        if report.clock_went_backwards {
            notes.push("Nothing drifted while time was confused.".to_string());
        } else if report.away_secs > report.simulated_secs {
            notes.push(format!(
                "Time away is capped, so only {} counted.",
                duration_words(report.simulated_secs)
            ));
        }
        self.banner = match welcome {
            Some(speech) => Some(Banner {
                title: if report.clock_went_backwards {
                    "Your clock went backwards.".to_string()
                } else {
                    format!("You were gone for {}.", duration_words(report.away_secs))
                },
                speech,
                changes: report.changes.clone(),
                notes,
                events,
                at: now,
            }),
            None if !events.is_empty() => Some(Banner {
                title: "While you were away".to_string(),
                speech: String::new(),
                changes: Vec::new(),
                notes,
                events,
                at: now,
            }),
            None => None,
        };

        self.thought = self.narrator.thought(&rock, &mut self.dice);
        self.rock = Some(rock);
        self.journal = journal;
        self.screen = Screen::Main;
        self.reset_timers(now);
        self.persist();
    }

    fn adopt(&mut self, name: &str, personality: Personality, now: i64) {
        let rock = Rock::new(name, personality, now, &mut self.dice);
        let mut journal = Journal::default();

        if let Some((old, _)) = self.previous.take()
            && self.saving_enabled
            && self.save_path.exists()
        {
            match save::preserve(&self.save_path, "retired", now) {
                Ok(path) => journal.push(
                    now,
                    EntryKind::System,
                    format!(
                        "{} retired to a quiet quarry. Its save lives on at {}.",
                        old.name,
                        path.display()
                    ),
                ),
                Err(e) => {
                    self.notice = Some(format!(
                        "Couldn't keep a copy of {}'s save ({e}). It has been replaced.",
                        old.name
                    ));
                }
            }
        }
        if let Some(note) = self.pending_note.take() {
            journal.push(now, EntryKind::System, note);
        }
        journal.push(
            now,
            EntryKind::System,
            format!(
                "Adopted {}, a {} {} rock. It did not adopt you back.",
                rock.name,
                personality.label().to_lowercase(),
                rock.formation.label()
            ),
        );
        let greeting = self.narrator.greeting(&rock, &mut self.dice);
        journal.push(now, EntryKind::Remark, greeting.clone());

        self.banner = Some(Banner {
            title: format!("Meet {}.", rock.name),
            speech: greeting,
            changes: Vec::new(),
            notes: Vec::new(),
            events: Vec::new(),
            at: now,
        });
        self.thought = self.narrator.thought(&rock, &mut self.dice);
        self.rock = Some(rock);
        self.journal = journal;
        self.screen = Screen::Main;
        self.expression = None;
        self.reset_timers(now);
        self.persist();
    }

    /// Advance timers and the simulation. Called every frame.
    pub fn tick(&mut self, now: i64) {
        let elapsed = now - self.last_tick;
        if elapsed < 0 {
            // The clock jumped backwards mid-session. Don't wait for it.
            self.reset_timers(now);
        }
        self.now = now;
        self.last_tick = now;
        if self.expression.is_some_and(|(_, until)| now >= until) {
            self.expression = None;
        }

        let Some(rock) = self.rock.as_mut() else {
            return;
        };
        if elapsed != 0 {
            rock.tick(elapsed.max(0) as f64, now);
            self.dirty = true;
        }

        if now >= self.next_idle_roll {
            self.next_idle_roll = now + IDLE_ROLL_SECS;
            let idle = rock.idle_roll(now, &mut self.dice);
            if !idle.is_empty() {
                let events = self.narrator.events(
                    rock,
                    idle.moved,
                    idle.rare_event,
                    &idle.unlocked,
                    &mut self.dice,
                );
                for (kind, text) in &events {
                    self.journal.push(now, *kind, text.clone());
                }
                self.banner = Some(Banner {
                    title: "Meanwhile, without your help".to_string(),
                    speech: String::new(),
                    changes: Vec::new(),
                    notes: Vec::new(),
                    events,
                    at: now,
                });
                self.persist();
            }
        }

        if let Some(rock) = &self.rock
            && now >= self.next_thought
        {
            self.thought = self.narrator.thought(rock, &mut self.dice);
            self.next_thought = now + i64::from(self.dice.between(THOUGHT_SECS.0, THOUGHT_SECS.1));
        }

        if self.dirty && now >= self.next_autosave {
            self.next_autosave = now + AUTOSAVE_SECS;
            self.persist();
        }
    }

    /// Save now. Failures become a notice rather than a crash.
    pub fn persist(&mut self) {
        if !self.saving_enabled {
            return;
        }
        let Some(rock) = &self.rock else {
            return;
        };
        let data = SaveData::new(rock.clone(), self.journal.clone(), self.now);
        match save::store(&self.save_path, &data) {
            Ok(()) => {
                self.dirty = false;
                if self.save_failing {
                    self.save_failing = false;
                    self.notice = Some("Saving works again.".to_string());
                }
            }
            Err(e) => {
                if !self.save_failing {
                    self.notice = Some(format!(
                        "Couldn't save to {}: {e}",
                        self.save_path.display()
                    ));
                }
                self.save_failing = true;
            }
        }
    }

    pub fn mood(&self) -> Option<Mood> {
        self.rock.as_ref().map(|r| r.mood(self.now))
    }

    /// The action the rock's face is still reacting to, if any.
    pub fn expression(&self) -> Option<Action> {
        self.expression.map(|(action, _)| action)
    }

    /// The rock's parting words, once the app has quit.
    pub fn farewell(&self) -> Option<&str> {
        self.farewell.as_deref()
    }

    pub fn quit(&mut self) {
        if self.rock.is_none()
            && let Some((rock, journal)) = self.previous.take()
        {
            self.rock = Some(rock);
            self.journal = journal;
        }
        if let Some(rock) = &self.rock {
            self.farewell = Some(self.narrator.farewell(rock, &mut self.dice));
        }
        self.persist();
        self.should_quit = true;
    }

    /// Do something to the rock and hear about it.
    pub fn act(&mut self, action: Action, now: i64) {
        let Some(rock) = self.rock.as_mut() else {
            return;
        };
        let mood = rock.mood(now);
        let outcome = rock.perform(action, now, &mut self.dice);
        self.after_action(outcome, mood, now);
    }

    fn after_action(&mut self, outcome: Outcome, mood: Mood, now: i64) {
        let Some(rock) = &self.rock else {
            return;
        };
        let reaction = self.narrator.react(rock, &outcome, mood, &mut self.dice);
        self.journal
            .push_action(now, outcome.action, reaction.speech.clone());
        for (kind, text) in &reaction.events {
            self.journal.push(now, *kind, text.clone());
        }

        let mut notes = Vec::new();
        if outcome.repeated {
            notes.push(format!(
                "Same as last time: half the boredom relief, +{REPEAT_PENALTY:.0} disappointment."
            ));
        }
        if outcome.overdone {
            notes.push(
                match outcome.action {
                    Action::Water => "Already soaked, so the extra water mostly annoyed it.",
                    _ => "Already gleaming, so it got bored and slightly insulted.",
                }
                .to_string(),
            );
        }
        self.banner = Some(Banner {
            title: action_title(outcome.action, &rock.name),
            speech: reaction.speech,
            changes: outcome.changes,
            notes,
            events: reaction.events,
            at: now,
        });
        self.expression = Some((outcome.action, now + EXPRESSION_SECS));
        if !self.save_failing {
            self.notice = None;
        }
        self.persist();
    }

    fn submit_rename(&mut self, input: &str, now: i64) {
        let Some(rock) = self.rock.as_mut() else {
            return;
        };
        let old_name = rock.name.clone();
        let mood = rock.mood(now);
        match rock.rename(input, now, &mut self.dice) {
            Some(outcome) => {
                let new_name = rock.name.clone();
                self.journal.push(
                    now,
                    EntryKind::System,
                    format!("Renamed {old_name} to {new_name}."),
                );
                self.screen = Screen::Main;
                self.after_action(outcome, mood, now);
            }
            None if sanitize_name(input).is_none() => {
                self.notice = Some("A name needs at least one visible character.".to_string());
            }
            None => {
                self.notice = Some(format!(
                    "That's already its name. {old_name} noticed you hesitate."
                ));
                self.screen = Screen::Main;
            }
        }
    }

    /// Handle one key press.
    pub fn handle_key(&mut self, key: KeyEvent, now: i64) {
        self.tick(now);
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C'))
        {
            self.quit();
            return;
        }
        match self.screen {
            Screen::Setup { .. } => self.setup_key(key, now),
            Screen::Main => self.main_key(key, now),
            Screen::Rename { .. } => self.rename_key(key, now),
            Screen::ConfirmNew => self.confirm_key(key),
            Screen::Journal { .. } | Screen::Discoveries { .. } | Screen::Help { .. } => {
                self.overlay_key(key)
            }
        }
    }

    fn main_key(&mut self, key: KeyEvent, now: i64) {
        // Esc deliberately doesn't quit here: it's how overlays close, and a
        // double tap shouldn't end the session.
        let KeyCode::Char(c) = key.code else {
            return;
        };
        let c = c.to_ascii_lowercase();
        match action_for_key(c) {
            Some(Action::Rename) => {
                let suggestion = self.narrator.name_idea(&mut self.dice).to_string();
                self.screen = Screen::Rename {
                    input: String::new(),
                    suggestion,
                };
            }
            Some(action) => self.act(action, now),
            None => match c {
                'j' => self.screen = Screen::Journal { scroll: 0 },
                'a' => self.screen = Screen::Discoveries { scroll: 0 },
                '?' | 'h' => self.screen = Screen::Help { scroll: 0 },
                'n' => self.screen = Screen::ConfirmNew,
                'q' => self.quit(),
                _ => {}
            },
        }
    }

    fn setup_key(&mut self, key: KeyEvent, now: i64) {
        let Screen::Setup {
            step,
            name,
            selected,
            suggestion,
        } = &mut self.screen
        else {
            return;
        };
        match *step {
            SetupStep::Name => match key.code {
                KeyCode::Enter => {
                    if sanitize_name(name).is_none() {
                        *name = suggestion.clone();
                    }
                    *step = SetupStep::Personality;
                }
                KeyCode::Tab => {
                    *name = suggestion.clone();
                    *suggestion = self.narrator.name_idea(&mut self.dice).to_string();
                }
                KeyCode::Esc => {
                    if let Some((rock, journal)) = self.previous.take() {
                        self.rock = Some(rock);
                        self.journal = journal;
                        self.screen = Screen::Main;
                        self.reset_timers(now);
                    } else {
                        self.quit();
                    }
                }
                _ => {
                    edit_text(name, &key);
                }
            },
            SetupStep::Personality => match key.code {
                KeyCode::Up | KeyCode::Left | KeyCode::BackTab => {
                    *selected = (*selected + Personality::ALL.len() - 1) % Personality::ALL.len();
                }
                KeyCode::Down | KeyCode::Right | KeyCode::Tab => {
                    *selected = (*selected + 1) % Personality::ALL.len();
                }
                KeyCode::Char(c @ '1'..='3') => {
                    *selected = (c as usize) - ('1' as usize);
                }
                KeyCode::Esc => *step = SetupStep::Name,
                KeyCode::Enter => {
                    let name = name.clone();
                    let personality = Personality::ALL[*selected % Personality::ALL.len()];
                    self.adopt(&name, personality, now);
                }
                _ => {}
            },
        }
    }

    fn rename_key(&mut self, key: KeyEvent, now: i64) {
        let Screen::Rename { input, suggestion } = &mut self.screen else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.screen = Screen::Main,
            KeyCode::Tab => {
                *input = suggestion.clone();
                *suggestion = self.narrator.name_idea(&mut self.dice).to_string();
            }
            KeyCode::Enter => {
                let input = input.clone();
                self.submit_rename(&input, now);
            }
            _ => {
                edit_text(input, &key);
            }
        }
    }

    fn confirm_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y' | 'Y') => {
                if let Some(rock) = self.rock.take() {
                    let journal = std::mem::take(&mut self.journal);
                    self.previous = Some((rock, journal));
                }
                self.banner = None;
                self.begin_setup();
            }
            KeyCode::Char('n' | 'N' | 'q') | KeyCode::Esc | KeyCode::Enter => {
                self.screen = Screen::Main;
            }
            _ => {}
        }
    }

    fn overlay_key(&mut self, key: KeyEvent) {
        let max = self.max_scroll.get();
        let (scroll, toggle) = match &mut self.screen {
            Screen::Journal { scroll } => (scroll, 'j'),
            Screen::Discoveries { scroll } => (scroll, 'a'),
            Screen::Help { scroll } => (scroll, '?'),
            _ => return,
        };
        match key.code {
            KeyCode::Up => *scroll = scroll.saturating_sub(1),
            KeyCode::Down => *scroll = (*scroll + 1).min(max),
            KeyCode::PageUp => *scroll = scroll.saturating_sub(10),
            KeyCode::PageDown | KeyCode::Char(' ') => *scroll = (*scroll + 10).min(max),
            KeyCode::Home => *scroll = 0,
            KeyCode::End => *scroll = max,
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => self.screen = Screen::Main,
            KeyCode::Char(c) if c == toggle || (toggle == '?' && c == 'h') => {
                self.screen = Screen::Main
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::DAY;

    const T0: i64 = 1_750_000_000;

    fn press(app: &mut App, code: KeyCode, now: i64) {
        app.handle_key(KeyEvent::new(code, KeyModifiers::NONE), now);
    }

    fn type_text(app: &mut App, text: &str, now: i64) {
        for c in text.chars() {
            press(app, KeyCode::Char(c), now);
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir()
            .join(format!(
                "sedimentary-app-{name}-{}-{nanos}",
                std::process::id()
            ))
            .join("save.json")
    }

    fn adopted(name: &str) -> (App, PathBuf) {
        let path = scratch(name);
        let mut app = App::load(path.clone(), T0, Dice::seeded(5));
        type_text(&mut app, "Gerald", T0);
        press(&mut app, KeyCode::Enter, T0);
        press(&mut app, KeyCode::Down, T0);
        press(&mut app, KeyCode::Enter, T0);
        (app, path)
    }

    fn cleanup(path: &std::path::Path) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn digits_and_letters_map_to_actions() {
        assert_eq!(action_for_key('1'), Some(Action::Feed));
        assert_eq!(action_for_key('7'), Some(Action::Rename));
        assert_eq!(action_for_key('8'), None);
        assert_eq!(action_for_key('0'), None);
        assert_eq!(action_for_key('k'), Some(Action::Walk));
        assert_eq!(action_for_key('j'), None);
    }

    #[test]
    fn first_launch_walks_through_setup() {
        let (app, path) = adopted("setup");
        let rock = app.rock.as_ref().expect("a rock was adopted");
        assert_eq!(rock.name, "Gerald");
        assert_eq!(rock.personality, Personality::Existential);
        assert_eq!(app.screen, Screen::Main);
        assert!(path.exists(), "adoption should save immediately");
        cleanup(&path);
    }

    #[test]
    fn blank_name_takes_the_suggestion() {
        let path = scratch("blank");
        let mut app = App::load(path.clone(), T0, Dice::seeded(8));
        let Screen::Setup { suggestion, .. } = app.screen.clone() else {
            panic!("expected setup");
        };
        press(&mut app, KeyCode::Enter, T0);
        press(&mut app, KeyCode::Enter, T0);
        assert_eq!(app.rock.unwrap().name, suggestion);
        cleanup(&path);
    }

    #[test]
    fn actions_update_stats_banner_and_journal() {
        let (mut app, path) = adopted("act");
        let before = app.rock.as_ref().unwrap().stats.clone();
        let entries = app.journal.len();
        press(&mut app, KeyCode::Char('w'), T0 + 1);
        let rock = app.rock.as_ref().unwrap();
        assert!(rock.stats.moisture > before.moisture);
        let banner = app.banner.as_ref().unwrap();
        assert!(banner.title.contains("watered"));
        assert!(!banner.speech.is_empty());
        assert!(app.journal.len() > entries);
        assert_eq!(app.expression(), Some(Action::Water));
        app.tick(T0 + 10);
        assert_eq!(app.expression(), None);
        cleanup(&path);
    }

    #[test]
    fn rename_flow_and_rejections() {
        let (mut app, path) = adopted("rename");
        press(&mut app, KeyCode::Char('r'), T0 + 1);
        assert!(matches!(app.screen, Screen::Rename { .. }));
        type_text(&mut app, "   ", T0 + 1);
        press(&mut app, KeyCode::Enter, T0 + 1);
        assert!(
            matches!(app.screen, Screen::Rename { .. }),
            "blank names stay open"
        );
        assert!(app.notice.is_some());
        for _ in 0..3 {
            press(&mut app, KeyCode::Backspace, T0 + 1);
        }
        type_text(&mut app, "Sir Reginald Gneiss", T0 + 1);
        press(&mut app, KeyCode::Enter, T0 + 1);
        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.rock.as_ref().unwrap().name, "Sir Reginald Gneiss");
        assert_eq!(app.rock.as_ref().unwrap().counters.renames, 1);
        cleanup(&path);
    }

    #[test]
    fn new_rock_needs_confirmation_and_can_be_cancelled() {
        let (mut app, path) = adopted("newrock");
        press(&mut app, KeyCode::Char('n'), T0 + 1);
        assert_eq!(app.screen, Screen::ConfirmNew);
        press(&mut app, KeyCode::Char('n'), T0 + 1);
        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.rock.as_ref().unwrap().name, "Gerald");

        // Confirm, then back out of setup: Gerald comes back.
        press(&mut app, KeyCode::Char('n'), T0 + 2);
        press(&mut app, KeyCode::Char('y'), T0 + 2);
        assert!(matches!(app.screen, Screen::Setup { .. }));
        assert!(app.rock.is_none());
        press(&mut app, KeyCode::Esc, T0 + 2);
        assert_eq!(app.rock.as_ref().unwrap().name, "Gerald");
        assert!(!app.journal.is_empty());

        // Confirm and finish: the old save is retired, not destroyed.
        press(&mut app, KeyCode::Char('n'), T0 + 3);
        press(&mut app, KeyCode::Char('y'), T0 + 3);
        type_text(&mut app, "Cliff", T0 + 3);
        press(&mut app, KeyCode::Enter, T0 + 3);
        press(&mut app, KeyCode::Enter, T0 + 3);
        assert_eq!(app.rock.as_ref().unwrap().name, "Cliff");
        let dir = path.parent().unwrap();
        let retired = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().contains("retired"));
        assert!(retired, "the previous rock's save should be kept");
        cleanup(&path);
    }

    #[test]
    fn quitting_during_new_rock_setup_keeps_the_old_rock() {
        let (mut app, path) = adopted("quitsetup");
        press(&mut app, KeyCode::Char('n'), T0 + 1);
        press(&mut app, KeyCode::Char('y'), T0 + 1);
        app.handle_key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            T0 + 1,
        );
        assert!(app.should_quit);
        let reopened = App::load(path.clone(), T0 + 2, Dice::seeded(1));
        assert_eq!(reopened.rock.unwrap().name, "Gerald");
        cleanup(&path);
    }

    #[test]
    fn reopening_after_a_month_is_capped_and_welcoming() {
        let (mut app, path) = adopted("month");
        app.quit();
        let later = T0 + 31 * DAY;
        let app = App::load(path.clone(), later, Dice::seeded(2));
        assert_eq!(app.screen, Screen::Main);
        let banner = app.banner.as_ref().expect("a welcome back");
        assert!(banner.title.contains("31 days"), "{}", banner.title);
        assert!(banner.notes.iter().any(|n| n.contains("capped")));
        let stats = &app.rock.as_ref().unwrap().stats;
        assert!(stats.boredom <= 85.0 && stats.moisture >= 10.0);
        cleanup(&path);
    }

    #[test]
    fn overlays_open_scroll_and_close() {
        let (mut app, path) = adopted("overlay");
        press(&mut app, KeyCode::Char('j'), T0 + 1);
        app.max_scroll.set(5);
        for _ in 0..20 {
            press(&mut app, KeyCode::Down, T0 + 1);
        }
        assert_eq!(app.screen, Screen::Journal { scroll: 5 });
        press(&mut app, KeyCode::Char('j'), T0 + 1);
        assert_eq!(app.screen, Screen::Main);
        press(&mut app, KeyCode::Char('?'), T0 + 1);
        assert!(matches!(app.screen, Screen::Help { .. }));
        press(&mut app, KeyCode::Esc, T0 + 1);
        press(&mut app, KeyCode::Char('a'), T0 + 1);
        assert!(matches!(app.screen, Screen::Discoveries { .. }));
        press(&mut app, KeyCode::Char('q'), T0 + 1);
        assert_eq!(app.screen, Screen::Main);
        assert!(!app.should_quit, "q closes overlays before it quits");
        press(&mut app, KeyCode::Esc, T0 + 1);
        assert!(
            !app.should_quit,
            "a stray Esc on the main screen doesn't quit"
        );
        press(&mut app, KeyCode::Char('q'), T0 + 1);
        assert!(app.should_quit);
        cleanup(&path);
    }

    #[test]
    fn idle_time_drifts_and_eventually_does_something_historic() {
        let (mut app, path) = adopted("idle");
        // Rare events relieve boredom, so moisture is the honest witness to time.
        let start = app.rock.as_ref().unwrap().stats.moisture;
        let mut now = T0;
        for _ in 0..4_000 {
            now += 15;
            app.tick(now);
        }
        let rock = app.rock.as_ref().unwrap();
        assert!(rock.stats.moisture < start);
        assert!(rock.counters.moves > 0, "the rock should have moved by now");
        cleanup(&path);
    }
}
