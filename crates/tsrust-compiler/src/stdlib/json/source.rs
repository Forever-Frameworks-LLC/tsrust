//! Validated source cursors for schema decoding without an intermediate value tree.
use super::{Error, Limits};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// A cursor borrows the original document. Child cursors retain that same lifetime;
/// owned strings and containers are constructed only when the decoder requests them.
#[derive(Clone, Copy, Debug)]
pub struct Node<'source> { text: &'source str }

pub fn document(input: &str, limits: Limits) -> Result<Node<'_>, Error> {
    // Integer tokens are re-scanned only when one could have overflowed into a float.
    if super::typed::validate(input.as_bytes(), limits)? {
        super::validate_integer_tokens(input.as_bytes())?;
    }
    Ok(Node { text: input.trim() })
}

/// Schema decoding follows the underlying Rust parser's default recursion guard.
/// Complete syntax and integer validation still precede all schema callbacks.
pub fn document_plain(input:&str)->Result<Node<'_>,Error>{
    if super::typed::validate_plain(input)? {
        super::validate_integer_tokens(input.as_bytes())?;
    }
    Ok(Node{text:input.trim()})
}

impl<'source> Node<'source> {
    pub fn text(self) -> &'source str { self.text }
    pub fn kind(self) -> &'static str {
        match self.text.as_bytes()[0] {
            b'n' => "null", b't' | b'f' => "boolean", b'"' => "string",
            b'[' => "array", b'{' => "object", _ => "number",
        }
    }
    /// The document was already checked, including ignored fields and duplicates.
    /// Re-reading a cursor never repeats global budget accounting or user callbacks.
    pub fn read<T: serde::Deserialize<'source>>(self) -> Result<T, Error> {
        serde_json::from_str(self.text).map_err(|error| {
            let message=error.to_string();
            let code=["number-precision", "integer-range", "integer-fractional"]
                .into_iter().find(|code| message.starts_with(&format!("tsrust:{code}")))
                .unwrap_or("shape-mismatch");
            super::located(self.text.as_bytes(),error,Some(code))
        })
    }
    pub fn int(self) -> Result<i64, Error> { self.read::<super::typed::ExactInt>().map(|value|value.0) }
    pub fn number(self) -> Result<f64, Error> { self.read::<super::typed::ExactNumber>().map(|value|value.0) }
    pub fn boolean(self) -> Result<bool, Error> { self.read() }
    pub fn string(self) -> Result<Cow<'source,str>, Error> { self.read::<Text<'source>>().map(|value|value.0) }

    /// Decode only the field index. Values remain source slices, including nested
    /// containers. Last duplicate wins before any field validator can run.
    pub fn fields(self) -> Result<BTreeMap<Cow<'source,str>,Node<'source>>, Error> {
        self.read::<Fields<'source>>().map(|fields|fields.0)
    }

    /// Visit array elements in order without collecting a temporary element array.
    /// False stops callbacks; remaining source slices are discarded by the visitor.
    pub fn elements(self, mut visit: impl FnMut(usize, Node<'source>) -> bool) -> Result<(), Error> {
        struct Elements<'callback,F>(&'callback mut F);
        impl<'source,F:FnMut(usize,Node<'source>)->bool> serde::de::Visitor<'source> for Elements<'_,F> {
            type Value=();
            fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result {f.write_str("an array")}
            fn visit_seq<A:serde::de::SeqAccess<'source>>(self,mut sequence:A)->Result<(),A::Error> {
                let mut index=0;
                let mut active=true;
                while let Some(value)=sequence.next_element::<Node<'source>>()? {
                    if active {active=(self.0)(index,value);}
                    index+=1;
                }
                Ok(())
            }
        }
        let mut decoder=serde_json::Deserializer::from_str(self.text);
        serde::Deserializer::deserialize_seq(&mut decoder,Elements(&mut visit))
            .map_err(|error|super::located(self.text.as_bytes(),error,Some("shape-mismatch")))
    }

    /// Explicit retention requests an owned dynamic value as final output. Ordinary
    /// typed fields never call this conversion; it is not a validation bridge.
    pub fn retain(self) -> super::Value {
        serde_json::from_str(self.text).expect("validated source retained as JSON")
    }
}

// These adapters are private construction paths. They only read descendants of a
// validated document, so Serde's permissive RawValue scan cannot bypass validation.
impl<'source> serde::Deserialize<'source> for Node<'source> {
    fn deserialize<D:serde::Deserializer<'source>>(decoder:D)->Result<Self,D::Error> {
        let raw=<&'source serde_json::value::RawValue as serde::Deserialize>::deserialize(decoder)?;
        Ok(Self{text:raw.get()})
    }
}
struct Text<'source>(Cow<'source,str>);
impl<'source> serde::Deserialize<'source> for Text<'source> {
    fn deserialize<D:serde::Deserializer<'source>>(decoder:D)->Result<Self,D::Error> {
        struct Visitor;
        impl<'source> serde::de::Visitor<'source> for Visitor {
            type Value=Text<'source>;
            fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("a string")}
            fn visit_borrowed_str<E:serde::de::Error>(self,value:&'source str)->Result<Self::Value,E>{Ok(Text(Cow::Borrowed(value)))}
            fn visit_str<E:serde::de::Error>(self,value:&str)->Result<Self::Value,E>{Ok(Text(Cow::Owned(value.into())))}
            fn visit_string<E:serde::de::Error>(self,value:String)->Result<Self::Value,E>{Ok(Text(Cow::Owned(value)))}
        }
        decoder.deserialize_str(Visitor)
    }
}
struct Fields<'source>(BTreeMap<Cow<'source,str>,Node<'source>>);
impl<'source> serde::Deserialize<'source> for Fields<'source> {
    fn deserialize<D:serde::Deserializer<'source>>(decoder:D)->Result<Self,D::Error> {
        struct Visitor;
        impl<'source> serde::de::Visitor<'source> for Visitor {
            type Value=Fields<'source>;
            fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("an object")}
            fn visit_map<A:serde::de::MapAccess<'source>>(self,mut object:A)->Result<Self::Value,A::Error> {
                let mut fields=BTreeMap::new();
                while let Some(Text(key))=object.next_key::<Text<'source>>()? {
                    fields.insert(key,object.next_value()?);
                }
                Ok(Fields(fields))
            }
        }
        decoder.deserialize_map(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits()->Limits {Limits{bytes:4096,depth:8,entries:32,nodes:64}}

    #[test]
    fn plain_document_matches_serde_recursion_and_checks_discarded_content() {
        for depth in [64,65,126,127,128,129] {
            let input="[".repeat(depth)+"0"+&"]".repeat(depth);
            assert_eq!(document_plain(&input).is_ok(),serde_json::from_str::<serde_json::Value>(&input).is_ok(),"depth {depth}");
        }
        for source in [r#"{"x":"\ud800","x":0}"#,r#"{"x":18446744073709551616,"x":0}"#,"1e400","[1] null"] {
            assert!(document_plain(source).is_err(),"{source}");
        }
        let source=r#"{"x":9007199254740993,"escaped":"\u00e9"}"#;
        let fields=document_plain(source).unwrap().fields().unwrap();
        assert_eq!(fields["x"].int().unwrap(),9_007_199_254_740_993);
        assert_eq!(fields["escaped"].string().unwrap(),"é");
    }

    #[test]
    fn fields_borrow_original_source_and_resolve_duplicates_before_validation() {
        let source=String::from(r#"{"name":"old","\u006eame":"new","nested":{"items":[1,2]}}"#);
        let fields=document(&source,limits()).unwrap().fields().unwrap();
        assert_eq!(fields.len(),2);
        assert_eq!(fields["name"].string().unwrap(),"new");
        let nested=fields["nested"].text();
        assert_eq!(nested,r#"{"items":[1,2]}"#);
        assert_eq!(nested.as_ptr(),source[source.find("{\"items").unwrap()..].as_ptr());
        let values=fields["nested"].fields().unwrap();
        let mut seen=Vec::new();
        values["items"].elements(|index,node|{seen.push((index,node.int().unwrap()));true}).unwrap();
        assert_eq!(seen,[(0,1),(1,2)]);
    }
    #[test]
    fn unicode_and_exact_numeric_contracts_match_typed_decoding() {
        let source=document(r#"["plain","\u00e9\ud83d\ude00",9007199254740993,-0.0,18446744073709551615]"#,limits()).unwrap();
        let mut nodes=Vec::new();source.elements(|_,node|{nodes.push(node);true}).unwrap();
        assert!(matches!(nodes[0].string().unwrap(),Cow::Borrowed("plain")));
        assert!(matches!(nodes[1].string().unwrap(),Cow::Owned(_)));
        assert_eq!(nodes[1].string().unwrap(),"é😀");
        assert_eq!(nodes[2].int().unwrap(),9_007_199_254_740_993);
        assert_eq!(nodes[2].number().unwrap_err().code,"number-precision");
        assert!(nodes[3].number().unwrap().is_sign_negative());
        assert_eq!(nodes[4].int().unwrap_err().code,"integer-range");
    }
    #[test]
    fn rejection_stops_callbacks_and_explicit_retention_is_owned() {
        let input=document("[1,2,3]",limits()).unwrap();let mut visited=0;
        input.elements(|_,_|{visited+=1;false}).unwrap();assert_eq!(visited,1);
        let output={let source=String::from(r#"{"nested":[1,"two"]}"#);document(&source,limits()).unwrap().retain()};
        assert_eq!(output["nested"][1],"two");
    }
    #[test]
    fn document_checks_ignored_duplicate_content_and_global_budgets() {
        for source in [r#"{"x":"\ud800","x":0}"#,r#"{"ignored":18446744073709551616}"#, "[1] null"] {
            assert!(document(source,limits()).is_err(),"{source}");
        }
        assert_eq!(document("[[0]]",Limits{depth:1,..limits()}).unwrap_err().code,"depth-limit");
        assert_eq!(document("[0,1]",Limits{nodes:2,..limits()}).unwrap_err().code,"node-limit");
        assert_eq!(document("[0,1]",Limits{entries:1,..limits()}).unwrap_err().code,"entry-limit");
    }
    #[test]
    fn source_cursor_limits_accept_exact_boundaries_and_count_duplicate_entries() {
        let exact=Limits{bytes:5,depth:1,entries:2,nodes:3};
        assert!(document("[0,1]",exact).is_ok());
        for (limits,code) in [
            (Limits{bytes:4,..exact},"byte-limit"),
            (Limits{depth:0,..exact},"depth-limit"),
            (Limits{entries:1,..exact},"entry-limit"),
            (Limits{nodes:2,..exact},"node-limit"),
        ] { assert_eq!(document("[0,1]",limits).unwrap_err().code,code); }
        let duplicate=r#"{"x":0,"x":1}"#;
        assert_eq!(document(duplicate,Limits{entries:1,..limits()}).unwrap_err().code,"entry-limit");
        assert_eq!(document(duplicate,Limits{nodes:2,..limits()}).unwrap_err().code,"node-limit");
        let fields=document(duplicate,Limits{entries:2,nodes:3,..limits()}).unwrap().fields().unwrap();
        assert_eq!(fields.len(),1);
        assert_eq!(fields["x"].int().unwrap(),1);
    }

}

#[cfg(test)]
mod scalar_cursor_tests {
    use super::*;
    #[test]
    fn cursor_categories_and_boolean_reads_do_not_coerce() {
        let limits=Limits{bytes:128,depth:4,entries:4,nodes:8};
        for (text,kind) in [("null","null"),("true","boolean"),("false","boolean"),("1","number"),("\"x\"","string"),("[]","array"),("{}","object")] {
            let node=document(text,limits).unwrap();assert_eq!(node.kind(),kind);
            match text {
                "true"=>assert!(node.boolean().unwrap()),
                "false"=>assert!(!node.boolean().unwrap()),
                _=>assert_eq!(node.boolean().unwrap_err().code,"shape-mismatch"),
            }
        }
    }
}
