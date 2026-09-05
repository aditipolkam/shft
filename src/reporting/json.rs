use crate::{error::Result, reporting::Report};

pub fn print(report: &Report) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}
