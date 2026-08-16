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
    use crate::test_util::make_expense;

    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

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

        // u64::MAX: fails the `usize` conversion on 32-bit hosts, and
        // `checked_add` overflows on 64-bit hosts.
        let res = closest_subset_to_target(&expenses, u64::MAX);
        assert!(res.is_err());

        // usize::MAX passes the `usize` conversion on both host widths, so the
        // error comes from `target + max_value` overflowing `checked_add`.
        let res = closest_subset_to_target(&expenses, u64::try_from(usize::MAX).unwrap());
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

    /// Minimum subset sum `>= target` over all subsets (None if none exists).
    fn bruteforce_min_over(amounts: &[i64], target: u64) -> Option<u64> {
        let mut sums = vec![0_u64];

        for &a in amounts {
            let a = u64::try_from(a).unwrap();
            let additions: Vec<u64> = sums.iter().map(|&s| s.saturating_add(a)).collect();
            sums.extend(additions);
        }

        sums.into_iter().filter(|&s| s >= target).min()
    }

    proptest! {
        #[test]
        fn prop_oracle_matches_bruteforce(
            amounts in prop::collection::vec(0_i64..=50, 0..=8),
            target in 0_u64..=1_000,
        ) {
            let expenses: Vec<Expense> = amounts
                .iter()
                .enumerate()
                .map(|(i, &a)| make_expense(i64::try_from(i).unwrap(), a))
                .collect();

            let res = closest_subset_to_target(&expenses, target);
            prop_assert!(res.is_ok(), "amounts={amounts:?} target={target}");
            let subset = res.unwrap();

            let actual: u64 = subset
                .iter()
                .map(|e| u64::try_from(e.unit_amount.0).unwrap())
                .sum();
            if let Some(expected) = bruteforce_min_over(&amounts, target) {
                prop_assert_eq!(actual, expected, "amounts={:?} target={}", amounts, target);
            } else {
                prop_assert!(
                    subset.is_empty(),
                    "expected empty result: amounts={:?} target={}",
                    amounts,
                    target,
                );
            }

            let ids: Vec<i64> = subset.iter().map(|e| e.id.0).collect();
            prop_assert_eq!(
                ids.len(),
                ids.iter().collect::<HashSet<_>>().len(),
                "duplicate expense chosen: amounts={:?} target={}",
                amounts,
                target,
            );
        }
    }

    proptest! {
        #[test]
        fn prop_structural_invariants(
            amounts in prop::collection::vec(0_i64..=10_000, 0..=20),
            target in 0_u64..=50_000,
        ) {
            let expenses: Vec<Expense> = amounts
                .iter()
                .enumerate()
                .map(|(i, &a)| make_expense(i64::try_from(i).unwrap(), a))
                .collect();

            let whole: u64 = amounts.iter().map(|&a| u64::try_from(a).unwrap()).sum();
            let res = closest_subset_to_target(&expenses, target);
            prop_assert!(res.is_ok(), "amounts={amounts:?} target={target}");
            let subset = res.unwrap();

            if target == 0 {
                prop_assert!(subset.is_empty(), "amounts={amounts:?} target={target}");
            } else if whole < target {
                prop_assert!(subset.is_empty(), "amounts={amounts:?} target={target}");
            } else {
                prop_assert!(!subset.is_empty(), "amounts={amounts:?} target={target}");
                let sum: u64 = subset
                    .iter()
                    .map(|e| u64::try_from(e.unit_amount.0).unwrap())
                    .sum();
                prop_assert!(sum >= target, "amounts={amounts:?} target={target}");

                let max_amt = amounts.iter().copied().max().unwrap();
                if max_amt > 0 {
                    prop_assert!(
                        sum < target.checked_add(u64::try_from(max_amt).unwrap()).unwrap(),
                        "amounts={amounts:?} target={target}",
                    );
                }

                let min_chosen = subset.iter().map(|e| e.unit_amount.0).min().unwrap();
                let min_chosen = u64::try_from(min_chosen).unwrap();
                let removed_sum = sum - min_chosen;
                prop_assert!(
                    removed_sum < target,
                    "subset minus its smallest element still meets target (not minimal): \
                     amounts={amounts:?} target={target}",
                );

                let ids: Vec<i64> = subset.iter().map(|e| e.id.0).collect();
                prop_assert_eq!(
                    ids.len(),
                    ids.iter().collect::<HashSet<_>>().len(),
                    "duplicate expense chosen: amounts={:?} target={}",
                    amounts,
                    target,
                );
            }
        }
    }

    proptest! {
        #[test]
        fn prop_negative_amounts_error(
            amounts in prop::collection::vec(-50_i64..=50, 0..=20),
            target in 0_u64..=1_000,
        ) {
            let expenses: Vec<Expense> = amounts
                .iter()
                .enumerate()
                .map(|(i, &a)| make_expense(i64::try_from(i).unwrap(), a))
                .collect();

            let res = closest_subset_to_target(&expenses, target);
            if amounts.is_empty() {
                prop_assert!(res.is_ok(), "amounts={amounts:?} target={target}");
                prop_assert!(res.unwrap().is_empty(), "amounts={amounts:?} target={target}");
            } else if amounts.iter().any(|&a| a < 0) {
                prop_assert!(res.is_err(), "expected error: amounts={amounts:?} target={target}");
            } else {
                prop_assert!(res.is_ok(), "expected ok: amounts={amounts:?} target={target}");
            }
        }
    }
}
