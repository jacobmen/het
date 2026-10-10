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

use crate::common::{EXPENSE_A, EXPENSE_A_BYTES, het};

mod common;

fn run_add(cmd: &mut Command, file: &Path, amount: &str) {
    cmd.arg("add")
        .arg("--file")
        .arg(file)
        .arg("--amount")
        .arg(amount)
        .assert()
        .success();
}

fn run_retrieve(cmd: &mut Command, out: &Path, amount: &str) {
    cmd.arg("retrieve")
        .arg("--out")
        .arg(out)
        .arg(amount)
        .assert()
        .success();
}

#[test]
fn config_db_path_used_for_add_and_retrieve() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    let db_path = temp.child("confdb/het.db");
    temp.child("confdb").create_dir_all()?;
    config_home
        .child("het/config.toml")
        .write_str(&format!("db_path = '{}'", db_path.path().display()))?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path());
    run_add(&mut cmd, expense.path(), "100");

    assert!(db_path.path().exists());
    assert!(!temp.child(".local/share/het/het.db").path().exists());

    let out_dir = temp.child("out");
    out_dir.create_dir_all()?;
    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path());
    run_retrieve(&mut cmd, out_dir.path(), "100");

    let expense_date = Local::now().naive_local().date();
    let extracted = out_dir.join(format!("{expense_date}_10000_expense_a.pdf"));
    assert_eq!(EXPENSE_A_BYTES.to_vec(), fs::read(&extracted)?);

    Ok(())
}

#[test]
fn config_db_path_found_in_home_config_when_xdg_config_home_unset() -> Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.child("confdb/het.db");
    temp.child("confdb").create_dir_all()?;
    temp.child(".config/het/config.toml")
        .write_str(&format!("db_path = '{}'", db_path.path().display()))?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path());
    run_add(&mut cmd, expense.path(), "100");

    assert!(db_path.path().exists());
    assert!(!temp.child(".local/share/het/het.db").path().exists());

    Ok(())
}

#[test]
fn config_db_path_found_in_home_config_when_xdg_config_home_empty() -> Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.child("confdb/het.db");
    temp.child("confdb").create_dir_all()?;
    temp.child(".config/het/config.toml")
        .write_str(&format!("db_path = '{}'", db_path.path().display()))?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path()).env("XDG_CONFIG_HOME", "");
    run_add(&mut cmd, expense.path(), "100");

    assert!(db_path.path().exists());
    assert!(!temp.child(".local/share/het/het.db").path().exists());

    Ok(())
}

#[test]
fn db_flag_overrides_config_db_path() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    let config_db_path = temp.child("confdb/het.db");
    config_home
        .child("het/config.toml")
        .write_str(&format!("db_path = '{}'", config_db_path.path().display()))?;

    let override_db_path = temp.child("ovr/het.db");
    temp.child("ovr").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .arg("--db")
        .arg(override_db_path.path());
    run_add(&mut cmd, expense.path(), "100");

    assert!(override_db_path.path().exists());
    assert!(!config_db_path.path().exists());

    let out_dir = temp.child("out");
    out_dir.create_dir_all()?;
    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .arg("--db")
        .arg(override_db_path.path());
    run_retrieve(&mut cmd, out_dir.path(), "100");

    let expense_date = Local::now().naive_local().date();
    let extracted = out_dir.join(format!("{expense_date}_10000_expense_a.pdf"));
    assert_eq!(EXPENSE_A_BYTES.to_vec(), fs::read(&extracted)?);

    Ok(())
}

#[test]
fn config_db_path_overrides_data_home_default() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    let data_home = temp.child("data-home");
    data_home.child("het").create_dir_all()?;
    let db_path = temp.child("confdb/het.db");
    temp.child("confdb").create_dir_all()?;
    config_home
        .child("het/config.toml")
        .write_str(&format!("db_path = '{}'", db_path.path().display()))?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("XDG_DATA_HOME", data_home.path());
    run_add(&mut cmd, expense.path(), "100");

    assert!(db_path.path().exists());
    assert!(!data_home.child("het/het.db").path().exists());

    Ok(())
}

#[test]
fn db_flag_does_not_bypass_config_parse() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    config_home
        .child("het/config.toml")
        .write_str("not [[ toml")?;

    let override_db_path = temp.child("ovr/het.db");
    temp.child("ovr").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let output = het()?
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .arg("--db")
        .arg(override_db_path.path())
        .arg("add")
        .arg("--file")
        .arg(expense.path())
        .arg("--amount")
        .arg("100")
        .assert()
        .failure()
        .get_output()
        .to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("TOML parse error"),
        "unexpected stderr: {stderr}"
    );
    assert!(!override_db_path.path().exists());

    Ok(())
}

#[test]
fn invalid_config_toml_fails_without_creating_db() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    config_home
        .child("het/config.toml")
        .write_str("not [[ toml")?;
    let data_home = temp.child("data-home");
    data_home.child("het").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let output = het()?
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("XDG_DATA_HOME", data_home.path())
        .arg("add")
        .arg("--file")
        .arg(expense.path())
        .arg("--amount")
        .arg("100")
        .assert()
        .failure()
        .get_output()
        .to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("TOML parse error"),
        "unexpected stderr: {stderr}"
    );
    assert!(!data_home.child("het/het.db").path().exists());

    Ok(())
}

#[test]
fn config_missing_db_path_field_fails() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    config_home
        .child("het/config.toml")
        .write_str("foo = \"bar\"")?;
    let data_home = temp.child("data-home");
    data_home.child("het").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let output = het()?
        .env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("XDG_DATA_HOME", data_home.path())
        .arg("add")
        .arg("--file")
        .arg(expense.path())
        .arg("--amount")
        .arg("100")
        .assert()
        .failure()
        .get_output()
        .to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing field `db_path`"),
        "unexpected stderr: {stderr}"
    );
    assert!(!data_home.child("het/het.db").path().exists());

    Ok(())
}

#[test]
fn default_db_path_uses_xdg_data_home() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    let data_home = temp.child("data-home");
    data_home.child("het").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("XDG_DATA_HOME", data_home.path());
    run_add(&mut cmd, expense.path(), "100");

    assert!(data_home.child("het/het.db").path().exists());
    assert!(!temp.child(".local/share/het/het.db").path().exists());

    Ok(())
}

#[test]
fn default_db_path_when_xdg_data_home_empty() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    temp.child(".local/share/het").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("XDG_DATA_HOME", "");
    run_add(&mut cmd, expense.path(), "100");

    assert!(temp.child(".local/share/het/het.db").path().exists());
    assert!(!temp.child("data-home/het/het.db").path().exists());

    Ok(())
}

#[test]
fn default_db_path_when_xdg_data_home_unset() -> Result<()> {
    let temp = TempDir::new()?;
    let config_home = temp.child("cfg-home");
    config_home.create_dir_all()?;
    temp.child(".local/share/het").create_dir_all()?;

    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    let mut cmd = het()?;
    cmd.env("HOME", temp.path())
        .env("XDG_CONFIG_HOME", config_home.path());
    run_add(&mut cmd, expense.path(), "100");

    assert!(temp.child(".local/share/het/het.db").path().exists());
    assert!(!temp.child("data-home/het/het.db").path().exists());

    Ok(())
}

#[test]
fn unset_home_fails_before_db_open() -> Result<()> {
    let temp = TempDir::new()?;
    let db = temp.child("het.db");
    let expense = temp.child(EXPENSE_A);
    expense.write_binary(EXPENSE_A_BYTES)?;

    het()?
        .env_remove("HOME")
        .arg("--db")
        .arg(db.path())
        .arg("add")
        .arg("--file")
        .arg(expense.path())
        .arg("--amount")
        .arg("100")
        .assert()
        .failure()
        .stdout("$HOME is unset; required environment variable\n");

    assert!(!db.path().exists());

    Ok(())
}
