use anyhow::{Result, anyhow};
use chrono::NaiveDate;

use crate::sql::Repository;

#[derive(PartialEq, Debug)]
pub struct ExpenseId(pub i64);

#[derive(PartialEq, Debug)]
pub struct ExpenseName(pub String);

#[derive(PartialEq, Debug)]
pub struct FileDataType(pub String);

#[derive(PartialEq, Debug)]
pub struct ExpenseUnitAmount(pub i64);

#[derive(PartialEq, Debug)]
pub struct ExpenseFileData(pub Vec<u8>);

pub struct Expense {
    pub id: ExpenseId,
    pub name: ExpenseName,
    pub file_data_type: FileDataType,
    pub expense_date: NaiveDate,
    pub unit_amount: ExpenseUnitAmount,
    pub compressed_file_data: ExpenseFileData,
    pub is_deleted: bool,
}

pub struct ExpenseService<R: Repository> {
    repository: R,
}

impl<R: Repository> ExpenseService<R> {
    pub fn new(repository: R) -> Self {
        ExpenseService { repository }
    }

    pub fn create_expense_table(&self) -> Result<()> {
        self.repository.create_expense_table()
    }

    pub fn get_all_expenses(&self) -> Result<Vec<Expense>> {
        let expense_rows = self.repository.get_all_expenses()?;

        expense_rows
            .into_iter()
            .map(|r| {
                Ok(Expense {
                    id: ExpenseId(r.id),
                    name: ExpenseName(r.name),
                    file_data_type: FileDataType(r.file_data_type),
                    expense_date: NaiveDate::parse_from_str(&r.expense_date, "%Y-%m-%d")?,
                    unit_amount: ExpenseUnitAmount(r.unit_amount),
                    compressed_file_data: ExpenseFileData(r.compressed_file_data),
                    is_deleted: match r.is_deleted {
                        0 => Ok(false),
                        1 => Ok(true),
                        _ => Err(anyhow!("unknown is_deleted column: {}", r.is_deleted)),
                    }?,
                })
            })
            .collect::<Result<_>>()
    }

    pub fn create_new_expense(
        &self,
        name: &ExpenseName,
        file_data_type: &FileDataType,
        expense_date: &NaiveDate,
        unit_amount: ExpenseUnitAmount,
        compressed_file_data: &ExpenseFileData,
    ) -> Result<()> {
        self.repository.create_new_expense(
            &name.0,
            &file_data_type.0,
            &expense_date.format("%Y-%m-%d").to_string(),
            unit_amount.0,
            &compressed_file_data.0,
        )
    }

    pub fn mark_expenses_as_deleted(&self, expense_ids: &[ExpenseId]) -> Result<()> {
        self.repository
            .mark_expenses_as_deleted(&expense_ids.iter().map(|e| e.0).collect::<Vec<i64>>())
    }
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
            Ok(vec![ExpenseRow {
                id: 1,
                name: "expense".to_string(),
                file_data_type: "pdf".to_string(),
                expense_date: "2026-01-07".to_string(),
                unit_amount: 2000,
                compressed_file_data: vec![0x1],
                is_deleted: 0,
            }])
        }

        fn create_new_expense(
            &self,
            name: &str,
            file_data_type: &str,
            expense_date: &str,
            unit_amount: i64,
            compressed_file_data: &[u8],
        ) -> Result<()> {
            assert_eq!("expense", name);
            assert_eq!("pdf", file_data_type);
            assert_eq!("2026-01-07", expense_date);
            assert_eq!(2000, unit_amount);
            assert_eq!(vec![0x1], compressed_file_data);

            Ok(())
        }

        fn mark_expenses_as_deleted(&self, _expense_ids: &[i64]) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_get_all_expenses() -> Result<()> {
        let expense_service = ExpenseService::new(TestRepository {});

        let expenses = expense_service.get_all_expenses()?;
        assert_eq!(1, expenses.len());

        let expense = &expenses[0];

        assert_eq!(ExpenseId(1), expense.id);
        assert_eq!(ExpenseName("expense".to_string()), expense.name);
        assert_eq!(FileDataType("pdf".to_string()), expense.file_data_type);
        assert_eq!(
            NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
            expense.expense_date
        );
        assert_eq!(ExpenseUnitAmount(2000), expense.unit_amount);
        assert_eq!(ExpenseFileData(vec![0x1]), expense.compressed_file_data);
        assert_eq!(false, expense.is_deleted);

        Ok(())
    }

    #[test]
    fn test_create_new_expense() -> Result<()> {
        let expense_service = ExpenseService::new(TestRepository {});

        expense_service.create_new_expense(
            &ExpenseName("expense".to_string()),
            &FileDataType("pdf".to_string()),
            &NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
            ExpenseUnitAmount(2000),
            &ExpenseFileData(vec![0x1]),
        )?;

        Ok(())
    }
}
