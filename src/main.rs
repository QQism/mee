use std::io::{stdout, Write, Stdout};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size, ScrollUp, ScrollDown},
    QueueableCommand,
    queue,
    Result,
};

struct TerminalSize {
    cols: u16,
    rows: u16,
}

fn match_event() -> Result<()> {
    let mut stdout = stdout();
    let mut line = String::new();
    let mut current_max_column = 0;

    let (terminal_cols, terminal_rows) = size()?;

    let mut terminal_size = TerminalSize {
        cols: terminal_cols,
        rows: terminal_rows,
    };

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
                echo(&mut stdout, &mut terminal_size, line.clone())?;
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
                queue!(stdout, Clear(ClearType::CurrentLine), cursor::SavePosition, cursor::MoveToColumn(0)).expect("Error");
                // print!("{}", line);
                write!(stdout, "{}", line)?;
                queue!(stdout, cursor::RestorePosition, cursor::MoveLeft(1)).expect("Error");

                current_max_column -= 1;

                show_suggestions(&mut stdout, &mut terminal_size, line.clone())?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c), ..
            }) => {
                let (column, _) = cursor::position().unwrap();

                if line.len() == (column as usize) {
                    write!(stdout, "{}", c)?;
                    // queue!(stdout, cursor("{}", c))?;
                    line.push(c);
                } else if line.len() < (column as usize) {

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
                    // print!("{}", line);
                    write!(stdout, "{}", line)?;
                    queue!(stdout, cursor::RestorePosition, cursor::MoveRight(1)).expect("Error");
                }

                current_max_column += 1;

                show_suggestions(&mut stdout, &mut terminal_size, line.clone())?;
            }
            Event::Resize(columns, rows) => {
                println!("Terminal size changed: Columns {} Rows {}", columns, rows);
                terminal_size.cols = columns;
                terminal_size.rows = rows;
            }
            _ => {}
        }

        stdout.flush()?;
    }

    Ok(())
}

fn clear_suggestions(stdout: &mut Stdout, terminal_size: &mut TerminalSize) -> Result<()> {
    let (col, _) = cursor::position().unwrap();
    queue!(stdout,
           cursor::SavePosition,
           cursor::MoveToNextLine(1),
           Clear(ClearType::CurrentLine),
           cursor::MoveToNextLine(1),
           Clear(ClearType::CurrentLine),
           cursor::MoveToNextLine(1),
           Clear(ClearType::CurrentLine),
           cursor::RestorePosition)?;

    Ok(())
}

fn show_suggestions(mut stdout: &mut Stdout, terminal_size: &mut TerminalSize, line: String) -> Result<()> {
    clear_suggestions(&mut stdout, terminal_size)?;

    // let (_, rows) = cursor::position().unwrap();

    // if (rows + 3) >= terminal_size.rows {
    //     queue!(stdout,
    //            cursor::SavePosition,
    //            ScrollUp(3),
    //            cursor::RestorePosition)?;
    // }

    queue!(stdout, cursor::SavePosition, cursor::MoveToNextLine(1))?;
    write!(stdout, "\u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}")?;
    queue!(stdout, cursor::MoveToNextLine(1))?;
    write!(stdout, "\u{2502} {} \u{2502}", line)?;
    queue!(stdout, cursor::MoveToNextLine(1))?;
    write!(stdout, "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}")?;
    queue!(stdout, cursor::RestorePosition)?;

    Ok(())
}

fn echo(mut stdout: &mut Stdout, terminal_size: &mut TerminalSize, line: String) -> Result<()> {
    clear_suggestions(&mut stdout, terminal_size)?;

    queue!(stdout, cursor::MoveToNextLine(1))?;

    write!(stdout, "You typed: {}", line)?;

    queue!(stdout, cursor::MoveToNextLine(1))?;

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
