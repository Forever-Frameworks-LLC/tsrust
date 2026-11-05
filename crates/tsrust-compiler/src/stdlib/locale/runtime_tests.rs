use tsrust_native_locale::*;
#[test]fn explicit_locale_defaults_ranges_and_exact_bridge(){
 let locale=parse_locale("EN-us").unwrap();assert_eq!(format_locale(&locale),"en-US");assert!(parse_locale("en_US").is_err());
 let numeric=parse_locale("en-u-kn-true").unwrap();let numeric=create_collator(&numeric,None).unwrap();assert_eq!(compare(&numeric,"2","10"),-1);
 let en=create_collator(&locale,None).unwrap();assert_eq!(string_compare("e\u{301}","é",&en),0);
 let segmenter=create_segmenter();assert!(segment(&segmenter,"").is_empty());assert_eq!(segment(&segmenter,"e\u{301}🦀"),vec![Range{start:0,end:3},Range{start:3,end:7}]);
 let number=create_number_format(&locale,None).unwrap();assert_eq!(format_int(&number,i64::MIN),"-9,223,372,036,854,775,808");
 assert_eq!(format_decimal(&number,"12345678901234567890.12"),"12,345,678,901,234,567,890.12");
 for digits in [-1,i64::MIN,32769,i64::MAX]{assert!(matches!(create_number_format(&locale,Some(&NumberFormatOptions{fraction_digits:Some(digits),..Default::default()})),Err(Error{code:"Range",..})));}
 let fixed=create_number_format(&locale,Some(&NumberFormatOptions{fraction_digits:Some(2),grouping:Some("never".into())})).unwrap();assert_eq!(format_int(&fixed,1),"1.00");assert_eq!(format_decimal(&fixed,"2.345"),"2.34");assert_eq!(format_decimal(&fixed,"2.355"),"2.36");
 for digits in [32767,32768]{let fixed=create_number_format(&locale,Some(&NumberFormatOptions{fraction_digits:Some(digits),grouping:Some("never".into())})).unwrap();let value=format_int(&fixed,1);assert_eq!(value.len(),digits as usize+2);assert!(value[2..].bytes().all(|v|v==b'0'));}
}
#[test]fn compiled_owner_transfer_matches_backend(){fn transferable<T:Send+Sync>(){}transferable::<Locale>();transferable::<Collator>();transferable::<Segmenter>();transferable::<NumberFormat>();}
#[test]fn numbering_symbols_fallback_is_preserved(){let locale=parse_locale("en-u-nu-arab").unwrap();let number=create_number_format(&locale,None).unwrap();assert_eq!(format_decimal(&number,"1234.5"),"١,٢٣٤.٥");}
