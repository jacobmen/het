use std::cell::{Cell, RefCell};

use anyhow::Result;
use chrono::NaiveDate;

use crate::{
    expense_service::{
        Expense, ExpenseFileData, ExpenseId, ExpenseName, ExpenseUnitAmount, FileDataType,
    },
    sql::{ExpenseRow, Repository},
};

#[derive(Debug, Default)]
pub struct InMemoryRepository {
    state: RefCell<Vec<ExpenseRow>>,
    next_id: Cell<i64>,
}

impl InMemoryRepository {
    #[must_use]
    pub fn new(rows: Vec<ExpenseRow>) -> Self {
        let next_id = rows.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        Self {
            state: RefCell::new(rows),
            next_id: Cell::new(next_id),
        }
    }
}

impl Repository for InMemoryRepository {
    fn create_expense_table(&self) -> Result<()> {
        Ok(())
    }

    fn get_all_expenses(&self) -> Result<Vec<ExpenseRow>> {
        Ok(self
            .state
            .borrow()
            .iter()
            .filter(|row| !row.is_deleted)
            .cloned()
            .collect())
    }

    fn create_new_expense(
        &self,
        name: &str,
        file_data_type: &str,
        expense_date: &str,
        unit_amount: i64,
        compressed_file_data: &[u8],
    ) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.push(ExpenseRow {
            id: self.next_id.get(),
            name: name.into(),
            file_data_type: file_data_type.into(),
            expense_date: expense_date.into(),
            unit_amount,
            compressed_file_data: compressed_file_data.to_vec(),
            is_deleted: false,
        });
        self.next_id.set(self.next_id.get() + 1);
        Ok(())
    }

    fn mark_expenses_as_deleted(&self, expense_ids: &[i64]) -> Result<()> {
        let mut state = self.state.borrow_mut();
        for row in state.iter_mut().filter(|row| expense_ids.contains(&row.id)) {
            row.is_deleted = true;
        }
        Ok(())
    }
}

#[must_use]
pub fn make_expense(id: i64, amount: i64) -> Expense {
    Expense {
        id: ExpenseId(id),
        name: ExpenseName("test_expense".to_string()),
        file_data_type: FileDataType("pdf".to_string()),
        date: NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
        unit_amount: ExpenseUnitAmount(amount),
        compressed_file_data: ExpenseFileData(vec![1, 2, 3]),
        is_deleted: false,
    }
}
