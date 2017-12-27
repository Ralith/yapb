//! # Yet Another Progress Bar
//!
//! This library provides lightweight tools for rendering Unicode progress indicators. Unlike most similar libraries, it
//! performs no IO internally, instead providing `Display` implementations. Handling the details of any particular
//! output device is left to the user.
//!
//! # Examples
//! The `termion` crate can be used to implement good behavior on an ANSI terminal:
//!
//! ```no_run
//! extern crate yapb;
//! extern crate termion;
//! use std::{thread, time};
//! use std::io::{self, Write};
//! use yapb::{Bar, Progress};
//!
//! fn main() {
//!   let mut bar = yapb::Bar::new();
//!   print!("{}", termion::cursor::Save);
//!   for i in 0..100 {
//!     bar.set(i * (u32::max_value() / 100));
//!     let (width, _) = termion::terminal_size().unwrap();
//!     print!("{}{}[{:width$}]",
//!            termion::clear::AfterCursor, termion::cursor::Restore,
//!            bar, width = width as usize - 2);
//!     io::stdout().flush().unwrap();
//!     thread::sleep(time::Duration::from_millis(100));
//!   }
//! }
//! ```

use std::fmt::{self, Write, Display};

/// Types that can display a progress state
///
/// Progress values are unrestricted 32-bit integers. The "finished" state is `u32::max_value()`. Implementations of
/// these two setters should only be a handful of instructions, with all complexity deferred to the `Display` impl.
pub trait Progress: Display {
    /// Set the amount of progress to an absolute value.
    fn set(&mut self, value: u32);
    /// Advance the current state `count` times.
    fn step(&mut self, count: u32);
}

/// A high-resolution progress bar using block elements
///
/// Note that the associated `Progress::step` implementation saturates.
///
/// # Examples
/// ```
/// # use yapb::*;
/// let mut bar = Bar::new();
/// bar.set(55 * (u32::max_value() / 100));
/// assert_eq!(format!("[{:10}]", bar), "[█████▌    ]");
/// ```
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Bar {
    progress: u32,
}

impl Bar {
    pub fn new() -> Self { Bar {
        progress: 0,
    }}

    pub fn get(&self) -> u32 { self.progress }
}

impl Progress for Bar {
    fn set(&mut self, state: u32) { self.progress = state; }
    fn step(&mut self, count: u32) { self.progress.saturating_add(count); }
}

impl Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(80) as u32;
        // Scale by width, rounding to nearest
        let approx = (((self.progress as u64) * (width as u64 * 8) + u32::max_value() as u64) >> 32) as u32;
        let whole = approx / 8;
        for _ in 0..whole {
            f.write_char('█')?;
        }
        let fraction = approx % 8;
        let fill = f.fill();
        if fraction != 0 {
            f.write_char(match fraction {
                1 => '▏',
                2 => '▎',
                3 => '▍',
                4 => '▌',
                5 => '▋',
                6 => '▊',
                7 => '▉',
                _ => unreachable!(),
            })?;
        } else if whole < width {
            f.write_char(fill)?;
        }
        for _ in whole..(width - 1) {
            f.write_char(fill)?;
        }
        Ok(())
    }
}

/// A spinner that cycles through 256 states by counting in binary using braille
///
/// # Examples
/// ```
/// # use yapb::*;
/// let mut spinner = Counter256::new();
/// assert_eq!(format!("{}", spinner), "⠀");
/// spinner.step(0x0F);	
/// assert_eq!(format!("{}", spinner), "⡇");
/// spinner.step(0xF0);
/// assert_eq!(format!("{}", spinner), "⣿");
/// ```
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Counter256 {
    state: u8
}

impl Counter256 {
    pub fn new() -> Self { Self { state: 0 } }
}

impl Progress for Counter256 {
    fn set(&mut self, state: u32) { self.state = state as u8; }
    fn step(&mut self, count: u32) { self.state = self.state.wrapping_add(count as u8); }
}

fn braille_binary(value: u8) -> char {
    // Rearrange bits for consistency
    let value = (value & 0b10000111)
        | ((value & 0b00001000) << 3)
        | ((value & 0b01110000) >> 1);
    unsafe { ::std::char::from_u32_unchecked(0x2800 + value as u32) }
}

impl Display for Counter256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(braille_binary(self.state))
    }
}

/// A spinner that cycles through 8 states with a single spinning braille dot
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Spinner8 {
    state: u8
}

const SPINNER8_STATES: [char; 8] = ['⡀', '⠄', '⠂', '⠁', '⠈', '⠐', '⠠', '⢀'];

impl Spinner8 {
    pub fn new() -> Self { Self { state: 0 } }
}

impl Progress for Spinner8 {
    fn set(&mut self, state: u32) { self.state = state as u8 % SPINNER8_STATES.len() as u8; }
    fn step(&mut self, count: u32) { self.state = self.state.wrapping_add(count as u8) % SPINNER8_STATES.len() as u8; }
}

impl Display for Spinner8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(*unsafe { SPINNER8_STATES.get_unchecked(self.state as usize) })
    }
}

/// A spinner that cycles through 16 states by counting in binary using block elements
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Counter16 {
    state: u8
}

const COUNTER16_STATES: [char; 16] = [' ', '▘', '▖', '▌', '▝', '▀', '▞', '▛', '▗', '▚', '▄', '▙', '▐', '▜', '▟', '█'];

impl Counter16 {
    pub fn new() -> Self { Self { state: 0 } }
}

impl Progress for Counter16 {
    fn set(&mut self, state: u32) { self.state = state as u8 % COUNTER16_STATES.len() as u8; }
    fn step(&mut self, count: u32) { self.state = self.state.wrapping_add(count as u8) % COUNTER16_STATES.len() as u8; }
}

impl Display for Counter16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(*unsafe { COUNTER16_STATES.get_unchecked(self.state as usize) })
    }
}

/// A spinner that cycles through 4 states with a single spinning block element
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Spinner4 {
    state: u8
}

const SPINNER4_STATES: [char; 4] = ['▖', '▘', '▝', '▗'];

impl Spinner4 {
    pub fn new() -> Self { Self { state: 0 } }
}

impl Progress for Spinner4 {
    fn set(&mut self, state: u32) { self.state = state as u8 % SPINNER4_STATES.len() as u8; }
    fn step(&mut self, count: u32) { self.state = self.state.wrapping_add(count as u8) % SPINNER4_STATES.len() as u8; }
}

impl Display for Spinner4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(*unsafe { SPINNER4_STATES.get_unchecked(self.state as usize) })
    }
}

/// A spinner that cycles through many states with a snake made of 1-6 braille dots
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Clone)]
pub struct Snake {
    state: u32
}

impl Snake {
    pub fn new() -> Self { Self { state: 0 } }
}

impl Progress for Snake {
    fn set(&mut self, state: u32) { self.state = state; }
    fn step(&mut self, count: u32) { self.state = self.state.wrapping_add(count); }
}

impl Display for Snake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const WOBBLE: u32 = 5;
        let length = (((self.state % (2*WOBBLE)) as i32 - (WOBBLE as i32)).abs() + 1) as u32;
        let bits = !(0xFFu8 << length);
        let position = (WOBBLE * (self.state / (2*WOBBLE)) + (self.state % (2*WOBBLE)).saturating_sub(WOBBLE)) as u8;
        let snake = bits.rotate_right(position as u32);
        // Reverse most significant nybble
        let value = snake & 0xF
            | ((snake & 0b10000000) >> 3)
            | ((snake & 0b01000000) >> 1)
            | ((snake & 0b00100000) << 1)
            | ((snake & 0b00010000) << 3);
        f.write_char(braille_binary(value))
    }
}

