use chrono::{DateTime,Utc,NaiveTime,FixedOffset,Offset,LocalResult,SecondsFormat,Timelike,TimeZone as ChronoTimeZone};
#[derive(Clone)]
pub struct Time(NaiveTime);
#[derive(Clone)]
pub struct Instant(DateTime<Utc>);
#[derive(Clone)]
pub struct Duration(i64);
#[derive(Clone)]
pub enum TimeZone { Named(chrono_tz::Tz), Fixed(FixedOffset) }
const TIME_RANGE:Error=("Range","Time, instant, offset or duration exceeds its supported range");
const TIME_SYNTAX:Error=("Syntax","Expected padded time or explicit-offset ISO timestamp; leap seconds are unsupported");
fn checked_instant(value:DateTime<Utc>)->Result<Instant,Error>{if (1..=9999).contains(&value.year()){Ok(Instant(value))}else{Err(TIME_RANGE)}}
pub fn parse_time(text:&str)->Result<Time,Error>{
    let bytes=text.as_bytes();
    if !text.is_ascii() || !(8..=18).contains(&bytes.len()) || bytes[2]!=b':' || bytes[5]!=b':' || ![0,1,3,4,6,7].iter().all(|&i|bytes[i].is_ascii_digit()){return Err(TIME_SYNTAX)}
    let nanos=if bytes.len()==8{0}else{
        if bytes.len()<10 || bytes[8]!=b'.' || !bytes[9..].iter().all(|b|b.is_ascii_digit()){return Err(TIME_SYNTAX)}
        text[9..].parse::<u32>().map_err(|_|TIME_SYNTAX)?*10u32.pow((18-bytes.len()) as u32)
    };
    let pair=|i|u32::from(bytes[i]-b'0')*10+u32::from(bytes[i+1]-b'0');
    NaiveTime::from_hms_nano_opt(pair(0),pair(3),pair(6),nanos).map(Time).ok_or(TIME_RANGE)
}
pub fn format_time(value:&Time)->String{
    let mut result=value.0.format("%H:%M:%S").to_string();
    if value.0.nanosecond()!=0{result.push('.');result.push_str(format!("{:09}",value.0.nanosecond()).trim_end_matches('0'));}result
}
pub fn compare_times(left:&Time,right:&Time)->i64{ordering(left.0.cmp(&right.0))}
fn ordering(value:std::cmp::Ordering)->i64{match value{std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}}
pub fn parse_iso(text:&str)->Result<Instant,Error>{
    if !text.is_ascii() || !(20..=35).contains(&text.len()) || text.as_bytes()[10]!=b'T'{return Err(TIME_SYNTAX)}
    parse_date(&text[..10])?;
    let end=if text.ends_with('Z'){text.len()-1}else{
        let end=text.len()-6;let offset=&text.as_bytes()[end..];
        if !matches!(offset[0],b'+'|b'-') || offset[3]!=b':' || ![1,2,4,5].iter().all(|&i|offset[i].is_ascii_digit()) || &text[end..]=="-00:00"{return Err(TIME_SYNTAX)}
        end
    };
    if end<11{return Err(TIME_SYNTAX)}parse_time(&text[11..end])?;
    checked_instant(DateTime::parse_from_rfc3339(text).map_err(|_|TIME_SYNTAX)?.with_timezone(&Utc))
}
pub fn format_iso(value:&Instant)->String{value.0.to_rfc3339_opts(SecondsFormat::AutoSi,true)}
pub fn now()->Result<Instant,Error>{
    let elapsed=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let (duration,sign)=match elapsed{Ok(value)=>(value,1i128),Err(error)=>(error.duration(),-1i128)};
    let nanos=sign*(i128::from(duration.as_secs())*1_000_000_000+i128::from(duration.subsec_nanos()));
    let seconds=i64::try_from(nanos.div_euclid(1_000_000_000)).map_err(|_|TIME_RANGE)?;
    checked_instant(DateTime::from_timestamp(seconds,nanos.rem_euclid(1_000_000_000) as u32).ok_or(TIME_RANGE)?)
}
pub fn from_unix_milliseconds(value:i64)->Result<Instant,Error>{checked_instant(DateTime::from_timestamp_millis(value).ok_or(TIME_RANGE)?)}
pub fn to_unix_milliseconds(value:&Instant)->Result<i64,Error>{
    if value.0.timestamp_subsec_nanos()%1_000_000!=0{return Err(("Precision","Instant has sub-millisecond precision"))}Ok(value.0.timestamp_millis())
}
pub fn from_unix_seconds(value:i64)->Result<Instant,Error>{checked_instant(DateTime::from_timestamp(value,0).ok_or(TIME_RANGE)?)}
pub fn to_unix_seconds(value:&Instant)->Result<i64,Error>{if value.0.timestamp_subsec_nanos()!=0{return Err(("Precision","Instant has fractional seconds"))}Ok(value.0.timestamp())}
pub fn compare_instants(left:&Instant,right:&Instant)->i64{ordering(left.0.cmp(&right.0))}
pub fn duration_from_nanoseconds(value:i64)->Duration{Duration(value)}
pub fn duration_from_milliseconds(value:i64)->Result<Duration,Error>{value.checked_mul(1_000_000).map(Duration).ok_or(TIME_RANGE)}
pub fn duration_from_seconds(value:i64)->Result<Duration,Error>{value.checked_mul(1_000_000_000).map(Duration).ok_or(TIME_RANGE)}
pub fn duration_to_nanoseconds(value:&Duration)->i64{value.0}
pub fn duration_to_milliseconds(value:&Duration)->Result<i64,Error>{if value.0%1_000_000!=0{return Err(("Precision","Duration has sub-millisecond precision"))}Ok(value.0/1_000_000)}
pub fn add_durations(left:&Duration,right:&Duration)->Result<Duration,Error>{left.0.checked_add(right.0).map(Duration).ok_or(TIME_RANGE)}
pub fn subtract_durations(left:&Duration,right:&Duration)->Result<Duration,Error>{left.0.checked_sub(right.0).map(Duration).ok_or(TIME_RANGE)}
pub fn add_duration(value:&Instant,amount:&Duration)->Result<Instant,Error>{checked_instant(value.0.checked_add_signed(TimeDelta::nanoseconds(amount.0)).ok_or(TIME_RANGE)?)}
pub fn difference(later:&Instant,earlier:&Instant)->Result<Duration,Error>{(later.0-earlier.0).num_nanoseconds().map(Duration).ok_or(TIME_RANGE)}
pub fn parse_time_zone(name:&str)->Result<TimeZone,Error>{
    if name.len()>128{return Err(("Zone","Time zone name exceeds 128 bytes"))}
    name.parse::<chrono_tz::Tz>().map(TimeZone::Named).map_err(|_|("Zone","Unknown case-sensitive IANA time zone name"))
}
pub fn fixed_offset(seconds:i64)->Result<TimeZone,Error>{
    i32::try_from(seconds).ok().and_then(FixedOffset::east_opt).map(TimeZone::Fixed).ok_or(TIME_RANGE)
}
pub fn zone_name(zone:&TimeZone)->String{match zone{TimeZone::Named(value)=>value.name().to_owned(),TimeZone::Fixed(value)=>value.to_string()}}
pub fn offset_seconds(value:&Instant,zone:&TimeZone)->i64{match zone{TimeZone::Named(zone)=>i64::from(value.0.with_timezone(zone).offset().fix().local_minus_utc()),TimeZone::Fixed(zone)=>i64::from(zone.local_minus_utc())}}
fn local(value:&Instant,zone:&TimeZone)->Result<chrono::NaiveDateTime,Error>{
    value.0.naive_utc().checked_add_signed(TimeDelta::seconds(offset_seconds(value,zone))).ok_or(TIME_RANGE)
}
pub fn local_date(value:&Instant,zone:&TimeZone)->Result<Date,Error>{checked(local(value,zone)?.date())}
pub fn local_time(value:&Instant,zone:&TimeZone)->Result<Time,Error>{Ok(Time(local(value,zone)?.time()))}
pub fn resolve_date_time(date:&Date,time:&Time,zone:&TimeZone,disambiguation:&str)->Result<Instant,Error>{
    if !matches!(disambiguation,"reject"|"earlier"|"later"){return Err(("Disambiguation","Expected reject, earlier or later"))}
    let local=date.0.and_time(time.0);
    let result=match zone{
        TimeZone::Named(zone)=>zone.from_local_datetime(&local).map(|value|value.with_timezone(&Utc)),
        TimeZone::Fixed(zone)=>zone.from_local_datetime(&local).map(|value|value.with_timezone(&Utc))
    };
    match result{
        LocalResult::Single(value)=>checked_instant(value),
        LocalResult::None=>Err(("NonexistentTime","Local time falls in a zone transition gap")),
        LocalResult::Ambiguous(first,second)=>match disambiguation{
            "earlier"=>checked_instant(first.min(second)),"later"=>checked_instant(first.max(second)),
            _=>Err(("AmbiguousTime","Local time occurs twice; choose earlier or later explicitly"))
        }
    }
}
