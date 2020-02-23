use std::io::{stdout, Write, Stdout};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand,
    queue,
    Result,
};

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
                    queue!(stdout, cursor::MoveRight(1)).expect("Error");
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                let (column, _) = cursor::position().unwrap();
                // to delete a letter, cursor column should be at least 1, then move 1 step left to zero and delete the first letter
                if line.is_empty() || column == 0 {
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
                queue!(stdout, cursor::RestorePosition).expect("Error");
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
                    let (str1, str2) = line.split_at(column as usize);

                    let first_str = String::from(str1);
                    let second_str = String::from(str2);

                    queue!(stdout,
                           Clear(ClearType::CurrentLine),
                           cursor::SavePosition, cursor::MoveToColumn(0)).expect("Error");

                    line.clear();
                    line.push_str(&first_str);
                    line.push(c);
                    line.push_str(&second_str);
                    print!("{}", line);
                    queue!(stdout, cursor::RestorePosition, cursor::MoveRight(1)).expect("Error");
                }

                current_max_column += 1;

                show_sugesstion(&mut stdout, &mut line);
            }
            _ => {}
        }

        stdout.flush()?;
    }

    Ok(())
}

fn show_sugesstion(stdout: &mut Stdout, line: &mut String) -> Result<()> {
    queue!(stdout,
           cursor::SavePosition,
           cursor::MoveToNextLine(1),
           Clear(ClearType::CurrentLine))?;
    print!("{}", line);
    queue!(stdout, cursor::RestorePosition)?;

    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnableMouseCapture)?;

    match_event()?;

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
