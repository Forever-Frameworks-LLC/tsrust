#[test]
fn temporal_values_preserve_precision_and_resolve_dst_explicitly(){
 use tsrust_native_date::*;
 let instant=|s|parse_iso(s).unwrap();let date=|s|parse_date(s).unwrap();let time=|s|parse_time(s).unwrap();
 assert_eq!(format_time(&time("23:59:59.123456789")),"23:59:59.123456789");
 assert_eq!(format_time(&time("00:00:00.100000000")),"00:00:00.1");
 for s in ["24:00:00","23:59:60","1:00:00","12:00","12:00:00.","12:00:00.1234567890"]{assert!(parse_time(s).is_err(),"{s}")}
 assert_eq!(format_iso(&instant("2024-01-01T01:00:00.123456789+01:00")),"2024-01-01T00:00:00.123456789Z");
 for s in ["2024-01-01","2024-01-01T00:00:00","2024-01-01 00:00:00Z","2024-01-01T00:00:00-00:00","2024-01-01T23:59:60Z","0001-01-01T00:00:00+01:00","9999-12-31T23:59:59-01:00"]{assert!(parse_iso(s).is_err(),"{s}")}
 assert_eq!(format_iso(&from_unix_milliseconds(-1).unwrap()),"1969-12-31T23:59:59.999Z");
 assert_eq!(to_unix_milliseconds(&from_unix_milliseconds(-1).unwrap()).unwrap(),-1);
 assert_eq!(to_unix_seconds(&from_unix_seconds(-1).unwrap()).unwrap(),-1);
 assert!(to_unix_milliseconds(&instant("1970-01-01T00:00:00.000000001Z")).is_err());
 assert!(to_unix_seconds(&instant("1970-01-01T00:00:00.001Z")).is_err());
 assert!(from_unix_milliseconds(i64::MAX).is_err());assert!(from_unix_seconds(i64::MIN).is_err());
 assert!(duration_from_seconds(i64::MAX).is_err());assert!(duration_from_milliseconds(i64::MIN).is_err());
 assert!(duration_to_milliseconds(&duration_from_nanoseconds(-1)).is_err());
 assert_eq!(duration_to_milliseconds(&duration_from_milliseconds(-1).unwrap()).unwrap(),-1);
 assert!(add_durations(&duration_from_nanoseconds(i64::MAX),&duration_from_nanoseconds(1)).is_err());
 assert!(subtract_durations(&duration_from_nanoseconds(i64::MIN),&duration_from_nanoseconds(1)).is_err());
 assert!(difference(&instant("9999-01-01T00:00:00Z"),&instant("0001-01-01T00:00:00Z")).is_err());
 let epoch=from_unix_seconds(0).unwrap();let before=add_duration(&epoch,&duration_from_nanoseconds(-1)).unwrap();
 assert_eq!(format_iso(&before),"1969-12-31T23:59:59.999999999Z");assert_eq!(duration_to_nanoseconds(&difference(&epoch,&before).unwrap()),1);
 let ny=parse_time_zone("America/New_York").unwrap();let fold=date("2024-11-03");let half=time("01:30:00");
 assert!(resolve_date_time(&fold,&half,&ny,"reject").is_err());
 let earlier=resolve_date_time(&fold,&half,&ny,"earlier").unwrap();let later=resolve_date_time(&fold,&half,&ny,"later").unwrap();
 assert_eq!(format_iso(&earlier),"2024-11-03T05:30:00Z");assert_eq!(format_iso(&later),"2024-11-03T06:30:00Z");
 assert_eq!(duration_to_nanoseconds(&difference(&later,&earlier).unwrap()),3_600_000_000_000);
 assert_eq!(format_date(&local_date(&earlier,&ny).unwrap()),"2024-11-03");assert_eq!(format_time(&local_time(&earlier,&ny).unwrap()),"01:30:00");assert_eq!(offset_seconds(&earlier,&ny),-14400);
 for policy in ["reject","earlier","later"]{assert!(resolve_date_time(&date("2024-03-10"),&time("02:30:00"),&ny,policy).is_err())}
 assert!(resolve_date_time(&fold,&half,&ny,"guess").is_err());assert!(parse_time_zone("america/new_york").is_err());
 let lord=parse_time_zone("Australia/Lord_Howe").unwrap();let fold=date("2024-04-07");let half=time("01:45:00");
 let a=resolve_date_time(&fold,&half,&lord,"earlier").unwrap();let b=resolve_date_time(&fold,&half,&lord,"later").unwrap();
 assert_eq!(duration_to_nanoseconds(&difference(&b,&a).unwrap()),1_800_000_000_000);
 let apia=parse_time_zone("Pacific/Apia").unwrap();assert!(resolve_date_time(&date("2011-12-30"),&time("12:00:00"),&apia,"later").is_err());
 let offset=fixed_offset(19800).unwrap();assert_eq!(zone_name(&offset),"+05:30");assert_eq!(offset_seconds(&epoch,&offset),19800);assert!(fixed_offset(86400).is_err());
 assert!(local_date(&instant("0001-01-01T00:00:00Z"),&fixed_offset(-3600).unwrap()).is_err());
 assert_eq!(compare_times(&time("01:00:00"),&time("02:00:00")),-1);assert_eq!(compare_instants(&earlier,&later),-1);assert!(now().is_ok());
}

#[test]
fn floor_second_preserves_the_second_before_and_after_epoch(){
 use tsrust_native_date::*;
 for (value,expected,nanos) in [("1969-12-31T23:59:59.999Z","1969-12-31T23:59:59Z",999_000_000),("2024-01-01T00:00:00.123Z","2024-01-01T00:00:00Z",123_000_000)]{
  let value=parse_iso(value).unwrap();let floor=start_of_second(&value);
  assert_eq!(format_iso(&floor),expected);assert_eq!(duration_to_nanoseconds(&difference(&value,&floor).unwrap()),nanos);
  assert_eq!(format_iso(&start_of_second(&floor)),expected);
 }
}
