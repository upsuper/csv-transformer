use crate::column_ref::ColumnRef;
use crate::transform::{Transform, TransformedColumns};
use anyhow::{ensure, Context, Result};
use itertools::Itertools;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::iter;
use std::path::{Path, PathBuf};
use std::str;
use structopt::StructOpt;

mod column_ref;
mod transform;

#[derive(StructOpt)]
enum Action {
    Extract {
        #[structopt(parse(from_os_str))]
        original: PathBuf,
    },
    Transform {
        #[structopt(parse(from_os_str))]
        original: PathBuf,
        #[structopt(parse(from_os_str))]
        transform: PathBuf,
    },
}

fn main() -> Result<()> {
    match Action::from_args() {
        Action::Extract { original } => do_extract(&original),
        Action::Transform {
            original,
            transform,
        } => do_transform(&original, &transform),
    }
}

fn do_extract(original: &Path) -> Result<()> {
    let data = parse_csv(original).context("parse original file")?;
    let columns = data
        .headers
        .into_iter()
        .enumerate()
        .map(|(index, header)| ColumnRef { index, header })
        .collect_vec();
    let stdout = io::stdout();
    let stdout = stdout.lock();
    serde_yaml::to_writer(stdout, &columns).context("write extract result")?;
    Ok(())
}

fn do_transform(original: &Path, transform: &Path) -> Result<()> {
    let original_data = parse_csv(original).context("parse original file")?;
    let transform = File::open(transform).context("open transform file")?;
    let transform = BufReader::new(transform);
    let new_columns: Vec<TransformedColumns> =
        serde_yaml::from_reader(transform).context("parse transform file")?;

    // Check all the column references
    new_columns
        .iter()
        .map(|c| {
            c.validate(|col| {
                let is_valid = original_data
                    .headers
                    .get(col.index)
                    .map(|header| header == &col.header)
                    .unwrap_or(false);
                ensure!(is_valid, "invalid column reference: {}", col);
                Ok(())
            })
        })
        .collect::<Result<()>>()?;

    // Output the result
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let mut writer = csv::Writer::from_writer(stdout);
    // Write the header
    for col in new_columns.iter() {
        col.write_headers(&mut writer)?;
    }
    writer
        .write_record(iter::empty::<&[u8]>())
        .context("write header")?;
    // Write the records
    for (i, record) in original_data.values.iter().enumerate() {
        for col in new_columns.iter() {
            col.write_fields(&record, &mut writer)
                .with_context(|| format!("transform record {}", i))?;
        }
        writer
            .write_record(iter::empty::<&[u8]>())
            .context("write record")?;
    }

    Ok(())
}

struct CsvData {
    headers: Vec<String>,
    values: Vec<Vec<String>>,
}

fn parse_csv(path: &Path) -> Result<CsvData> {
    let mut reader = csv::Reader::from_path(path).context("open csv file")?;
    let headers = reader
        .headers()
        .context("read headers")?
        .iter()
        .map(|header| header.to_string())
        .collect();
    let values = reader
        .records()
        .enumerate()
        .map(|(i, record)| {
            let record = record.with_context(|| format!("read record {}", i))?;
            Ok(record.iter().map(|r| r.to_string()).collect())
        })
        .collect::<Result<_>>()?;
    Ok(CsvData { headers, values })
}
