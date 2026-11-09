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
    app_config::get_app_config,
    cli::Args,
    expense_service::ExpenseService,
    retrieve_expenses::{RetrieveExpensesSummary, retrieve_expenses},
    sql::SqlRepository,
    xdg_dirs::{XdgVar, config_home_dir, data_home_dir, expand_tilde},
};
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use rusqlite::{Connection, OpenFlags};
use std::{fs, path::PathBuf, process::exit};

mod add_expense;
mod algo;
mod app_config;
mod cli;
mod expense_service;
mod retrieve_expenses;
mod sql;
mod xdg_dirs;

#[cfg(test)]
mod test_util;

fn main() -> Result<()> {
    let args = Args::parse();

    let Ok(home_env_var) = std::env::var("HOME") else {
        println!("$HOME is unset; required environment variable");
        exit(1);
    };

    let expense_service = open_expense_service(&home_env_var, args.db)?;

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

fn open_expense_service(
    home_env_var: &str,
    db_override: Option<PathBuf>,
) -> Result<ExpenseService<SqlRepository>> {
    let xdg_config_home =
        std::env::var("XDG_CONFIG_HOME").map_or(XdgVar::Missing, |value| XdgVar::Present { value });

    let config_file_path = config_home_dir(&xdg_config_home, home_env_var).join("config.toml");
    let config_file_content = if config_file_path.try_exists().with_context(|| {
        format!(
            "failed to check for existence of {}",
            config_file_path.display()
        )
    })? {
        Some(fs::read_to_string(&config_file_path)?)
    } else {
        None
    };

    let app_config = match config_file_content {
        Some(content) => Some(get_app_config(&content)?),
        None => None,
    };

    let xdg_data_home =
        std::env::var("XDG_DATA_HOME").map_or(XdgVar::Missing, |value| XdgVar::Present { value });

    let db_path = match (db_override, app_config) {
        (Some(db_path), _) => db_path,
        (None, Some(app_config)) => app_config.db_path,
        (None, None) => data_home_dir(&xdg_data_home, home_env_var).join("het.db"),
    };

    let db_path = expand_tilde(&db_path, home_env_var);

    let connection = Connection::open_with_flags(
        &db_path,
        OpenFlags::default() | OpenFlags::SQLITE_OPEN_EXRESCODE,
    )
    .with_context(|| format!("failed to open {}", db_path.display()))?;

    let sql_repository = SqlRepository::try_new(connection)?;
    let expense_service = ExpenseService::new(sql_repository);

    expense_service.create_expense_table()?;

    Ok(expense_service)
}
