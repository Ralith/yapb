//! Helpers to display compact human-readable numbers

use std::fmt::{self, Write, Display};

/// Given an exact value `x`, return the same value scaled to the nearest lesser binary prefix, and the prefix in
/// question.
pub fn binary(x: f64) -> (f64, Option<&'static str>) {
    const TABLE: [&'static str; 8] = [
        "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi", "Yi"
    ];

    let mut divisor = 1024.0;
    if x < divisor { return (x, None); }
    let (last, most) = TABLE.split_last().unwrap();
    for prefix in most {
        let next = divisor * 1024.0;
        if next > x {
            return (x / divisor, Some(prefix));
        }
        divisor = next;
    }
    (x / divisor, Some(last))
}

/// Given an exact value `x`, return the same value scaled to the nearest lesser SI prefix, and the prefix in question.
pub fn si(x: f64) -> (f64, Option<&'static str>) {
    const SMALL: [&'static str; 8] = [
        "m", "µ", "n", "p", "f", "a", "z", "y",
    ];
    const LARGE: [&'static str; 8] = [
        "k", "M", "G", "T", "P", "E", "Z", "Y"
    ];

    if x.abs() < 1.0 {
        let mut divisor = 1e-3;
        let (last, most) = SMALL.split_last().unwrap();
        for prefix in most {
            let next = divisor * 1e-3;
            if next < x.abs() {
                return (x / divisor, Some(prefix));
            }
            divisor = next;
        }
        (x / divisor, Some(last))
    } else if x.abs() < 1e3 {
        (x, None)
    } else {
        let mut divisor = 1e3;
        let (last, most) = LARGE.split_last().unwrap();
        for prefix in most {
            let next = divisor * 1e3;
            if next > x.abs() {
                return (x / divisor, Some(prefix));
            }
            divisor = next;
        }
        (x / divisor, Some(last))
    }
}

/// Format `value` compactly with exactly `figures` significant figures
///
/// For compactness, exponential notation is used for values that are larger than `1eN` or smaller than `1e-N`.
pub fn fmt_sigfigs(f: &mut fmt::Formatter, value: f64, figures: usize) -> fmt::Result {
    if value == 0.0 { return write!(f, "{:.*}", figures - 1, 0.0); }
    let log = value.abs().log10() as isize;
    if log < 0 || log >= figures as isize {
        write!(f, "{:.*e}", figures - 1, value)
    } else {
        write!(f, "{:.*}", figures - (log + 1) as usize, value)
    }
}

/// Helper struct to format a float with `format_sigfigs`
#[derive(Debug, Copy, Clone)]
pub struct SigFigs(pub f64, pub usize);
impl Display for SigFigs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_sigfigs(f, self.0, self.1)
    }
}


/// Helper struct to compactly format a value with a binary unit prefix
///
/// If the provided value is equal to 0 or is in [1e-2, 1e28), this will produce at most 7 ASCII characters.
///
/// # Examples
/// ```
/// assert_eq!(format!("{}B/s", yapb::prefix::Binary(12345.0)), "12.1 KiB/s");
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Binary(pub f64);
impl Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 1.0 && self.0 >= 1e-2 {
            write!(f, "{:.2} ", self.0)?;
        } else {
            let (value, prefix) = binary(self.0);
            if value >= 1000.0 {
                fmt_sigfigs(f, value, 4)?;
            } else {
                fmt_sigfigs(f, value, 3)?;
            }
            f.write_char(' ')?;
            if let Some(prefix) = prefix {
                f.write_str(prefix)?;
            }
        }
        Ok(())
    }
}

/// Helper struct to compactly format a value with a SI unit prefix
///
/// If the provided value is in [1e-24, 1e28), this will produce at most 6 ASCII characters.
#[derive(Debug, Copy, Clone)]
pub struct Scientific(pub f64);
impl Display for Scientific {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (value, prefix) = si(self.0);
        fmt_sigfigs(f, value, 3)?;
        f.write_char(' ')?;
        if let Some(prefix) = prefix {
            f.write_str(prefix)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_prefixes() {
        assert_eq!(binary(2.0 * 1024.0), (2.0, Some("Ki")));
        assert_eq!(binary(2.0 * 1024.0 * 1024.0), (2.0, Some("Mi")));
    }

    #[test]
    fn si_prefixes() {
        assert_eq!(si(2e3), (2.0, Some("k")));
        assert_eq!(si(2e6), (2.0, Some("M")));
    }

    #[test]
    fn sigfig_sanity() {
        assert_eq!(SigFigs(1.0, 1).to_string(), "1");
        assert_eq!(SigFigs(1.0, 2).to_string(), "1.0");
        assert_eq!(SigFigs(0.1, 1).to_string(), "1e-1");
        assert_eq!(SigFigs(0.1, 2).to_string(), "1.0e-1");
        assert_eq!(SigFigs(10.0, 1).to_string(), "1e1");
        assert_eq!(SigFigs(10.0, 2).to_string(), "10");
    }

    #[test]
    fn binary_fmt() {
        assert_eq!(Binary(0.0).to_string(), "0.00 ");
        assert_eq!(Binary(0.001).to_string(), "1.00e-3 ");
        assert_eq!(Binary(0.01).to_string(), "0.01 ");
        assert_eq!(Binary(1023.0).to_string(), "1023 ");
        assert_eq!(Binary(2.0 * 1024.0).to_string(), "2.00 Ki");
        assert_eq!(Binary(1023.0 * 1024.0).to_string(), "1023 Ki");
    }

    #[test]
    fn scientific_fmt() {
        assert_eq!(Scientific(0.001).to_string(), "1.00 m");
        assert_eq!(Scientific(0.01).to_string(), "10.0 m");
        assert_eq!(Scientific(2.0 * 1000.0).to_string(), "2.00 k");
        assert_eq!(Scientific(999.0 * 1000.0).to_string(), "999 k");
    }
}
