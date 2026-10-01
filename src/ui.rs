//! The terminal interface. Reads [`App`] and draws it; never changes the game.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph};

use crate::achievements::Achievement;
use crate::app::{App, Screen, SetupStep};
use crate::art::{self, CANVAS_WIDTH, Look, Tone};
use crate::clock::{duration_words, short_ago};
use crate::journal::{Entry, EntryKind};
use crate::rock::{
    Action, BOREDOM_DRIFT, DISAPPOINTMENT_EASE, DISAPPOINTMENT_RISE, DRIFT_CEILING, DRIFT_FLOOR,
    MOISTURE_DRIFT, OFFLINE_CAP_SECS, OVERPOLISH_AT, OVERWATER_AT, POLISH_DRIFT, Personality,
    REPEAT_PENALTY, Rock, STAT_MAX, Stat,
};

pub const MIN_WIDTH: u16 = 50;
pub const MIN_HEIGHT: u16 = 24;
/// Border, one row of ground, the rock itself, and at least one line of thought.
const HABITAT_MIN: u16 = 11;
/// From this width up, the condition and field notes get their own column.
const WIDE_AT: u16 = 80;
/// How long stat changes stay highlighted next to the bars.
const DELTA_SECS: i64 = 10;

/// An earthy 256-colour palette.
mod pal {
    use ratatui::style::Color;

    pub const OCHRE: Color = Color::Indexed(179);
    pub const SAND: Color = Color::Indexed(180);
    pub const CLAY: Color = Color::Indexed(137);
    pub const SHALE: Color = Color::Indexed(243);
    pub const STONE: Color = Color::Indexed(250);
    pub const BRIGHT: Color = Color::Indexed(255);
    pub const DIM: Color = Color::Indexed(245);
    pub const FAINT: Color = Color::Indexed(240);
    pub const DEEP: Color = Color::Indexed(238);
    pub const WATER: Color = Color::Indexed(74);
    pub const GLINT: Color = Color::Indexed(222);
    pub const LICHEN: Color = Color::Indexed(139);
    pub const RUST: Color = Color::Indexed(167);
    pub const MOSS: Color = Color::Indexed(107);
    pub const ORCHID: Color = Color::Indexed(176);
}

fn fg(color: Color) -> Style {
    Style::new().fg(color)
}

fn voice_color(p: Personality) -> Color {
    match p {
        Personality::Judgmental => Color::Indexed(187),
        Personality::Existential => Color::Indexed(146),
        Personality::SuspiciouslySupportive => Color::Indexed(218),
    }
}

/// What the thought bubble says the rock is doing.
fn musing(p: Personality) -> &'static str {
    match p {
        Personality::Judgmental => "silently judging",
        Personality::Existential => "pondering the void",
        Personality::SuspiciouslySupportive => "rooting for you",
    }
}

fn stat_color(s: Stat) -> Color {
    match s {
        Stat::Moisture => pal::WATER,
        Stat::Polish => pal::GLINT,
        Stat::Boredom => pal::LICHEN,
        Stat::Disappointment => pal::RUST,
    }
}

/// Whether a change in this direction is good news for the rock.
fn is_good(stat: Stat, delta: f64) -> bool {
    match stat {
        Stat::Moisture | Stat::Polish => delta > 0.0,
        Stat::Boredom | Stat::Disappointment => delta < 0.0,
    }
}

fn mood_color(sarcasm: u8) -> Color {
    match sarcasm {
        0 => pal::MOSS,
        1 => pal::OCHRE,
        2 => Color::Indexed(173),
        _ => pal::RUST,
    }
}

fn tone_style(tone: Tone, rock_color: Color, voice: Color) -> Style {
    match tone {
        Tone::Blank => Style::new(),
        Tone::Outline => fg(rock_color),
        Tone::Texture => fg(pal::SHALE),
        Tone::Eye => fg(pal::BRIGHT).add_modifier(Modifier::BOLD),
        Tone::Mouth => fg(voice),
        Tone::Sparkle => fg(pal::GLINT).add_modifier(Modifier::BOLD),
        Tone::Water => fg(pal::WATER),
        Tone::Moss => fg(pal::MOSS),
        Tone::Dust => fg(pal::CLAY),
        Tone::Crack => fg(pal::RUST),
        Tone::Snore => fg(pal::LICHEN),
    }
}

fn short_label(action: Action) -> &'static str {
    match action {
        Action::Feed => "feed",
        Action::Water => "water",
        Action::Polish => "polish",
        Action::Talk => "talk",
        Action::Walk => "walk",
        Action::Stare => "stare",
        Action::Rename => "rename",
    }
}

fn to_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

fn width_of(text: &str) -> usize {
    Span::raw(text).width()
}

/// Word-wrap to `width` columns. Long words are split.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_w = 0;
    for word in text.split_whitespace() {
        let w = width_of(word);
        if current_w > 0 && current_w + 1 + w > width {
            lines.push(std::mem::take(&mut current));
            current_w = 0;
        }
        if w > width {
            for ch in word.chars() {
                let cw = width_of(ch.encode_utf8(&mut [0; 4]));
                if current_w + cw > width && current_w > 0 {
                    lines.push(std::mem::take(&mut current));
                    current_w = 0;
                }
                current.push(ch);
                current_w += cw;
            }
            continue;
        }
        if current_w > 0 {
            current.push(' ');
            current_w += 1;
        }
        current.push_str(word);
        current_w += w;
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

fn wrapped(text: &str, width: usize, style: Style) -> Vec<Line<'static>> {
    wrap(text, width)
        .into_iter()
        .map(|l| Line::from(Span::styled(l, style)))
        .collect()
}

fn with_ellipsis_if_needed(text: &str, width: usize) -> String {
    if width_of(text) <= width {
        text.to_string()
    } else {
        with_ellipsis(text, width)
    }
}

/// Mark a line as cut short, keeping it within `width`.
fn with_ellipsis(line: &str, width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for ch in line.chars() {
        let w = width_of(ch.encode_utf8(&mut [0; 4]));
        if used + w + 1 > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    format!("{}…", out.trim_end())
}

fn signed(delta: f64) -> String {
    if (delta - delta.round()).abs() < 0.05 {
        format!("{delta:+.0}")
    } else {
        format!("{delta:+.1}")
    }
}

fn age_phrase(myr: u32) -> String {
    if myr >= 1_000 {
        format!("about {:.1} billion years old", f64::from(myr) / 1_000.0)
    } else {
        format!("about {myr} million years old")
    }
}

fn adopted_phrase(rock: &Rock, now: i64) -> String {
    let secs = rock.age_secs(now);
    if secs < 60 {
        "adopted just now".to_string()
    } else {
        format!("yours for {}", duration_words(secs))
    }
}

fn displacement(rock: &Rock) -> String {
    let n = rock.position.unsigned_abs();
    let side = if rock.position < 0 { "left" } else { "right" };
    match (n, rock.counters.moves) {
        (0, 0) => "has not moved. yet.".to_string(),
        (0, _) => "back where it started, somehow".to_string(),
        (1, _) => format!("1 character {side} of where it started"),
        _ => format!("{n} characters {side} of where it started"),
    }
}

fn panel(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(pal::FAINT))
        .title(Span::styled(format!(" {title} "), fg(pal::OCHRE)))
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

fn put(buf: &mut Buffer, bounds: Rect, x: i32, y: i32, ch: char, style: Style) {
    let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
        return;
    };
    if x < bounds.x || x >= bounds.right() || y < bounds.y || y >= bounds.bottom() {
        return;
    }
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch).set_style(style);
    }
}

// ---------------------------------------------------------------- controls

type Controls = Vec<Vec<(&'static str, &'static str)>>;

fn controls(app: &App, narrow: bool) -> Controls {
    match &app.screen {
        Screen::Setup {
            step: SetupStep::Name,
            ..
        } => vec![
            vec![
                ("type", "a name"),
                ("Tab", "suggest"),
                ("Enter", "continue"),
            ],
            vec![("Esc", if app.rock.is_some() { "back" } else { "quit" })],
        ],
        Screen::Setup { .. } => vec![
            vec![("↑↓", "choose"), ("1-3", "pick"), ("Enter", "adopt")],
            vec![("Esc", "back")],
        ],
        Screen::Main => {
            let actions = Action::ALL
                .iter()
                .map(|a| (key_label(*a), short_label(*a)))
                .collect();
            let rest = if narrow {
                vec![
                    ("j", "notes"),
                    ("a", "finds"),
                    ("?", "help"),
                    ("n", "new"),
                    ("q", "quit"),
                ]
            } else {
                vec![
                    ("j", "field notes"),
                    ("a", "discoveries"),
                    ("?", "help"),
                    ("n", "new rock"),
                    ("q", "quit"),
                ]
            };
            vec![actions, rest]
        }
        Screen::Rename { .. } => vec![
            vec![("type", "a name"), ("Tab", "suggest"), ("Enter", "rename")],
            vec![("Esc", "cancel")],
        ],
        Screen::ConfirmNew => vec![vec![("y", "retire it, start over"), ("n", "keep it")]],
        Screen::Journal { .. } | Screen::Discoveries { .. } | Screen::Help { .. } => vec![
            vec![("↑↓", "scroll"), ("PgUp PgDn", "page")],
            vec![("Esc", "close")],
        ],
    }
}

fn key_label(action: Action) -> &'static str {
    match action {
        Action::Feed => "f",
        Action::Water => "w",
        Action::Polish => "p",
        Action::Talk => "t",
        Action::Walk => "k",
        Action::Stare => "s",
        Action::Rename => "r",
    }
}

/// Pack control hints into as few lines as fit.
fn control_lines(groups: &Controls, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Vec<Span<'static>>> = vec![Vec::new()];
    let mut used = 0;
    for (g, group) in groups.iter().enumerate() {
        for (i, (key, label)) in group.iter().enumerate() {
            let gap = match (used, i, g) {
                (0, _, _) => 0,
                (_, 0, g) if g > 0 => 5,
                _ => 2,
            };
            let item_w = width_of(key) + 1 + width_of(label);
            if used > 0 && used + gap + item_w > width {
                lines.push(Vec::new());
                used = 0;
            }
            let line = lines.last_mut().expect("at least one line");
            if used > 0 {
                line.push(Span::raw(" ".repeat(gap)));
                used += gap;
            }
            line.push(Span::styled(
                *key,
                fg(pal::OCHRE).add_modifier(Modifier::BOLD),
            ));
            line.push(Span::styled(format!(" {label}"), fg(pal::DIM)));
            used += item_w;
        }
    }
    lines.into_iter().map(Line::from).collect()
}

// ---------------------------------------------------------------- entry

/// Draw the whole interface.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small(frame, area);
        return;
    }
    let narrow = area.width < WIDE_AT;
    let hints = control_lines(&controls(app, narrow), usize::from(area.width) - 2);
    let [body, footer] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(to_u16(hints.len()))])
            .areas(area);
    frame.render_widget(
        Paragraph::new(hints),
        Rect::new(footer.x + 1, footer.y, footer.width - 1, footer.height),
    );

    if let Screen::Setup {
        step,
        name,
        selected,
        suggestion,
    } = &app.screen
    {
        draw_setup(frame, body, app, *step, name, *selected, suggestion);
        return;
    }
    let Some(rock) = &app.rock else {
        return;
    };
    draw_main(frame, body, app, rock, narrow);

    match &app.screen {
        Screen::Rename { input, suggestion } => {
            draw_rename(frame, body, app, rock, input, suggestion)
        }
        Screen::ConfirmNew => draw_confirm(frame, body, rock),
        Screen::Journal { scroll } => {
            let width = usize::from(body.width.min(84)).saturating_sub(6);
            let lines = journal_lines(app, rock, width, usize::MAX, true);
            let title = format!("Field notes ({} entries)", app.journal.len());
            draw_scroller(frame, body, app, &title, lines, *scroll);
        }
        Screen::Discoveries { scroll } => {
            let width = usize::from(body.width.min(84)).saturating_sub(6);
            draw_scroller(
                frame,
                body,
                app,
                "Discoveries",
                discovery_lines(app, rock, width),
                *scroll,
            );
        }
        Screen::Help { scroll } => {
            let width = usize::from(body.width.min(84)).saturating_sub(6);
            draw_scroller(
                frame,
                body,
                app,
                "How this works",
                help_lines(app, width),
                *scroll,
            );
        }
        _ => {}
    }
}

fn draw_too_small(frame: &mut Frame, area: Rect) {
    let text = format!(
        "Sedimentary needs at least {MIN_WIDTH}x{MIN_HEIGHT}. This is {}x{}. \
         Your rock refuses to be cropped.",
        area.width, area.height
    );
    let lines = wrapped(
        &text,
        usize::from(area.width.saturating_sub(2)),
        fg(pal::SAND),
    );
    let height = to_u16(lines.len());
    let rect = centered(area, area.width.saturating_sub(2), height);
    frame.render_widget(Paragraph::new(lines).centered(), rect);
}

// ---------------------------------------------------------------- main view

fn draw_main(frame: &mut Frame, area: Rect, app: &App, rock: &Rock, narrow: bool) {
    let latest_width = usize::from(area.width).saturating_sub(4);
    let latest = latest_lines(app, rock, latest_width);
    let condition_h = match (narrow, area.height >= 30) {
        (true, true) => 4,
        (true, false) => 2,
        (false, _) => 0,
    };
    // The response gets whatever the habitat can spare, but never most of the screen.
    let spare = area
        .height
        .saturating_sub(2 + condition_h + HABITAT_MIN)
        .min(area.height / 2);
    let latest_h = (to_u16(latest.len()) + 2).clamp(4, spare.max(4));

    let [header, body, latest_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(latest_h),
    ])
    .areas(area);

    draw_header(frame, header, app, rock);
    draw_latest(frame, latest_area, app, latest);

    if narrow {
        let bordered = condition_h == 4;
        let [habitat, condition] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(condition_h)]).areas(body);
        draw_habitat(frame, habitat, app, rock);
        draw_condition_compact(frame, condition, app, rock, bordered);
    } else {
        let side_w = (area.width * 2 / 5).clamp(32, 46);
        let [habitat, side] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(side_w)]).areas(body);
        draw_habitat(frame, habitat, app, rock);
        let condition_h = match side.height {
            h if h >= 30 => 18,
            h if h >= 20 => 14,
            _ => 10,
        };
        if side.height >= condition_h + 4 {
            let [condition, notes] =
                Layout::vertical([Constraint::Length(condition_h), Constraint::Fill(1)])
                    .areas(side);
            draw_condition(frame, condition, app, rock);
            draw_notes(frame, notes, app, rock);
        } else {
            draw_condition(frame, side, app, rock);
        }
    }
}

fn split_line(frame: &mut Frame, area: Rect, left: Line<'static>, right: Line<'static>) {
    let fits = left.width() + right.width() + 2 <= usize::from(area.width);
    frame.render_widget(Paragraph::new(left), area);
    if fits {
        frame.render_widget(Paragraph::new(right).right_aligned(), area);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, rock: &Rock) {
    let area = Rect::new(
        area.x + 1,
        area.y,
        area.width.saturating_sub(2),
        area.height,
    );
    let [top, bottom] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
    let width = usize::from(area.width);
    let voice = voice_color(rock.personality);
    let mood = rock.mood(app.now);
    let mood_line = Line::from(Span::styled(mood.label(), fg(mood_color(mood.sarcasm))));

    // Longest description that fits, from chatty to terse.
    let personality = rock.personality.label().to_lowercase();
    let formation = rock.formation.label();
    let age = age_phrase(rock.age_myr);
    let terse_age = age.trim_start_matches("about ").to_string();
    let candidates = [
        ("a ", format!(" {formation} rock, {age}")),
        ("", format!(" {formation} rock, {terse_age}")),
        ("", format!(", {terse_age}")),
        ("", String::new()),
    ];
    let describe = |(prefix, rest): &(&str, String)| {
        Line::from(vec![
            Span::styled(prefix.to_string(), fg(pal::DIM)),
            Span::styled(personality.clone(), fg(voice)),
            Span::styled(rest.clone(), fg(pal::DIM)),
        ])
    };
    let with_mood = candidates
        .iter()
        .map(describe)
        .find(|line| line.width() + mood_line.width() + 2 <= width);

    let name = Line::from(Span::styled(
        rock.name.clone(),
        fg(pal::SAND).add_modifier(Modifier::BOLD),
    ));
    let status = if !app.saving_enabled {
        Line::from(Span::styled("not saving this session", fg(pal::RUST)))
    } else if with_mood.is_some() {
        Line::from(Span::styled(adopted_phrase(rock, app.now), fg(pal::DIM)))
    } else {
        mood_line.clone()
    };
    split_line(frame, top, name, status);

    match with_mood {
        Some(about) => split_line(frame, bottom, about, mood_line),
        None => {
            let about = candidates
                .iter()
                .map(describe)
                .find(|line| line.width() <= width)
                .unwrap_or_else(|| describe(&candidates[3]));
            frame.render_widget(Paragraph::new(about), bottom);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BubbleStyle {
    Boxed,
    Plain { tail: bool },
}

fn draw_habitat(frame: &mut Frame, area: Rect, app: &App, rock: &Rock) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(pal::FAINT))
        .title(Span::styled(" Sedimentary ", fg(pal::OCHRE)))
        .title_bottom(
            Line::from(Span::styled(
                format!(" {} ", displacement(rock)),
                fg(pal::DIM),
            ))
            .right_aligned(),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 8 || inner.height < 4 {
        return;
    }

    let height = usize::from(inner.height);
    let ground = match height {
        h if h >= 18 => 3,
        h if h >= 14 => 2,
        _ => 1,
    };
    let avail = height - ground;
    let text_w = usize::from(inner.width).saturating_sub(6).clamp(10, 52);
    let thought = wrap(&app.thought, text_w);
    let n = thought.len();

    // Prefer a boxed bubble and the full picture; give things up as space runs out.
    let plan = [8, 7]
        .into_iter()
        .find(|rows| avail >= rows + n + 3)
        .map(|rows| (rows, BubbleStyle::Boxed, n))
        .or_else(|| {
            [8, 7]
                .into_iter()
                .find(|rows| avail > rows + n)
                .map(|rows| (rows, BubbleStyle::Plain { tail: true }, n))
        })
        .unwrap_or_else(|| {
            let rows = avail.min(7);
            (rows, BubbleStyle::Plain { tail: false }, avail - rows)
        });
    let (canvas_rows, style, shown) = plan;

    let ground_top = i32::from(inner.bottom()) - ground as i32;
    let base_x = i32::from(inner.x) + (i32::from(inner.width) - CANVAS_WIDTH as i32) / 2;
    let max_x = (i32::from(inner.right()) - CANVAS_WIDTH as i32).max(i32::from(inner.x));
    let rock_x = (base_x + rock.position).clamp(i32::from(inner.x), max_x);
    let center = rock_x + 15;

    let look = Look {
        personality: rock.personality,
        formation: rock.formation,
        sarcasm: rock.mood(app.now).sarcasm,
        expression: app.expression(),
        moisture: rock.stats.moisture,
        polish: rock.stats.polish,
        boredom: rock.stats.boredom,
        frame: app.frame,
    };
    let canvas = art::render(&look);
    let rock_color = match rock.stats.polish {
        p if p >= 70.0 => pal::BRIGHT,
        p if p < 25.0 => pal::SHALE,
        _ => pal::STONE,
    };
    let voice = voice_color(rock.personality);
    let buf = frame.buffer_mut();

    // Ground: a little cross-section of strata, with a fossil if there's room.
    for g in 0..ground {
        let y = ground_top + g as i32;
        for x in inner.x..inner.right() {
            let col = usize::from(x - inner.x);
            let speckle = (col * 7 + g * 3).is_multiple_of(11);
            let (ch, color) = match g {
                0 => ('▀', pal::SAND),
                1 if speckle => ('▓', pal::CLAY),
                1 => ('▒', pal::CLAY),
                _ if speckle => ('▒', pal::DEEP),
                _ => ('░', pal::DEEP),
            };
            put(buf, inner, i32::from(x), y, ch, fg(color));
        }
    }
    if ground >= 2 {
        let fossil_x = i32::from(inner.x) + i32::from(inner.width) * 4 / 5;
        put(buf, inner, fossil_x, ground_top + 1, '@', fg(pal::OCHRE));
        if rock.position != 0 {
            put(buf, inner, base_x + 15, ground_top + 1, 'x', fg(pal::GLINT));
        }
    }

    // The rock. Art rows 0..8 sit above the ground; row 8 is drawn on the surface.
    let first_row = 8 - canvas_rows;
    for (r, row) in canvas.iter().enumerate() {
        if r < first_row {
            continue;
        }
        let y = ground_top - 8 + r as i32;
        for (c, cell) in row.iter().enumerate() {
            if cell.tone != Tone::Blank {
                put(
                    buf,
                    inner,
                    rock_x + c as i32,
                    y,
                    cell.ch,
                    tone_style(cell.tone, rock_color, voice),
                );
            }
        }
    }

    // The thought, just above the rock.
    if shown == 0 {
        return;
    }
    let canvas_top = ground_top - canvas_rows as i32;
    let mut lines = thought[..shown.min(n)].to_vec();
    if shown < n
        && let Some(last) = lines.last_mut()
    {
        *last = with_ellipsis(last, text_w);
    }
    let widest = lines.iter().map(|l| width_of(l)).max().unwrap_or(0);
    let title = format!(" {} ", musing(rock.personality));
    match style {
        BubbleStyle::Boxed => {
            let bubble_w = (widest.max(width_of(&title) + 2) + 4).min(usize::from(inner.width));
            let bubble_h = shown + 2;
            let top = canvas_top - 1 - bubble_h as i32;
            let left = (center - bubble_w as i32 / 2).clamp(
                i32::from(inner.x),
                i32::from(inner.right()) - bubble_w as i32,
            );
            let rect = Rect::new(
                to_u16(left.max(0) as usize),
                to_u16(top.max(0) as usize),
                to_u16(bubble_w),
                to_u16(bubble_h),
            )
            .intersection(inner);
            let bubble = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(fg(pal::SHALE))
                .title(Span::styled(
                    title,
                    fg(voice).add_modifier(Modifier::ITALIC),
                ));
            let text: Vec<Line> = lines
                .iter()
                .map(|l| Line::from(Span::styled(l.clone(), fg(pal::STONE))))
                .collect();
            let text_rect = bubble.inner(rect);
            frame.render_widget(bubble, rect);
            let padded = Rect::new(
                text_rect.x + 1,
                text_rect.y,
                text_rect.width.saturating_sub(1),
                text_rect.height,
            );
            frame.render_widget(Paragraph::new(text), padded);
            put(
                frame.buffer_mut(),
                inner,
                center + 3,
                canvas_top - 1,
                'o',
                fg(pal::SHALE),
            );
        }
        BubbleStyle::Plain { tail } => {
            let tail_rows = i32::from(tail);
            let top = canvas_top - tail_rows - shown as i32;
            let buf = frame.buffer_mut();
            for (i, line) in lines.iter().enumerate() {
                let w = width_of(line) as i32;
                let left = (center - w / 2).clamp(i32::from(inner.x), i32::from(inner.right()) - w);
                let y = top + i as i32;
                if let (Ok(x), Ok(y)) = (u16::try_from(left), u16::try_from(y))
                    && y >= inner.y
                    && y < inner.bottom()
                {
                    buf.set_stringn(
                        x,
                        y,
                        line,
                        usize::from(inner.right().saturating_sub(x)),
                        fg(voice).add_modifier(Modifier::ITALIC),
                    );
                }
            }
            if tail {
                put(buf, inner, center + 3, canvas_top - 1, 'o', fg(pal::SHALE));
            }
        }
    }
}

fn recent_delta(app: &App, stat: Stat) -> Option<f64> {
    let banner = app.banner.as_ref()?;
    if app.now - banner.at > DELTA_SECS {
        return None;
    }
    banner
        .changes
        .iter()
        .find(|(s, _)| *s == stat)
        .map(|(_, d)| *d)
}

fn bar(value: f64, width: usize, color: Color) -> Vec<Span<'static>> {
    let filled = ((value / STAT_MAX) * width as f64).round() as usize;
    let filled = filled.min(width);
    vec![
        Span::styled("━".repeat(filled), fg(color)),
        Span::styled("─".repeat(width - filled), fg(pal::DEEP)),
    ]
}

fn delta_span(app: &App, stat: Stat) -> Span<'static> {
    match recent_delta(app, stat) {
        Some(d) => {
            let color = if is_good(stat, d) {
                pal::MOSS
            } else {
                pal::RUST
            };
            Span::styled(format!(" {:>4}", signed(d)), fg(color))
        }
        None => Span::raw("     "),
    }
}

/// Rows the condition panel wants per stat, given its inner height.
fn rows_per_stat(inner_height: u16) -> usize {
    match inner_height {
        h if h >= 16 => 4,
        h if h >= 12 => 3,
        _ => 2,
    }
}

fn draw_condition(frame: &mut Frame, area: Rect, app: &App, rock: &Rock) {
    let block = panel("Condition");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let inner = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    );
    let per_stat = rows_per_stat(inner.height);
    let width = usize::from(inner.width);
    let mut lines = Vec::new();
    for stat in Stat::ALL {
        let value = rock.stats.get(stat);
        let color = stat_color(stat);
        let label = Span::styled(stat.label(), fg(color).add_modifier(Modifier::BOLD));
        let describe = stat.describe(value);
        if per_stat >= 3 {
            // Label and number, a full-width bar, then the verdict underneath.
            let number = format!("{value:.0}");
            let delta = match recent_delta(app, stat) {
                Some(d) => {
                    let color = if is_good(stat, d) {
                        pal::MOSS
                    } else {
                        pal::RUST
                    };
                    Span::styled(format!("  {}", signed(d)), fg(color))
                }
                None => Span::raw(""),
            };
            let pad = width.saturating_sub(stat.label().len() + number.len() + delta.width());
            lines.push(Line::from(vec![
                label,
                Span::raw(" ".repeat(pad)),
                Span::styled(number, fg(pal::STONE)),
                delta,
            ]));
            lines.push(Line::from(bar(value, width, color)));
            lines.push(Line::from(Span::styled(
                with_ellipsis_if_needed(describe, width),
                fg(pal::DIM).add_modifier(Modifier::ITALIC),
            )));
            if per_stat == 4 {
                lines.push(Line::default());
            }
        } else {
            let room = width.saturating_sub(stat.label().len() + 2);
            let describe = with_ellipsis_if_needed(describe, room);
            let pad = width.saturating_sub(stat.label().len() + width_of(&describe));
            lines.push(Line::from(vec![
                label,
                Span::raw(" ".repeat(pad)),
                Span::styled(describe, fg(pal::DIM)),
            ]));
            let mut meter = bar(value, width.saturating_sub(9), color);
            meter.push(Span::styled(format!(" {value:>3.0}"), fg(pal::STONE)));
            meter.push(delta_span(app, stat));
            lines.push(Line::from(meter));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_condition_compact(frame: &mut Frame, area: Rect, app: &App, rock: &Rock, bordered: bool) {
    let inner = if bordered {
        let block = panel("Condition");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    } else {
        area
    };
    let inner = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    );
    let half = usize::from(inner.width) / 2;
    let columns = [
        [Stat::Moisture, Stat::Boredom],
        [Stat::Polish, Stat::Disappointment],
    ];
    let mut lines = vec![Vec::new(), Vec::new()];
    for (c, column) in columns.iter().enumerate() {
        let label_w = column.iter().map(|s| s.label().len()).max().unwrap_or(0);
        let col_w = if c == 0 {
            half.saturating_sub(2)
        } else {
            usize::from(inner.width) - half
        };
        for (r, stat) in column.iter().enumerate() {
            let value = rock.stats.get(*stat);
            let bar_w = col_w.saturating_sub(label_w + 5).max(2);
            let line = &mut lines[r];
            if c == 1 {
                line.push(Span::raw("  "));
            }
            line.push(Span::styled(
                format!("{:<label_w$} ", stat.label()),
                fg(stat_color(*stat)),
            ));
            line.extend(bar(value, bar_w, stat_color(*stat)));
            let delta = recent_delta(app, *stat).filter(|d| d.abs() >= 0.5);
            let number = Span::styled(
                format!("{:>4.0}", value),
                match delta {
                    Some(d) if is_good(*stat, d) => fg(pal::MOSS),
                    Some(_) => fg(pal::RUST),
                    None => fg(pal::STONE),
                },
            );
            line.push(number);
        }
    }
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>()),
        inner,
    );
}

fn entry_lines(entry: &Entry, now: i64, width: usize, voice: Color) -> Vec<Line<'static>> {
    let ago = format!("{:>4}  ", short_ago(now - entry.at));
    let body_w = width.saturating_sub(ago.len()).max(8);
    let (text, style) = match entry.kind {
        EntryKind::Action => {
            let label = entry.action.map(|a| a.label()).unwrap_or("Did something");
            (format!("{label}: \"{}\"", entry.text), fg(voice))
        }
        EntryKind::Movement => (entry.text.clone(), fg(pal::GLINT)),
        EntryKind::Rare => (entry.text.clone(), fg(pal::ORCHID)),
        EntryKind::Discovery => (
            entry.text.clone(),
            fg(pal::OCHRE).add_modifier(Modifier::BOLD),
        ),
        EntryKind::Remark => (
            format!("\"{}\"", entry.text),
            fg(voice).add_modifier(Modifier::ITALIC),
        ),
        EntryKind::System => (entry.text.clone(), fg(pal::DIM)),
    };
    let label_len = match (entry.kind, entry.action) {
        (EntryKind::Action, Some(a)) => a.label().len() + 1,
        _ => 0,
    };
    wrap(&text, body_w)
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let gutter = if i == 0 {
                Span::styled(ago.clone(), fg(pal::FAINT))
            } else {
                Span::raw(" ".repeat(ago.len()))
            };
            if i == 0 && label_len > 0 && line.len() >= label_len {
                let (label, rest) = line.split_at(label_len);
                Line::from(vec![
                    gutter,
                    Span::styled(label.to_string(), fg(pal::OCHRE)),
                    Span::styled(rest.to_string(), style),
                ])
            } else {
                Line::from(vec![gutter, Span::styled(line, style)])
            }
        })
        .collect()
}

fn journal_lines(
    app: &App,
    rock: &Rock,
    width: usize,
    max_lines: usize,
    spaced: bool,
) -> Vec<Line<'static>> {
    let voice = voice_color(rock.personality);
    let mut out = Vec::new();
    for entry in app.journal.recent() {
        let lines = entry_lines(entry, app.now, width, voice);
        if !out.is_empty() && out.len() + lines.len() > max_lines {
            break;
        }
        out.extend(lines);
        if spaced {
            out.push(Line::default());
        }
    }
    if out.is_empty() {
        out.push(Line::from(Span::styled(
            "Nothing yet. Do something regrettable.",
            fg(pal::DIM),
        )));
    }
    out
}

fn draw_notes(frame: &mut Frame, area: Rect, app: &App, rock: &Rock) {
    let block = panel("Field notes")
        .title_bottom(Line::from(Span::styled(" j for all ", fg(pal::FAINT))).right_aligned());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let inner = Rect::new(
        inner.x,
        inner.y,
        inner.width.saturating_sub(1),
        inner.height,
    );
    let lines = journal_lines(
        app,
        rock,
        usize::from(inner.width),
        usize::from(inner.height),
        false,
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

fn latest_lines(app: &App, rock: &Rock, width: usize) -> Vec<Line<'static>> {
    let voice = voice_color(rock.personality);
    let mut out = Vec::new();
    if let Some(notice) = &app.notice {
        out.extend(wrapped(notice, width, fg(pal::RUST)));
    }
    let Some(banner) = &app.banner else {
        match app.journal.recent().find(|e| e.kind == EntryKind::Action) {
            Some(entry) => {
                out.push(Line::from(Span::styled(
                    "Last time",
                    fg(pal::SAND).add_modifier(Modifier::BOLD),
                )));
                out.extend(entry_lines(entry, app.now, width, voice));
            }
            None => {
                out.push(Line::from(Span::styled(
                    "Nothing has happened yet.",
                    fg(pal::SAND).add_modifier(Modifier::BOLD),
                )));
                out.extend(wrapped(
                    &format!(
                        "Press a key below. {} waited out the Cambrian; it can wait for you.",
                        rock.name
                    ),
                    width,
                    fg(pal::DIM),
                ));
            }
        }
        return out;
    };

    out.extend(wrapped(
        &banner.title,
        width,
        fg(pal::SAND).add_modifier(Modifier::BOLD),
    ));
    if !banner.speech.is_empty() {
        out.extend(wrapped(&format!("\"{}\"", banner.speech), width, fg(voice)));
    }

    // Stat changes, packed onto as few lines as fit.
    let mut line: Vec<Span<'static>> = Vec::new();
    let mut used = 0;
    for (stat, delta) in &banner.changes {
        let label = format!("{} ", stat.label());
        let value = signed(*delta);
        let w = label.len() + value.len();
        if used > 0 && used + 3 + w > width {
            out.push(Line::from(std::mem::take(&mut line)));
            used = 0;
        }
        if used > 0 {
            line.push(Span::raw("   "));
            used += 3;
        }
        line.push(Span::styled(label, fg(pal::DIM)));
        let color = if is_good(*stat, *delta) {
            pal::MOSS
        } else {
            pal::RUST
        };
        line.push(Span::styled(value, fg(color).add_modifier(Modifier::BOLD)));
        used += w;
    }
    if !line.is_empty() {
        out.push(Line::from(line));
    }

    for note in &banner.notes {
        out.extend(wrapped(
            note,
            width,
            fg(pal::DIM).add_modifier(Modifier::ITALIC),
        ));
    }
    for (kind, text) in &banner.events {
        let style = match kind {
            EntryKind::Movement => fg(pal::GLINT).add_modifier(Modifier::BOLD),
            EntryKind::Rare => fg(pal::ORCHID),
            EntryKind::Discovery => fg(pal::OCHRE).add_modifier(Modifier::BOLD),
            EntryKind::Remark => fg(voice).add_modifier(Modifier::ITALIC),
            _ => fg(pal::DIM),
        };
        let text = match kind {
            EntryKind::Remark => format!("\"{text}\""),
            _ => text.clone(),
        };
        out.extend(wrapped(&text, width, style));
    }
    out
}

fn draw_latest(frame: &mut Frame, area: Rect, app: &App, lines: Vec<Line<'static>>) {
    let mut block = panel("Latest");
    if let Some(banner) = &app.banner {
        let ago = app.now - banner.at;
        let when = if ago < 60 {
            "just now".to_string()
        } else {
            format!("{} ago", duration_words(ago))
        };
        block = block
            .title(Line::from(Span::styled(format!(" {when} "), fg(pal::FAINT))).right_aligned());
    }
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let inner = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    );
    let mut lines = lines;
    let room = usize::from(inner.height);
    if lines.len() > room && room > 0 {
        lines.truncate(room - 1);
        lines.push(Line::from(Span::styled(
            "… the rest is in the field notes (j)",
            fg(pal::FAINT),
        )));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------- overlays

fn popup(frame: &mut Frame, area: Rect, width: u16, height: u16, title: &str) -> Rect {
    let rect = centered(area, width, height);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(pal::OCHRE))
        .title(Span::styled(
            format!(" {title} "),
            fg(pal::SAND).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    )
}

fn draw_scroller(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    title: &str,
    lines: Vec<Line<'static>>,
    scroll: usize,
) {
    let width = area.width.saturating_sub(4).min(84);
    let height = area.height.saturating_sub(2).max(5);
    let rect = centered(area, width, height);
    frame.render_widget(Clear, rect);
    let inner_h = usize::from(rect.height.saturating_sub(2));
    let max_scroll = lines.len().saturating_sub(inner_h);
    app.max_scroll.set(max_scroll);
    let scroll = scroll.min(max_scroll);

    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(fg(pal::OCHRE))
        .title(Span::styled(
            format!(" {title} "),
            fg(pal::SAND).add_modifier(Modifier::BOLD),
        ));
    if max_scroll > 0 {
        let hint = match scroll {
            0 => " more below ".to_string(),
            s if s >= max_scroll => " end ".to_string(),
            s => format!(" {}% ", s * 100 / max_scroll),
        };
        block = block.title_bottom(Line::from(Span::styled(hint, fg(pal::DIM))).right_aligned());
    }
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    let inner = Rect::new(
        inner.x + 1,
        inner.y,
        inner.width.saturating_sub(2),
        inner.height,
    );
    frame.render_widget(Paragraph::new(lines).scroll((to_u16(scroll), 0)), inner);
}

fn discovery_lines(app: &App, rock: &Rock, width: usize) -> Vec<Line<'static>> {
    let found = Achievement::ALL.iter().filter(|a| rock.has(**a)).count();
    let mut out = wrapped(
        &format!(
            "{found} of {} discovered. The rest are hidden, as is tradition.",
            Achievement::ALL.len()
        ),
        width,
        fg(pal::DIM),
    );
    out.push(Line::default());
    let mut locked = 0;
    for achievement in Achievement::ALL {
        if let Some(at) = rock.achievements.get(achievement.id()) {
            out.push(Line::from(vec![
                Span::styled("* ", fg(pal::GLINT)),
                Span::styled(
                    achievement.title(),
                    fg(pal::OCHRE).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("  {}", short_ago(app.now - at)), fg(pal::FAINT)),
            ]));
            for line in wrap(achievement.description(), width.saturating_sub(2)) {
                out.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line, fg(pal::STONE)),
                ]));
            }
        } else {
            out.push(Line::from(vec![
                Span::styled("? ??? ", fg(pal::FAINT)),
                Span::styled(LOCKED[locked % LOCKED.len()], fg(pal::DEEP)),
            ]));
            locked += 1;
        }
    }
    out
}

/// What a hidden discovery says about itself. Not much.
const LOCKED: [&str; 5] = [
    "Not yet.",
    "Classified, geologically.",
    "You'll know it when it happens.",
    "Undiscovered. Possibly undiscoverable. Probably not.",
    "The rock knows. The rock isn't telling.",
];

fn help_lines(app: &App, width: usize) -> Vec<Line<'static>> {
    let heading = |text: &str| {
        Line::from(Span::styled(
            text.to_string(),
            fg(pal::SAND).add_modifier(Modifier::BOLD),
        ))
    };
    let body = fg(pal::STONE);
    let mut out = vec![heading("What each action does")];
    for (i, action) in Action::ALL.into_iter().enumerate() {
        let mut spans = vec![
            Span::styled(
                format!("{} {} ", action.key(), i + 1),
                fg(pal::OCHRE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{:<17}", action.label()), body),
        ];
        let effects: Vec<String> = action
            .effects()
            .iter()
            .map(|(stat, d)| format!("{} {}", stat.label(), signed(*d)))
            .collect();
        let text = effects.join(", ");
        let room = width.saturating_sub(22);
        let mut first = true;
        for line in wrap(&text, room.max(12)) {
            if !first {
                out.push(Line::from(std::mem::take(&mut spans)));
                spans.push(Span::raw(" ".repeat(22)));
            }
            spans.push(Span::styled(line, fg(pal::DIM)));
            first = false;
        }
        out.push(Line::from(spans));
    }
    out.push(Line::default());
    out.push(heading("How it works"));
    let rules = [
        format!(
            "Stats run from 0 to {STAT_MAX:.0}. Every hour, moisture {}, polish {} and boredom {}. \
             A bored rock grows more disappointed ({} an hour); an entertained one slowly forgives \
             ({} an hour). Time alone never pushes a stat below {DRIFT_FLOOR:.0} or above \
             {DRIFT_CEILING:.0}; only you can do that.",
            signed(MOISTURE_DRIFT),
            signed(POLISH_DRIFT),
            signed(BOREDOM_DRIFT),
            signed(DISAPPOINTMENT_RISE),
            signed(DISAPPOINTMENT_EASE),
        ),
        format!(
            "Doing the same thing twice in a row gives half the boredom relief and \
             +{REPEAT_PENALTY:.0} disappointment. Watering a rock at {OVERWATER_AT:.0}+ moisture or \
             polishing one at {OVERPOLISH_AT:.0}+ polish backfires."
        ),
        "Neglect makes it more sarcastic. It cannot die. It can only get better at saying what \
         it thinks of you."
            .to_string(),
        format!(
            "Time away counts for at most {}, so a month off won't ruin anything. A clock that \
             runs backwards costs nothing.",
            duration_words(OFFLINE_CAP_SECS)
        ),
        "Every so often the rock moves one character. This is a big deal. Some discoveries are \
         hidden; press a to see what you've found."
            .to_string(),
    ];
    for rule in rules {
        out.extend(wrapped(&rule, width, body));
        out.push(Line::default());
    }
    out.push(heading("Your save"));
    out.extend(wrapped(
        &app.save_path.display().to_string(),
        width,
        fg(pal::GLINT),
    ));
    out.extend(wrapped(
        "Saved after every action and every 30 seconds. Set SEDIMENTARY_SAVE to use another \
         file. Unreadable saves are set aside, never deleted.",
        width,
        fg(pal::DIM),
    ));
    out
}

fn draw_rename(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    rock: &Rock,
    input: &str,
    suggestion: &str,
) {
    let width = area.width.saturating_sub(4).min(56);
    let inner = popup(frame, area, width, 7, &format!("Rename {}", rock.name));
    let w = usize::from(inner.width);
    let mut lines = wrapped(
        "It will remember this. It remembers everything.",
        w,
        fg(pal::DIM),
    );
    lines.truncate(1);
    lines.push(Line::default());
    let field = if input.is_empty() {
        Line::from(vec![
            Span::styled("> ", fg(pal::OCHRE)),
            Span::styled(suggestion.to_string(), fg(pal::FAINT)),
        ])
    } else {
        Line::from(vec![
            Span::styled("> ", fg(pal::OCHRE)),
            Span::styled(
                input.to_string(),
                fg(pal::BRIGHT).add_modifier(Modifier::BOLD),
            ),
        ])
    };
    lines.push(field);
    lines.push(Line::default());
    match &app.notice {
        Some(notice) => lines.push(Line::from(Span::styled(notice.clone(), fg(pal::RUST)))),
        None => lines.push(Line::from(Span::styled(
            format!("Tab fills in \"{suggestion}\"."),
            fg(pal::DIM),
        ))),
    }
    frame.render_widget(Paragraph::new(lines), inner);
    let cursor_x = inner.x + 2 + to_u16(width_of(input));
    if cursor_x < inner.right() {
        frame.set_cursor_position((cursor_x, inner.y + 2));
    }
}

fn draw_confirm(frame: &mut Frame, area: Rect, rock: &Rock) {
    let width = area.width.saturating_sub(4).min(56);
    let text = format!(
        "Retire {name} and adopt a new rock? {name}'s save will be set aside as a backup \
         file, and {name} will pretend not to mind.",
        name = rock.name
    );
    let w = usize::from(width.saturating_sub(4));
    let mut lines = wrapped(&text, w, fg(pal::STONE));
    let height = to_u16(lines.len() + 4);
    let inner = popup(frame, area, width, height, "Start over?");
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("y", fg(pal::OCHRE).add_modifier(Modifier::BOLD)),
        Span::styled(" retire it    ", fg(pal::DIM)),
        Span::styled("n", fg(pal::OCHRE).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" keep {}", rock.name), fg(pal::DIM)),
    ]));
    frame.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------- setup

fn draw_setup(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    step: SetupStep,
    name: &str,
    selected: usize,
    suggestion: &str,
) {
    let col_w = area.width.saturating_sub(4).min(66);
    let w = usize::from(col_w);
    let personality = Personality::ALL[selected % Personality::ALL.len()];

    let mut form: Vec<Line<'static>> = Vec::new();
    let mut cursor_row = None;
    match step {
        SetupStep::Name => {
            form.push(Line::from(Span::styled(
                "What will you call your rock?",
                fg(pal::SAND).add_modifier(Modifier::BOLD),
            )));
            form.push(Line::default());
            cursor_row = Some(form.len());
            form.push(if name.is_empty() {
                Line::from(vec![
                    Span::styled("> ", fg(pal::OCHRE)),
                    Span::styled(suggestion.to_string(), fg(pal::FAINT)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("> ", fg(pal::OCHRE)),
                    Span::styled(
                        name.to_string(),
                        fg(pal::BRIGHT).add_modifier(Modifier::BOLD),
                    ),
                ])
            });
            form.push(Line::default());
            form.extend(wrapped(
                &format!("Tab takes the suggestion. Leave it blank and it'll be {suggestion}."),
                w,
                fg(pal::DIM),
            ));
        }
        SetupStep::Personality => {
            let shown = if name.is_empty() { suggestion } else { name };
            form.extend(wrapped(
                &format!("Choose a temperament for {shown}."),
                w,
                fg(pal::SAND).add_modifier(Modifier::BOLD),
            ));
            form.push(Line::default());
            for (i, p) in Personality::ALL.iter().enumerate() {
                let chosen = i == selected % Personality::ALL.len();
                let marker = if chosen { "> " } else { "  " };
                let style = if chosen {
                    fg(voice_color(*p)).add_modifier(Modifier::BOLD)
                } else {
                    fg(pal::STONE)
                };
                form.push(Line::from(vec![
                    Span::styled(marker, fg(pal::OCHRE)),
                    Span::styled(format!("{} ", i + 1), fg(pal::FAINT)),
                    Span::styled(p.label(), style),
                ]));
                for line in wrap(p.tagline(), w.saturating_sub(4)) {
                    form.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(line, fg(if chosen { pal::DIM } else { pal::FAINT })),
                    ]));
                }
            }
            form.push(Line::default());
            form.extend(wrapped(
                &format!("\"{}\"", personality.sample()),
                w,
                fg(voice_color(personality)).add_modifier(Modifier::ITALIC),
            ));
        }
    }

    let title_h = 3;
    let picture_h = 10; // 8 art rows, the surface, one stratum
    let form_h = form.len();
    let total_with_art = title_h + picture_h + 1 + form_h;
    let show_art = usize::from(area.height) >= total_with_art;
    let total = if show_art {
        total_with_art
    } else {
        title_h + form_h
    };
    let top = area.y + area.height.saturating_sub(to_u16(total)) / 2;
    let left = area.x + (area.width - col_w) / 2;

    let title = vec![
        Line::from(Span::styled(
            "Sedimentary",
            fg(pal::OCHRE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "a pet rock that is deeply unimpressed with you",
            fg(pal::DIM),
        )),
    ];
    frame.render_widget(
        Paragraph::new(title).centered(),
        Rect::new(left, top, col_w, 2).intersection(area),
    );

    let mut y = top + to_u16(title_h);
    if show_art {
        let look = Look::preview(personality, app.frame);
        let look = if step == SetupStep::Name {
            Look {
                personality: Personality::Judgmental,
                ..look
            }
        } else {
            look
        };
        let canvas = art::render(&look);
        let rock_x = i32::from(left) + (i32::from(col_w) - CANVAS_WIDTH as i32) / 2;
        let strip = Rect::new(left, y, col_w, to_u16(picture_h)).intersection(area);
        let voice = voice_color(look.personality);
        let buf = frame.buffer_mut();
        let surface = i32::from(y) + 8;
        let strip_w = usize::from(col_w).min(CANVAS_WIDTH + 12);
        let strip_x = i32::from(left) + (i32::from(col_w) - strip_w as i32) / 2;
        for i in 0..strip_w as i32 {
            put(buf, strip, strip_x + i, surface, '▀', fg(pal::SAND));
            let ch = if i % 9 == 4 { '▓' } else { '▒' };
            put(buf, strip, strip_x + i, surface + 1, ch, fg(pal::CLAY));
        }
        for (r, row) in canvas.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                if cell.tone != Tone::Blank {
                    put(
                        buf,
                        strip,
                        rock_x + c as i32,
                        i32::from(y) + r as i32,
                        cell.ch,
                        tone_style(cell.tone, pal::STONE, voice),
                    );
                }
            }
        }
        y += to_u16(picture_h + 1);
    }

    let form_rect = Rect::new(left, y, col_w, to_u16(form_h)).intersection(area);
    frame.render_widget(Paragraph::new(form), form_rect);

    if let Some(row) = cursor_row {
        let x = form_rect.x + 2 + to_u16(width_of(name));
        let y = form_rect.y + to_u16(row);
        if x < form_rect.right() && y < form_rect.bottom() {
            frame.set_cursor_position((x, y));
        }
    }

    if let Some(notice) = &app.notice {
        let lines = wrapped(
            notice,
            usize::from(area.width.saturating_sub(4)),
            fg(pal::RUST),
        );
        let h = to_u16(lines.len()).min(area.height);
        frame.render_widget(
            Paragraph::new(lines).centered(),
            Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), h),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_respects_width_and_keeps_words() {
        let lines = wrap("the quick brown fox jumps over the lazy dog", 10);
        assert!(lines.iter().all(|l| width_of(l) <= 10), "{lines:?}");
        assert_eq!(
            lines.join(" "),
            "the quick brown fox jumps over the lazy dog"
        );
        let long = wrap("supercalifragilistic", 6);
        assert!(long.iter().all(|l| width_of(l) <= 6));
        assert_eq!(wrap("", 5), vec![String::new()]);
    }

    #[test]
    fn signed_formats_cleanly() {
        assert_eq!(signed(6.0), "+6");
        assert_eq!(signed(-5.0), "-5");
        assert_eq!(signed(2.5), "+2.5");
    }

    #[test]
    fn ages_read_naturally() {
        assert_eq!(age_phrase(340), "about 340 million years old");
        assert_eq!(age_phrase(2_140), "about 2.1 billion years old");
    }
}
