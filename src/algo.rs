use crate::expense_service::Expense;
use anyhow::{Result, anyhow};

pub fn closest_subset_to_target(expenses: &[Expense], target: u64) -> Result<Vec<&Expense>> {
    if expenses.is_empty() {
        return Ok(vec![]);
    }

    let max_value = expenses
        .iter()
        .max_by_key(|e| &e.unit_amount)
        .ok_or(anyhow!("failed to extract unit amount"))?
        .unit_amount
        .0 as u64;
    let limit = target + max_value;

    let mut parent: Vec<Option<usize>> = vec![None; (limit + 1) as usize];
    parent[0] = Some(0);

    for (idx, expense) in expenses.iter().enumerate() {
        let val = expense.unit_amount.0 as usize;

        for i in (val..=limit.try_into()?).rev() {
            if parent[i - val].is_some() && parent[i].is_none() {
                parent[i] = Some(idx);
            }
        }
    }

    let best_sum_option = (target..=limit).find(|i| parent[*i as usize].is_some());

    let best_sum = match best_sum_option {
        Some(bs) => bs,
        None => return Ok(vec![]),
    };

    let mut chosen_indices = Vec::new();
    let mut curr = best_sum as usize;

    while curr > 0 {
        let idx = parent[curr].unwrap();
        chosen_indices.push(idx);
        curr -= expenses[idx].unit_amount.0 as usize;
    }

    let mut chosens_expenses = Vec::new();

    for idx in chosen_indices.iter() {
        chosens_expenses.push(&expenses[*idx]);
    }

    Ok(chosens_expenses)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::expense_service::{
        ExpenseFileData, ExpenseId, ExpenseName, ExpenseUnitAmount, FileDataType,
    };

    use super::*;

    fn make_expense(id: i64, amount: i64) -> Expense {
        Expense {
            id: ExpenseId(id),
            name: ExpenseName("test_expense".to_string()),
            file_data_type: FileDataType("pdf".to_string()),
            expense_date: NaiveDate::parse_from_str("2026-01-07", "%Y-%m-%d").unwrap(),
            unit_amount: ExpenseUnitAmount(amount),
            compressed_file_data: ExpenseFileData(vec![1, 2, 3]),
            is_deleted: false,
        }
    }

    #[test]
    fn test_no_viable_subset() {
        let exp1 = make_expense(1, 3);
        let exp2 = make_expense(2, 4);
        let expenses = vec![exp1, exp2];
        let target = 10;

        let res = closest_subset_to_target(&expenses, target);

        assert!(res.is_ok());

        let subset = res.unwrap();
        assert!(subset.is_empty());
    }

    #[test]
    fn test_subset_is_whole_list() {
        let exp1 = make_expense(1, 6);
        let exp2 = make_expense(2, 4);
        let expenses = vec![exp1, exp2];
        let target = 10;

        let res = closest_subset_to_target(&expenses, target);

        assert!(res.is_ok());

        let subset = res.unwrap();

        assert_eq!(2, subset.len());
        assert!(subset.contains(&&make_expense(1, 6)));
        assert!(subset.contains(&&make_expense(2, 4)));
    }

    #[test]
    fn test_subset_overshoots_target() {
        let exp1 = make_expense(1, 6);
        let exp2 = make_expense(2, 4);
        let expenses = vec![exp1, exp2];
        let target = 9;

        let res = closest_subset_to_target(&expenses, target);

        assert!(res.is_ok());

        let subset = res.unwrap();

        assert_eq!(2, subset.len());
        assert!(subset.contains(&&make_expense(1, 6)));
        assert!(subset.contains(&&make_expense(2, 4)));
    }

    #[test]
    fn test_subset_is_equal_to_target() {
        let exp1 = make_expense(1, 6);
        let exp2 = make_expense(2, 4);
        let exp3 = make_expense(3, 5);
        let expenses = vec![exp1, exp2, exp3];
        let target = 10;

        let res = closest_subset_to_target(&expenses, target);

        assert!(res.is_ok());

        let subset = res.unwrap();

        assert_eq!(2, subset.len());
        assert!(subset.contains(&&make_expense(1, 6)));
        assert!(subset.contains(&&make_expense(2, 4)));
    }

    #[test]
    fn test_subset_is_closest_to_target() {
        let exp1 = make_expense(1, 6);
        let exp2 = make_expense(2, 4);
        let exp3 = make_expense(3, 5);
        let expenses = vec![exp1, exp2, exp3];
        let target = 8;

        let res = closest_subset_to_target(&expenses, target);

        assert!(res.is_ok());

        let subset = res.unwrap();

        assert_eq!(2, subset.len());
        assert!(subset.contains(&&make_expense(2, 4)));
        assert!(subset.contains(&&make_expense(3, 5)));
    }
}
