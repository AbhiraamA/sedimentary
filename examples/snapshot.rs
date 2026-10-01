use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sedimentary::app::App;
use sedimentary::dice::Dice;
use sedimentary::ui;

fn dump(app: &App, w: u16, h: u16, label: &str) {
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| ui::draw(f, app)).unwrap();
    let buf = t.backend().buffer().clone();
    println!("==== {label} ({w}x{h}) ====");
    for y in 0..h {
        let mut s = String::new();
        for x in 0..w {
            s.push_str(buf[(x, y)].symbol());
        }
        println!("{}|", s.trim_end());
    }
}

fn key(app: &mut App, c: KeyCode, now: i64) {
    app.handle_key(KeyEvent::new(c, KeyModifiers::NONE), now);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let which = args.get(1).map(String::as_str).unwrap_or("all");
    let dir = std::env::temp_dir().join(format!("sed-snap-{}", std::process::id()));
    let path = dir.join("save.json");
    let t0 = 1_750_000_000;
    let mut app = App::load(path.clone(), t0, Dice::seeded(3));
    if which == "setup" || which == "all" {
        dump(&app, 80, 30, "setup name");
    }
    for c in "Gerald".chars() {
        key(&mut app, KeyCode::Char(c), t0);
    }
    key(&mut app, KeyCode::Enter, t0);
    if which == "setup" || which == "all" {
        dump(&app, 80, 30, "setup personality");
        dump(&app, 50, 24, "setup personality small");
    }
    key(&mut app, KeyCode::Enter, t0);
    if which == "main" || which == "all" {
        dump(&app, 100, 32, "main fresh");
        dump(&app, 130, 42, "large");
    }
    key(&mut app, KeyCode::Char('f'), t0 + 2);
    if which == "main" || which == "all" {
        dump(&app, 100, 32, "after feed");
        dump(&app, 80, 24, "after feed 80x24");
        dump(&app, 60, 24, "narrow 60x24");
        dump(&app, 50, 24, "min 50x24");
        dump(&app, 40, 15, "too small");
    }
    if which == "states" || which == "all" {
        let r = app.rock.as_mut().unwrap();
        r.stats.moisture = 95.0;
        r.stats.polish = 20.0;
        r.stats.boredom = 90.0;
        r.stats.disappointment = 90.0;
        r.position = 5;
        r.counters.moves = 7;
        app.tick(t0 + 100);
        dump(&app, 100, 32, "soggy bored grimy moved");
        let r = app.rock.as_mut().unwrap();
        r.stats.moisture = 5.0;
        r.stats.polish = 99.0;
        dump(&app, 100, 32, "dry polished");
    }
    if which == "overlays" || which == "all" {
        key(&mut app, KeyCode::Char('?'), t0 + 120);
        dump(&app, 90, 30, "help");
        key(&mut app, KeyCode::Esc, t0 + 120);
        key(&mut app, KeyCode::Char('a'), t0 + 120);
        dump(&app, 90, 30, "discoveries");
        key(&mut app, KeyCode::Esc, t0 + 120);
        key(&mut app, KeyCode::Char('j'), t0 + 120);
        dump(&app, 90, 30, "journal");
        key(&mut app, KeyCode::Esc, t0 + 120);
        key(&mut app, KeyCode::Char('r'), t0 + 120);
        for c in "Cliff".chars() {
            key(&mut app, KeyCode::Char(c), t0 + 120);
        }
        dump(&app, 90, 30, "rename");
        key(&mut app, KeyCode::Esc, t0 + 120);
        key(&mut app, KeyCode::Char('n'), t0 + 120);
        dump(&app, 90, 30, "confirm new");
    }
    let _ = std::fs::remove_dir_all(dir);
}
