type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {let input=value2.as_ref();let locale="en".parse::<icu_locale::Locale>().unwrap();let value=icu_decimal::DecimalFormatter::try_new((&locale).into(),Default::default()).unwrap();let mut checksum=value1;for _ in 0..value0{checksum+=value.format_to_string(&fixed_decimal::Decimal::from(1234)).len()as i64;}checksum}
fn main(){}
