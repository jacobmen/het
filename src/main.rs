use std::{
    fs::{self},
    path::Path,
};

use crate::{
    cli::Args,
    expense_service::{
        ExpenseFileData, ExpenseName, ExpenseService, ExpenseUnitAmount, FileDataType,
    },
    sql::{Repository, SqlRepository},
};
use anyhow::{Context, Result, anyhow};
use chrono::Local;
use clap::Parser;
use rusqlite::Connection;

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
        cli::Commands::Retrieve { amount, out } => todo!(),
    }

    Ok(())
}

fn add_expense<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    file_path: &Path,
    input_amount: f64,
) -> Result<()> {
    let file_name = file_path
        .file_name()
        .ok_or(anyhow!(
            "path doesn't contain file name: {}",
            file_path.to_string_lossy()
        ))?
        .to_string_lossy();

    let (expense_name, file_data_type) = file_name.split_once(".").ok_or(anyhow!(
        "file name doesn't contain delimiter `.`: {}",
        file_name
    ))?;

    if expense_name.is_empty() {
        return Err(anyhow!("expense name is empty"));
    } else if file_data_type.is_empty() {
        return Err(anyhow!("file data type is empty"));
    }

    let unit_amount = (input_amount * 100.0).round() as i64;

    if dryrun {
        println!("Dryrun add");
        println!("\texpense=`{}`", expense_name);
        println!("\tfile_data_type=`{}`", file_data_type);
        println!("\tunit_amount=`{}`", unit_amount);
        return Ok(());
    }

    let compressed_file_data = ExpenseFileData(lzma::compress(&fs::read(file_path)?, 9)?);
    let expense_date = Local::now().naive_local().date();

    expense_service.create_new_expense(
        &ExpenseName(expense_name.to_string()),
        &FileDataType(file_data_type.to_string()),
        &expense_date,
        ExpenseUnitAmount(unit_amount),
        &compressed_file_data,
    )?;

    println!("Created expense");
    println!("\texpense=`{}`", expense_name);
    println!("\tfile_data_type=`{}`", file_data_type);
    println!(
        "\texpense_date=`{}`",
        expense_date.format("%Y-%m-%d").to_string()
    );
    println!("\tunit_amount=`{}`", unit_amount);
    println!("\tcompressed_data_size=`{}`", compressed_file_data.0.len());

    Ok(())
}
