use std::path::Path;

use crate::{
    expense_service::{
        ExpenseFileData, ExpenseName, ExpenseService, ExpenseUnitAmount, FileDataType,
    },
    sql::Repository,
};
use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use rusty_money::{Money, iso};

pub enum AddExpenseSummary {
    DryRun {
        name: ExpenseName,
        file_data_type: FileDataType,
        unit_amount: ExpenseUnitAmount,
    },
    Created {
        name: ExpenseName,
        file_data_type: FileDataType,
        expense_date: NaiveDate,
        unit_amount: ExpenseUnitAmount,
        compressed_data_size: usize,
    },
}

pub fn add_expense<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    file_path: &Path,
    file_contents: &[u8],
    expense_date: NaiveDate,
    input_amount: Money<'static, iso::Currency>,
) -> Result<AddExpenseSummary> {
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
        return Ok(AddExpenseSummary::DryRun {
            name: ExpenseName(expense_name.to_string()),
            file_data_type: FileDataType(file_data_type.to_string()),
            unit_amount: ExpenseUnitAmount(unit_amount),
        });
    }

    let compressed_file_data = ExpenseFileData(lzma::compress(file_contents, 9)?);

    expense_service.create_new_expense(
        &ExpenseName(expense_name.to_string()),
        &FileDataType(file_data_type.to_string()),
        expense_date,
        ExpenseUnitAmount(unit_amount),
        &compressed_file_data,
    )?;

    Ok(AddExpenseSummary::Created {
        name: ExpenseName(expense_name.to_string()),
        file_data_type: FileDataType(file_data_type.to_string()),
        expense_date,
        unit_amount: ExpenseUnitAmount(unit_amount),
        compressed_data_size: compressed_file_data.0.len(),
    })
}

#[cfg(test)]
mod tests {
    use crate::test_util::InMemoryRepository;

    use super::*;

    const CONTENTS: &[u8] = b"sample expense contents";
    const FIXED_DATE: NaiveDate = NaiveDate::from_ymd_opt(2026, 1, 7).unwrap();

    #[test]
    fn test_add_expense_happy_path() -> Result<()> {
        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![]));

        let summary = add_expense(
            &expense_service,
            false,
            Path::new("Cargo.toml"),
            CONTENTS,
            FIXED_DATE,
            Money::from_minor(10_000, iso::USD),
        )?;
        match summary {
            AddExpenseSummary::Created {
                name,
                file_data_type,
                expense_date,
                unit_amount,
                compressed_data_size,
            } => {
                assert_eq!(ExpenseName("Cargo".into()), name);
                assert_eq!(FileDataType("toml".into()), file_data_type);
                assert_eq!(FIXED_DATE, expense_date);
                assert_eq!(ExpenseUnitAmount(10_000), unit_amount);
                assert_eq!(lzma::compress(CONTENTS, 9)?.len(), compressed_data_size);
            }
            AddExpenseSummary::DryRun { .. } => unreachable!("expected a real add"),
        }

        let expenses = expense_service.get_all_expenses()?;
        assert_eq!(1, expenses.len());

        let expense = &expenses[0];
        assert_eq!(ExpenseName("Cargo".into()), expense.name);
        assert_eq!(FileDataType("toml".into()), expense.file_data_type);
        assert_eq!(FIXED_DATE, expense.date);
        assert_eq!(ExpenseUnitAmount(10_000), expense.unit_amount);
        assert_eq!(
            ExpenseFileData(lzma::compress(CONTENTS, 9)?),
            expense.compressed_file_data
        );

        Ok(())
    }

    #[test]
    fn test_add_expense_dryrun() -> Result<()> {
        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![]));

        let summary = add_expense(
            &expense_service,
            true,
            Path::new("dryrun.pdf"),
            b"unused contents",
            FIXED_DATE,
            Money::from_minor(10_000, iso::USD),
        )?;
        match summary {
            AddExpenseSummary::DryRun {
                name,
                file_data_type,
                unit_amount,
            } => {
                assert_eq!(ExpenseName("dryrun".into()), name);
                assert_eq!(FileDataType("pdf".into()), file_data_type);
                assert_eq!(ExpenseUnitAmount(10_000), unit_amount);
            }
            AddExpenseSummary::Created { .. } => unreachable!("expected a dryrun summary"),
        }
        assert!(
            expense_service.get_all_expenses()?.is_empty(),
            "dryrun must not persist anything"
        );
        Ok(())
    }

    #[test]
    fn test_no_delimiter() {
        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![]));

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new("./Cargo"),
                b"contents",
                FIXED_DATE,
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }

    #[test]
    fn test_no_file_name() {
        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![]));

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new(".abcd"),
                b"contents",
                FIXED_DATE,
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }

    #[test]
    fn test_no_file_data_type() {
        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![]));

        assert!(
            add_expense(
                &expense_service,
                false,
                Path::new("abcd"),
                b"contents",
                FIXED_DATE,
                Money::from_minor(10_000, iso::USD)
            )
            .is_err()
        );
    }
}
