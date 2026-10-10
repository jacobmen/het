use anyhow::Result;
use assert_cmd::Command;

pub const EXPENSE_A: &str = "expense_a.pdf";
pub const EXPENSE_A_BYTES: &[u8] = b"pdf data for a";

pub fn het() -> Result<Command> {
    let mut cmd = Command::cargo_bin("het")?;
    cmd.env_remove("XDG_CONFIG_HOME");
    cmd.env_remove("XDG_DATA_HOME");
    Ok(cmd)
}
