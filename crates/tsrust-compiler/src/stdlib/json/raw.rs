//! Validated source-backed JSON fragments; output policy is handled separately.
use super::{Error, Limits};

/// Holds only a real borrow of the validated UTF-8 input. No dynamic tree or
/// replacement source buffer is retained. Serde may use temporary escape scratch.
pub struct Fragment<'a> { text: &'a str }
impl<'a> Fragment<'a> {
    pub fn text(&self) -> &'a str { self.text }
}

pub fn borrow(input: &str, limits: Limits) -> Result<Fragment<'_>, Error> {
    super::typed::validate(input.as_bytes(), limits)?;
    Ok(Fragment { text: input })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> Limits { Limits { bytes: 128, depth: 4, entries: 4, nodes: 12 } }
    #[test]
    fn fragments_borrow_source_without_rewriting_tokens() {
        let source = String::from(" \n{\"z\":18446744073709551615,\"a\":-0.0,\"text\":\"\\u0061\"} \t");
        let fragment = borrow(&source, limits()).unwrap();
        assert_eq!(fragment.text(), source);
        assert_eq!(fragment.text().as_ptr(), source.as_ptr());
        assert_eq!(fragment.text().len(), source.len());
    }
    #[test]
    fn fragment_validation_enforces_all_input_budgets() {
        for (source, limits, code) in [
            ("null ", Limits { bytes: 4, ..limits() }, "byte-limit"),
            ("[[0]]", Limits { depth: 1, ..limits() }, "depth-limit"),
            ("[0,1]", Limits { entries: 1, ..limits() }, "entry-limit"),
            ("[0]", Limits { nodes: 1, ..limits() }, "node-limit"),
            ("null true", limits(), "trailing-data"),
            ("/*comment*/null", limits(), "invalid-json"),
            ("\"\\ud800\"", limits(), "invalid-json"),
        ] {
            assert_eq!(borrow(source, limits).err().unwrap().code, code, "{source}");
        }
        assert_eq!(borrow("null", Limits { depth: 65, ..limits() }).err().unwrap().code, "invalid-limits");
    }
}
