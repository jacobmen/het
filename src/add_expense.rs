use std::{fs, path::Path};

use crate::{
    expense_service::{
        ExpenseFileData, ExpenseName, ExpenseService, ExpenseUnitAmount, FileDataType,
    },
    sql::Repository,
};
use anyhow::{Result, anyhow};
use chrono::Local;
use rusty_money::{Money, iso};

pub fn add_expense<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    file_path: &Path,
    input_amount: Money<'static, iso::Currency>,
) -> Result<()> {
    let file_name = file_path
        .file_name()
        .ok_or_else(|| {
            anyhow!(
                "path doesn't contain file name: {}",
                file_path.to_string_lossy()
            )
        })?
        .to_string_lossy();

    let (expense_name, file_data_type) = file_name
        .split_once('.')
        .ok_or_else(|| anyhow!("file name doesn't contain delimiter `.`: {file_name}"))?;

    if expense_name.is_empty() {
        return Err(anyhow!("expense name is empty"));
    } else if file_data_type.is_empty() {
        return Err(anyhow!("file data type is empty"));
    }

    let unit_amount = input_amount.to_minor_units();

    if dryrun {
        println!("Dryrun add");
        println!("\texpense=`{expense_name}`");
        println!("\tfile_data_type=`{file_data_type}`");
        println!("\tunit_amount=`{unit_amount}`");
        return Ok(());
    }

    let compressed_file_data = ExpenseFileData(lzma::compress(&fs::read(file_path)?, 9)?);
    let expense_date = Local::now().naive_local().date();

    expense_service.create_new_expense(
        &ExpenseName(expense_name.to_string()),
        &FileDataType(file_data_type.to_string()),
        expense_date,
        ExpenseUnitAmount(unit_amount),
        &compressed_file_data,
    )?;

    println!("Created expense");
    println!("\texpense=`{expense_name}`");
    println!("\tfile_data_type=`{file_data_type}`");
    println!("\texpense_date=`{}`", expense_date.format("%Y-%m-%d"));
    println!("\tunit_amount=`{unit_amount}`");
    println!("\tcompressed_data_size=`{}`", compressed_file_data.0.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::sql::ExpenseRow;

    use super::*;

    struct TestRepository;

    impl Repository for TestRepository {
        fn create_expense_table(&self) -> Result<()> {
            Ok(())
        }

        fn get_all_expenses(&self) -> Result<Vec<ExpenseRow>> {
            Ok(vec![])
        }

        fn create_new_expense(
            &self,
            name: &str,
            file_data_type: &str,
            _expense_date: &str,
            unit_amount: i64,
            _compressed_file_data: &[u8],
        ) -> Result<()> {
            assert_eq!("Cargo", name);
            assert_eq!("toml", file_data_type);
            assert_eq!(10_000, unit_amount);

            Ok(())
        }

        fn mark_expenses_as_deleted(&self, _expense_ids: &[i64]) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_add_expense_happy_path() -> Result<()> {
        let expense_service = ExpenseService::new(TestRepository {});

        add_expense(
            &expense_service,
            false,
            Path::new("./Cargo.toml"),
            Money::from_minor(10_000, iso::USD),
        )?;
        Ok(())
    }

    #[test]
    fn test_no_delimiter() {
        let expense_service = ExpenseService::new(TestRepository {});

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new("./Cargo"),
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }

    #[test]
    fn test_no_file_name() {
        let expense_service = ExpenseService::new(TestRepository {});

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new(".abcd"),
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }

    #[test]
    fn test_no_file_data_type() {
        let expense_service = ExpenseService::new(TestRepository {});

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new("abcd"),
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }

    #[test]
    fn test_no_file() {
        let expense_service = ExpenseService::new(TestRepository {});

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new("abcd.1234"),
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }
}
