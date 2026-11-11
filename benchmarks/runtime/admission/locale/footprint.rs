// One source and identical rustc/link flags for every control. Deltas include
// reachable ICU service code + compiled data, not data bytes alone.
use std::hint::black_box;
fn main(){
 let input=std::env::args().nth(1).unwrap_or_else(||"12345.6700".to_owned());
 let owner=black_box(input).parse::<bigdecimal::BigDecimal>().unwrap().normalized();
 let mut decimal=fixed_decimal::Decimal::try_from_str(&owner.to_plain_string()).unwrap();
 decimal.round(-2);let mut checksum=black_box(decimal.to_string()).len();
 #[cfg(any(footprint_locale,footprint_all))]{let mut locale="iw-IL".parse::<icu_locale::Locale>().unwrap();icu_locale::LocaleCanonicalizer::new_extended().canonicalize(&mut locale);checksum+=black_box(locale.to_string()).len();}
 #[cfg(any(footprint_collator,footprint_all))]{let locale="de".parse::<icu_locale::Locale>().unwrap();let collator=icu_collator::Collator::try_new(locale.into(),Default::default()).unwrap();checksum+=usize::from(black_box(collator.compare(black_box("é2"),black_box("e10")))==std::cmp::Ordering::Less);}
 #[cfg(any(footprint_decimal,footprint_all))]{let locale="hi-IN".parse::<icu_locale::Locale>().unwrap();let formatter=icu_decimal::DecimalFormatter::try_new(locale.into(),Default::default()).unwrap();checksum+=black_box(formatter.format_to_string(black_box(&decimal))).len();}
 #[cfg(any(footprint_grapheme,footprint_all))]{checksum+=icu_segmenter::GraphemeClusterSegmenter::new().segment_str(black_box("e\u{301}🦀👨‍👩‍👧‍👦🇺🇳")).sum::<usize>();}
 println!("{}",black_box(checksum));
}
