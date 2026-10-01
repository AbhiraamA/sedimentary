//! Local JSON persistence.
//!
//! Writes are atomic (temp file, then rename). A save that can't be read is
//! never overwritten: it is moved aside first, and if that fails the game
//! refuses to save over it for the rest of the session.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::journal::Journal;
use crate::rock::Rock;

pub const FORMAT_VERSION: u32 = 1;
/// Overrides the save location (handy for tests and for keeping several rocks).
pub const PATH_ENV: &str = "SEDIMENTARY_SAVE";
const FILE_NAME: &str = "save.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SaveData {
    pub format_version: u32,
    /// When this file was written (Unix seconds).
    pub saved_at: i64,
    pub rock: Rock,
    #[serde(default)]
    pub journal: Journal,
}

impl SaveData {
    pub fn new(rock: Rock, journal: Journal, now: i64) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            saved_at: now,
            rock,
            journal,
        }
    }
}

#[derive(Debug)]
pub enum Loaded {
    /// No save yet. Time to adopt.
    Fresh,
    Existing(Box<SaveData>),
    /// The save couldn't be used. `backup` is where the original now lives;
    /// `None` means it couldn't be moved and is still at the save path.
    Recovered {
        backup: Option<PathBuf>,
        problem: String,
    },
}

/// `$SEDIMENTARY_SAVE`, else the platform data directory, else the current
/// directory.
pub fn default_save_path() -> PathBuf {
    if let Some(custom) = std::env::var_os(PATH_ENV).filter(|v| !v.is_empty()) {
        return PathBuf::from(custom);
    }
    match dirs::data_dir() {
        Some(dir) => dir.join("sedimentary").join(FILE_NAME),
        None => PathBuf::from("sedimentary-save.json"),
    }
}

/// Parse save contents, rejecting anything this version can't understand.
pub fn parse(text: &str) -> Result<SaveData, String> {
    if text.trim().is_empty() {
        return Err("the file is empty".to_string());
    }
    let data: SaveData =
        serde_json::from_str(text).map_err(|e| format!("it isn't a valid save ({e})"))?;
    if data.format_version > FORMAT_VERSION {
        return Err(format!(
            "it was written by a newer version (format {})",
            data.format_version
        ));
    }
    Ok(data)
}

/// Load the save at `path`. Anything unreadable is moved aside, never deleted.
pub fn load(path: &Path, now: i64) -> Loaded {
    let problem = match fs::read(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Loaded::Fresh,
        Err(e) => format!("it couldn't be read ({e})"),
        Ok(bytes) => match String::from_utf8(bytes) {
            Err(_) => "it isn't text".to_string(),
            Ok(text) => match parse(&text) {
                Ok(data) => return Loaded::Existing(Box::new(data)),
                Err(problem) => problem,
            },
        },
    };
    Loaded::Recovered {
        backup: preserve(path, "unreadable", now).ok(),
        problem,
    }
}

/// Write the save atomically, creating directories as needed.
pub fn store(path: &Path, data: &SaveData) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data).map_err(io::Error::other)?;
    let tmp = sibling(path, "tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// Move the file at `path` to a labelled sibling such as
/// `save.unreadable-1750000000.json` and return where it went. Falls back to
/// copying if it can't be renamed.
pub fn preserve(path: &Path, label: &str, now: i64) -> io::Result<PathBuf> {
    let target = free_name(path, &format!("{label}-{now}"));
    match fs::rename(path, &target) {
        Ok(()) => Ok(target),
        Err(rename_err) => match fs::copy(path, &target) {
            Ok(_) => Ok(target),
            Err(_) => Err(rename_err),
        },
    }
}

/// `dir/save.json` + `tag` -> `dir/save.tag.json`.
fn sibling(path: &Path, tag: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "save".to_string());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_else(|| "json".to_string());
    path.with_file_name(format!("{stem}.{tag}.{ext}"))
}

fn free_name(path: &Path, tag: &str) -> PathBuf {
    let first = sibling(path, tag);
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| sibling(path, &format!("{tag}-{n}")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::Dice;
    use crate::rock::{Action, Personality};

    fn scratch(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "sedimentary-unit-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.join("save.json")
    }

    #[test]
    fn sibling_names_keep_the_extension() {
        let p = Path::new("/x/save.json");
        assert_eq!(sibling(p, "tmp"), Path::new("/x/save.tmp.json"));
        assert_eq!(
            sibling(p, "unreadable-5"),
            Path::new("/x/save.unreadable-5.json")
        );
    }

    #[test]
    fn store_then_load_round_trips() {
        let path = scratch("roundtrip");
        let mut dice = Dice::seeded(9);
        let mut rock = Rock::new("Cliff", Personality::Existential, 1_000, &mut dice);
        rock.perform(Action::Polish, 1_010, &mut dice);
        let data = SaveData::new(rock, Journal::default(), 1_020);
        store(&path, &data).unwrap();
        match load(&path, 2_000) {
            Loaded::Existing(loaded) => assert_eq!(*loaded, data),
            other => panic!("expected a save, got {other:?}"),
        }
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn rejects_saves_from_the_future() {
        let mut dice = Dice::seeded(1);
        let rock = Rock::new("Mica", Personality::Judgmental, 1, &mut dice);
        let mut data = SaveData::new(rock, Journal::default(), 1);
        data.format_version = FORMAT_VERSION + 1;
        let text = serde_json::to_string(&data).unwrap();
        assert!(parse(&text).unwrap_err().contains("newer version"));
        assert!(parse("   ").unwrap_err().contains("empty"));
    }

    #[test]
    fn missing_file_is_fresh() {
        let path = scratch("missing");
        assert!(matches!(load(&path, 1), Loaded::Fresh));
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
