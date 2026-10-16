#[test]
fn bounded_decimal_operations_never_round_to_claim_success(){
 use tsrust_native_decimal::*;
 let value=|text|parse_decimal(text).unwrap();
 assert_eq!(format(&add(&value("0.1"),&value("0.2")).unwrap()),"0.3");
 assert_eq!(format_fixed(&multiply(&value("19.99"),&from_int(3)).unwrap(),2).unwrap(),"59.97");
 assert_eq!(format(&divide_exact(&from_int(1),&from_int(8)).unwrap()),"0.125");
 assert!(divide_exact(&from_int(1),&from_int(3)).is_err());
 assert!(divide_exact(&from_int(1),&from_int(0)).is_err());
 assert!(multiply(&value("0.0000000000000000000000000001"),&value("0.1")).is_err());
 assert!(add(&value("99999999999999999999999999999999999999"),&from_int(1)).is_err());
 assert_eq!(format(&value("-0.000")),"0");assert_eq!(compare(&value("1.0"),&value("1")),0);
 assert_eq!(to_int(&value("9223372036854775807")).unwrap(),i64::MAX);assert!(to_int(&value("9223372036854775808")).is_err());assert!(to_int(&value("1.1")).is_err());
 assert_eq!(to_number(&value("1.25")).unwrap(),1.25);assert!(to_number(&value("0.1")).is_err());assert!(to_number(&value("9007199254740993")).is_err());
 assert!(format_fixed(&value("1.001"),2).is_err());assert!(format_fixed(&value("1"),29).is_err());
 for text in [""," 1","1e2","1.",".1","NaN","1_000","--1","1.2.3"]{assert!(parse_decimal(text).is_err(),"{text}");}
 assert!(parse_decimal(&"0".repeat(129)).is_err());
 assert_eq!(format(&subtract(&value("1.2"),&value("0.3")).unwrap()),"0.9");
}
