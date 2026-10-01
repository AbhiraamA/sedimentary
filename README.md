# Sedimentary

A virtual pet rock for your terminal. It is deeply unimpressed with you.

```
cargo run --release
```

Needs stable Rust 1.88 or newer. Everything runs offline; there is no server and no AI model, just a rock and a large supply of opinions.

## Playing

On first launch you name your rock and pick a temperament:

- **Judgmental**: has standards. You do not meet them.
- **Existential**: has been staring into the void since the Precambrian.
- **Suspiciously Supportive**: believes in you. Has been taking notes.

Then look after it, or don't. It can't die. Neglect only makes it more sarcastic.

| Key | Action | | Key | Screen |
|---|---|---|---|---|
| `f` / `1` | Feed | | `j` | Field notes (journal) |
| `w` / `2` | Water | | `a` | Discoveries |
| `p` / `3` | Polish | | `?` / `h` | Help and the rules |
| `t` / `4` | Talk | | `n` | New rock (asks first) |
| `k` / `5` | Take for a walk | | `q` | Quit (saves) |
| `s` / `6` | Stare | | `Esc` | Close or go back |
| `r` / `7` | Rename | | `Ctrl+C` | Quit from anywhere (saves) |

The controls for the current screen are always shown at the bottom.

## How it works

The rock has four stats from 0 to 100: Moisture, Polish, Boredom and Disappointment. Every action shows exactly what it changed, and the help screen (`?`) lists each action's effects, generated from the same table the game uses.

- Over time it dries out, dulls, and gets bored. A bored rock grows more disappointed. Time alone never pushes a stat below 10 or above 85.
- Repeating an action gives half the boredom relief and adds disappointment. Watering a soaked rock or polishing a gleaming one backfires.
- Time away counts for at most 2 days, so a month off leaves it grumpy but fine. A clock that jumps backwards costs nothing.
- Occasionally it moves one character. This is treated as a historic event.
- There are 14 hidden discoveries.

## Your save

Saved after every action, every 30 seconds while open, and on quit.

| System | Location |
|---|---|
| Linux | `~/.local/share/sedimentary/save.json` (or `$XDG_DATA_HOME`) |
| macOS | `~/Library/Application Support/sedimentary/save.json` |
| Windows | `%APPDATA%\sedimentary\save.json` |

Run `sedimentary --save-path` to print yours. Set `SEDIMENTARY_SAVE=/some/file.json` to use another file (one per rock, if you're that kind of person).

Saves are written atomically. If a save can't be read, it is renamed to `save.unreadable-<time>.json` before a new rock is adopted, never deleted. Starting a new rock keeps the old one as `save.retired-<time>.json`.

## Development

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --example snapshot   # print screens as text at several sizes
```

The pet logic (`rock.rs`) knows nothing about text, terminals or files. Dialogue lives in `lines.rs` and is chosen by `responses.rs`. Persistence is `save.rs`, and the interface is `art.rs` plus `ui.rs`, glued together by `app.rs`.
