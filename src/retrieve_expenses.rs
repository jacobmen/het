use std::path::Path;

use crate::{
    algo::closest_subset_to_target,
    expense_service::{Expense, ExpenseId, ExpenseService},
    sql::Repository,
};
use anyhow::{Result, anyhow};

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
    todo!()
}
