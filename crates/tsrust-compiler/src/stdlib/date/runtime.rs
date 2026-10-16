    use chrono::{Datelike,NaiveDate,Months,TimeDelta};
    type Error=(&'static str,&'static str);
    const RANGE:Error=("Range","Date must be valid and within years 0001 through 9999");
    #[derive(Clone)]
    pub struct Date(NaiveDate);
    fn checked(value:NaiveDate)->Result<Date,Error>{if (1..=9999).contains(&value.year()){Ok(Date(value))}else{Err(RANGE)}}
    pub fn parse_date(text:&str)->Result<Date,Error>{
        let b=text.as_bytes();
        if b.len()!=10 || b[4]!=b'-' || b[7]!=b'-' || !b.iter().enumerate().all(|(i,c)|i==4||i==7||c.is_ascii_digit()) {return Err(("Syntax","Expected YYYY-MM-DD with exactly four year digits"))}
        checked(NaiveDate::parse_from_str(text,"%Y-%m-%d").map_err(|_|RANGE)?)
    }
    pub fn format_date(value:&Date)->String{value.0.format("%Y-%m-%d").to_string()}
    pub fn add_days(value:&Date,amount:i64)->Result<Date,Error>{checked(value.0.checked_add_signed(TimeDelta::try_days(amount).ok_or(RANGE)?).ok_or(RANGE)?)}
    pub fn add_months(value:&Date,amount:i64)->Result<Date,Error>{
        let count=Months::new(u32::try_from(amount.unsigned_abs()).map_err(|_|RANGE)?);
        let result=if amount<0{value.0.checked_sub_months(count)}else{value.0.checked_add_months(count)};
        checked(result.ok_or(RANGE)?)
    }
    pub fn difference_in_calendar_days(later:&Date,earlier:&Date)->i64{(later.0-earlier.0).num_days()}
    pub fn compare_asc(left:&Date,right:&Date)->i64{match left.0.cmp(&right.0){std::cmp::Ordering::Less=>-1,std::cmp::Ordering::Equal=>0,std::cmp::Ordering::Greater=>1}}
    pub fn is_leap_year(value:&Date)->bool{NaiveDate::from_ymd_opt(value.0.year(),2,29).is_some()}
    pub fn get_year(value:&Date)->i64{i64::from(value.0.year())}
    pub fn get_month(value:&Date)->i64{i64::from(value.0.month0())}
    pub fn get_date(value:&Date)->i64{i64::from(value.0.day())}
    pub fn start_of_month(value:&Date)->Date{Date(value.0.with_day(1).unwrap())}
    pub fn end_of_month(value:&Date)->Date{Date((28..=31).rev().find_map(|day|value.0.with_day(day)).unwrap())}
    pub fn format(value:&Date,pattern:&str)->Result<String,Error>{
        const FORMAT:Error=("Format","Unsupported date token, unclosed quoted literal or pattern longer than 256 bytes");
        if pattern.len()>256{return Err(FORMAT)}
        let chars:Vec<char>=pattern.chars().collect();let mut i=0;let mut output=String::new();
        while i<chars.len(){
            let ch=chars[i];
            if ch=='\''{
                i+=1;if chars.get(i)==Some(&'\''){output.push('\'');i+=1;continue}
                let mut closed=false;
                while i<chars.len(){
                    if chars[i]=='\''{i+=1;if chars.get(i)==Some(&'\''){output.push('\'');i+=1;}else{closed=true;break}}else{output.push(chars[i]);i+=1}
                }
                if !closed{return Err(FORMAT)}continue
            }
            if ch.is_ascii_alphabetic(){
                let start=i;while i<chars.len()&&chars[i]==ch{i+=1}
                let token:String=chars[start..i].iter().collect();
                let spec=match token.as_str(){"yyyy"=>"%Y","yy"=>"%y","MMMM"=>"%B","MMM"=>"%b","MM"=>"%m","M"=>"%-m","dd"=>"%d","d"=>"%-d","EEEE"=>"%A","EEE"=>"%a",_=>return Err(FORMAT)};
                output.push_str(&value.0.format(spec).to_string());
            }else{output.push(ch);i+=1}
        }
        Ok(output)
    }
