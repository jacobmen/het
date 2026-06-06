use std::path::PathBuf;

use crate::{
    add_expense::add_expense,
    cli::Args,
    expense_service::ExpenseService,
    sql::{Repository, SqlRepository},
};
use anyhow::{Context, Result};
use clap::Parser;
use rusqlite::Connection;

mod add_expense;
mod cli;
mod expense_service;
mod sql;

fn main() -> Result<()> {
    let args = Args::parse();

    let connection = Connection::open(&args.db)
        .with_context(|| format!("failed to open {}", args.db.display()))?;

    let sql_repository = SqlRepository::try_new(connection)?;
    let expense_service = ExpenseService::new(sql_repository);

    expense_service.create_expense_table()?;

    match args.command {
        cli::Commands::Add { file, amount } => {
            add_expense(&expense_service, args.dryrun, &file, amount)
                .with_context(|| format!("failed to create expense for `{}`", file.display()))?
        }
        cli::Commands::Retrieve { amount, out } => {
            retrieve_expenses(&expense_service, args.dryrun, amount, &out)
                .with_context(|| format!("failed to retrieve expenses"))?
        }
    }

    Ok(())
}

fn retrieve_expenses<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    amount: f64,
    out_path: &PathBuf,
) -> Result<()> {
    Ok(())
}
