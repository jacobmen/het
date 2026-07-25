use std::{fs, path::Path};

use crate::{
    algo::closest_subset_to_target,
    expense_service::{Expense, ExpenseId, ExpenseService},
    sql::Repository,
};
use anyhow::{Context, Result, anyhow};

pub fn retrieve_expenses<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    amount: f64,
    out_path: &Path,
) -> Result<()> {
    let expenses = expense_service.get_all_expenses()?;

    // TODO: turn into type safe parse-don't-validate pattern
    if expenses.iter().any(|e| e.is_deleted) {
        return Err(anyhow!(
            "invariant broken, retrieved expenses contain deleted entry"
        ));
    }

    let target_unit_amount = (amount * 100.0).round() as u64;
    let closest_subset = closest_subset_to_target(&expenses, target_unit_amount)?;

    if closest_subset.is_empty() {
        println!("no expenses reach ${}", amount);
        return Ok(());
    }

    if dryrun {
        println!("target unit amount: {}", target_unit_amount);
        closest_subset
            .iter()
            .for_each(|e| println!("{}: {}", e.name.0, e.unit_amount.0));

        return Ok(());
    }

    write_expense_files(&closest_subset, out_path)?;

    let expenses_to_delete: Vec<ExpenseId> = closest_subset.iter().map(|e| e.id).collect();
    expense_service.mark_expenses_as_deleted(&expenses_to_delete)?;

    Ok(())
}

fn write_expense_files(expenses: &[&Expense], out_path: &Path) -> Result<()> {
    for expense in expenses.iter() {
        let file_name = build_expense_file_name(expense);
        let file_write_path = out_path.join(&file_name);
        let file_contents = lzma::decompress(&expense.compressed_file_data.0)
            .with_context(|| format!("failed to decompress data for {}", file_name))?;

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
        expense.expense_date, expense.unit_amount.0, expense.name.0, expense.file_data_type.0
    )
}

#[cfg(test)]
mod tests {
    use crate::expense_service::{ExpenseFileData, ExpenseName, ExpenseUnitAmount, FileDataType};
    use crate::sql::ExpenseRow;
    use chrono::NaiveDate;

    use super::*;

    struct TestRepository;

    impl Repository for TestRepository {
        fn create_expense_table(&self) -> Result<()> {
            Ok(())
        }

        fn get_all_expenses(&self) -> Result<Vec<ExpenseRow>> {
            Ok(vec![
                ExpenseRow {
                    id: 1,
                    name: "e2e1".to_string(),
                    file_data_type: "jpg".to_string(),
                    expense_date: "2026-04-27".to_string(),
                    unit_amount: 300,
                    compressed_file_data: lzma::compress(&[0x1u8, 0x2u8], 9)?,
                    is_deleted: 0,
                },
                ExpenseRow {
                    id: 2,
                    name: "e2e2".to_string(),
                    file_data_type: "pdf".to_string(),
                    expense_date: "2025-03-01".to_string(),
                    unit_amount: 500,
                    compressed_file_data: lzma::compress(&[0x3u8, 0x4u8], 9)?,
                    is_deleted: 0,
                },
            ])
        }

        fn create_new_expense(
            &self,
            _name: &str,
            _file_data_type: &str,
            _expense_date: &str,
            _unit_amount: i64,
            _compressed_file_data: &[u8],
        ) -> Result<()> {
            Ok(())
        }

        fn mark_expenses_as_deleted(&self, _expense_ids: &[i64]) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_retrieve_expenses_e2e() -> Result<()> {
        let expense_service = ExpenseService::new(TestRepository {});
        let out_path = Path::new("./");

        retrieve_expenses(&expense_service, false, 7.0, &out_path)?;

        let expenses = expense_service.get_all_expenses()?;

        assert_eq!(2, expenses.len());

        let exp1 = &expenses[0];
        let exp2 = &expenses[1];

        let exp1_path = out_path.join(build_expense_file_name(&exp1));
        let exp2_path = out_path.join(build_expense_file_name(&exp2));

        assert!(fs::exists(&exp1_path)?, "exp1 file doesn't exist");
        assert!(fs::exists(&exp2_path)?, "exp2 file doesn't exist");

        let exp1_file_content = fs::read(&exp1_path)?;
        let exp2_file_content = fs::read(&exp2_path)?;

        assert_eq!(vec![0x1, 0x2], exp1_file_content);
        assert_eq!(vec![0x3, 0x4], exp2_file_content);

        fs::remove_file(&exp1_path)?;
        fs::remove_file(&exp2_path)?;

        Ok(())
    }

    #[test]
    fn test_write_expenses_e2e() -> Result<()> {
        let out_path = Path::new("./");

        let exp1 = Expense {
            id: ExpenseId(1),
            name: ExpenseName("exp1".to_string()),
            file_data_type: FileDataType("png".to_string()),
            expense_date: NaiveDate::parse_from_str("2026-01-07", "%Y-%m-%d").unwrap(),
            unit_amount: ExpenseUnitAmount(30000),
            compressed_file_data: ExpenseFileData(lzma::compress(&[0x1u8, 0x2u8], 9)?),
            is_deleted: false,
        };
        let exp2 = Expense {
            id: ExpenseId(2),
            name: ExpenseName("exp2".to_string()),
            file_data_type: FileDataType("pdf".to_string()),
            expense_date: NaiveDate::parse_from_str("2026-04-27", "%Y-%m-%d").unwrap(),
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

        fs::remove_file(&exp1_path)?;
        fs::remove_file(&exp2_path)?;

        Ok(())
    }
}
