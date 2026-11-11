type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {let input=value2.as_ref();let mut checksum=value1;for _ in 0..value0{let mut value=input.parse::<icu_locale::Locale>().unwrap();icu_locale::LocaleCanonicalizer::new_extended().canonicalize(&mut value);checksum+=value.to_string().len()as i64;}checksum}
fn main(){}
