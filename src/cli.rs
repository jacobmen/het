use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rusty_money::{Money, Round, iso};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Override default DB path
    #[arg(long, value_name = "DB", default_value = "het.db")]
    pub db: PathBuf,

    /// Dry-run operation without any state changes
    #[arg(short, long)]
    pub dryrun: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a health expense to the tracker
    Add {
        /// Expense file to ingest
        #[arg(short, long)]
        file: PathBuf,

        /// Value of expense
        #[arg(short, long, value_parser = parse_money)]
        amount: Money<'static, iso::Currency>,
    },

    /// Retrieve health expenses summing as close as possible (and at minimum) to the provided value
    Retrieve {
        /// Target amount for expenses to reach
        #[arg(value_parser = parse_money)]
        amount: Money<'static, iso::Currency>,

        /// Output path of expense files
        #[arg(short, long, value_name = "OUT_PATH", default_value = ".")]
        out: PathBuf,
    },
}

fn parse_money(s: &str) -> Result<Money<'static, iso::Currency>, String> {
    let money = Money::from_str(s, iso::USD).map_err(|_| format!("`{s}` isn't a number"))?;
    if !money.is_positive() {
        return Err(format!("`{s}` is less than $0.00"));
    }

    let money = money.round(2, Round::HalfUp);
    // `to_minor_units` yields 0 when the value exceeds i64 range.
    // Round trip to detect overflow
    if Money::from_minor(money.to_minor_units(), iso::USD) != money {
        return Err(format!("`{s}` is too large"));
    }

    Ok(money)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_amount() {
        assert!(parse_money("10").is_ok_and(|amount| amount == Money::from_minor(1000, iso::USD)));
        assert!(parse_money("5.25").is_ok_and(|amount| amount == Money::from_minor(525, iso::USD)));
    }

    #[test]
    fn test_rounding_to_cent() {
        let amount = parse_money("10.005").unwrap();
        assert_eq!(Money::from_minor(1001, iso::USD), amount);
        assert_eq!(1001, amount.to_minor_units());
    }

    #[test]
    fn test_amount_too_large() {
        assert!(parse_money("100000000000000000000").is_err());
        assert!(parse_money("92233720368547758.08").is_err());
        assert!(parse_money("92233720368547758.07").is_ok());
    }

    #[test]
    fn test_negative_amount() {
        assert!(parse_money("-1").is_err());
    }

    #[test]
    fn test_non_numeric_input() {
        assert!(parse_money("abcd").is_err());
    }
}
