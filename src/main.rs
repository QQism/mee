use std::io::{stdout, Write, Stdout};
use crossterm::{
    cursor,
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size, ScrollUp},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
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
    let mut selected_suggestion_idx: i32 = -1;
    let mut is_selecting: u8 = 0;
    let mut suggestions: Vec<String> = Vec::new();

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
                match is_selecting {
                    1 => {
                        is_selecting = 0;
                        // Fill the suggestion
                    }
                    _ => {
                        echo(&mut stdout, &mut terminal_size, line.clone())?;
                        line.clear();
                    }
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Left, .. }) => { stdout.queue(cursor::MoveLeft(1)).expect("Error"); }
            Event::Key(KeyEvent {code: KeyCode::Right, .. }) => {
                let (column, _) = cursor::position().unwrap();
                if column < current_max_column {
                    queue!(stdout, cursor::MoveRight(1)).expect("Error");
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Tab, .. }) => { 
                selected_suggestion_idx += 1;

                if selected_suggestion_idx >= (suggestions.len() as i32) {
                    selected_suggestion_idx = 0;
                }
                
                show_suggestions(&mut stdout, &mut terminal_size, &suggestions, selected_suggestion_idx)?;
            }
            Event::Key(KeyEvent { code: KeyCode::BackTab, .. }) => { 
                selected_suggestion_idx -= 1;

                if selected_suggestion_idx <= -1 {
                    selected_suggestion_idx = (suggestions.len() as i32) - 1;
                }

                show_suggestions(&mut stdout, &mut terminal_size, &suggestions, selected_suggestion_idx)?;
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

                token = get_current_token(line.clone(), current_col);

                suggestions = get_suggestions(token.clone(), &words);
                show_suggestions(&mut stdout, &mut terminal_size, &suggestions, selected_suggestion_idx)?;
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

                    write!(stdout, "{}", line)?;
                    queue!(stdout, cursor::RestorePosition, cursor::MoveRight(1)).expect("Error");

                    token = get_current_token(line.clone(), current_col);
                }

                current_max_column += 1;

                // To detect the current token, first check the left side
                // let new_col = current_col + 1;
                // then check the right side

                suggestions = get_suggestions(token.clone(), &words);
                show_suggestions(&mut stdout, &mut terminal_size, &suggestions, selected_suggestion_idx)?;
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

fn show_suggestions(mut stdout: &mut Stdout, terminal_size: &mut TerminalSize, suggestions: &Vec<String>, selected_suggestion_idx: i32) -> Result<()> {
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

    let mut idx = 0;
    for word in suggestions {
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

        if idx == selected_suggestion_idx {
            queue!(stdout, 
                SetBackgroundColor(Color::White),
                SetForegroundColor(Color::Black))?;

            write!(stdout, "{}", word)?;

            queue!(stdout, ResetColor)?;

            write!(stdout, " ")?;
        } else {
            write!(stdout, "{} ", word)?;
        }

        // queue!(stdout, 
        //     SetBackgroundColor(Color::White),
        //     SetForegroundColor(Color::Black))?;

        // write!(stdout, "{}", word)?;

        // queue!(stdout, ResetColor)?;

        // write!(stdout, " ")?;
        // write!(stdout, "{} ", word)?;
        idx += 1;
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

fn get_suggestions(token: String, words: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    let mut adding_suggestion: u16 = 0;

    for word in words {
        if word.starts_with(&token.to_lowercase()) {
            result.push(word.to_string());
            adding_suggestion = 1;
        } else if adding_suggestion == 1 {
            // No need to check till the end of the list
            // break;
        }
    }

    result
}

fn get_current_token(line: String, current_col: u16) -> String {
    let mut token = String::new();

    let line_count = line.chars().count() as u16;

    let mut new_col = (current_col - 1) as usize;
    let mut left_char_iter = line.chars().rev();

    // Check the left side
    let mut left_part = String::new();

    if new_col > 0 {
        let c = left_char_iter.nth((line_count as usize) - new_col); // reverse iter reaches to the current char, and discards all the right letters

        if c != None && c != Some(' ') {
            left_part.push(c.unwrap());

            loop {
                let c = left_char_iter.next();

                if c == None || c == Some(' ') {
                    break;
                }

                left_part.push(c.unwrap());
            }
        } 
    }
    // Then check the right side
    let mut right_part = String::new();
    let mut right_char_iter = line.chars();

    new_col = (current_col-1) as usize;

    if new_col < (line_count as usize) {
        let c = right_char_iter.nth(new_col);

        if c != None && c != Some(' ') {
            right_part.push(c.unwrap());

            loop {
                let c = right_char_iter.next();

                if c == None || c == Some(' ') {
                    break;
                }

                right_part.push(c.unwrap());
            }
        }
    }

    token.clear();

    loop {
        let c = left_part.pop();
        if c == None {
            break;
        }

        token.push(c.unwrap());
    }

    token.push_str(&right_part);

    token
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
