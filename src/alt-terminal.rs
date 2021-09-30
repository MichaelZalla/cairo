use std::io::{stdout};
use crossterm::{
    cursor::{Show, Hide, MoveTo},
    event::{poll, read, Event},
    execute,
    style::{Color, Print, SetForegroundColor, ResetColor},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};
use std::time::Duration;

fn main() -> Result<()> {

	execute!(
        stdout(),
        EnterAlternateScreen,
        SetForegroundColor(Color::Magenta),
        Hide
    )?;

	loop {

		if poll(Duration::from_millis(500))? {
            match read()? {
                Event::Key(_) => break,
                _ => {}
            }
        } else {
            execute!(
                stdout(),
                Clear(ClearType::All),
                MoveTo(0, 0),
                Print("Press enter to exit...")
            )?;
        }

	}

	execute!(stdout(), ResetColor, Show, LeaveAlternateScreen)?;

	Ok(())

}