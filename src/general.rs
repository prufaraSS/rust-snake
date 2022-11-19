pub mod error_handling {
    use std::io::{stderr};
    use std::fs::File;
    use crossterm::{
        execute, Result,
        style::Print,
        style::Stylize,
        event::{
            read,
            Event
        }
    };
    use crate::free_window;

    fn cry(about:&str) {
        execute!(
            stderr(),
            Print(about.red()),
        ).handle();
        loop { //wait for input before panic!!
            if let Ok(res) = read() {
                if let Event::Key(_) = res {break}
            } else {break}
        };
        free_window(0,0);
    }

    pub trait ReadHandling {
        fn handle_read(self,filename:&str) -> String;
    }

    impl ReadHandling for std::io::Result<String> {
        fn handle_read(self,filename:&str) -> String {
            match self {
                Ok(t) => t,
                Err(_) => {
                    cry(&format!("Can't read file: \"{}\"",filename));
                    panic!();
                }
            }
        }
    }

    pub trait OpenHandling {
        fn handle_open(self,filename:&str) -> File;
    }

    impl OpenHandling for std::io::Result<File> {
        fn handle_open(self,filename:&str) -> File {
            match self {
                Ok(f) => f,
                Err(_) => {
                    cry(&format!("Can't open file: \"{}\"",filename));
                    panic!();
                }
            }
        }
    }

    pub trait WriteHandling {
        fn handle_write(self);
    }

    impl WriteHandling for std::io::Result<usize> {
        fn handle_write(self) {
            match self {
                Ok(_) => (),
                Err(_) => {
                    cry("Can't write data in file");
                    panic!();
                }
            }
        }
    }

    pub trait TerminalHandling {
        fn handle(self);
    }

    impl TerminalHandling for Result<()> {
        fn handle(self) { //this error is caused by trying to write in buffer
            self.expect("ERROR: Unable to access terminal buffer.");
        } //so you can't cry about it
    }
}

pub mod graphics {
    use std::io::stdout;
    use crossterm::{
        queue,
        cursor::{MoveTo},
        style::{Print}
    };
    use crate::general::error_handling::TerminalHandling;

    pub fn draw_simple_ascii_picture(ascii:&String,x:u16,y:u16) {
        let mut pos = 0u16;
        let mut ascii = ascii.lines();
        while let Some(line) = ascii.next() {
            queue!(
                stdout(),
                MoveTo(x,y+pos),
                Print(line)
            ).handle();
            pos += 1;
        }
    }
}

pub mod input {
    use crossterm::{
        event::{
            poll,
            read,
            Event,
            MouseEventKind,
            MouseButton,
            KeyCode
        }
    };
    
    use std::time::Duration;
    
    use crate::{
        Snake,
        Direction,
        DirectionFunctionality
    };

    pub struct Cursor {
        pub x:u16,
        pub y:u16,
        pub hover:u8
    }
    
    pub enum InputResult {
        Continue,
        Click,
        Draw,
        Abort
    }

    fn receive_input(polltime:Duration) -> Result<Event,InputResult> {
        if let Duration::MAX = polltime {} //don't wait for input forever, just read()
        else {
            match poll(polltime) {
                Ok(result) => if let false = result {
                    return Err(InputResult::Continue)
                },
                Err(_) => return Err(InputResult::Abort)
            }
        }
        
        match read() {
            Ok(data) => return Ok(data),
            Err(_) => return Err(InputResult::Abort)
        }
    }
    
    pub fn cursor_input(cursor:&mut Cursor,polltime:Duration) -> InputResult {
        if polltime.is_zero() == true { return InputResult::Continue }; //you can't make input faster than 0.000s
        let input = match receive_input(polltime) {
            Ok(data) => data,
            Err(reason) => return reason
        };
        match &input {
            Event::Mouse(event) => {
                match event.kind {
                    MouseEventKind::Moved => {
                        cursor.x = event.column;
                        cursor.y = event.row;
                    },
                    MouseEventKind::Down(button) => {
                        if let MouseButton::Left = button {
                            return InputResult::Click
                        }
                    },
                    MouseEventKind::Drag(button) => {
                        cursor.x = event.column;
                        cursor.y = event.row;
                        if let MouseButton::Left = button {
                            return InputResult::Draw
                        }
                    }
                    _ => ()
                }
            },
            Event::Key(event) => {
                match event.code {
                    KeyCode::Right => cursor.x += 1,
                    KeyCode::Left => cursor.x -= 1,
                    KeyCode::Up => cursor.y -= 1,
                    KeyCode::Down => cursor.y += 1,
                    KeyCode::Enter => return InputResult::Click,
                    KeyCode::Esc => return InputResult::Abort,
                    _ => ()
                }
            },
            _ => ()
        }
        InputResult::Continue
    } //shrinked it so whole function perfectly fits my monitor pog (edit: no more :/)

    pub fn game_input(snake:&mut Snake,speed:Duration) -> InputResult {
        let input = match receive_input(speed) {
            Ok(data) => data,
            Err(reason) => return reason
        };
        match &input {
            Event::Key(event) => {
                if let KeyCode::Esc = event.code {return InputResult::Abort}
                let dir = match event.code {
                    KeyCode::Right => Direction::Right,
                    KeyCode::Left => Direction::Left,
                    KeyCode::Up => Direction::Up,
                    KeyCode::Down => Direction::Down,
                    _ => snake.direction.copy()
                };
                snake.last_input = dir.copy();
                if dir.is_opposite_of(&snake.prev_move) == false { snake.direction = dir }
            },
            _ => ()
        }
        InputResult::Continue
    }
}