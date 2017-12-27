extern crate yapb;
extern crate termion;

use std::{thread, time};
use std::io::{self, Write};

use yapb::*;

fn main() {
    let mut s256 = Counter256::new();
    let mut s16 = Counter16::new();
    let mut s8 = Spinner8::new();
    let mut s4 = Spinner4::new();
    let mut snake = Snake::new();
    let mut bar = Bar::new();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    write!(stdout, "{}", termion::cursor::Save).unwrap();
    for i in 0..1000 {
        s4.set(i >> 2);
        s8.set(i >> 1);
        s16.set(i >> 2);
        s256.set(i >> 1);
        snake.set(i);
        bar.set(i * (u32::max_value() / 1000));

        let (width, _) = termion::terminal_size().unwrap();
        write!(stdout, "{}{}{} {} {} {} {} [{:width$}]",
               termion::clear::AfterCursor, termion::cursor::Restore,
               s4, s8, s16, s256, snake, bar, width = width as usize - 12).unwrap();
        stdout.flush().unwrap();
        thread::sleep(time::Duration::from_millis(50));
    }
}
