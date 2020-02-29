use std::io::{stdout, Write, Stdout};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size, ScrollUp},
    QueueableCommand,
    queue,
    Result,
};

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

struct TerminalSize {
    cols: u16,
    rows: u16,
}

fn match_event() -> Result<()> {
    let mut stdout = stdout();
    let mut line = String::new();
    let mut token = String::new();
    let mut current_max_column = 0;

    let (terminal_cols, terminal_rows) = size()?;

    let mut terminal_size = TerminalSize {
        cols: terminal_cols,
        rows: terminal_rows,
    };

    let words = load_words();

    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL
            }) => {
                clear_suggestions(&mut stdout)?;
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
                let (current_col, _) = cursor::position().unwrap();
                // to delete a letter, cursor column should be at least 1, then move 1 step left to zero and delete the first letter
                if line.is_empty() || current_col == 0 {
                    continue;
                }

                line.remove((current_col-1) as usize);
                queue!(stdout, Clear(ClearType::CurrentLine), cursor::SavePosition, cursor::MoveToColumn(0)).expect("Error");
                write!(stdout, "{}", line)?;
                queue!(stdout, cursor::RestorePosition, cursor::MoveLeft(1)).expect("Error");

                current_max_column -= 1;

                // Detect the current token
                let mut new_col = (current_col - 1) as usize;
                let mut char_iter = line.chars();

                // Check the left side
                let mut left_part = String::new();

                if new_col > 0 {
                    loop {
                        // println!("new_col {}", new_col);
                        let c = char_iter.nth(new_col-1);

                        if c == None || c == Some(' ') {
                            break;
                        }

                        left_part.push(c.unwrap());

                        new_col -= 1;

                        if new_col < 1 {
                            break;
                        }
                    }
                }
                // Then check the right side
                let mut right_part = String::new();

                new_col = (current_col-1) as usize;

                let line_count = (line.chars().count() as u16);

                if new_col < (line_count as usize) {

                    loop {
                        let c = char_iter.nth(new_col-1);

                        if c == None || c == Some(' ') {
                            break;
                        }

                        right_part.push(c.unwrap());

                        new_col += 1;

                        if new_col >= (line_count as usize) {
                            break;
                        }
                    }
                }

                token.clear();
                println!("\nLeft: {}", left_part);
                println!("\nRight: {}", right_part);
                loop {
                    let c = left_part.pop();
                    if c == None {
                        break;
                    }

                    token.push(c.unwrap());
                }

                token.push_str(&right_part);
                println!("\nToken: {}", token);

                show_suggestions(&mut stdout, &mut terminal_size, token.clone(), &words)?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c), ..
            }) => {let (current_col, _) = cursor::position().unwrap();

                if line.len() == (current_col as usize) {
                    write!(stdout, "{}", c)?;
                    // queue!(stdout, cursor("{}", c))?;
                    line.push(c);

                    // update the current token

                    if c == ' ' {
                        token.clear();
                    } else {
                        token.push(c);
                    }
                } else if line.len() < (current_col as usize) {
                    // Not possible, the cursor cannot be further the current line
                } else {
                    let (str1, str2) = line.split_at(current_col as usize);

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

                // To detect the current token, first check the left side
                let new_col = current_col + 1;
                // then check the right side

                show_suggestions(&mut stdout, &mut terminal_size, token.clone(), &words)?;
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

fn clear_suggestions(stdout: &mut Stdout) -> Result<()> {
    queue!(stdout,
           cursor::SavePosition,
           cursor::MoveToNextLine(1),
           Clear(ClearType::FromCursorDown),
           cursor::RestorePosition)?;

    Ok(())
}

fn show_suggestions(mut stdout: &mut Stdout, terminal_size: &mut TerminalSize, line: String, words: &Vec<String>) -> Result<()> {
    clear_suggestions(&mut stdout)?;

    let (_, rows) = cursor::position().unwrap();

    let suggestion_height = 3; // need to >= 3

    if (rows + suggestion_height) >= terminal_size.rows {
        queue!(stdout,
               // Scroll up and move the cursor back to where it was
               ScrollUp(suggestion_height),
               cursor::MoveUp(suggestion_height)
        )?;
    }

    queue!(stdout, cursor::SavePosition, cursor::MoveToNextLine(1))?;

    // Render the top border
    write!(stdout, "\u{256D}")?; // ╭

    for _col in 0..terminal_size.cols-2 {
        write!(stdout, "\u{2500}")?; // ─
    }

    write!(stdout, "\u{256E}")?; // ╮

    // Render the body

    let mut current_suggestion_line = 0;

    queue!(stdout, cursor::MoveToNextLine(1))?;

    // Render the left border
    write!(stdout, "\u{2502} ")?; // │

    // Start at col 3rd
    let mut col = 3;

    for word in words {
        if word.starts_with(&line.to_lowercase()) {
            let word_spaces = word.chars().count() + 1; // there is one space between suggestions, so +1;

            col += word_spaces;

            if col > ((terminal_size.cols-3) as usize) {
                // close the current line
                // Render the right border
                queue!(stdout, cursor::MoveToColumn(terminal_size.cols-1))?;
                write!(stdout, " \u{2502}")?; // │

                // if still in the suggestion area
                if current_suggestion_line < suggestion_height {
                    queue!(stdout, cursor::MoveToNextLine(1))?;
                    col = 3 + word_spaces;
                    current_suggestion_line += 1;

                    // Render the left border
                    write!(stdout, "\u{2502} ")?; // │
                } else {
                    break;
                }
            }

            write!(stdout, "{} ", word)?;
        }
    }

    // write!(stdout, "{}", line)?;

    // Render the right border
    queue!(stdout, cursor::MoveToColumn(terminal_size.cols-1))?;
    write!(stdout, " \u{2502}")?; // │

    queue!(stdout, cursor::MoveToNextLine(1))?;

    // Render the bottom border
    write!(stdout, "\u{2570}")?; // ╰

    for _col in 0..terminal_size.cols-2 {
        write!(stdout, "\u{2500}")?; // ─
    }

    write!(stdout, "\u{256F}")?; // ╯

    queue!(stdout, cursor::RestorePosition)?;

    Ok(())
}

fn echo(mut stdout: &mut Stdout, terminal_size: &mut TerminalSize, line: String) -> Result<()> {
    clear_suggestions(&mut stdout)?;

    queue!(stdout, cursor::MoveToNextLine(1))?;

    write!(stdout, "You typed: {}", line)?;

    queue!(stdout, cursor::MoveToNextLine(1))?;

    Ok(())
}

fn load_words() -> Vec<String> {
    let file = File::open("words.txt").expect("Cannot find the words.txt file");
    let reader = BufReader::new(file);

    let mut words: Vec<String> = Vec::new();

    for line in reader.lines() {
        let uline = line.unwrap();
        words.push(uline);
    }

    words
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();

    let (cols, rows) = size()?;

    println!("Terminal size {} cols x {} rows\n\r", cols, rows);

    execute!(stdout, EnableMouseCapture)?;

    match_event()?;

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
