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

use std::{fs, path::Path};

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use chrono::Local;

const EXPENSE_A: &str = "expense_a.pdf";
const EXPENSE_B: &str = "expense_b.png";
const EXPENSE_C: &str = "expense_c.jpeg";

const EXPENSE_A_BYTES: &[u8] = b"pdf data for a";
const EXPENSE_B_BYTES: &[u8] = b"png data for b";
const EXPENSE_C_BYTES: &[u8] = b"jpeg data for c";

fn add_expense(db: &Path, file: &Path, amount: &str) -> Result<()> {
    Command::cargo_bin("het")?
        .arg("--db")
        .arg(db)
        .arg("add")
        .arg("--file")
        .arg(file)
        .arg("--amount")
        .arg(amount)
        .assert()
        .success();
    Ok(())
}

fn retrieve_expenses(db: &Path, out: &Path, amount: &str) -> Result<()> {
    Command::cargo_bin("het")?
        .arg("--db")
        .arg(db)
        .arg("retrieve")
        .arg("--out")
        .arg(out)
        .arg(amount)
        .assert()
        .success();
    Ok(())
}

#[test]
fn dryrun_add_prints_and_persists_nothing() -> Result<()> {
    let temp = TempDir::new()?;
    let db = temp.child("het.db");
    let expense = temp.child("dryrun.pdf");
    expense.write_binary(b"ignored")?;

    Command::cargo_bin("het")?
        .arg("--db")
        .arg(db.path())
        .arg("--dryrun")
        .arg("add")
        .arg("--file")
        .arg(expense.path())
        .arg("--amount")
        .arg("10")
        .assert()
        .success()
        .stdout("Dryrun add\n\texpense=`dryrun`\n\tfile_data_type=`pdf`\n\tunit_amount=`1000`\n");
    Command::cargo_bin("het")?
        .arg("--db")
        .arg(db.path())
        .arg("retrieve")
        .arg("--out")
        .arg(temp.path())
        .arg("10")
        .assert()
        .success()
        .stdout("no expenses reach $10.00\n");
    Ok(())
}

#[test]
fn add_and_retrieve_expenses_end_to_end() -> Result<()> {
    let temp = TempDir::new()?;
    let db = temp.child("het.db");

    let expense_a = temp.child(EXPENSE_A);
    expense_a.write_binary(EXPENSE_A_BYTES)?;
    let expense_b = temp.child(EXPENSE_B);
    expense_b.write_binary(EXPENSE_B_BYTES)?;
    let expense_c = temp.child(EXPENSE_C);
    expense_c.write_binary(EXPENSE_C_BYTES)?;

    add_expense(db.path(), expense_a.path(), "100")?;
    add_expense(db.path(), expense_b.path(), "50")?;
    add_expense(db.path(), expense_c.path(), "25.50")?;

    let expense_date = Local::now().naive_local().date();

    let out_dir = temp.child("retrieved");
    out_dir.create_dir_all()?;

    retrieve_expenses(db.path(), out_dir.path(), "115")?;

    let extracted_a = out_dir.join(format!("{expense_date}_10000_expense_a.pdf"));
    let extracted_c = out_dir.join(format!("{expense_date}_2550_expense_c.jpeg"));
    let extracted_b = out_dir.join(format!("{expense_date}_5000_expense_b.png"));

    assert_eq!(EXPENSE_A_BYTES.to_vec(), fs::read(&extracted_a)?);
    assert_eq!(EXPENSE_C_BYTES.to_vec(), fs::read(&extracted_c)?);
    assert!(!extracted_b.exists());

    Command::cargo_bin("het")?
        .arg("--db")
        .arg(db.path())
        .arg("retrieve")
        .arg("--out")
        .arg(out_dir.path())
        .arg("100")
        .assert()
        .success()
        .stdout("no expenses reach $100.00\n");

    retrieve_expenses(db.path(), out_dir.path(), "40")?;

    assert_eq!(EXPENSE_B_BYTES.to_vec(), fs::read(&extracted_b)?);

    Ok(())
}
