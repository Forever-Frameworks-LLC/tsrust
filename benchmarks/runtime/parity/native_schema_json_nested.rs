// Hand-written typed serde decoding for the fixed nested-schema workload.
// Ordinary owned Vecs, checked integer arithmetic, and validation during decode.
type String = std::borrow::Cow<'static, str>;

#[derive(serde::Deserialize)]
#[serde(try_from = "i64")]
struct BoundedInt(i64);

impl TryFrom<i64> for BoundedInt {
    type Error = &'static str;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if (0..=1000).contains(&value) { Ok(Self(value)) } else { Err("Integer outside schema bounds") }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Group { values: Vec<BoundedInt> }

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Document { groups: Vec<Group> }

fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let (n, seed, input) = (value0, value1, value2);
    let mut checksum = seed;
    for _ in 0..n {
        let value: Document = serde_json::from_str(input).expect("Expected nested success");
        assert_eq!(value.groups.len(), 4, "Wrong groups");
        for group in value.groups {
            assert_eq!(group.values.len(), 16, "Wrong values");
            for item in group.values {
                checksum = checksum.checked_add(item.0).expect("integer addition overflow");
            }
        }
    }
    checksum
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoder_checks_fields_types_and_bounds() {
        for input in [
            r#"{"groups":[{"values":[-1]}]}"#,
            r#"{"groups":[{"values":[1001]}]}"#,
            r#"{"groups":[{"values":[1.5]}]}"#,
            r#"{"groups":[{"values":[],"extra":true}]}"#,
            r#"{"groups":[{}]}"#,
        ] { assert!(serde_json::from_str::<Document>(input).is_err(), "{input}"); }
        let valid: Document = serde_json::from_str(r#"{"groups":[{"values":[0,1000]}]}"#).unwrap();
        assert_eq!(valid.groups[0].values[1].0, 1000);
    }
}
