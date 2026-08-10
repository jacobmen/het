#![cfg_attr(
    test,
    allow(
        clippy::panic,
        clippy::panic_in_result_fn,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::todo,
        clippy::unreachable,
        clippy::arithmetic_side_effects,
    )
)]

use crate::{
    add_expense::{AddExpenseSummary, add_expense},
    cli::Args,
    expense_service::ExpenseService,
    retrieve_expenses::{RetrieveExpensesSummary, retrieve_expenses},
    sql::SqlRepository,
};
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use rusqlite::Connection;
use std::fs;

mod add_expense;
mod algo;
mod cli;
mod expense_service;
mod retrieve_expenses;
mod sql;

#[cfg(test)]
mod test_util;

fn main() -> Result<()> {
    let args = Args::parse();

    let connection = Connection::open(&args.db)
        .with_context(|| format!("failed to open {}", args.db.display()))?;

    let sql_repository = SqlRepository::try_new(connection)?;
    let expense_service = ExpenseService::new(sql_repository);

    expense_service.create_expense_table()?;

    match args.command {
        cli::Commands::Add { file, amount } => {
            let expense_date = Local::now().naive_local().date();
            let file_contents = fs::read(&file)
                .with_context(|| format!("failed to read expense file `{}`", file.display()))?;

            let summary = add_expense(
                &expense_service,
                args.dryrun,
                &file,
                &file_contents,
                expense_date,
                amount,
            )
            .with_context(|| format!("failed to create expense for `{}`", file.display()))?;

            match summary {
                AddExpenseSummary::DryRun {
                    name,
                    file_data_type,
                    unit_amount,
                } => {
                    println!("Dryrun add");
                    println!("\texpense=`{}`", name.0);
                    println!("\tfile_data_type=`{}`", file_data_type.0);
                    println!("\tunit_amount=`{}`", unit_amount.0);
                }
                AddExpenseSummary::Created {
                    name,
                    file_data_type,
                    expense_date,
                    unit_amount,
                    compressed_data_size,
                } => {
                    println!("Created expense");
                    println!("\texpense=`{}`", name.0);
                    println!("\tfile_data_type=`{}`", file_data_type.0);
                    println!("\texpense_date=`{}`", expense_date.format("%Y-%m-%d"));
                    println!("\tunit_amount=`{}`", unit_amount.0);
                    println!("\tcompressed_data_size=`{compressed_data_size}`");
                }
            }
        }
        cli::Commands::Retrieve { amount, out } => {
            let summary = retrieve_expenses(&expense_service, args.dryrun, amount)
                .with_context(|| "failed to retrieve expenses".to_string())?;

            match summary {
                RetrieveExpensesSummary::NoMatch => println!("no expenses reach {amount}"),
                RetrieveExpensesSummary::DryRun {
                    target_unit_amount,
                    expenses,
                } => {
                    println!("target unit amount: {target_unit_amount}");
                    for (name, unit_amount) in expenses {
                        println!("{}: {}", name.0, unit_amount.0);
                    }
                }
                RetrieveExpensesSummary::Retrieved { files } => {
                    for file in &files {
                        let file_write_path = out.join(&file.name);
                        fs::write(&file_write_path, &file.contents).with_context(|| {
                            format!(
                                "failed to write expense file to {}",
                                file_write_path.display()
                            )
                        })?;
                    }

                    let expense_ids = files.iter().map(|file| file.id).collect::<Vec<_>>();
                    expense_service.mark_expenses_as_deleted(&expense_ids)?;
                }
            }
        }
    }

    Ok(())
}
