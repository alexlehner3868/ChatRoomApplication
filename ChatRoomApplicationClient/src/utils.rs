use std::io::{self, Write, stdout};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

// Function to erase the last line that was printed to the terminal
pub fn erase_last_line() {
    print!("\x1b[1A"); // Cursor moves up a line in the terminal
    print!("\x1b[2K"); // Clear the line that the cursor is now on
    io::stdout().flush().unwrap();
}

// Function to clear the current line 
pub fn erase_current_line() {
    print!("\r\x1B[K");
    io::stdout().flush().unwrap();
}

// Function to print '*' to hide passwords
pub fn read_sensitive_information() -> std::io::Result<String> {
    // Get every key stroke immediately and dont print
    enable_raw_mode()?;  

    let mut stdout = stdout();
    let mut password = String::new();

    loop {
        // Read keyboard input
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                // User finished typing
                KeyCode::Enter => {
                    disable_raw_mode()?; // Reset env
                    println!();
                    return Ok(password); // Return password
                }
                KeyCode::Char(c) => {
                    password.push(c); // Store the char typed
                    print!("*");
                    stdout.flush()?; // Print a "*"
                }
                KeyCode::Backspace => {
                    if password.pop().is_some() { // Remove last char from password
                        print!("\x08 \x08"); // Remove the last printed *
                        stdout.flush()?;
                    }
                }
                _ => {}
            }
        }
    }
}