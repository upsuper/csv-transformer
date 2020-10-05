use super::Transform;
use crate::column_ref::ColumnRef;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::borrow::Cow;
use std::io::Write;

/// Join multiple columns together
#[derive(Deserialize)]
pub struct Join {
    /// New header of the joined column
    header: String,
    columns: Vec<ValueMap>,
    /// Separator of values, `", "` by default
    sep: Option<String>,
}

struct ValueMap(ValueMapInternal);

#[derive(Deserialize)]
struct ValueMapInternal {
    column: ColumnRef,
    /// Format to transform the value, using `{}` for the original value
    format: Option<String>,
}

impl Transform for Join {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()> {
        self.columns
            .iter()
            .map(|c| check_ref(&c.0.column))
            .collect()
    }

    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()> {
        writer.write_field(&self.header).context("write header")
    }

    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()> {
        let sep = self.sep.as_deref().unwrap_or(", ");
        let values = self.columns.iter().filter_map(|c| {
            let ValueMapInternal { column, format } = &c.0;
            match record[column.index].trim() {
                "" => None,
                value => Some(if let Some(format) = &format {
                    Cow::Owned(format.replace("{}", value))
                } else {
                    Cow::Borrowed(value)
                }),
            }
        });
        writer
            .write_field(itertools::join(values, sep))
            .context("write field")
    }
}

mod de {
    use super::{ValueMap, ValueMapInternal};
    use crate::column_ref::ColumnRef;
    use serde::de::value::MapAccessDeserializer;
    use serde::de::{self, IntoDeserializer, MapAccess, Visitor};
    use serde::{Deserialize, Deserializer};
    use std::fmt;

    impl<'de> Deserialize<'de> for ValueMap {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(ValueMapVisitor).map(ValueMap)
        }
    }

    struct ValueMapVisitor;

    impl<'de> Visitor<'de> for ValueMapVisitor {
        type Value = ValueMapInternal;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("column reference string or transformation object")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let column = ColumnRef::deserialize(v.into_deserializer())?;
            Ok(ValueMapInternal {
                column,
                format: None,
            })
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            Deserialize::deserialize(MapAccessDeserializer::new(map))
        }
    }
}
