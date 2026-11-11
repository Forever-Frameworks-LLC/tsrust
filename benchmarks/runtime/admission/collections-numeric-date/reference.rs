// Admission controls only. Reuse the emitted ordering implementation unchanged.
include!("../../../../crates/tsrust-compiler/src/stdlib/collections/ordered_runtime.rs");
include!("../../../../crates/tsrust-compiler/src/stdlib/collections/ordered_key_runtime.rs");
mod calendar {
    include!("../../../../crates/tsrust-compiler/src/stdlib/date/runtime.rs");
}
use chrono::{Datelike, Timelike};
use std::{collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque}, hint::black_box};

fn digest(values: impl IntoIterator<Item=i64>) -> u64 {
    values.into_iter().fold(7_u64, |sum, value| sum.wrapping_mul(31).wrapping_add(value as u64))
}
fn heap_snapshot(heap: &BinaryHeap<TsrOrdered<i64>>, sorted_vec: bool) -> Vec<i64> {
    let sorted = if sorted_vec { heap.clone().into_sorted_vec() } else {
        let mut items:Vec<_> = heap.iter().cloned().collect(); items.sort_unstable(); items
    };
    sorted.into_iter().map(|item| item.0).collect()
}
fn heap_values(size:usize,pattern:&str)->Vec<i64>{(0..size).map(|i|match pattern{"seeded"=>((i*73+19)%size) as i64,"reverse"=>i as i64,"duplicates"=>((i*73+19)%7) as i64,_=>(size-1-i) as i64}).collect()}
fn verify() {
    let empty:BTreeSet<String> = BTreeSet::new(); assert_eq!(empty.first(), None); assert_eq!(empty.last(), None);
    let empty:BTreeMap<String,i64> = BTreeMap::new(); assert_eq!(empty.first_key_value(), None); assert_eq!(empty.last_key_value(), None);
    let mut set = BTreeSet::from([String::from("b"),String::from("a")]);
    let snapshot:Vec<_> = set.iter().cloned().collect(); set.clear(); assert_eq!(snapshot,["a","b"]);
    let entries:Vec<_> = snapshot.iter().map(|key|(key.clone(),key.clone())).collect(); assert_eq!(entries[0],("a".into(),"a".into()));
    let mut deque = VecDeque::from([vec![1],vec![2],vec![3]]); deque.pop_front(); deque.push_back(vec![4]);
    let mut snapshot:Vec<_> = deque.iter().cloned().collect(); snapshot[0].push(9); assert_eq!(deque[0],[2]); assert_eq!(snapshot,[vec![2,9],vec![3],vec![4]]);
    for size in [0,1,8,1024] {
        let heap:BinaryHeap<_>=(0..size).rev().map(|i|TsrOrdered::new(i as i64)).collect();
        let expected:Vec<_>=(0..size as i64).collect();
        assert_eq!(heap_snapshot(&heap,true),expected); assert_eq!(heap_snapshot(&heap,false),expected); assert_eq!(heap.len(),size);
    }
    for pattern in ["seeded","reverse","duplicates"] {for size in [0,1,8,1024] {
        let input=heap_values(size,pattern);let heap:BinaryHeap<_>=input.iter().copied().map(TsrOrdered::new).collect();let mut expected=input;expected.sort_unstable();
        assert_eq!(heap_snapshot(&heap,true),expected);assert_eq!(heap_snapshot(&heap,false),expected);assert_eq!(heap.len(),size);
    }}
    let mut heap=BinaryHeap::new(); for value in [f64::NAN,-0.0,0.0,f64::NEG_INFINITY,f64::INFINITY,2.0] {heap.push(TsrOrdered::new(value));}
    let sorted=heap.clone().into_sorted_vec();
    assert_eq!(sorted[0].0,f64::NEG_INFINITY); assert_eq!(sorted[1].0.to_bits(),0); assert_eq!(sorted[2].0.to_bits(),0);
    let mut alternative=heap.iter().cloned().collect::<Vec<_>>();alternative.sort_unstable();assert_eq!(alternative.iter().map(|v|v.0.to_bits()).collect::<Vec<_>>(),sorted.iter().map(|v|v.0.to_bits()).collect::<Vec<_>>());
    assert_eq!(sorted[3].0,2.0); assert_eq!(sorted[4].0,f64::INFINITY); assert!(sorted[5].0.is_nan()); assert_eq!(heap.len(),6);
    for (text,value) in [("0",0.0),("-0",-0.0),("+1.5",1.5),(".5",0.5),("1.",1.0),("1e3",1000.0),("1e999",f64::INFINITY),("-1e-999",-0.0)] {assert_eq!(text.parse::<f64>().unwrap().to_bits(),value.to_bits());}
    for text in ["NaN","nan","inf","-inf","Infinity","+infinity"] {assert!(!text.parse::<f64>().unwrap().is_finite());}
    for text in [""," "," 1","1 ","1x","0x10","1_000","1e"] {assert!(text.parse::<f64>().is_err());}
    for (text,radix,value) in [("+10",10,10),("-10",2,-2),("ff",16,255),("Z",36,35),("9223372036854775807",10,i64::MAX),("-9223372036854775808",10,i64::MIN)] {assert_eq!(i64::from_str_radix(text,radix),Ok(value));}
    use std::num::IntErrorKind;
    for text in ["","+"," 1","1 ","1_0","0x10"] {assert!(matches!(i64::from_str_radix(text,10).unwrap_err().kind(),IntErrorKind::Empty|IntErrorKind::InvalidDigit));}
    assert_eq!(i64::from_str_radix("9223372036854775808",10).unwrap_err().kind(),&IntErrorKind::PosOverflow);
    assert_eq!(i64::from_str_radix("-9223372036854775809",10).unwrap_err().kind(),&IntErrorKind::NegOverflow);
    assert_eq!(i64::MAX.checked_add(1),None); assert_eq!(i64::MIN.checked_sub(1),None); assert_eq!(i64::MAX.checked_mul(2),None);
    assert_eq!(i64::MIN.checked_div(-1),None); assert_eq!(i64::MIN.checked_rem(-1),None); assert_eq!(1_i64.checked_div(0),None); assert_eq!(1_i64.checked_rem(0),None);
    assert_eq!((-7_i64).checked_div(3),Some(-2)); assert_eq!((-7_i64).checked_rem(3),Some(-1));
    for (text,weekday,leap) in [("0001-01-01",1,false),("1900-03-01",4,false),("2000-02-29",2,true),("2024-11-03",0,true),("9999-12-31",5,false)] {
        let date=chrono::NaiveDate::parse_from_str(text,"%Y-%m-%d").unwrap(); assert_eq!(date.weekday().num_days_from_sunday(),weekday);
        let start=date.with_month(1).unwrap().with_day(1).unwrap(); let end=date.with_day(1).unwrap().with_month(12).unwrap().with_day(31).unwrap();
        assert_eq!(start.ordinal(),1); assert_eq!(end.ordinal(),if leap{366}else{365});
        let wrapped=calendar::parse_date(text).unwrap(); assert_eq!(calendar::get_year(&wrapped),i64::from(date.year()));
    }
    for (text,expected) in [("00:00:00",(0,0,0,0)),("23:59:59.999999999",(23,59,59,999999999)),("12:34:56.000000001",(12,34,56,1))] {
        let time=chrono::NaiveTime::parse_from_str(text,"%H:%M:%S%.f").unwrap(); assert_eq!((time.hour(),time.minute(),time.second(),time.nanosecond()),expected);
    }
    println!("{{\"kind\":\"correctness\",\"passed\":true}}");
}

fn kernel(name:&str,size:usize)->Box<dyn FnMut()->u64> {
    let set:BTreeSet<_>=(0..size as i64).collect(); let map:BTreeMap<_,_>=(0..size as i64).map(|v|(v,v)).collect();
    let mut deque:VecDeque<_>=(0..size as i64).collect(); if size>0 {let first=deque.pop_front().unwrap();deque.push_back(first);}
    let pattern=name.rsplit('-').next().unwrap();let heap:BinaryHeap<_>=heap_values(size,pattern).into_iter().map(TsrOrdered::new).collect();
    match name {
        "endpoints"=>Box::new(move||{let set=black_box(&set);let map=black_box(&map);digest([set.first().copied().unwrap_or(-1),set.last().copied().unwrap_or(-1),map.first_key_value().map(|(key,_)|*key).unwrap_or(-1),map.last_key_value().map(|(key,_)|*key).unwrap_or(-1)])}),
        "set-snapshot"=>Box::new(move||{let values:Vec<_>=black_box(&set).iter().copied().collect();digest(black_box(values))}),
        "deque-snapshot"=>Box::new(move||{let values:Vec<_>=black_box(&deque).iter().copied().collect();digest(black_box(values))}),
        "heap-sorted-vec"|"heap-sorted-vec-seeded"|"heap-sorted-vec-reverse"|"heap-sorted-vec-duplicates"=>Box::new(move||digest(black_box(heap_snapshot(black_box(&heap),true)))),
        "heap-sort-snapshot"|"heap-sort-snapshot-seeded"|"heap-sort-snapshot-reverse"|"heap-sort-snapshot-duplicates"=>Box::new(move||digest(black_box(heap_snapshot(black_box(&heap),false)))),
        "float-parse"=>Box::new(||digest(["1.5","-0","1e999","-1e-999"].map(|text|black_box(text).parse::<f64>().unwrap().to_bits() as i64))),
        "radix-parse"=>Box::new(||digest([( "ff",16),("-100",2),("z",36)].map(|(text,radix)|i64::from_str_radix(black_box(text),black_box(radix)).unwrap()))),
        "checked"=>Box::new(||{let a=black_box(123_i64);let b=black_box(7_i64);digest([a.checked_add(b),a.checked_sub(b),a.checked_mul(b),a.checked_div(b),a.checked_rem(b)].map(Option::unwrap))}),
        "date-time"=>{let date=chrono::NaiveDate::from_ymd_opt(2024,2,29).unwrap();let time=chrono::NaiveTime::from_hms_nano_opt(23,59,59,999999999).unwrap();Box::new(move||{let date=black_box(date);let time=black_box(time);digest([i64::from(date.weekday().num_days_from_sunday()),i64::from(date.with_day(1).unwrap().with_month(1).unwrap().ordinal()),i64::from(date.with_day(1).unwrap().with_month(12).unwrap().with_day(31).unwrap().ordinal()),i64::from(time.hour()),i64::from(time.minute()),i64::from(time.second()),i64::from(time.nanosecond())])})},
        _=>panic!("unknown kernel"),
    }
}
