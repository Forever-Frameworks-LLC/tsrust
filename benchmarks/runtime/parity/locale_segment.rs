type String=std::borrow::Cow<'static,str>;
fn segment_ranges(value:icu_segmenter::GraphemeClusterSegmenterBorrowed<'static>,input:&str)->Vec<(i64,i64)>{let mut start=0;value.segment_str(input).skip(1).map(|end|{let pair=(start as i64,end as i64);start=end;pair}).collect()}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {let input=value2.as_ref();let value=icu_segmenter::GraphemeClusterSegmenter::new();let mut checksum=value1;for _ in 0..value0{let ranges=segment_ranges(value,input);for(start,end)in ranges{checksum+=start+end+1;}}checksum}
fn main(){}
