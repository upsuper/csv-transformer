use super::Transform;
use crate::column_ref::ColumnRef;
use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::io::Write;

/// Reformat the timestamp with the given spec
#[derive(Deserialize)]
pub struct Timestamp {
    column: ColumnRef,
    /// Optional header, if omitted, the header of the reference column would be used
    header: Option<String>,
    /// Format to parse the timestamp in syntax of chrono's strftime
    from: String,
    /// Format to serialize the timestamp in syntax of chrono's strftime
    to: String,
}

impl Transform for Timestamp {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()> {
        check_ref(&self.column)
    }

    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()> {
        writer
            .write_field(self.header.as_deref().unwrap_or(&self.column.header))
            .context("write header")
    }

    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()> {
        let value = &record[self.column.index];
        let time = NaiveDateTime::parse_from_str(value, &self.from)
            .with_context(|| format!("parse timestamp: {}", value))?;
        writer
            .write_field(time.format(&self.to).to_string())
            .context("write field")
    }
}
