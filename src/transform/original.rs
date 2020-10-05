use super::Transform;
use crate::column_ref::ColumnRef;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::Write;

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Original(pub(super) ColumnRef);

impl Transform for Original {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()> {
        check_ref(&self.0)
    }

    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()> {
        writer.write_field(&self.0.header).context("write header")
    }

    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()> {
        writer
            .write_field(&record[self.0.index])
            .context("write field")
    }
}
