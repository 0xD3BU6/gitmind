use std::error::Error;
use std::path::PathBuf;

use gitmind::{App, Tui, ui};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).map(PathBuf::from);
    if matches!(path.as_deref().and_then(|p| p.to_str()), Some("-h" | "--help")) {
        println!("usage: gitmind [REPO_PATH]");
        return Ok(());
    }

    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::with_path(path);

    // Run the loop and always restore the terminal, even on error.
    let result = run(&mut tui, &mut app);
    tui.exit()?;
    result
}

fn run(tui: &mut Tui, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        tui.draw(|f| ui::draw(f, app))?;
        if app.tick()? {
            break;
        }
    }
    Ok(())
}
