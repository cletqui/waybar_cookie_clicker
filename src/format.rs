pub fn cookies(n: f64) -> String {
    match n {
        n if n < 1_000.0 => format!("{n:.0}"),
        n if n < 1_000_000.0 => format!("{:.1}k", n / 1_000.0),
        n if n < 1_000_000_000.0 => format!("{:.1}M", n / 1_000_000.0),
        n if n < 1_000_000_000_000.0 => format!("{:.1}B", n / 1_000_000_000.0),
        n => format!("{:.1}T", n / 1_000_000_000_000.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_zero() {
        assert_eq!(cookies(0.0), "0");
    }

    #[test]
    fn formats_below_thousand() {
        assert_eq!(cookies(1.0), "1");
        assert_eq!(cookies(999.0), "999");
    }

    #[test]
    fn formats_thousands() {
        assert_eq!(cookies(1_000.0), "1.0k");
        assert_eq!(cookies(1_500.0), "1.5k");
        assert_eq!(cookies(999_900.0), "999.9k");
    }

    #[test]
    fn formats_millions() {
        assert_eq!(cookies(1_000_000.0), "1.0M");
        assert_eq!(cookies(2_500_000.0), "2.5M");
    }

    #[test]
    fn formats_billions() {
        assert_eq!(cookies(1_000_000_000.0), "1.0B");
    }

    #[test]
    fn formats_trillions() {
        assert_eq!(cookies(1_000_000_000_000.0), "1.0T");
        assert_eq!(cookies(5_000_000_000_000.0), "5.0T");
    }
}
