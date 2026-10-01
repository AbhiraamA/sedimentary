//! The rock, drawn in text. Produces a grid of tagged characters; the UI
//! decides what colour each tag gets. No terminal code in here.

use crate::rock::{Action, Formation, Personality};

/// Width and height of the rock itself.
pub const ROCK_WIDTH: usize = 22;
pub const ROCK_HEIGHT: usize = 7;
/// Room around the rock for sparkles, moss, drips and snoring.
const PAD_X: usize = 4;
pub const CANVAS_WIDTH: usize = ROCK_WIDTH + 2 * PAD_X;
pub const CANVAS_HEIGHT: usize = ROCK_HEIGHT + 2;

// `E` marks the eyes, `MMMM` the mouth. Every row is exactly 22 columns.
const TEMPLATE: [&str; ROCK_HEIGHT] = [
    "      _.-~~~~~-._     ",
    "   .-'           '-.  ",
    "  /    E       E    \\ ",
    " |                   |",
    " |       MMMM       / ",
    "  \\.              .'  ",
    "    '-..______..-'    ",
];

/// What a character is, so the UI can colour it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Blank,
    Outline,
    Texture,
    Eye,
    Mouth,
    Sparkle,
    Water,
    Moss,
    Dust,
    Crack,
    Snore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub tone: Tone,
}

impl Cell {
    const BLANK: Cell = Cell {
        ch: ' ',
        tone: Tone::Blank,
    };
}

/// Everything that changes how the rock looks.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub personality: Personality,
    pub formation: Formation,
    /// 0..=3, from [`crate::rock::Mood`].
    pub sarcasm: u8,
    /// A brief reaction to the last thing you did.
    pub expression: Option<Action>,
    pub moisture: f64,
    pub polish: f64,
    pub boredom: f64,
    /// Animation counter (a few ticks per second).
    pub frame: u64,
}

impl Look {
    /// A freshly adopted rock, for previews.
    pub fn preview(personality: Personality, frame: u64) -> Self {
        Self {
            personality,
            formation: Formation::Sedimentary,
            sarcasm: 0,
            expression: None,
            moisture: 50.0,
            polish: 50.0,
            boredom: 30.0,
            frame,
        }
    }
}

/// Two eyes and a four-character mouth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face {
    pub left: char,
    pub right: char,
    pub mouth: [char; 4],
}

fn mouth(s: &str) -> [char; 4] {
    let mut out = [' '; 4];
    for (slot, ch) in out.iter_mut().zip(s.chars()) {
        *slot = ch;
    }
    out
}

fn face_of(left: char, right: char, m: &str) -> Face {
    Face {
        left,
        right,
        mouth: mouth(m),
    }
}

/// The face for a personality at a given sarcasm level, optionally reacting
/// to an action. Suspiciously Supportive rocks never stop smiling.
pub fn face(personality: Personality, sarcasm: u8, expression: Option<Action>) -> Face {
    let resting = match (personality, sarcasm.min(3)) {
        (Personality::Judgmental, 0) => face_of('o', 'o', " -- "),
        (Personality::Judgmental, 1) => face_of('-', '-', " -- "),
        (Personality::Judgmental, 2) => face_of('\\', '/', "____"),
        (Personality::Judgmental, _) => face_of('\\', '/', ".--."),
        (Personality::Existential, 0) => face_of('o', 'o', " .. "),
        (Personality::Existential, 1) => face_of('O', 'O', " -- "),
        (Personality::Existential, 2) => face_of('.', '.', " -- "),
        (Personality::Existential, _) => face_of('.', '.', ".--."),
        (Personality::SuspiciouslySupportive, 0 | 1) => face_of('^', '^', "\\__/"),
        (Personality::SuspiciouslySupportive, 2) => face_of('o', 'o', "\\__/"),
        (Personality::SuspiciouslySupportive, _) => face_of('O', 'O', "\\__/"),
    };
    let Some(action) = expression else {
        return resting;
    };
    let reacting = match action {
        Action::Feed => face_of('=', '=', " -- "),
        Action::Water => face_of('>', '<', " ~~ "),
        Action::Polish => face_of('^', '^', "\\__/"),
        Action::Talk => face_of('-', 'o', " -- "),
        Action::Walk => face_of('@', '@', " ~~ "),
        Action::Stare => face_of('O', 'O', "____"),
        Action::Rename => face_of('o', 'O', "  o "),
    };
    if personality == Personality::SuspiciouslySupportive {
        Face {
            mouth: resting.mouth,
            ..reacting
        }
    } else {
        reacting
    }
}

struct Grid {
    rows: Vec<Vec<Cell>>,
}

impl Grid {
    fn new() -> Self {
        Self {
            rows: vec![vec![Cell::BLANK; CANVAS_WIDTH]; CANVAS_HEIGHT],
        }
    }

    fn put(&mut self, row: usize, col: usize, ch: char, tone: Tone) {
        if let Some(cell) = self.rows.get_mut(row).and_then(|r| r.get_mut(col)) {
            *cell = Cell { ch, tone };
        }
    }

    /// Only draw on empty space, never over the rock.
    fn put_if_blank(&mut self, row: usize, col: usize, ch: char, tone: Tone) {
        if let Some(cell) = self.rows.get_mut(row).and_then(|r| r.get_mut(col))
            && cell.tone == Tone::Blank
        {
            *cell = Cell { ch, tone };
        }
    }

    /// Place on the rock's own coordinates (row/col inside the template).
    fn put_rock(&mut self, row: usize, col: usize, ch: char, tone: Tone) {
        self.put(row + 1, col + PAD_X, ch, tone);
    }
}

/// Mineral flecks that hint at how the rock formed.
fn texture(formation: Formation) -> [(usize, usize, char); 3] {
    match formation {
        Formation::Igneous => [(1, 9, '*'), (3, 4, '.'), (5, 13, '*')],
        Formation::Sedimentary => [(1, 9, '='), (3, 16, '-'), (5, 6, '=')],
        Formation::Metamorphic => [(1, 9, '~'), (3, 4, '~'), (5, 13, '~')],
    }
}

/// Draw the rock and its surroundings. Every row is [`CANVAS_WIDTH`] wide.
pub fn render(look: &Look) -> Vec<Vec<Cell>> {
    let mut grid = Grid::new();

    for (r, line) in TEMPLATE.iter().enumerate() {
        for (c, ch) in line.chars().enumerate() {
            if !matches!(ch, ' ' | 'E' | 'M') {
                grid.put_rock(r, c, ch, Tone::Outline);
            }
        }
    }

    for (r, c, ch) in texture(look.formation) {
        grid.put_rock(r, c, ch, Tone::Texture);
    }

    // Faces blink every few seconds unless they're busy reacting.
    let mut f = face(look.personality, look.sarcasm, look.expression);
    if look.expression.is_none() && look.frame.is_multiple_of(23) {
        f.left = '-';
        f.right = '-';
    }
    grid.put_rock(2, 7, f.left, Tone::Eye);
    grid.put_rock(2, 15, f.right, Tone::Eye);
    for (i, ch) in f.mouth.iter().enumerate() {
        if *ch != ' ' {
            grid.put_rock(4, 9 + i, *ch, Tone::Mouth);
        }
    }

    decorate(&mut grid, look);
    grid.rows
}

fn decorate(grid: &mut Grid, look: &Look) {
    let twinkle = (look.frame / 2).is_multiple_of(2);
    let bottom = CANVAS_HEIGHT - 1;

    if look.polish >= 70.0 {
        let (a, b) = if twinkle { ('+', '\'') } else { ('\'', '+') };
        grid.put_if_blank(1, 6, a, Tone::Sparkle);
        grid.put_if_blank(3, CANVAS_WIDTH - 4, b, Tone::Sparkle);
        grid.put_if_blank(0, CANVAS_WIDTH - 9, b, Tone::Sparkle);
        if look.polish >= 95.0 {
            grid.put_if_blank(6, 2, a, Tone::Sparkle);
            grid.put_rock(1, 13, '`', Tone::Sparkle);
        }
    } else if look.polish < 25.0 {
        for (row, col, ch) in [(2, 3, ','), (4, 2, '.'), (5, CANVAS_WIDTH - 5, '`')] {
            grid.put_if_blank(row, col, ch, Tone::Dust);
        }
        grid.put_rock(3, 12, '.', Tone::Dust);
        grid.put_rock(5, 15, ',', Tone::Dust);
    }

    if look.moisture >= 90.0 {
        for col in PAD_X..PAD_X + ROCK_WIDTH {
            let ch = if (col + look.frame as usize).is_multiple_of(3) {
                '-'
            } else {
                '~'
            };
            grid.put_if_blank(bottom, col, ch, Tone::Water);
        }
        grid.put_if_blank(5, PAD_X + ROCK_WIDTH - 1, '\'', Tone::Water);
    } else if look.moisture >= 70.0 {
        let drip = if twinkle { '\'' } else { ',' };
        grid.put_if_blank(4, PAD_X + ROCK_WIDTH - 1, drip, Tone::Water);
        grid.put_if_blank(bottom, PAD_X + 5, '.', Tone::Water);
        grid.put_if_blank(bottom, PAD_X + 16, ',', Tone::Water);
    } else if look.moisture <= 15.0 {
        grid.put_rock(5, 11, '\\', Tone::Crack);
        grid.put_rock(5, 12, '/', Tone::Crack);
        grid.put_rock(3, 17, '/', Tone::Crack);
    }

    if look.moisture >= 75.0 && look.polish < 40.0 {
        for (i, ch) in ",;,.,;,".chars().enumerate() {
            grid.put_if_blank(0, PAD_X + 8 + i, ch, Tone::Moss);
        }
        grid.put_rock(0, 7, ';', Tone::Moss);
    }

    if look.boredom >= 75.0 {
        let drift = (look.frame / 3 % 3) as usize;
        grid.put_if_blank(0, CANVAS_WIDTH - 7 + drift, 'z', Tone::Snore);
        grid.put_if_blank(0, CANVAS_WIDTH - 5 + drift, 'Z', Tone::Snore);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look() -> Look {
        Look::preview(Personality::Judgmental, 1)
    }

    fn has(look: &Look, tone: Tone) -> bool {
        render(look).iter().flatten().any(|c| c.tone == tone)
    }

    #[test]
    fn template_rows_are_equal_width() {
        for row in TEMPLATE {
            assert_eq!(row.chars().count(), ROCK_WIDTH, "{row:?}");
        }
    }

    #[test]
    fn every_variant_renders_a_full_rectangle() {
        let conditions = [(0.0, 0.0, 0.0), (100.0, 100.0, 100.0), (80.0, 10.0, 90.0)];
        for personality in Personality::ALL {
            for formation in Formation::ALL {
                for sarcasm in 0..=3 {
                    for expression in [None, Some(Action::Water), Some(Action::Walk)] {
                        for (moisture, polish, boredom) in conditions {
                            for frame in 0..7 {
                                let canvas = render(&Look {
                                    personality,
                                    formation,
                                    sarcasm,
                                    expression,
                                    moisture,
                                    polish,
                                    boredom,
                                    frame,
                                });
                                assert_eq!(canvas.len(), CANVAS_HEIGHT);
                                assert!(canvas.iter().all(|r| r.len() == CANVAS_WIDTH));
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn eyes_land_where_the_template_says() {
        let canvas = render(&look());
        assert_eq!(canvas[3][PAD_X + 7].tone, Tone::Eye);
        assert_eq!(canvas[3][PAD_X + 15].tone, Tone::Eye);
    }

    #[test]
    fn conditions_show_up_in_the_picture() {
        let base = look();
        assert!(!has(&base, Tone::Water));
        let soaked = Look {
            moisture: 95.0,
            ..base
        };
        assert!(has(&soaked, Tone::Water));
        let parched = Look {
            moisture: 5.0,
            ..base
        };
        assert!(has(&parched, Tone::Crack));
        assert!(has(
            &Look {
                polish: 99.0,
                ..base
            },
            Tone::Sparkle
        ));
        assert!(has(
            &Look {
                polish: 5.0,
                ..base
            },
            Tone::Dust
        ));
        assert!(has(
            &Look {
                boredom: 90.0,
                ..base
            },
            Tone::Snore
        ));
        let mossy = Look {
            moisture: 80.0,
            polish: 10.0,
            ..base
        };
        assert!(has(&mossy, Tone::Moss));
    }

    #[test]
    fn supportive_rocks_keep_smiling() {
        for action in Action::ALL {
            let f = face(Personality::SuspiciouslySupportive, 3, Some(action));
            assert_eq!(f.mouth, mouth("\\__/"));
        }
    }
}
