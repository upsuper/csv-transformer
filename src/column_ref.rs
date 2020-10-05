use serde::de;
use serde::de::Unexpected;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str;
use std::str::FromStr;

pub struct ColumnRef {
    pub index: usize,
    pub header: String,
}

pub struct InvalidColumnRef;

impl FromStr for ColumnRef {
    type Err = InvalidColumnRef;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s.find(':').ok_or(InvalidColumnRef)?;
        let index = ref_to_index(&s[..pos]).ok_or(InvalidColumnRef)?;
        Ok(ColumnRef {
            index,
            header: s[pos + 1..].trim().to_string(),
        })
    }
}

impl fmt::Display for ColumnRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = [0; 16];
        write!(f, "{}: {}", index_to_ref(self.index, &mut buf), self.header)
    }
}

impl Serialize for ColumnRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ColumnRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColumnRefVisitor)
    }
}

struct ColumnRefVisitor;

impl<'de> de::Visitor<'de> for ColumnRefVisitor {
    type Value = ColumnRef;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("column reference string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ColumnRef::from_str(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &""))
    }
}

fn index_to_ref(mut index: usize, buf: &mut [u8]) -> &str {
    let mut iter = buf.iter_mut().enumerate().rev();
    let (mut pos, dest) = iter.next().unwrap();
    *dest = b'A' + (index % 26) as u8;
    index /= 26;
    while index > 0 {
        let (next_pos, dest) = iter.next().unwrap();
        pos = next_pos;
        index -= 1;
        *dest = b'A' + (index % 26) as u8;
        index /= 26;
    }
    str::from_utf8(&buf[pos..]).unwrap()
}

fn ref_to_index(s: &str) -> Option<usize> {
    let mut iter = s.bytes();
    let mut index = match iter.next() {
        Some(b) if is_ref_byte(b) => (b - b'A') as usize,
        _ => return None,
    };
    for b in iter {
        if !(b'A'..=b'Z').contains(&b) {
            return None;
        }
        index = (index + 1) * 26;
        index += (b - b'A') as usize;
    }
    Some(index)
}

fn is_ref_byte(b: u8) -> bool {
    (b'A'..=b'Z').contains(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_index_to_ref() {
        let mut buf = [0; 10];
        let mut index = 0;
        check_refs(move |expected: &'_ [u8]| {
            assert_eq!(
                index_to_ref(index, &mut buf),
                str::from_utf8(expected).unwrap()
            );
            index += 1;
        });
    }

    #[test]
    fn check_ref_to_index() {
        let mut index = 0;
        check_refs(move |input: &'_ [u8]| {
            assert_eq!(ref_to_index(str::from_utf8(input).unwrap()), Some(index));
            index += 1;
        });
    }

    fn check_refs(mut check_next: impl FnMut(&'_ [u8])) {
        for x in b'A'..=b'Z' {
            check_next(&[x]);
        }
        for x in b'A'..=b'Z' {
            for y in b'A'..=b'Z' {
                check_next(&[x, y]);
            }
        }
        for x in b'A'..=b'Z' {
            for y in b'A'..=b'Z' {
                for z in b'A'..=b'Z' {
                    check_next(&[x, y, z]);
                }
            }
        }
    }
}
