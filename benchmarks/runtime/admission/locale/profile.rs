#[cfg(feature="thread-safe-locale")]
include!("../../../../crates/tsrust-compiler/src/stdlib/locale/runtime.rs");
#[cfg(feature="thread-safe-locale")]
fn public_owner_send_sync_witness(){
    fn require<T:Send+Sync>(){}
    require::<tsrust_native_locale::Locale>();require::<tsrust_native_locale::Collator>();
    require::<tsrust_native_locale::Segmenter>();require::<tsrust_native_locale::NumberFormat>();
}
include!("reference.rs");
use std::{hint::black_box,time::Instant};
#[cfg(feature="allocation-probe")]
mod allocation {
    include!("../../probes/counting-allocator.rs");
    pub fn begin()->usize{let baseline=LIVE.load(Relaxed);ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);REQUESTED.store(0,Relaxed);PEAK.store(baseline,Relaxed);baseline}
    pub fn end(baseline:usize)->(usize,usize,usize,i64,i64){(ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),REQUESTED.load(Relaxed),LIVE.load(Relaxed)as i64-baseline as i64,PEAK.load(Relaxed)as i64-baseline as i64)}
}
struct Context { collator:CollatorBorrowed<'static>, formatter:DecimalFormatter, decimal:Decimal, normalized:Decimal, bigdecimal:bigdecimal::BigDecimal, text:String, right:String, tag:&'static str, input:&'static str, canonicalizer:LocaleCanonicalizer }
impl Context {fn new(scenario:&str,size:usize)->Self {
    let(tag,input,text)=match scenario {
        "unicode"=>("de","-12345678901234567890.1200","e\u{301}🦀👨‍👩‍👧‍👦🇺🇳".repeat(size)),
        "indic"=>("hi-IN","12345678.9000","नमस्ते".repeat(size)),
        _=>("en","12345.6700","ascii text ".repeat(size))};
    let right=format!("{text}z");
    let bigdecimal=input.parse::<bigdecimal::BigDecimal>().unwrap().normalized();
    let normalized=Decimal::try_from_str(&bigdecimal.to_plain_string()).unwrap();
    Self{right,normalized,bigdecimal,collator:collator(tag),formatter:formatter(tag,None),decimal:Decimal::try_from_str(input).unwrap(),text,tag,input,canonicalizer:LocaleCanonicalizer::new_extended()}
}}
fn kernel(name:&str,c:&Context)->u64 {
    match name {
        "locale_parse"=>black_box(locale(black_box(c.tag))).to_string().len()as u64,
        "canonicalize_reuse"=>{let mut l=locale(black_box("iw-IL"));c.canonicalizer.canonicalize(&mut l);black_box(l).to_string().len()as u64},
        "canonicalize_construct"=>black_box(canonical(black_box("iw-IL"))).len()as u64,
        "collator_construct"=>{let v=collator(black_box(c.tag));(compare(&v,"é2","e10")+1)as u64},
        "collator_reuse"|"string_locale_compare_reuse"=>(compare(black_box(&c.collator),black_box("é2"),black_box("e10"))+1)as u64,
        "collator_long_reuse"=>(compare(black_box(&c.collator),black_box(&c.text),black_box(&c.right))+1)as u64,
        "collator_long_construct"=>{let value=collator(black_box(c.tag));(compare(&value,black_box(&c.text),black_box(&c.right))+1)as u64},
        "number_construct"=>black_box(formatter(black_box(c.tag),None)).format_to_string(black_box(&c.decimal)).len()as u64,
        "number_fixed_input"=>black_box(c.formatter.format_to_string(black_box(&c.decimal))).len()as u64,
        "number_fixed_text_roundtrip"=>{let text=black_box(c.decimal.to_string());let input=Decimal::try_from_str(&text).unwrap();black_box(c.formatter.format_to_string(&input)).len()as u64},
        "number_normalized_fixed_input"=>black_box(c.formatter.format_to_string(black_box(&c.normalized))).len()as u64,
        "number_bigdecimal_bridge"=>{let text=black_box(black_box(&c.bigdecimal).to_plain_string());let input=Decimal::try_from_str(&text).unwrap();black_box(c.formatter.format_to_string(&input)).len()as u64},
        "number_parse_input"=>{let input=Decimal::try_from_str(black_box(c.input)).unwrap();black_box(c.formatter.format_to_string(&input)).len()as u64},
        "number_round_half_even"=>{let mut input=black_box(c.decimal.clone());input.round(-2);black_box(c.formatter.format_to_string(&input)).len()as u64},
        "grapheme_direct"=>GraphemeClusterSegmenter::new().segment_str(black_box(&c.text)).skip(1).map(|end|end as u64).sum(),
        "grapheme_ranges"=>black_box(ranges(black_box(&c.text))).iter().map(|&(_,end)|end as u64).sum(),
        "grapheme_strings"=>{let mut start=0;let strings=GraphemeClusterSegmenter::new().segment_str(black_box(&c.text)).skip(1).map(|end|{let value=c.text[start..end].to_owned();start=end;value}).collect::<Vec<_>>();let mut end=0;black_box(strings).iter().map(|s|{end+=s.len()as u64;end}).sum()},
        _=>panic!("unknown kernel")
    }
}
fn measure(name:&str,scenario:&str,size:usize){let c=Context::new(scenario,size);
    #[cfg(feature="allocation-probe")]{for _ in 0..8{black_box(kernel(name,&c));}let baseline=allocation::begin();let checksum=black_box(kernel(name,&c));let(allocations,reallocations,requested,retained,peak)=allocation::end(baseline);println!("{{\"kind\":\"allocations\",\"kernel\":{name:?},\"scenario\":{scenario:?},\"size\":{size},\"checksum\":{checksum},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak}}}");}
    #[cfg(not(feature="allocation-probe"))]{let rounds=32;let mut durations=Vec::new();let mut checksums=Vec::new();for sample in 0..39{let started=Instant::now();let mut checksum=0;for _ in 0..rounds{checksum+=black_box(kernel(name,&c));}let elapsed=started.elapsed().as_nanos();if sample>=8{durations.push(elapsed);checksums.push(checksum);}}println!("{{\"kind\":\"timing\",\"kernel\":{name:?},\"scenario\":{scenario:?},\"size\":{size},\"rounds\":{rounds},\"durations_ns\":{durations:?},\"checksums\":{checksums:?}}}");}
}
fn main(){if std::env::args().any(|v|v=="--check"){correctness();return;}
for name in ["locale_parse","canonicalize_reuse","canonicalize_construct","collator_construct","collator_reuse","string_locale_compare_reuse","number_construct","number_fixed_input","number_fixed_text_roundtrip","number_parse_input","number_normalized_fixed_input","number_bigdecimal_bridge","number_round_half_even"]{for scenario in ["ascii","unicode","indic"]{measure(name,scenario,32);}}
for size in [16,64,256,1024]{for name in ["grapheme_direct","grapheme_ranges","grapheme_strings","collator_long_reuse","collator_long_construct"]{for scenario in ["ascii","unicode","indic"]{measure(name,scenario,size);}}}}
