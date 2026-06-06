use std::path::Path;

use crate::{expense_service::ExpenseService, sql::Repository};
use anyhow::Result;

pub fn retrieve_expenses<R: Repository>(
    _expense_service: &ExpenseService<R>,
    _dryrun: bool,
    _amount: f64,
    _out_path: &Path,
) -> Result<()> {
    Ok(())
}
