use waybuilder::build;
use waybuilder::model;

mod data;
mod persistence;
mod ui;

use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use wayfinder_core::aon::GameSystem;

use ui::app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let system = if std::env::args().any(|a| a == "--sf2e") {
        GameSystem::Starfinder
    } else {
        GameSystem::Pathfinder
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, system).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    system: GameSystem,
) -> anyhow::Result<()> {
    let mut app = App::new(system);
    loop {
        app.poll_loader();
        app.tick();
        terminal.draw(|f| ui::render::draw(f, &app))?;
        ui::events::poll_event(&mut app)?;
        if app.quit {
            return Ok(());
        }
    }
}
