type String=std::borrow::Cow<'static,str>;
fn ordering(value:std::cmp::Ordering)->i64{match value{std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {let input=value2.as_ref();let locale="en".parse::<icu_locale::Locale>().unwrap();let value=icu_collator::Collator::try_new((&locale).into(),Default::default()).unwrap();let mut checksum=value1;for _ in 0..value0{checksum+=ordering(value.compare(input,"e10"))+2;}checksum}
fn main(){}
