// Independent Rust literal replacement and substitution-token expansion.
// Standard replace/replacen handle plain templates; token matches use a
// single-pass owned String builder and original-input prefix/suffix views.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct Probe{trace:i64,calls:i64}
fn mark(probe:&mut Probe,index:i64,tag:i64)->i64{probe.trace=probe.trace.checked_mul(131).unwrap().checked_add(probe.calls).unwrap().checked_add(index%65521).unwrap().checked_add(65521).unwrap().checked_add(tag).unwrap()%65521;probe.calls=probe.calls.checked_add(1).unwrap();index}
fn markers(probe:&Probe)->Array<String>{let mut row=vec![String::Borrowed("trace")];for (index,initial) in [probe.trace,probe.calls].into_iter().enumerate(){if index==1{row.push(String::Borrowed("calls"));}let mut value=initial;for bit in (0..16).rev(){let factor=1_i64<<bit;if value>=factor{row.push(String::Borrowed("1"));value=value.checked_sub(factor).unwrap();}else{row.push(String::Borrowed("0"));}}}Array(row)}
fn expand(output:&mut std::string::String,template:&str,text:&str,start:usize,end:usize){
    let mut characters=template.chars();while let Some(character)=characters.next(){if character!='$'{output.push(character);continue;}
        match characters.next(){Some('$')=>output.push('$'),Some('&')=>output.push_str(&text[start..end]),Some('`')=>output.push_str(&text[..start]),Some('\'')=>output.push_str(&text[end..]),Some(other)=>{output.push('$');output.push(other);},None=>output.push('$')}
    }
}
fn replace(text:&str,pattern:&str,template:&str,all:bool)->std::string::String{
    if !template.contains('$'){return if all{text.replace(pattern,template)}else{text.replacen(pattern,template,1)};}
    let mut output=std::string::String::with_capacity(text.len());let mut cursor=0;
    for (start,matched) in text.match_indices(pattern).take(if all{usize::MAX}else{1}){let end=start.checked_add(matched.len()).unwrap();output.push_str(&text[cursor..start]);expand(&mut output,template,text,start,end);cursor=end;}
    output.push_str(&text[cursor..]);output
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let pool=[String::Owned(value2.to_string()),String::Borrowed("ababa\u{0000}😀")];let patterns=["a","aa","missing","\u{0000}"];let replacements=["X","é😀","\u{0000}",""];let mut probe=Probe{trace:value1%65521,calls:0};let mut output=Vec::with_capacity(usize::try_from(value0.checked_add(1).unwrap()).unwrap());
    for i in 0..value0{let coordinate=i.checked_add(value1).unwrap();let receiver=usize::try_from(mark(&mut probe,coordinate%2,1)).unwrap();let pattern=usize::try_from(mark(&mut probe,coordinate%4,2)).unwrap();let template=usize::try_from(mark(&mut probe,coordinate%4,3)).unwrap();output.push(Array(vec![String::Owned(replace(pool[receiver].as_ref(),patterns[pattern],replacements[template],false))]));}
    output.push(markers(&probe));output
}
fn main(){}
