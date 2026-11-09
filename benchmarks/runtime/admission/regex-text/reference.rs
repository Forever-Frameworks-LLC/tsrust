// Admission prototype only; no generated API or accepted facade performance.
// Exact regex1.13.1 default features, index checksum:
// f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d.
use regex::{Regex, RegexBuilder};
#[derive(Clone, Debug, PartialEq, Eq)]
struct Range { start: i64, end: i64 }
fn range(value: regex::Match<'_>) -> Range { Range { start:i64::try_from(value.start()).expect("UTF-8 offset exceeds int"), end:i64::try_from(value.end()).expect("UTF-8 offset exceeds int") } }
struct Compiled { inner: Regex }
impl Compiled {
    fn compile(pattern:&str)->Result<Self,regex::Error>{RegexBuilder::new(pattern).build().map(|inner|Self{inner})}
    fn is_match(&self,text:&str)->bool{self.inner.is_match(text)}
    fn find(&self,text:&str)->Option<Range>{self.inner.find(text).map(range)}
    fn find_all(&self,text:&str)->Vec<Range>{self.inner.find_iter(text).map(range).collect()}
    fn captures(&self,text:&str)->Option<Vec<Option<Range>>>{self.inner.captures(text).map(|value|value.iter().map(|slot|slot.map(range)).collect())}
    fn captures_all(&self,text:&str)->Vec<Vec<Option<Range>>>{self.inner.captures_iter(text).map(|value|value.iter().map(|slot|slot.map(range)).collect()).collect()}
    fn replace(&self,text:&str,replacement_template:&str)->String{self.inner.replace(text,replacement_template).into_owned()}
    fn replace_all(&self,text:&str,replacement_template:&str)->String{self.inner.replace_all(text,replacement_template).into_owned()}
    fn split(&self,text:&str)->Vec<Range>{let mut start=0;let mut output=Vec::new();for found in self.inner.find_iter(text){output.push(Range{start,end:i64::try_from(found.start()).expect("UTF-8 offset exceeds int")});start=i64::try_from(found.end()).expect("UTF-8 offset exceeds int");}output.push(Range{start,end:i64::try_from(text.len()).expect("UTF-8 offset exceeds int")});output}
    // String.match/matchAll/search delegate these same borrowed operations.
    fn string_match(&self,text:&str)->Option<Vec<Option<Range>>>{self.captures(text)}
    fn string_match_all(&self,text:&str)->Vec<Vec<Option<Range>>>{self.captures_all(text)}
    fn string_search(&self,text:&str)->i64{self.find(text).map_or(-1,|value|value.start)}
}
fn quote(value:&str)->String{let mut out=String::from("\"");for ch in value.chars(){match ch{'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),ch if ch<' '=>out.push_str(&format!("\\u{:04x}",ch as u32)),ch=>out.push(ch)}}out.push('"');out}
fn range_json(value:&Range)->String{format!("[{},{}]",value.start,value.end)}
fn optional_json(value:Option<&Range>)->String{value.map_or_else(||"null".into(),range_json)}
fn slots_json(values:&[Option<Range>])->String{format!("[{}]",values.iter().map(|value|optional_json(value.as_ref())).collect::<Vec<_>>().join(","))}
fn snapshots_json(values:&[Vec<Option<Range>>])->String{format!("[{}]",values.iter().map(|value|slots_json(value)).collect::<Vec<_>>().join(","))}
fn observe(id:&str,pattern:&str,text:&str,replacement:&str){
    let compiled=Compiled::compile(pattern).unwrap();let found=compiled.find(text);let all=compiled.find_all(text);let captures=compiled.captures(text);let captures_all=compiled.captures_all(text);let split=compiled.split(text);
    assert_eq!(compiled.string_match(text),captures);assert_eq!(compiled.string_match_all(text),captures_all);
    let direct_split=compiled.inner.split(text).collect::<Vec<_>>();assert_eq!(split.iter().map(|value|&text[usize::try_from(value.start).unwrap()..usize::try_from(value.end).unwrap()]).collect::<Vec<_>>(),direct_split);
    println!("{{\"kind\":\"correctness\",\"case\":{},\"isMatch\":{},\"find\":{},\"findAll\":[{}],\"captures\":{},\"capturesAll\":{},\"replace\":{},\"replaceAll\":{},\"split\":[{}],\"search\":{}}}",quote(id),compiled.is_match(text),optional_json(found.as_ref()),all.iter().map(range_json).collect::<Vec<_>>().join(","),captures.as_ref().map_or_else(||"null".into(),|value|slots_json(value)),snapshots_json(&captures_all),quote(&compiled.replace(text,replacement)),quote(&compiled.replace_all(text,replacement)),split.iter().map(range_json).collect::<Vec<_>>().join(","),compiled.string_search(text));
}
fn correctness(){
    for(id,pattern,text,replacement)in[
        ("ascii",r"([a-z]+)=([0-9]+)","a=12 b=3","${2}:$1 $$"),
        ("unicode",r"(é)(🦀)?","xé🦀 é","${2}/${1}"),
        ("absent-empty",r"(a)?()b","b ab","${1}:${2}"),
        ("no-match",r"z+","héllo","${0}"),
        ("empty-input",r"","","x"),
        ("zero-width",r"","é🦀","x"),
        ("adjacent-empty",r"a*","ab","x"),
        ("named-template",r"(?P<word>[a-z]+)","cat dog","${word}:$word:$0:$$:${missing}"),
        ("split-boundaries",r",",",a,,b,","|"),
        ("controls",r"(x)","x\n\t\"\\","$1"),
    ]{observe(id,pattern,text,replacement);}
    for pattern in ["(",r"(?=x)",r"(a)\1"]{assert!(Compiled::compile(pattern).is_err());}
    assert!(RegexBuilder::new(r"\w").size_limit(1).build().is_err());
    let literal="a+b[é].*";let escaped=regex::escape(literal);assert!(Regex::new(&format!("^{escaped}$")).unwrap().is_match(literal));
    let snapshots={let text=String::from("é🦀");let compiled=Compiled::compile(".").unwrap();compiled.find_all(&text)};assert_eq!(snapshots,vec![Range{start:0,end:2},Range{start:2,end:6}]);
    println!("{{\"kind\":\"contracts\",\"unsupportedPatterns\":3,\"explicitLimitRejected\":true,\"escapeRoundTrip\":true,\"ownedRangesSurviveOwners\":true}}");
}
