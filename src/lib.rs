//! # Yet Another Progress Bar
//!
//! This library provides lightweight tools for rendering progress indicators and related information. Unlike most
//! similar libraries, it performs no IO internally, instead providing `Display` implementations. Handling the details
//! of any particular output device is left to the user.
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
//!   let mut bar = Bar::new();
//!   print!("{}", termion::cursor::Save);
//!   for i in 0..100 {
//!     bar.set(i as f32 / 100.0);
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

pub mod prefix;

/// Indicators that communicate a proportion of progress towards a known end point
pub trait Progress: Display {
    /// Set the amount of progress
    ///
    /// `value` must be in [0, 1]. Implementations should be trivial, with any complexity deferred to the
    /// `Display` implementation.
    fn set(&mut self, value: f32);
}

/// An unusually high-resolution progress bar using Unicode block elements
///
/// # Examples
/// ```
/// # use yapb::*;
/// let mut bar = Bar::new();
/// bar.set(0.55);
/// assert_eq!(format!("[{:10}]", bar), "[█████▌    ]");
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Bar {
    progress: f32,
}

impl Bar {
    pub fn new() -> Self { Bar {
        progress: 0.0,
    }}

    pub fn get(&self) -> f32 { self.progress }
}

impl Progress for Bar {
    fn set(&mut self, value: f32) { self.progress = value; }
}

impl Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = f.width().unwrap_or(80) as u32;
        // Scale by width, rounding to nearest
        let count = width as f32 * self.progress.max(0.0).min(1.0);
        let whole = count.trunc() as u32;
        for _ in 0..whole {
            f.write_char('█')?;
        }
        let fraction = (count.fract() * 8.0).trunc() as u32;
        let fill = f.fill();
        if whole < width {
            f.write_char(match fraction {
                0 => fill,
                1 => '▏',
                2 => '▎',
                3 => '▍',
                4 => '▌',
                5 => '▋',
                6 => '▊',
                7 => '▉',
                _ => unreachable!(),
            })?;
            for _ in whole..(width - 1) {
                f.write_char(fill)?;
            }
        }
        Ok(())
    }
}

/// Indicators that animate through some number of states to indicate activity with indefinite duration
///
/// Incrementing a state by 1 advances by one frame of animation. Implementations of these two setters should only be a
/// handful of instructions, with all complexity deferred to the `Display` impl.
pub trait Spinner: Display {
    /// Set a specific state
    fn set(&mut self, value: u32);
    /// Advance the current state `count` times.
    fn step(&mut self, count: u32);
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

impl Spinner for Counter256 {
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

impl Spinner for Spinner8 {
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

impl Spinner for Counter16 {
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

impl Spinner for Spinner4 {
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

impl Spinner for Snake {
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

/// Exponential moving average, useful for computing throughput
#[derive(Debug, Copy, Clone)]
pub struct MovingAverage {
    alpha: f32,
    value: f32,
}

impl MovingAverage {
    /// `alpha` is in (0, 1] describing how responsive to be to each update
    pub fn new(alpha: f32, initial: f32) -> Self { Self { alpha, value: initial } }

    /// Update with a new sample
    pub fn update(&mut self, value: f32) {
        self.value = self.alpha * value + (1.0 - self.alpha) * self.value;
    }

    /// Get the current average value
    pub fn get(&self) -> f32 { self.value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_sanity() {
        let mut bar = Bar::new();
        assert_eq!(format!("{:10}", bar), "          ");
        bar.set(1.0);
        assert_eq!(format!("{:10}", bar), "██████████");
    }
}
