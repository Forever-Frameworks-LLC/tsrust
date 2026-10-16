#[test]
fn calendar_dates_preserve_bounds_and_month_end_rules(){
 use tsrust_native_date::*;
 let date=|s|parse_date(s).unwrap();
 for s in ["0001-01-01","9999-12-31","2000-02-29","2024-02-29"]{assert_eq!(format_date(&date(s)),s)}
 for s in ["0000-01-01","10000-01-01","2023-02-29","1900-02-29","2024-13-01","2024-04-31","2024-1-01"," 2024-01-01","2024-01-01T00:00:00Z"]{assert!(parse_date(s).is_err(),"{s}")}
 assert_eq!(format_date(&add_days(&date("2024-02-28"),1).unwrap()),"2024-02-29");
 assert_eq!(format_date(&add_days(&date("2024-03-01"),-1).unwrap()),"2024-02-29");
 assert_eq!(format_date(&add_months(&date("2024-01-31"),1).unwrap()),"2024-02-29");
 assert_eq!(format_date(&add_months(&date("2024-03-31"),-1).unwrap()),"2024-02-29");
 assert_eq!(format_date(&add_months(&date("2024-02-29"),12).unwrap()),"2025-02-28");
 for amount in [i64::MIN,i64::MAX]{assert!(add_days(&date("2024-01-01"),amount).is_err());assert!(add_months(&date("2024-01-01"),amount).is_err())}
 assert!(add_days(&date("9999-12-31"),1).is_err());assert!(add_days(&date("0001-01-01"),-1).is_err());
 assert_eq!(difference_in_calendar_days(&date("2024-03-01"),&date("2024-02-28")),2);
 assert_eq!(compare_asc(&date("2024-01-01"),&date("2024-01-02")),-1);
 assert!(is_leap_year(&date("2000-01-01")));assert!(!is_leap_year(&date("1900-01-01")));
 let d=date("2024-02-29");assert_eq!((get_year(&d),get_month(&d),get_date(&d)),(2024,1,29));
 assert_eq!(format_date(&start_of_month(&d)),"2024-02-01");assert_eq!(format_date(&end_of_month(&d)),"2024-02-29");
 assert_eq!(format(&d,"EEEE, MMMM d, yyyy").unwrap(),"Thursday, February 29, 2024");
 assert_eq!(format(&d,"'Date:' yyyy-MM-dd ''").unwrap(),"Date: 2024-02-29 '");
 for pattern in ["YYYY-MM-DD","yyyy 'unclosed","HH:mm"]{assert!(format(&d,pattern).is_err())}
 assert!(format(&d,&"x".repeat(257)).is_err());
}
