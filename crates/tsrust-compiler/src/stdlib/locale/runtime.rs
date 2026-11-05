mod tsrust_native_locale {
    use icu_collator::{Collator as BackendCollator,CollatorBorrowed,options::{CollatorOptions as BackendCollatorOptions,Strength,AlternateHandling,MaxVariable,CaseLevel}};
    use icu_decimal::{DecimalFormatter,options::{DecimalFormatterOptions,GroupingStrategy}};
    use icu_locale::LocaleCanonicalizer;
    use fixed_decimal::Decimal;
    pub struct Locale(icu_locale::Locale);
    pub struct Collator(CollatorBorrowed<'static>);
    pub struct Segmenter;
    pub struct NumberFormat {formatter:DecimalFormatter,fraction_position:Option<i16>}
    #[derive(Clone,Debug,PartialEq,Eq)]pub struct Range{pub start:i64,pub end:i64}
    #[derive(Debug)]pub struct Error{pub code:&'static str,pub message:std::string::String}
    #[derive(Default)]pub struct CollatorOptions{pub strength:Option<std::string::String>,pub alternate_handling:Option<std::string::String>,pub max_variable:Option<std::string::String>,pub case_level:Option<bool>}
    #[derive(Default)]pub struct NumberFormatOptions{pub grouping:Option<std::string::String>,pub fraction_digits:Option<i64>}
    fn error(code:&'static str,message:impl ToString)->Error{Error{code,message:message.to_string()}}
    pub fn parse_locale(text:&str)->Result<Locale,Error>{let mut value=text.parse::<icu_locale::Locale>().map_err(|e|error("Syntax",e))?;LocaleCanonicalizer::new_extended().canonicalize(&mut value);Ok(Locale(value))}
    pub fn format_locale(value:&Locale)->std::string::String{value.0.to_string()}
    pub fn create_collator(locale:&Locale,options:Option<&CollatorOptions>)->Result<Collator,Error>{
        let mut backend=BackendCollatorOptions::default();
        if let Some(options)=options{
            backend.strength=match options.strength.as_deref(){None=>None,Some("primary")=>Some(Strength::Primary),Some("secondary")=>Some(Strength::Secondary),Some("tertiary")=>Some(Strength::Tertiary),Some("quaternary")=>Some(Strength::Quaternary),Some("identical")=>Some(Strength::Identical),_=>return Err(error("Option","Unsupported collation strength"))};
            backend.alternate_handling=match options.alternate_handling.as_deref(){None=>None,Some("nonIgnorable")=>Some(AlternateHandling::NonIgnorable),Some("shifted")=>Some(AlternateHandling::Shifted),_=>return Err(error("Option","Unsupported alternate handling"))};
            backend.max_variable=match options.max_variable.as_deref(){None=>None,Some("space")=>Some(MaxVariable::Space),Some("punctuation")=>Some(MaxVariable::Punctuation),Some("symbol")=>Some(MaxVariable::Symbol),Some("currency")=>Some(MaxVariable::Currency),_=>return Err(error("Option","Unsupported max variable"))};
            backend.case_level=options.case_level.map(|value|if value{CaseLevel::On}else{CaseLevel::Off});
        }
        BackendCollator::try_new((&locale.0).into(),backend).map(Collator).map_err(|e|error("Data",e))
    }
    pub fn compare(value:&Collator,left:&str,right:&str)->i64{match value.0.compare(left,right){std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}}
    pub fn string_compare(left:&str,right:&str,value:&Collator)->i64{compare(value,left,right)}
    pub fn create_segmenter()->Segmenter{Segmenter}
    pub fn segment<'a>(_value:&Segmenter,text:&'a str)->impl Iterator<Item=Range>+'a{let mut start=0;icu_segmenter::GraphemeClusterSegmenter::new().segment_str(text).skip(1).map(move|end|{let range=Range{start:i64::try_from(start).expect("valid string offset"),end:i64::try_from(end).expect("valid string offset")};start=end;range})}
    pub fn create_number_format(locale:&Locale,options:Option<&NumberFormatOptions>)->Result<NumberFormat,Error>{
        let mut backend=DecimalFormatterOptions::default();let mut fraction_position=None;
        if let Some(options)=options{
            backend.grouping_strategy=match options.grouping.as_deref(){None=>None,Some("auto")=>Some(GroupingStrategy::Auto),Some("never")=>Some(GroupingStrategy::Never),Some("always")=>Some(GroupingStrategy::Always),Some("min2")=>Some(GroupingStrategy::Min2),_=>return Err(error("Option","Unsupported grouping strategy"))};
            if let Some(digits)=options.fraction_digits{if digits<0{return Err(error("Range","fractionDigits must be in 0..32768"));}fraction_position=Some(digits.checked_neg().and_then(|position|i16::try_from(position).ok()).ok_or_else(||error("Range","fractionDigits must be in 0..32768"))?);}
        }
        DecimalFormatter::try_new((&locale.0).into(),backend).map(|formatter|NumberFormat{formatter,fraction_position}).map_err(|e|error("Data",e))
    }
    fn formatted(value:&NumberFormat,mut decimal:Decimal)->std::string::String{if let Some(position)=value.fraction_position{decimal.round(position);}value.formatter.format_to_string(&decimal)}
    pub fn format_int(value:&NumberFormat,input:i64)->std::string::String{formatted(value,Decimal::from(input))}
    // The caller supplies existing normalized Decimal.format plain text, never f64.
    pub fn format_decimal(value:&NumberFormat,plain:&str)->std::string::String{formatted(value,Decimal::try_from_str(plain).expect("normalized Tsrust Decimal fits fixed_decimal"))}
}
