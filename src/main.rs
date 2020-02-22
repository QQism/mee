use std::io::{stdout, Write};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand,
    queue,
    Result,
};

const HELP: &str = r#"Blocking read()
 - Keyboard, mouse and terminal resize events enabled
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

fn match_event() -> Result<()> {
    let mut stdout = stdout();
    let mut line = String::new();
    let mut currentMaxColumn = 0;

    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL
            }) => {
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                // cursor::MoveToNextLine(1);
                // Print the suggestion move to the next line
                print!("\n\r>> ");
                line.clear();
            }
            Event::Key(KeyEvent { code: KeyCode::Left, .. }) => { stdout.queue(cursor::MoveLeft(1)).expect("Error"); }
            Event::Key(KeyEvent {code: KeyCode::Right, .. }) => {
                let (column, _) = cursor::position().unwrap();
                if column < currentMaxColumn {
                    stdout.queue(cursor::MoveRight(1)).expect("Error");
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                let (column, _) = cursor::position().unwrap();
                // line.remove(column.try_into().unwrap());
                line.remove(column as usize);
                queue!(stdout, cursor::MoveLeft(1), Clear(ClearType::CurrentLine));
                print!("{}", line);
                stdout.queue(cursor::SavePosition);
                println!("\n\r {}", line);
                stdout.queue(cursor::RestorePosition);
                currentMaxColumn -= 1;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c), ..
            }) => {
                // Print in the current line
                // Show suggestion in the next line
                print!("{}", c);
                line.push(c);
                stdout.queue(cursor::SavePosition)?;
                println!("\n\r {}", line);
                stdout.queue(cursor::RestorePosition)?;
                currentMaxColumn += 1;
            }
            _ => {}
        }

        stdout.flush()?;
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("{}", HELP);

    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnableMouseCapture)?;

    match_event()?;

    // if let Err(e) = print_events() {
    //     println!("Error: {:?}\r", e);
    // }

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
