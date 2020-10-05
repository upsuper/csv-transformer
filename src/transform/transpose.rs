use super::Transform;
use crate::column_ref::ColumnRef;
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;

/// Transpose values and their header across several columns
///
/// For example, it allows to transform
///
/// | Question? 1st | Question? 2nd | Question? 3rd |
/// | ------------- | ------------- | ------------- |
/// | Rust          | C++           | C             |
/// | Python        | C             | Rust          |
///
/// into
///
/// | Question? Rust | Question? C++ | Question? C | Question? Python |
/// | -------------- | ------------- | ----------- | ---------------- |
/// | 1st            | 2nd           | 3rd         |                  |
/// | 3rd            |               | 2nd         | 1st              |
///
/// If a value presents in multiple source columns, the first matching one would be picked.
///
/// It's an error if a non-empty value in source columns can't be mapped to a target column.
#[derive(Deserialize)]
pub struct Transpose {
    /// Source columns and the new value they map to in the new columns
    #[serde(deserialize_with = "de::deserialize_pair_seq")]
    sources: Vec<(ColumnRef, String)>,
    /// New columns' headers and corresponding values they represent in the source columns
    #[serde(deserialize_with = "de::deserialize_pair_seq")]
    columns: Vec<(String, String)>,
}

impl Transform for Transpose {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()> {
        self.sources
            .iter()
            .map(|(c, _)| check_ref(c))
            .collect::<Result<()>>()?;
        // Validate that each new column takes different values from the old columns.
        let mut value_to_new_column = HashMap::new();
        for (header, value) in self.columns.iter() {
            ensure!(
                !value.is_empty(),
                "transpose column corresponds to empty value: {}",
                header,
            );
            value_to_new_column
                .entry(value.as_str())
                .or_insert_with(|| Vec::new())
                .push(header.as_str());
        }
        for (value, headers) in value_to_new_column.iter() {
            ensure!(
                headers.len() == 1,
                "multiple transpose columns share the same value `{}`: {}",
                value,
                headers.join(", "),
            );
        }
        Ok(())
    }

    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()> {
        self.columns
            .iter()
            .map(|(h, _)| writer.write_field(h).context("write field"))
            .collect()
    }

    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()> {
        // Verify that every value has a column to go.
        for (col, _) in self.sources.iter() {
            let value = &record[col.index];
            if value.is_empty() {
                continue;
            }
            let target_column = self.columns.iter().find(|(_, v)| v == value);
            ensure!(
                target_column.is_some(),
                "value `{}` from column `{}` isn't a match to any new column",
                value,
                col.header,
            );
        }
        self.columns
            .iter()
            .map(|(_, value)| {
                let source = self.sources.iter().find(|(c, _)| &record[c.index] == value);
                writer
                    .write_field(source.map_or("", |(_, v)| v.as_str()))
                    .context("write field")
            })
            .collect()
    }
}

mod de {
    use serde::de::{MapAccess, Visitor};
    use serde::{Deserialize, Deserializer};
    use std::fmt;
    use std::marker::PhantomData;

    pub fn deserialize_pair_seq<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
    where
        D: Deserializer<'de>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        deserializer.deserialize_map(PairSeqVisitor(PhantomData))
    }

    struct PairSeqVisitor<K, V>(PhantomData<fn() -> Vec<(K, V)>>);

    impl<'de, K, V> Visitor<'de> for PairSeqVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = Vec<(K, V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut result = Vec::with_capacity(map.size_hint().unwrap_or(0));
            while let Some((key, value)) = map.next_entry()? {
                result.push((key, value));
            }
            Ok(result)
        }
    }
}
