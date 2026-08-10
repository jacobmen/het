use crate::{
    algo::closest_subset_to_target,
    expense_service::{Expense, ExpenseId, ExpenseName, ExpenseService, ExpenseUnitAmount},
    sql::Repository,
};
use anyhow::{Context, Result};
use rusty_money::{Money, iso};

#[derive(PartialEq, Eq, Debug)]
pub struct ExpenseFile {
    pub id: ExpenseId,
    pub name: String,
    pub contents: Vec<u8>,
}

pub enum RetrieveExpensesSummary {
    NoMatch,
    DryRun {
        target_unit_amount: u64,
        expenses: Vec<(ExpenseName, ExpenseUnitAmount)>,
    },
    Retrieved {
        files: Vec<ExpenseFile>,
    },
}

pub fn retrieve_expenses<R: Repository>(
    expense_service: &ExpenseService<R>,
    dryrun: bool,
    amount: Money<'static, iso::Currency>,
) -> Result<RetrieveExpensesSummary> {
    let expenses = expense_service.get_all_expenses()?;

    let target_unit_amount = u64::try_from(amount.to_minor_units())?;
    let closest_subset = closest_subset_to_target(&expenses, target_unit_amount)?;

    if closest_subset.is_empty() {
        return Ok(RetrieveExpensesSummary::NoMatch);
    }

    if dryrun {
        return Ok(RetrieveExpensesSummary::DryRun {
            target_unit_amount,
            expenses: closest_subset
                .iter()
                .map(|expense| (ExpenseName(expense.name.0.clone()), expense.unit_amount))
                .collect(),
        });
    }

    let files = build_expense_files(&closest_subset)?;
    Ok(RetrieveExpensesSummary::Retrieved { files })
}

fn build_expense_files(expenses: &[&Expense]) -> Result<Vec<ExpenseFile>> {
    let mut files = Vec::with_capacity(expenses.len());
    for expense in expenses {
        let name = build_expense_file_name(expense);
        let contents = lzma::decompress(&expense.compressed_file_data.0)
            .with_context(|| format!("failed to decompress data for {name}"))?;
        files.push(ExpenseFile {
            id: expense.id,
            name,
            contents,
        });
    }
    Ok(files)
}

fn build_expense_file_name(expense: &Expense) -> String {
    format!(
        "{}_{}_{}.{}",
        expense.date, expense.unit_amount.0, expense.name.0, expense.file_data_type.0
    )
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::expense_service::{ExpenseFileData, ExpenseName, ExpenseUnitAmount, FileDataType};
    use crate::sql::ExpenseRow;
    use crate::test_util::InMemoryRepository;

    use super::*;

    #[test]
    fn test_retrieve_expenses_builds_files_without_deleting() -> Result<()> {
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
        let summary = retrieve_expenses(&expense_service, false, Money::from_minor(700, iso::USD))?;

        let RetrieveExpensesSummary::Retrieved { files } = summary else {
            panic!("expected Retrieved summary");
        };

        assert_eq!(files.len(), 2);
        for file in &files {
            match file.name.as_str() {
                "2026-04-27_300_e2e1.jpg" => {
                    assert_eq!(file.id, ExpenseId(1));
                    assert_eq!(file.contents, vec![0x1, 0x2]);
                }
                "2025-03-01_500_e2e2.pdf" => {
                    assert_eq!(file.id, ExpenseId(2));
                    assert_eq!(file.contents, vec![0x3, 0x4]);
                }
                other => panic!("unexpected file name: {other}"),
            }
        }

        assert_eq!(
            expense_service.get_all_expenses()?.len(),
            2,
            "retrieve_expenses must not delete"
        );

        Ok(())
    }

    #[test]
    fn test_retrieve_expenses_dryrun() -> Result<()> {
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

        let summary = retrieve_expenses(&expense_service, true, Money::from_minor(700, iso::USD))?;

        let RetrieveExpensesSummary::DryRun {
            target_unit_amount,
            expenses,
        } = summary
        else {
            panic!("expected DryRun summary");
        };

        assert_eq!(target_unit_amount, 700);
        assert_eq!(expenses.len(), 2);
        assert!(expenses.contains(&(ExpenseName("e2e1".into()), ExpenseUnitAmount(300))));
        assert!(expenses.contains(&(ExpenseName("e2e2".into()), ExpenseUnitAmount(500))));

        assert_eq!(
            expense_service.get_all_expenses()?.len(),
            2,
            "Dryrun must not delete"
        );

        Ok(())
    }

    #[test]
    fn test_retrieve_expenses_no_match() -> Result<()> {
        let expense_service = ExpenseService::new(InMemoryRepository::new(vec![ExpenseRow {
            id: 1,
            name: "e2e1".to_string(),
            file_data_type: "jpg".to_string(),
            expense_date: "2026-04-27".to_string(),
            unit_amount: 300,
            compressed_file_data: lzma::compress(&[0x1u8, 0x2u8], 9)?,
            is_deleted: false,
        }]));

        let summary =
            retrieve_expenses(&expense_service, false, Money::from_minor(9000, iso::USD))?;

        assert!(matches!(summary, RetrieveExpensesSummary::NoMatch));

        assert_eq!(
            expense_service.get_all_expenses()?.len(),
            1,
            "No-match must not delete."
        );

        Ok(())
    }

    #[test]
    fn test_build_expense_files() -> Result<()> {
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

        let files = build_expense_files(&[&exp1, &exp2])?;

        assert_eq!(
            files,
            vec![
                ExpenseFile {
                    id: ExpenseId(1),
                    name: "2026-01-07_30000_exp1.png".into(),
                    contents: vec![0x1, 0x2],
                },
                ExpenseFile {
                    id: ExpenseId(2),
                    name: "2026-04-27_500_exp2.pdf".into(),
                    contents: vec![0x3, 0x4],
                },
            ]
        );

        Ok(())
    }
}
