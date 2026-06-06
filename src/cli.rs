use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
        #[arg(short, long, value_parser = amount_in_range)]
        amount: f64,
    },

    /// Retrieve health expenses summing as close as possible (and at minimum) to the provided value
    Retrieve {
        /// Target amount for expenses to reach
        #[arg(value_parser = amount_in_range)]
        amount: f64,

        /// Output path of expense files
        #[arg(short, long, value_name = "OUT_PATH", default_value = ".")]
        out: PathBuf,
    },
}

fn amount_in_range(s: &str) -> Result<f64, String> {
    let amount: f64 = s.parse().map_err(|_| format!("`{s}` isn't a number"))?;

    if amount > 0.0 {
        Ok(amount)
    } else {
        Err(format!("`{amount}` is less than $0.00"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_amount() {
        assert!(amount_in_range("10").is_ok_and(|amount| amount == 10.0));
    }

    #[test]
    fn test_negative_amount() {
        assert!(amount_in_range("-1").is_err());
    }

    #[test]
    fn test_non_numeric_input() {
        assert!(amount_in_range("abcd").is_err());
    }
}
