use std::rc::Rc;

use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, types::Value};

pub struct ExpenseRow {
    pub id: i64,
    pub name: String,
    pub file_data_type: String,
    pub expense_date: String,
    pub unit_amount: i64,
    pub compressed_file_data: Vec<u8>,
    pub is_deleted: u8,
}

pub struct SqlRepository {
    connection: Connection,
}

pub trait Repository {
    fn create_expense_table(&self) -> Result<()>;

    fn get_all_expenses(&self) -> Result<Vec<ExpenseRow>>;

    fn create_new_expense(
        &self,
        name: String,
        file_data_type: String,
        expense_date: String,
        unit_amount: i64,
        compressed_file_data: Vec<u8>,
    ) -> Result<()>;

    fn mark_expenses_as_deleted(&self, expense_ids: &[i64]) -> Result<()>;
}

impl SqlRepository {
    pub fn try_new(connection: Connection) -> Result<Self> {
        rusqlite::vtab::array::load_module(&connection)
            .with_context(|| "failed to load rarray sqlite module")?;

        Ok(SqlRepository { connection })
    }
}

impl Repository for SqlRepository {
    fn create_expense_table(&self) -> Result<()> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS expenses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                file_data_type TEXT NOT NULL,
                expense_date TEXT NOT NULL,
                unit_amount INTEGER NOT NULL,
                compressed_file_data BLOB NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0 CHECK (is_deleted IN (0, 1)),
                UNIQUE(name, file_data_type, expense_date, unit_amount)
            );
            CREATE INDEX idx_expenses_is_deleted ON expenses(is_deleted);",
        )?;

        Ok(())
    }

    fn get_all_expenses(&self) -> Result<Vec<ExpenseRow>> {
        let mut stmt = self
            .connection
            .prepare(
                "SELECT
                      id,
                      name,
                      file_data_type,
                      expense_date,
                      unit_amount,
                      compressed_file_data,
                      is_deleted
                 FROM expenses
                 WHERE is_deleted = 0;",
            )
            .with_context(|| "failed to fetch expenses")?;

        let expense_iter = stmt
            .query_map([], |row| {
                Ok(ExpenseRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_data_type: row.get(2)?,
                    expense_date: row.get(3)?,
                    unit_amount: row.get(4)?,
                    compressed_file_data: row.get(5)?,
                    is_deleted: row.get(6)?,
                })
            })
            .with_context(|| "failed to query DB")?;

        expense_iter
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| "failed to build expense row")
    }

    fn create_new_expense(
        &self,
        name: String,
        file_data_type: String,
        expense_date: String,
        unit_amount: i64,
        compressed_file_data: Vec<u8>,
    ) -> Result<()> {
        let inserted = self.connection.execute(
            "INSERT INTO expenses (
                 name,
                 file_data_type,
                 expense_date,
                 unit_amount,
                 compressed_file_data
             ) VALUES (
                 ?1,
                 ?2,
                 ?3,
                 ?4,
                 ?5
             );",
            (
                &name,
                &file_data_type,
                &expense_date,
                &unit_amount,
                &compressed_file_data,
            ),
        )?;

        if inserted == 1 {
            Ok(())
        } else {
            Err(anyhow!("failed to insert expense row"))
        }
    }

    fn mark_expenses_as_deleted(&self, expense_ids: &[i64]) -> Result<()> {
        let bind_values = Rc::new(
            expense_ids
                .iter()
                .copied()
                .map(Value::from)
                .collect::<Vec<Value>>(),
        );

        let marked = self.connection.execute(
            "UPDATE expenses
             SET is_deleted = 1
             WHERE id IN rarray(?1);",
            (bind_values,),
        )?;

        if marked == expense_ids.len() {
            Ok(())
        } else {
            Err(anyhow!(
                "failed to mark correct number of rows as deleted. expected {}, got {}",
                expense_ids.len(),
                marked
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_expense() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        let sql_repository = SqlRepository::try_new(connection)?;

        sql_repository.create_expense_table()?;

        sql_repository.create_new_expense(
            "exp1".to_string(),
            "pdf".to_string(),
            "2025-01-01".to_string(),
            2000,
            vec![0x1, 0x1],
        )?;

        let expenses = sql_repository.get_all_expenses()?;
        assert_eq!(1, expenses.len());

        let expense = &expenses[0];

        assert_eq!("exp1", expense.name);
        assert_eq!("pdf", expense.file_data_type);
        assert_eq!("2025-01-01", expense.expense_date);
        assert_eq!(2000, expense.unit_amount);
        assert_eq!(vec![0x1, 0x1], expense.compressed_file_data);
        assert_eq!(0, expense.is_deleted);

        Ok(())
    }

    #[test]
    fn test_mark_expense_as_deleted() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        let sql_repository = SqlRepository::try_new(connection)?;

        sql_repository.create_expense_table()?;

        sql_repository.create_new_expense(
            "exp1".to_string(),
            "pdf".to_string(),
            "2025-01-01".to_string(),
            2000,
            vec![0x1, 0x1],
        )?;
        sql_repository.create_new_expense(
            "exp2".to_string(),
            "png".to_string(),
            "2026-01-01".to_string(),
            1000,
            vec![0xa, 0xb, 0xc],
        )?;
        sql_repository.create_new_expense(
            "exp3".to_string(),
            "jpeg".to_string(),
            "2026-06-01".to_string(),
            1500,
            vec![0x1, 0x2, 0x3],
        )?;

        let expenses = sql_repository.get_all_expenses()?;
        assert_eq!(3, expenses.len());

        let ids_to_delete = expenses
            .iter()
            .filter_map(|e| {
                if e.name == "exp1" || e.name == "exp2" {
                    Some(e.id)
                } else {
                    None
                }
            })
            .collect::<Vec<i64>>();
        sql_repository.mark_expenses_as_deleted(&ids_to_delete)?;

        let remaining_expenses = sql_repository.get_all_expenses()?;
        assert_eq!(1, remaining_expenses.len());

        let expense = &remaining_expenses[0];

        assert_eq!("exp3", expense.name);
        assert_eq!("jpeg", expense.file_data_type);
        assert_eq!("2026-06-01", expense.expense_date);
        assert_eq!(1500, expense.unit_amount);
        assert_eq!(vec![0x1, 0x2, 0x3], expense.compressed_file_data);
        assert_eq!(0, expense.is_deleted);

        Ok(())
    }

    #[test]
    fn test_expense_uniqueness() -> Result<()> {
        let connection = Connection::open_in_memory()?;
        let sql_repository = SqlRepository::try_new(connection)?;

        sql_repository.create_expense_table()?;

        sql_repository.create_new_expense(
            "exp1".to_string(),
            "pdf".to_string(),
            "2025-01-01".to_string(),
            2000,
            vec![0x1, 0x1],
        )?;

        assert!(
            sql_repository
                .create_new_expense(
                    "exp1".to_string(),
                    "pdf".to_string(),
                    "2025-01-01".to_string(),
                    2000,
                    vec![0x1, 0x1],
                )
                .is_err()
        );

        Ok(())
    }
}
