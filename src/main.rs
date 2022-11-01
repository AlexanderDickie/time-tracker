use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};

mod app;
mod ui;

static FILE_PATH: &str = "./storage.csv";
static DURATION: usize = 60 * 25;

#[tokio::main]
async fn main() {
    // setup crossterm
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen,  EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let tick_rate = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    let mut app = app::App::build(FILE_PATH);
    loop {
        terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {

                match key.code {
                    KeyCode::Char(c) => app.on_key(c),
                    _ => (),
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // restore terminal
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        ).unwrap();
    terminal.show_cursor().unwrap();

}






