use super::Transform;
use crate::column_ref::ColumnRef;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
pub struct Rename {
    header: String,
    column: ColumnRef,
}

impl Transform for Rename {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()> {
        check_ref(&self.column)
    }

    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()> {
        writer.write_field(&self.header).context("write header")
    }

    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()> {
        writer
            .write_field(&record[self.column.index])
            .context("write field")
    }
}
