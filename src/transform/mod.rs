use crate::column_ref::ColumnRef;
use anyhow::Result;
use serde::Deserialize;
use std::io::Write;

mod join;
mod original;
mod rename;
mod timestamp;
mod transpose;

#[derive(Deserialize)]
#[serde(transparent)]
pub struct TransformedColumns {
    #[serde(deserialize_with = "de::deserialize_internal")]
    internal: Internal,
}

pub trait Transform {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()>;
    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()>;
    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()>;
}

#[derive(Deserialize)]
#[serde(tag = "transform")]
#[serde(rename_all = "kebab-case")]
enum Internal {
    #[serde(skip)]
    Original(original::Original),
    Rename(rename::Rename),
    Timestamp(timestamp::Timestamp),
    Join(join::Join),
    Transpose(transpose::Transpose),
}

impl Transform for TransformedColumns {
    fn validate(&self, check_ref: impl Fn(&ColumnRef) -> Result<()>) -> Result<()> {
        match &self.internal {
            Internal::Original(o) => o.validate(check_ref),
            Internal::Timestamp(t) => t.validate(check_ref),
            Internal::Rename(r) => r.validate(check_ref),
            Internal::Join(j) => j.validate(check_ref),
            Internal::Transpose(t) => t.validate(check_ref),
        }
    }

    fn write_headers(&self, writer: &mut csv::Writer<impl Write>) -> Result<()> {
        match &self.internal {
            Internal::Original(o) => o.write_headers(writer),
            Internal::Timestamp(t) => t.write_headers(writer),
            Internal::Rename(r) => r.write_headers(writer),
            Internal::Join(j) => j.write_headers(writer),
            Internal::Transpose(t) => t.write_headers(writer),
        }
    }

    fn write_fields(&self, record: &[String], writer: &mut csv::Writer<impl Write>) -> Result<()> {
        match &self.internal {
            Internal::Original(o) => o.write_fields(record, writer),
            Internal::Timestamp(t) => t.write_fields(record, writer),
            Internal::Rename(r) => r.write_fields(record, writer),
            Internal::Join(j) => j.write_fields(record, writer),
            Internal::Transpose(t) => t.write_fields(record, writer),
        }
    }
}

mod de {
    use super::Internal;
    use crate::column_ref::ColumnRef;
    use serde::de::value::MapAccessDeserializer;
    use serde::de::{self, IntoDeserializer, MapAccess, Visitor};
    use serde::{Deserialize, Deserializer};
    use std::fmt;

    pub(super) fn deserialize_internal<'de, D>(deserializer: D) -> Result<Internal, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(InternalVisitor)
    }

    struct InternalVisitor;

    impl<'de> Visitor<'de> for InternalVisitor {
        type Value = Internal;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("column reference string or transformation object")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let column_ref = ColumnRef::deserialize(v.into_deserializer())?;
            Ok(Internal::Original(super::original::Original(column_ref)))
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            Deserialize::deserialize(MapAccessDeserializer::new(map))
        }
    }
}
