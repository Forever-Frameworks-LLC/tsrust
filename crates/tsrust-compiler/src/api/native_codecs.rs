//! Native schema composition and checked materialization into inferred DTO types.
//! The generated generic adapters are specialized and checked by the ordinary
//! compiler. They reconstruct records/tuples; they never assert JSON as a DTO.
use super::Schema;
use std::collections::BTreeMap;

fn text(value: &str) -> String { serde_json::to_string(value).expect("source string") }

#[derive(Clone)]
pub(super) struct Codec { pub ty: String, pub schema: String, pub adapt: String }

#[derive(Default)]
pub(super) struct Codecs {
    pub source: String,
    next: usize,
    seen: BTreeMap<(String, bool), Codec>,
}

impl Codecs {
    pub fn define(&mut self, shape: &Schema, strip: bool) -> Codec {
        let key = (serde_json::to_string(shape).expect("schema identity"), strip);
        if let Some(codec) = self.seen.get(&key) { return codec.clone(); }
        let id = self.next;
        self.next += 1;
        let schema = format!("_tsrustSchema{id}");
        let adapt = format!("_tsrustAdapt{id}");
        let named = format!("_TsrustWire{id}");
        let (ty, plan, body) = match shape {
            Schema::String => ("string".into(), "s.string()".into(), "return clone(value);".into()),
            Schema::Integer => ("int".into(), "s.int().min(-9007199254740991).max(9007199254740991)".into(), "return value;".into()),
            Schema::Number => ("number".into(), "s.number()".into(), "return value;".into()),
            Schema::Boolean => ("boolean".into(), "s.boolean()".into(), "return value;".into()),
            Schema::Null | Schema::Void => ("null".into(), "s.null()".into(), "return null;".into()),
            Schema::Literal { value } => (value.to_string(), format!("s.literal({value})"), "return clone(value);".into()),
            Schema::Object { properties, required } => {
                let mut fields = Vec::new();
                let mut plans = Vec::new();
                let mut values = Vec::new();
                for (key, child) in properties {
                    let literal = if let Schema::Literal { value } = child { Some(value.to_string()) } else { None };
                    let child = self.define(child, strip);
                    let optional = !required.contains(key);
                    let key = text(key);
                    fields.push(format!("{key}{}:{};", if optional { "?" } else { "" }, child.ty));
                    plans.push(format!("{key}:{}(){}", child.schema, if optional { ".optional()" } else { "" }));
                    // Keep known discriminants visible while the ordinary
                    // compiler selects a concrete record in a target union.
                    let value = literal.unwrap_or_else(|| format!("{}(value[{key}])", child.adapt));
                    values.push(format!("{key}:{}", if optional { format!("value[{key}]===undefined?undefined:{value}") } else { value }));
                }
                self.source.push_str(&format!("type {named}={{{}}};\n", fields.join("")));
                (named, format!("s.object({{{}}}){}", plans.join(","), if strip { ".strip()" } else { "" }), format!("return {{{}}};", values.join(",")))
            }
            Schema::Array { items } => {
                let child = self.define(items, strip);
                self.source.push_str(&format!("type {named}={}[];\n", child.ty));
                (named, format!("s.array({}())", child.schema), format!("const result:T=[];for(const element of value){{result.push({}(element));}}return result;", child.adapt))
            }
            Schema::Tuple { items } => {
                let children = items.iter().map(|child| self.define(child, strip)).collect::<Vec<_>>();
                self.source.push_str(&format!("type {named}=[{}];\n", children.iter().map(|child| child.ty.as_str()).collect::<Vec<_>>().join(",")));
                let element = match children.as_slice() {
                    [] => "s.null()".into(),
                    [only] => format!("{}()", only.schema),
                    _ => format!("s.union([{}])", children.iter().map(|child| format!("{}()", child.schema)).collect::<Vec<_>>().join(",")),
                };
                let mut refinement = String::new();
                let mut projection = Vec::new();
                let mut adaptation = Vec::new();
                for (index, child) in children.iter().enumerate() {
                    refinement.push_str(&format!("const checked{index}={}().safeParseJson(JSON.stringify(values[{index}]));if(!checked{index}.success){{return false;}}", child.schema));
                    projection.push(format!("{}({}().parse(JSON.parse(JSON.stringify(values[{index}]))))", child.adapt, child.schema));
                    adaptation.push(format!("{}(value[{index}])", child.adapt));
                }
                // ponytail: tuple composition revalidates each element. Replace
                // with a direct shared tuple plan if bridge measurements justify it.
                let plan = format!("s.array({element}).refine(values=>values.length==={}).refine(values=>{{{refinement}return true;}}).transform((values):{named}=>[{}])", children.len(), projection.join(","));
                (named, plan, format!("return [{}];", adaptation.join(",")))
            }
            Schema::Union { members } => {
                let children = members.iter().map(|child| self.define(child, strip)).collect::<Vec<_>>();
                self.source.push_str(&format!("type {named}={};\n", children.iter().map(|child| child.ty.as_str()).collect::<Vec<_>>().join("|")));
                let mut body = "const encoded=JSON.stringify(value);".to_owned();
                for (index, child) in children.iter().enumerate() {
                    body.push_str(&format!("const checked{index}={}().safeParseJson(encoded);if(checked{index}.success){{return {}(checked{index}.data);}}", child.schema, child.adapt));
                }
                body.push_str("throw 'Validated API union could not be materialized';");
                (named, format!("s.union([{}])", children.iter().map(|child| format!("{}()", child.schema)).collect::<Vec<_>>().join(",")), body)
            }
        };
        self.source.push_str(&format!("function {schema}(){{return {plan};}}\nfunction {adapt}<T,V>(value:V):T{{{body}}}\n"));
        let codec = Codec { ty, schema, adapt };
        self.seen.insert(key, codec.clone());
        codec
    }
}
