//! Static borrowed alternatives retain the concrete environments' lifetimes.
use crate::Mode;

pub(crate) fn emit_definition(output:&mut String,id:usize) {
    use std::fmt::Write;
    writeln!(output,"enum BorrowedCallable{id}<Left, Right> {{ Left(Left), Right(Right) }}").unwrap();
}

pub(crate) fn variant(index:usize,count:usize)->String {if count==2{if index==0{"Left".into()}else{"Right".into()}}else{format!("V{index}")}}
pub(crate) fn emit_definition_many(output:&mut String,id:usize,count:usize){
    if count==2{emit_definition(output,id);return;}
    use std::fmt::Write;
    let args=(0..count).map(|i|format!("Origin{i}")).collect::<Vec<_>>().join(",");
    let variants=(0..count).map(|i|format!("V{i}(Origin{i})")).collect::<Vec<_>>().join(",");
    writeln!(output,"enum BorrowedCallable{id}<{args}> {{{variants}}}").unwrap();
}

pub(crate) fn emit(output: &mut String, id: usize, mode: Mode, counted: bool) -> bool {
    use std::fmt::Write;
    let (trait_name, method, receiver) = match mode {
        Mode::Read => ("ReadCallback", "invoke_read", "&self"),
        Mode::Mutate => ("Callback", "invoke", "&mut self"),
        _ => return false,
    };
    let name = format!("BorrowedCallable{id}");
    emit_definition(output,id);
    writeln!(output, "impl<Args, Result, Left, Right> {trait_name}<Args, Result> for {name}<Left, Right> where Left: {trait_name}<Args, Result>, Right: {trait_name}<Args, Result> {{").unwrap();
    writeln!(output, "fn {method}({receiver}, args: Args) -> Result {{ match self {{ Self::Left(value) => value.{method}(args), Self::Right(value) => value.{method}(args) }} }}").unwrap();
    if counted {
        writeln!(output, "fn {method}_with_count({receiver}, args: Args, count: usize) -> Result {{ match self {{ Self::Left(value) => value.{method}_with_count(args, count), Self::Right(value) => value.{method}_with_count(args, count) }} }}").unwrap();
    }
    output.push_str("}\n");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrowed_dispatch_preserves_access_and_rejects_consuming_contracts() {
        for mode in [Mode::Read, Mode::Mutate] {
            let mut code = String::new();
            assert!(emit(&mut code, 7, mode, true));
            assert!(code.contains("enum BorrowedCallable7<Left, Right>"));
            assert!(!code.contains("Box<") && !code.contains("unsafe") && !code.contains("clone"));
        }
        let mut code = String::new();
        assert!(!emit(&mut code, 7, Mode::Consume, true));
        assert!(code.is_empty());
    }
    fn compile(source: &str, run: bool, level: &str) -> std::process::Output {
        use std::{fs, process::Command};
        let root = std::env::temp_dir().join(format!("tsrust-borrowed-dispatch-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        let path = root.join("witness.rs");
        let executable = root.join("witness");
        fs::write(&path, source).unwrap();
        let output = Command::new("rustc").args(["--edition=2024", "-C"]).arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&executable).output().unwrap();
        let output = if run {
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            Command::new(&executable).output().unwrap()
        } else { output };
        fs::remove_dir_all(&root).unwrap();
        output
    }

    const TRAITS: &str = r#"
        trait Callback<A,R> {
            fn invoke(&mut self,args:A)->R;
            fn invoke_with_count(&mut self,args:A,_count:usize)->R {self.invoke(args)}
        }
        trait ReadCallback<A,R> {
            fn invoke_read(&self,args:A)->R;
            fn invoke_read_with_count(&self,args:A,_count:usize)->R {self.invoke_read(args)}
        }
        struct Reader<'a>(&'a Vec<i64>);
        impl ReadCallback<(),usize> for Reader<'_> {
            fn invoke_read(&self,_:())->usize {self.0.len()}
            fn invoke_read_with_count(&self,_:(),count:usize)->usize {self.0.len()+count}
        }
        struct Mutator<'a>(&'a mut i64,i64);
        impl Callback<i64,Result<i64,&'static str>> for Mutator<'_> {
            fn invoke(&mut self,value:i64)->Result<i64,&'static str> {
                if value<0 {return Err("failure");}
                *self.0+=self.1+value;Ok(*self.0)
            }
            fn invoke_with_count(&mut self,value:i64,count:usize)->Result<i64,&'static str> {
                self.invoke(value+count as i64)
            }
        }
    "#;

    #[test]
    fn native_borrowed_dispatch_keeps_lifetimes_mutation_failures_and_counts() {
        let mut code=TRAITS.to_owned();
        assert!(emit(&mut code,7,Mode::Read,true));
        assert!(emit(&mut code,8,Mode::Mutate,true));
        code.push_str(r#"
            struct Owner<'a>(&'a std::cell::Cell<usize>);
            impl Drop for Owner<'_> {fn drop(&mut self){self.0.set(self.0.get()+1);}}
            struct Observer<'a>(&'a Owner<'a>);
            impl ReadCallback<(),usize> for Observer<'_> {
                fn invoke_read(&self,_:())->usize {self.0.0.get()}
            }
            fn main(){
                let drops=std::cell::Cell::new(0);
                {
                    let first=Owner(&drops);let second=Owner(&drops);
                    for flag in [true,false] {
                        let selected=if flag {BorrowedCallable7::Left(Observer(&first))} else {BorrowedCallable7::Right(Observer(&second))};
                        assert_eq!(selected.invoke_read(()),0);
                        drop(selected);assert_eq!(drops.get(),0);
                    }
                }
                assert_eq!(drops.get(),2);
                let left=vec![3];let right=vec![7,9];
                for flag in [true,false] {
                    let selected=if flag {BorrowedCallable7::Left(Reader(&left))} else {BorrowedCallable7::Right(Reader(&right))};
                    assert_eq!(selected.invoke_read(()),if flag {1} else {2});
                    assert_eq!(selected.invoke_read_with_count((),4),if flag {5} else {6});
                }
                for flag in [true,false] {
                    let mut left=0;let mut right=10;
                    {
                        let mut selected=if flag {BorrowedCallable8::Left(Mutator(&mut left,1))} else {BorrowedCallable8::Right(Mutator(&mut right,2))};
                        assert_eq!(selected.invoke(0),Ok(if flag {1} else {12}));
                        assert_eq!(selected.invoke(-1),Err("failure"));
                        assert_eq!(selected.invoke_with_count(0,2),Ok(if flag {4} else {16}));
                    }
                    assert_eq!((left,right),if flag {(4,10)} else {(0,16)});
                }
            }
        "#);
        for level in ["0", "3"] {
            let result=compile(&code,true,level);
            assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
            assert!(result.stdout.is_empty());
        }
    }

    #[test]
    fn native_borrowed_dispatch_rejects_escape_and_overlapping_mutation() {
        for (mode,id,body,diagnostic) in [
            (Mode::Read,7,r#"fn escape<'a>()->BorrowedCallable7<Reader<'a>,Reader<'a>> {let values=vec![1];BorrowedCallable7::Left(Reader(&values))} fn main(){}"#,"E0515"),
            (Mode::Mutate,8,r#"fn main(){let mut left=0;let mut right=10;let mut selected=if true {BorrowedCallable8::Left(Mutator(&mut left,1))} else {BorrowedCallable8::Right(Mutator(&mut right,2))};left=3;selected.invoke(0).unwrap();}"#,"E0506"),
        ] {
            let mut code=TRAITS.to_owned();assert!(emit(&mut code,id,mode,false));code.push_str(body);
            let result=compile(&code,false,"0");
            assert!(!result.status.success());
            assert!(String::from_utf8_lossy(&result.stderr).contains(diagnostic),"{}",String::from_utf8_lossy(&result.stderr));
        }
    }

}
