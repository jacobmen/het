use std::rc::Rc;

use crate::{cli::Args, sql::SqlRepository};
use anyhow::{Context, Result};
use clap::Parser;
use rusqlite::Connection;

mod cli;
mod sql;

fn main() -> Result<()> {
    let args = Args::parse();

    let connection = Rc::new(
        Connection::open(&args.db)
            .with_context(|| format!("failed to open {}", args.db.display()))?,
    );

    let sql_repository = SqlRepository::try_new(connection)?;

    Ok(())
}
