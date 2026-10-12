//! Validated source-backed JSON fragments with exact-text serialization.
use super::{Error, Limits};

/// Holds only a real borrow of the validated UTF-8 input. No dynamic tree or
/// replacement source buffer is retained. Serde may use temporary escape scratch.
pub struct Fragment<'a> { text: &'a str, depth: usize }
// Internal field adapter: callers must use typed::decode_borrowed so whole-input
// budgets and strict Unicode validation run before Serde extracts source slices.
impl<'de> serde::Deserialize<'de> for Fragment<'de> {
    fn deserialize<D:serde::Deserializer<'de>>(decoder:D)->Result<Self,D::Error>{
        let raw=<&'de serde_json::value::RawValue as serde::Deserialize<'de>>::deserialize(decoder)?;
        Ok(Self::validated(raw.get()))
    }
}
impl<'a> Fragment<'a> {
    // Both callers provide validated text. Cache only structural depth, retaining
    // the same real source borrow and no additional heap allocation.
    fn validated(text: &'a str) -> Self {
        let mut depth=0;
        let mut maximum=0;
        let mut quoted=false;
        let mut escaped=false;
        // Input is validated before Fragment construction. Only unquoted brackets
        // affect depth; escaped quotes and bracket characters in strings do not.
        for byte in text.bytes() {
            if quoted {
                if escaped { escaped=false; }
                else if byte==b'\\' { escaped=true; }
                else if byte==b'"' { quoted=false; }
            } else {
                match byte {
                    b'"'=>quoted=true,
                    b'['|b'{'=>{
                        depth+=1;maximum=maximum.max(depth);
                    },
                    b']'|b'}'=>depth-=1,
                    _=>{},
                }
            }
        }
        Self{text,depth:maximum}
    }

    pub fn text(&self) -> &'a str { self.text }
}

pub fn borrow(input: &str, limits: Limits) -> Result<Fragment<'_>, Error> {
    let depth=super::typed::validate_depth(input.as_bytes(), limits)?;
    Ok(Fragment{text:input,depth})
}

/// Internal serializer for an already validated fragment at its enclosing depth.
/// This uses the pinned serde_json raw-value serialization protocol because
/// deserializing a RawValue discards surrounding whitespace. No unsafe cast or
/// owned intermediate string is needed; the bounded output writer counts bytes.
pub struct Encoded<'a, 'source>(pub &'a Fragment<'source>, pub usize);
impl serde::Serialize for Encoded<'_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{Error, SerializeStruct};
        if self.1>64 || self.0.depth>64-self.1 {
            return Err(S::Error::custom("tsrust:depth-limit"));
        }
        const TOKEN:&str="$serde_json::private::RawValue";
        let mut raw=serializer.serialize_struct(TOKEN,1)?;
        raw.serialize_field(TOKEN,self.0.text)?;
        raw.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> Limits { Limits { bytes: 128, depth: 4, entries: 4, nodes: 12 } }
    #[test]
    fn raw_encoding_rejects_invalid_parent_depth_without_overflow() {
        let scalar=borrow("null",limits()).unwrap();
        for depth in [65,usize::MAX] {
            assert_eq!(super::super::typed::encode(&Encoded(&scalar,depth),128).unwrap_err().code,"depth-limit");
        }
    }
    #[test]
    fn embedded_serialization_preserves_text_and_combined_limits() {
        let text=" \n{\"z\":1e+02,\"a\":\"\\u0061\"} \t";
        let raw=borrow(text,limits()).unwrap();
        assert_eq!(super::super::typed::encode(&Encoded(&raw,0),128).unwrap(),text);
        assert_eq!(super::super::typed::encode(&vec![Encoded(&raw,1)],128).unwrap(),format!("[{text}]"));
        assert_eq!(super::super::typed::encode(&Encoded(&raw,0),text.len()-1).unwrap_err().code,"output-limit");
        assert_eq!(super::super::typed::encode(&Encoded(&raw,64),128).unwrap_err().code,"depth-limit");
        let scalar=borrow("null",limits()).unwrap();
        assert_eq!(super::super::typed::encode(&Encoded(&scalar,64),128).unwrap(),"null");
    }
    #[test]
    fn selective_fields_borrow_exact_source_tokens() {
        let source=String::from(r#"{"payload": { "n":18446744073709551615, "s":"\u0061" }}"#);
        let values:std::collections::BTreeMap<String,Fragment<'_>>=super::super::typed::decode_borrowed(source.as_bytes(),limits()).unwrap();
        let raw=values["payload"].text();
        assert_eq!(raw,r#"{ "n":18446744073709551615, "s":"\u0061" }"#);
        assert_eq!(raw.as_ptr(),source[12..].as_ptr());
        assert!(super::super::typed::decode_borrowed::<std::collections::BTreeMap<String,Fragment<'_>>>(br#"{"payload":"\ud800"}"#,limits()).is_err());
        assert!(super::super::typed::decode_borrowed::<std::collections::BTreeMap<String,Fragment<'_>>>(source.as_bytes(),Limits{nodes:1,..limits()}).is_err());
    }
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
