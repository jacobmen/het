use crate::expense_service::Expense;
use anyhow::{Result, anyhow};

pub fn closest_subset_to_target(expenses: &[Expense], target: u64) -> Result<Vec<&Expense>> {
    if expenses.is_empty() {
        return Ok(Vec::new());
    }

    let target =
        usize::try_from(target).map_err(|_| anyhow!("target value `{target}` out of range"))?;

    let unit_amounts: Vec<usize> = expenses
        .iter()
        .map(|e| usize::try_from(e.unit_amount.0))
        .collect::<Result<_, _>>()?;

    let max_value = unit_amounts
        .iter()
        .copied()
        .max()
        .ok_or_else(|| anyhow!("failed to extract max unit value"))?;

    let limit = target
        .checked_add(max_value)
        .ok_or_else(|| anyhow!("target plus max unit value overflows `usize`"))?;

    let table_len = limit
        .checked_add(1)
        .ok_or_else(|| anyhow!("subset-sum table too large"))?;

    let mut parent: Vec<Option<usize>> = vec![None; table_len];
    if let Some(sentinel) = parent.first_mut() {
        *sentinel = Some(usize::MAX);
    }

    for (idx, val) in unit_amounts.iter().copied().enumerate() {
        for i in (val..=limit).rev() {
            // i >= val by loop construction, so i - val is in [0, limit].
            let predecessor_reachable = i
                .checked_sub(val)
                .is_some_and(|j| parent.get(j).is_some_and(Option::is_some));
            if predecessor_reachable && let Some(cell @ None) = parent.get_mut(i) {
                *cell = Some(idx);
            }
        }
    }

    let Some(best_sum) = (target..=limit).find(|i| parent.get(*i).is_some_and(Option::is_some))
    else {
        return Ok(Vec::new());
    };

    let mut chosen_expenses = Vec::new();
    let mut curr = best_sum;

    while curr > 0 {
        let Some(Some(idx)) = parent.get(curr).copied() else {
            break;
        };
        if idx == usize::MAX {
            break;
        }
        let Some(val) = unit_amounts.get(idx).copied() else {
            return Err(anyhow!(
                "failed to backtrack subset sum at `{curr}`: expense index `{idx}` out of range"
            ));
        };
        let Some(expense) = expenses.get(idx) else {
            return Err(anyhow!(
                "failed to backtrack subset sum at `{curr}`: expense index `{idx}` out of range"
            ));
        };
        chosen_expenses.push(expense);
        curr = curr
            .checked_sub(val)
            .ok_or_else(|| anyhow!("failed to backtrack subset sum at `{curr}`"))?;
    }

    Ok(chosen_expenses)
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
            date: NaiveDate::parse_from_str("2026-01-07", "%Y-%m-%d").unwrap(),
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

    #[test]
    fn test_negative_unit_amount_errors() {
        let exp1 = make_expense(1, -5);
        let expenses = vec![exp1];

        let res = closest_subset_to_target(&expenses, 10);

        assert!(res.is_err());
    }

    #[test]
    fn test_target_overflow_errors() {
        let exp1 = make_expense(1, 10);
        let expenses = vec![exp1];

        let res = closest_subset_to_target(&expenses, u64::MAX);

        assert!(res.is_err());
    }

    #[test]
    fn test_zero_value_expense_not_chosen() {
        let exp1 = make_expense(1, 0);
        let exp2 = make_expense(2, 6);
        let exp3 = make_expense(3, 4);
        let expenses = vec![exp1, exp2, exp3];
        let target = 10;

        let res = closest_subset_to_target(&expenses, target);

        assert!(res.is_ok());

        let subset = res.unwrap();

        assert_eq!(2, subset.len());
        assert!(subset.contains(&&make_expense(2, 6)));
        assert!(subset.contains(&&make_expense(3, 4)));
        assert!(!subset.contains(&&make_expense(1, 0)));
    }

    #[test]
    fn test_zero_target_returns_empty() {
        let exp1 = make_expense(1, 6);
        let expenses = vec![exp1];

        let res = closest_subset_to_target(&expenses, 0);

        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[test]
    fn test_single_expense_below_and_at_target() {
        let exp1 = make_expense(1, 6);
        let expenses = vec![exp1];

        let below = closest_subset_to_target(&expenses, 10);
        assert!(below.is_ok());
        assert!(below.unwrap().is_empty());

        let reachable = closest_subset_to_target(&expenses, 5);
        assert!(reachable.is_ok());

        let subset = reachable.unwrap();
        assert_eq!(1, subset.len());
        assert!(subset.contains(&&make_expense(1, 6)));
    }
}
