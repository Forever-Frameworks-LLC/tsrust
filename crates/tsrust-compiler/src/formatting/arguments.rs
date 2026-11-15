//! Wrap direct parameters/arguments without counting nested expressions or generic types.
use super::{FormatOptions, types};
use crate::{Span, syntax::{Token, TokenKind}};

pub(super) fn layout(tokens: &[Token], generics: &[Span], options: &FormatOptions) -> (Vec<bool>, Vec<usize>) {
    let mut breaks=vec![false;tokens.len()];
    let mut changes=vec![0isize;tokens.len()+1];
    if options.argument_wrap_threshold==0 {return (breaks,vec![0;tokens.len()]);}
    let mut stack:Vec<(usize,Vec<usize>)>=Vec::new();
    for (index,token) in tokens.iter().enumerate() {
        if token.kind!=TokenKind::Symbol {continue;}
        match token.text.as_str() {
            "("|"["|"{"=>stack.push((index,Vec::new())),
            "<" if types::contains(generics,token.span.start)=>stack.push((index,Vec::new())),
            ","=>if let Some((open,commas))=stack.last_mut() && tokens[*open].text=="(" {commas.push(index);},
            ")"|"]"|"}"|">"=>{
                let expected=match token.text.as_str(){")"=>"(","]"=>"[","}"=>"{",_=>"<"};
                if stack.last().is_none_or(|(open,_)|tokens[*open].text!=expected){continue;}
                let (open,commas)=stack.pop().unwrap();
                if expected!="(" || !arguments(tokens,open,index){continue;}
                let trailing=commas.last()==index.checked_sub(1).as_ref();
                let count=usize::from(index>open+1)+commas.len()-usize::from(trailing);
                if count<=options.argument_wrap_threshold {continue;}
                breaks[open+1]=true;
                for comma in commas {breaks[comma+1]=true;}
                breaks[index]=true;
                changes[open+1]+=1;changes[index]-=1;
            }
            _=>{}
        }
    }
    let mut depth=0isize;
    let indentation=changes.into_iter().take(tokens.len()).map(|change|{depth+=change;depth as usize}).collect();
    (breaks,indentation)
}

fn arguments(tokens:&[Token],open:usize,close:usize)->bool {
    if tokens.get(close+1).is_some_and(|token|token.text=="=>" || token.text==":" && super::arrow_result_colon(tokens,close+1)){return true;}
    let Some(previous)=open.checked_sub(1).map(|index|&tokens[index])else{return false;};
    if previous.kind==TokenKind::Name {
        return !matches!(previous.text.as_str(),"if"|"for"|"while"|"switch"|"catch"|"match"|"return"|"throw"|"typeof"|"await"|"else"|"in"|"of");
    }
    previous.kind==TokenKind::Symbol && matches!(previous.text.as_str(),")"|"]"|">"|"?."|"!")
}

#[cfg(test)]
mod tests {
    use crate::{format,format_with_options,FormatOptions};
    #[test]
    fn wraps_parameters_calls_and_callbacks_with_nested_types_and_blocks() {
        for source in [
            "function App(path:string,requests:ReadOnly<Channel<string>>,results:Channel<Result<int,string>>,a:int,b:int,c:int):Node{return make(path,requests,results,a,b,c);}",
            "function main(){outer(1,2,3,4,5,(a:int,b:int,c:int,d:int,e:int,f:int):int=>{inner(1,2,3,4,5,6);return a;});}",
            "function main(){call<Map<string,int>>(a<b,c>d,{a:1,b:2},[1,2],other(1,2),6);}",
            "function main(){call(1,2,3,4,5,/* keep */6,);}",
        ] {
            let result=format(source).unwrap();
            assert!(result.contains("(\n"),"{result}");
            assert_eq!(format(&result).unwrap(),result,"{result}");
            let tokens=|source:&str|super::types::normalized_tokens(source).unwrap().into_iter().map(|token|(token.kind,token.text)).collect::<Vec<_>>();
            assert_eq!(tokens(source),tokens(&result));
        }
        let source="function main(){call(1,2,3,4,5);}";
        assert!(!format(source).unwrap().contains("call(\n"));
        let mut options=FormatOptions::default();options.argument_wrap_threshold=0;
        assert!(!format_with_options("function main(){call(1,2,3,4,5,6);}",&options).unwrap().contains("call(\n"));
        options.argument_wrap_threshold=2;
        assert!(format_with_options("function main(){call(1,2,3);}",&options).unwrap().contains("call(\n"));
        assert!(FormatOptions::overlay(&options,&serde_json::json!({"argumentWrapThreshold":101})).is_err());
    }
}
