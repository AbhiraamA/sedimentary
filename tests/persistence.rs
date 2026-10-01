//! End-to-end checks of saving, loading and coming back after time away.

use std::fs;
use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sedimentary::achievements::Achievement;
use sedimentary::app::{App, Screen};
use sedimentary::clock::{DAY, HOUR};
use sedimentary::dice::Dice;
use sedimentary::journal::{EntryKind, Journal};
use sedimentary::rock::{
    Action, DRIFT_CEILING, DRIFT_FLOOR, OFFLINE_CAP_SECS, Personality, Rock, Stat,
};
use sedimentary::save::{self, FORMAT_VERSION, Loaded, SaveData};

const T0: i64 = 1_750_000_000;

/// A fresh, empty directory for one test.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "sedimentary-it-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    fn save(&self) -> PathBuf {
        self.0.join("save.json")
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn press(app: &mut App, code: KeyCode, now: i64) {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE), now);
}

fn adopt(path: &Path, name: &str, now: i64) -> App {
    let mut app = App::load(path.to_path_buf(), now, Dice::seeded(77));
    for c in name.chars() {
        press(&mut app, KeyCode::Char(c), now);
    }
    press(&mut app, KeyCode::Enter, now);
    press(&mut app, KeyCode::Char('3'), now);
    press(&mut app, KeyCode::Enter, now);
    app
}

fn busy_rock(now: i64) -> (Rock, Journal) {
    let mut dice = Dice::seeded(4);
    let mut rock = Rock::new("Feldspar", Personality::Judgmental, now, &mut dice);
    let mut journal = Journal::default();
    for (i, action) in Action::ALL.iter().cycle().take(40).enumerate() {
        let at = now + i as i64 * 30;
        rock.perform(*action, at, &mut dice);
        journal.push_action(at, *action, format!("line {i}"));
    }
    journal.push(now, EntryKind::Movement, "It moved. Historians wept.");
    (rock, journal)
}

#[test]
fn a_save_survives_a_round_trip_exactly() {
    let dir = Scratch::new("roundtrip");
    let (rock, journal) = busy_rock(T0);
    let data = SaveData::new(rock, journal, T0 + 2_000);
    save::store(&dir.save(), &data).unwrap();

    match save::load(&dir.save(), T0 + 3_000) {
        Loaded::Existing(loaded) => assert_eq!(*loaded, data),
        other => panic!("expected the save back, got {other:?}"),
    }
    assert_eq!(dir.files(), vec!["save.json"], "no temp files left behind");
}

#[test]
fn the_save_file_is_readable_json_with_the_essentials() {
    let dir = Scratch::new("format");
    let (rock, journal) = busy_rock(T0);
    save::store(&dir.save(), &SaveData::new(rock, journal, T0)).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.save()).unwrap()).unwrap();
    assert_eq!(json["format_version"], FORMAT_VERSION);
    for key in ["saved_at", "rock", "journal"] {
        assert!(json.get(key).is_some(), "missing {key}");
    }
    for key in [
        "name",
        "personality",
        "stats",
        "last_seen",
        "born_at",
        "achievements",
    ] {
        assert!(json["rock"].get(key).is_some(), "missing rock.{key}");
    }
}

#[test]
fn a_malformed_save_is_preserved_before_anything_replaces_it() {
    let dir = Scratch::new("malformed");
    let garbage = "{ \"rock\": { \"name\": \"Gera";
    fs::write(dir.save(), garbage).unwrap();

    let mut app = App::load(dir.save(), T0, Dice::seeded(1));
    assert!(matches!(app.screen, Screen::Setup { .. }));
    assert!(app.saving_enabled);
    let notice = app
        .notice
        .clone()
        .expect("the player is told what happened");
    assert!(notice.contains("unreadable"), "{notice}");

    let backups: Vec<String> = dir
        .files()
        .into_iter()
        .filter(|f| f.contains("unreadable"))
        .collect();
    assert_eq!(backups.len(), 1, "{:?}", dir.files());
    let kept = fs::read_to_string(dir.0.join(&backups[0])).unwrap();
    assert_eq!(kept, garbage, "the original bytes are untouched");

    // Adopting a new rock writes a fresh save and notes the recovery.
    for c in "Mica".chars() {
        press(&mut app, KeyCode::Char(c), T0);
    }
    press(&mut app, KeyCode::Enter, T0);
    press(&mut app, KeyCode::Enter, T0);
    assert!(matches!(save::load(&dir.save(), T0), Loaded::Existing(_)));
    assert!(app.journal.recent().any(|e| e.text.contains("unreadable")));
}

#[test]
fn saves_from_a_newer_version_are_set_aside_not_overwritten() {
    let dir = Scratch::new("future");
    let (rock, journal) = busy_rock(T0);
    let mut data = SaveData::new(rock, journal, T0);
    data.format_version = FORMAT_VERSION + 1;
    let text = serde_json::to_string(&data).unwrap();
    fs::write(dir.save(), &text).unwrap();

    match save::load(&dir.save(), T0) {
        Loaded::Recovered {
            backup: Some(path),
            problem,
        } => {
            assert!(problem.contains("newer"));
            assert_eq!(fs::read_to_string(path).unwrap(), text);
        }
        other => panic!("expected recovery, got {other:?}"),
    }
    assert!(!dir.save().exists());
}

#[test]
fn hand_edited_nonsense_is_repaired_on_load() {
    let dir = Scratch::new("edited");
    let (mut rock, journal) = busy_rock(T0);
    rock.stats.moisture = 9_000.0;
    rock.stats.polish = -50.0;
    rock.position = 400;
    rock.name = "   ".to_string();
    rock.achievements.insert("not_a_real_one".into(), T0);
    rock.last_seen = T0 + 3_000;
    save::store(&dir.save(), &SaveData::new(rock, journal, T0)).unwrap();

    let app = App::load(dir.save(), T0 + 3_000, Dice::seeded(2));
    let rock = app.rock.expect("the rock loads");
    for stat in Stat::ALL {
        assert!((0.0..=100.0).contains(&rock.stats.get(stat)));
    }
    assert_eq!(rock.name, "Rock");
    assert!(rock.position.abs() <= 12);
    assert!(!rock.achievements.contains_key("not_a_real_one"));
}

#[test]
fn a_month_away_counts_the_same_as_the_cap() {
    let dir = Scratch::new("month");
    let mut app = adopt(&dir.save(), "Gravelyn", T0);
    app.quit();

    let month = App::load(dir.save(), T0 + 30 * DAY, Dice::seeded(9));
    let capped = App::load(dir.save(), T0 + OFFLINE_CAP_SECS, Dice::seeded(9));
    let (a, b) = (month.rock.unwrap(), capped.rock.unwrap());
    for stat in Stat::ALL {
        let (x, y) = (a.stats.get(stat), b.stats.get(stat));
        assert!((x - y).abs() < 1e-9, "{stat:?}: {x} vs {y}");
        assert!(
            (DRIFT_FLOOR - 1e-9..=DRIFT_CEILING + 1e-9).contains(&x)
                || stat == Stat::Disappointment,
            "{stat:?} drifted to {x}"
        );
    }
    assert!(a.has(Achievement::ProdigalOwner));
    assert_eq!(month.screen, Screen::Main, "still playable straight away");
}

#[test]
fn a_short_break_drifts_a_little() {
    let dir = Scratch::new("short");
    let mut app = adopt(&dir.save(), "Pebbles", T0);
    let before = app.rock.clone().unwrap().stats;
    app.quit();
    let back = App::load(dir.save(), T0 + 2 * HOUR, Dice::seeded(3));
    let after = back.rock.unwrap().stats;
    assert!(after.moisture < before.moisture);
    assert!(after.boredom > before.boredom);
    assert!(back.banner.is_none(), "two hours doesn't warrant a speech");
}

#[test]
fn a_clock_that_runs_backwards_costs_nothing() {
    let dir = Scratch::new("skew");
    let mut app = adopt(&dir.save(), "Dolomite", T0);
    app.quit();
    let before = app.rock.clone().unwrap().stats;

    let back = App::load(dir.save(), T0 - 5 * DAY, Dice::seeded(4));
    let rock = back.rock.clone().unwrap();
    assert_eq!(rock.stats, before);
    assert!(rock.has(Achievement::TemporalAnomaly));
    let banner = back.banner.as_ref().expect("the rock comments on it");
    assert!(banner.title.contains("backwards"));

    // When the clock is fixed, the gap is treated like any absence: capped.
    let mut back = back;
    back.quit();
    let later = App::load(dir.save(), T0, Dice::seeded(5));
    let stats = later.rock.unwrap().stats;
    assert!(stats.boredom <= DRIFT_CEILING && stats.moisture >= DRIFT_FLOOR);
    assert_eq!(later.screen, Screen::Main);
}

#[test]
fn every_action_saves_immediately() {
    let dir = Scratch::new("immediate");
    let mut app = adopt(&dir.save(), "Cliff", T0);
    for (i, action) in Action::ALL.iter().enumerate() {
        if *action == Action::Rename {
            continue;
        }
        let now = T0 + 10 + i as i64;
        app.act(*action, now);
        let Loaded::Existing(data) = save::load(&dir.save(), now) else {
            panic!("save should load");
        };
        assert_eq!(data.rock.last_action, Some(*action));
        assert_eq!(data.rock.stats, app.rock.as_ref().unwrap().stats);
    }
}
