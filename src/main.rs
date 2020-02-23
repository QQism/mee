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
    let mut current_max_column = 0;

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
                if column < current_max_column {
                    stdout.queue(cursor::MoveRight(1)).expect("Error");
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                let (column, _) = cursor::position().unwrap();
                // There is no letter at column 0, 
                // to delete a letter, that should start at column 1 (char index 0 of string)
                if line.len() == 0 || column == 0 {
                    continue;
                }
                line.remove((column-1) as usize);
                queue!(stdout, Clear(ClearType::CurrentLine), cursor::SavePosition).expect("Error");
                print!("\r{}", line);
                queue!(stdout, cursor::RestorePosition, cursor::MoveLeft(1)).expect("Error");

                queue!(stdout, cursor::SavePosition, 
                    cursor::MoveToNextLine(1),
                    Clear(ClearType::CurrentLine)
                ).expect("Errro");
                println!("{}", line);
                stdout.queue(cursor::RestorePosition).expect("Error");
                current_max_column -= 1;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c), ..
            }) => {
                let (column, _) = cursor::position().unwrap();
                // Print in the current line
                // Show suggestion in the next line

                if line.len() == (column as usize) {
                    print!("{}", c);
                    line.push(c);
                } else {
                    // Very inefficient, 0(n) for every operation, need sort of a link list data structure

                }
                
                stdout.queue(cursor::SavePosition)?;
                println!("\n\r{}", line);
                stdout.queue(cursor::RestorePosition)?;
                current_max_column += 1;
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
