use std::{fs, path::Path};

use crate::{
    algo::closest_subset_to_target,
    expense_service::{Expense, ExpenseId, ExpenseService},
    sql::Repository,
};
use anyhow::{Context, Result};
use rusty_money::{Money, iso};

pub fn retrieve_expenses<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    amount: Money<'static, iso::Currency>,
    out_path: &Path,
) -> Result<()> {
    let expenses = expense_service.get_all_expenses()?;

    let target_unit_amount = u64::try_from(amount.to_minor_units())?;
    let closest_subset = closest_subset_to_target(&expenses, target_unit_amount)?;

    if closest_subset.is_empty() {
        println!("no expenses reach {amount}");
        return Ok(());
    }

    if dryrun {
        println!("target unit amount: {target_unit_amount}");
        for expense in &closest_subset {
            println!("{}: {}", expense.name.0, expense.unit_amount.0);
        }

        return Ok(());
    }

    write_expense_files(&closest_subset, out_path)?;

    let expenses_to_delete: Vec<ExpenseId> = closest_subset.iter().map(|e| e.id).collect();
    expense_service.mark_expenses_as_deleted(&expenses_to_delete)?;

    Ok(())
}

fn write_expense_files(expenses: &[&Expense], out_path: &Path) -> Result<()> {
    for expense in expenses {
        let file_name = build_expense_file_name(expense);
        let file_write_path = out_path.join(&file_name);
        let file_contents = lzma::decompress(&expense.compressed_file_data.0)
            .with_context(|| format!("failed to decompress data for {file_name}"))?;

        fs::write(&file_write_path, &file_contents).with_context(|| {
            format!(
                "failed to write expense file to {}",
                file_write_path.display()
            )
        })?;
    }

    Ok(())
}

fn build_expense_file_name(expense: &Expense) -> String {
    format!(
        "{}_{}_{}.{}",
        expense.date, expense.unit_amount.0, expense.name.0, expense.file_data_type.0
    )
}

#[cfg(test)]
mod tests {
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use chrono::NaiveDate;

    use crate::expense_service::{ExpenseFileData, ExpenseName, ExpenseUnitAmount, FileDataType};
    use crate::sql::ExpenseRow;
    use crate::test_util::InMemoryRepository;

    use super::*;

    #[test]
    fn test_retrieve_expenses_e2e() -> Result<()> {
        let temp = TempDir::new()?;
        let out = temp.child("out");
        out.create_dir_all()?;

        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![
            ExpenseRow {
                id: 1,
                name: "e2e1".to_string(),
                file_data_type: "jpg".to_string(),
                expense_date: "2026-04-27".to_string(),
                unit_amount: 300,
                compressed_file_data: lzma::compress(&[0x1u8, 0x2u8], 9)?,
                is_deleted: false,
            },
            ExpenseRow {
                id: 2,
                name: "e2e2".to_string(),
                file_data_type: "pdf".to_string(),
                expense_date: "2025-03-01".to_string(),
                unit_amount: 500,
                compressed_file_data: lzma::compress(&[0x3u8, 0x4u8], 9)?,
                is_deleted: false,
            },
        ]));

        // Closest subset to $7.00 from $3.00 + $5.00 is both expenses ($8.00).
        retrieve_expenses(
            &expense_service,
            false,
            Money::from_minor(700, iso::USD),
            out.path(),
        )?;

        let exp1_path = out.join("2026-04-27_300_e2e1.jpg");
        let exp2_path = out.join("2025-03-01_500_e2e2.pdf");

        assert!(fs::exists(&exp1_path)?, "exp1 file doesn't exist");
        assert!(fs::exists(&exp2_path)?, "exp2 file doesn't exist");

        assert_eq!(vec![0x1, 0x2], fs::read(&exp1_path)?);
        assert_eq!(vec![0x3, 0x4], fs::read(&exp2_path)?);

        assert!(
            expense_service.get_all_expenses()?.is_empty(),
            "retrieved expenses must be marked deleted"
        );

        Ok(())
    }

    #[test]
    fn test_write_expenses_e2e() -> Result<()> {
        let temp = TempDir::new()?;
        let out = temp.child("out");
        out.create_dir_all()?;
        let out_path = out.path();

        let exp1 = Expense {
            id: ExpenseId(1),
            name: ExpenseName("exp1".to_string()),
            file_data_type: FileDataType("png".to_string()),
            date: NaiveDate::parse_from_str("2026-01-07", "%Y-%m-%d").unwrap(),
            unit_amount: ExpenseUnitAmount(30000),
            compressed_file_data: ExpenseFileData(lzma::compress(&[0x1u8, 0x2u8], 9)?),
            is_deleted: false,
        };
        let exp2 = Expense {
            id: ExpenseId(2),
            name: ExpenseName("exp2".to_string()),
            file_data_type: FileDataType("pdf".to_string()),
            date: NaiveDate::parse_from_str("2026-04-27", "%Y-%m-%d").unwrap(),
            unit_amount: ExpenseUnitAmount(500),
            compressed_file_data: ExpenseFileData(lzma::compress(&[0x3u8, 0x4u8], 9)?),
            is_deleted: false,
        };
        let expenses = [&exp1, &exp2];

        write_expense_files(&expenses, out_path)?;

        let exp1_path = out_path.join(build_expense_file_name(&exp1));
        let exp2_path = out_path.join(build_expense_file_name(&exp2));

        assert!(fs::exists(&exp1_path)?, "exp1 file doesn't exist");
        assert!(fs::exists(&exp2_path)?, "exp2 file doesn't exist");

        let exp1_file_content = fs::read(&exp1_path)?;
        let exp2_file_content = fs::read(&exp2_path)?;

        assert_eq!(vec![0x1, 0x2], exp1_file_content);
        assert_eq!(vec![0x3, 0x4], exp2_file_content);

        Ok(())
    }
}
