use std::io::{self, Write};

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