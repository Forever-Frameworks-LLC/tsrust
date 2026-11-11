// Admission prototype only: no generated Tsrust API measurements.
use icu_collator::{Collator, CollatorBorrowed, options::CollatorOptions};
use icu_decimal::{DecimalFormatter, options::{DecimalFormatterOptions, GroupingStrategy}};
use icu_locale::{Locale, LocaleCanonicalizer};
use icu_segmenter::GraphemeClusterSegmenter;
use fixed_decimal::Decimal;

fn locale(text: &str) -> Locale { text.parse().expect("fixture locale") }
fn canonical(text: &str) -> String {
    let mut value = locale(text);
    LocaleCanonicalizer::new_extended().canonicalize(&mut value);
    value.to_string()
}
fn collator(text: &str) -> CollatorBorrowed<'static> {
    Collator::try_new(locale(text).into(), CollatorOptions::default()).expect("compiled collation data")
}
fn formatter(text: &str, grouping: Option<GroupingStrategy>) -> DecimalFormatter {
    let mut options = DecimalFormatterOptions::default();
    options.grouping_strategy = grouping;
    DecimalFormatter::try_new(locale(text).into(), options).expect("compiled decimal data")
}
fn ranges(text: &str) -> Vec<(usize,usize)> {
    let mut previous = 0;
    GraphemeClusterSegmenter::new().segment_str(text).skip(1).map(|end| {
        let range = (previous,end); previous = end; range
    }).collect()
}
fn compare(value: &CollatorBorrowed<'static>, left: &str, right: &str) -> i8 {
    match value.compare(left,right) { std::cmp::Ordering::Less => -1, std::cmp::Ordering::Equal => 0, std::cmp::Ordering::Greater => 1 }
}
// The public nonnegative digit count maps to the backend's negative i16 magnitude.
fn fraction_position(digits:i64)->Result<i16,&'static str>{
    if digits<0{return Err("range");}
    digits.checked_neg().and_then(|position|i16::try_from(position).ok()).ok_or("range")
}
fn explicit_fraction(value:&mut Decimal,digits:i64)->Result<(),&'static str>{value.round(fraction_position(digits)?);Ok(())}
fn correctness() {
    for (input,expected) in [("EN-us","en-US"),("iw-IL","he-IL"),("in-ID","id-ID")] {
        let actual=canonical(input); assert_eq!(actual,expected);
        println!("{{\"kind\":\"canonical\",\"input\":{input:?},\"value\":{actual:?}}}");
    }
    for input in ["en_US","!","en--US"] { assert!(input.parse::<Locale>().is_err()); }
    for (tag,left,right,expected) in [("en","a","b",-1),("de","ä","z",-1),("sv","ä","z",1),("en-u-kn-true","2","10",-1),("en","é","e\u{301}",0)] {
        let actual=compare(&collator(tag),left,right); assert_eq!(actual,expected);
        println!("{{\"kind\":\"compare\",\"locale\":{tag:?},\"left\":{},\"right\":{},\"value\":{actual}}}",json_text(left),json_text(right));
    }
    for text in ["", "ascii", "e\u{301}🦀", "👨‍👩‍👧‍👦🇺🇳", "a\r\nb"] {
        let value=ranges(text); assert!(value.iter().all(|&(a,b)|a<b&&text.is_char_boundary(a)&&text.is_char_boundary(b)));
        assert_eq!(value.iter().map(|&(a,b)|b-a).sum::<usize>(),text.len());
        let ends=value.iter().map(|&(_,b)|b).collect::<Vec<_>>();
        println!("{{\"kind\":\"grapheme\",\"text\":{},\"ends\":{ends:?}}}",json_text(text));
    }
    for (tag,input) in [("en","-12345678901234567890.1200"),("de","12345.6700"),("hi-IN","12345678.90"),("ar-EG","-1234.50"),("en-u-nu-arab","1234.50"),("zz-ZZ","1234.50")] {
        let decimal=Decimal::try_from_str(input).unwrap(); let value=formatter(tag,None).format_to_string(&decimal);
        println!("{{\"kind\":\"format\",\"locale\":{tag:?},\"input\":{input:?},\"value\":{}}}",json_text(&value));
    }
    for (name,strategy,expected) in [("auto",GroupingStrategy::Auto,"1,234.50"),("always",GroupingStrategy::Always,"1,234.50"),("min2",GroupingStrategy::Min2,"1234.50"),("never",GroupingStrategy::Never,"1234.50")] {
        let value=formatter("en",Some(strategy)).format_to_string(&Decimal::try_from_str("1234.50").unwrap());assert_eq!(value,expected);
        println!("{{\"kind\":\"grouping\",\"strategy\":{name:?},\"value\":{value:?}}}");
    }
    // Existing Tsrust Decimal normalizes on construction; original scale is not retained.
    for (input,expected) in [("12345.6700","12345.67"),("-12345678901234567890.1200","-12345678901234567890.12"),("-0.000","0"),("99999999999999999999999999999999999999","99999999999999999999999999999999999999"),("0.0000000000000000000000000001","0.0000000000000000000000000001")] {
        let owner=input.parse::<bigdecimal::BigDecimal>().unwrap().normalized();
        let plain=owner.to_plain_string();assert_eq!(plain,expected);
        let fixed=Decimal::try_from_str(&plain).unwrap();let value=formatter("en",None).format_to_string(&fixed);
        println!("{{\"kind\":\"normalized_format\",\"input\":{input:?},\"plain\":{plain:?},\"value\":{}}}",json_text(&value));
    }
    let plain=formatter("en",Some(GroupingStrategy::Never));
    for (input,expected) in [("2.345","2.34"),("2.355","2.36"),("-2.345","-2.34"),("-2.355","-2.36")] {
        let mut value=Decimal::try_from_str(input).unwrap(); value.round(-2);
        let actual=plain.format_to_string(&value); assert_eq!(actual,expected);
        println!("{{\"kind\":\"round\",\"input\":{input:?},\"value\":{actual:?}}}");
    }
    for (input,digits,expected) in [("1",2,"1.00"),("1.2",4,"1.2000"),("-1.2",0,"-1"),("2.5",0,"2"),("3.5",0,"4")] {
        let mut value=Decimal::try_from_str(input).unwrap();explicit_fraction(&mut value,digits).unwrap();let actual=plain.format_to_string(&value);assert_eq!(actual,expected);
        println!("{{\"kind\":\"fraction_padding\",\"input\":{input:?},\"digits\":{digits},\"value\":{actual:?}}}");
    }
    for digits in [-1_i64,i64::MIN,32769,i64::MAX] {
        let mut value=Decimal::from(1);assert_eq!(explicit_fraction(&mut value,digits),Err("range"));assert_eq!(value.to_string(),"1");
        println!("{{\"kind\":\"fraction_range\",\"digits\":\"{digits}\",\"code\":\"range\",\"unchanged\":true}}");
    }
    for digits in [32767_i64,32768] {
        let mut value=Decimal::from(1);explicit_fraction(&mut value,digits).unwrap();let actual=plain.format_to_string(&value);
        assert_eq!(actual.len(),digits as usize+2);assert!(actual.starts_with("1."));assert!(actual.as_bytes()[2..].iter().all(|&byte|byte==b'0'));
        println!("{{\"kind\":\"fraction_edge\",\"digits\":{digits},\"bytes\":{},\"prefix\":\"1.\",\"zeros\":{digits}}}",actual.len());
    }
    assert_eq!(plain.format_to_string(&Decimal::from(i64::MIN)),i64::MIN.to_string());
}
fn json_text(text:&str)->String {
    let mut out=String::from("\"");for ch in text.chars(){match ch {'\r'=>out.push_str("\\r"),'\n'=>out.push_str("\\n"),'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),_=>out.push(ch)}}out.push('"');out
}
