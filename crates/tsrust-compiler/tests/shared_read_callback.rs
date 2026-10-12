//! Native ownership acceptance for the shared immutable matcher callback adapter.
trait ReadCallback<A, R> {
    fn invoke_read(&self, args: A) -> R;
    fn invoke_read_with_count(&self, args: A, _count: usize) -> R { self.invoke_read(args) }
}
trait Callback<A, R> {
    fn invoke(&mut self, args: A) -> R;
    fn invoke_with_count(&mut self, args: A, _count: usize) -> R { self.invoke(args) }
}
include!("../src/stdlib/testing/runtime/shared_callback.rs");
include!("../src/stdlib/testing/runtime/shared_callback_counted.rs");

struct IdentityReader {
    actual: Box<[i64]>,
    dropped: std::rc::Rc<std::cell::Cell<usize>>,
}
impl Drop for IdentityReader {
    fn drop(&mut self) { self.dropped.set(self.dropped.get()+1); }
}
impl ReadCallback<*const i64, bool> for IdentityReader {
    fn invoke_read(&self, expected: *const i64) -> bool { self.actual.as_ptr()==expected }
}
#[test]
fn shared_callback_keeps_noncloneable_environment_identity_and_drops_once() {
    let dropped=std::rc::Rc::new(std::cell::Cell::new(0));
    let reader=IdentityReader{actual:vec![1,2].into_boxed_slice(),dropped:dropped.clone()};
    let address=reader.actual.as_ptr();
    let shared=TsrSharedReadCallback::new(reader);
    let mut other=shared.clone();
    assert!(shared.invoke_read(address));assert!(other.invoke(address));
    drop(shared);assert_eq!(dropped.get(),0);assert!(other.invoke(address));
    drop(other);assert_eq!(dropped.get(),1);
}
struct ArgumentCount;
impl ReadCallback<usize,usize> for ArgumentCount {
    fn invoke_read(&self,args:usize)->usize {args}
    fn invoke_read_with_count(&self,args:usize,count:usize)->usize {args+count}
}
#[test]
fn shared_callback_preserves_optional_argument_count_dispatch() {
    let shared=TsrSharedReadCallback::new(ArgumentCount);
    let mut other=shared.clone();
    assert_eq!(shared.invoke_read_with_count(10,2),12);
    assert_eq!(other.invoke_with_count(10,3),13);
}

mod uncounted {
    trait ReadCallback<A,R> { fn invoke_read(&self,args:A)->R; }
    trait Callback<A,R> { fn invoke(&mut self,args:A)->R; }
    include!("../src/stdlib/testing/runtime/shared_callback.rs");
    include!("../src/stdlib/testing/runtime/shared_callback_read.rs");
    struct Reader;
    impl ReadCallback<i64,i64> for Reader { fn invoke_read(&self,args:i64)->i64 {args+1} }
    #[test]
    fn adapter_supports_uncounted_generated_callback_traits() {
        let original=TsrSharedReadCallback::new(Reader);let mut alias=original.clone();
        assert_eq!(original.invoke_read(3),4);assert_eq!(alias.invoke(8),9);
    }
}

mod support;
#[test]
fn adapter_shares_an_actual_generated_owned_comparison_environment() {
    let source=r#"
function comparison(actual:int[]):Function<boolean,[expected:ReadOnly<int[]>]>{return(expected:int[]):boolean=>{return Object.is(actual,expected);};}
function query(predicate:Function<boolean,[expected:ReadOnly<int[]>]>,values:int[]):boolean{return predicate(values);}
function main(){const items=[1,2];const matcher=comparison(items);console.log(query(matcher,[1,2]));}
"#;
    let checked=tsrust_compiler::check(source).unwrap();
    let mut rust=checked.emit_rust().replace("fn main()", "fn source_main()");
    rust.push_str(include_str!("../src/stdlib/testing/runtime/shared_callback.rs"));
    rust.push_str(include_str!("../src/stdlib/testing/runtime/shared_callback_read.rs"));
    rust.push_str(r#"
fn main() {
 let first=TsrSharedReadCallback::new(function0(NumericArray::from([1i64,2i64])));
 let second=first.clone();
 assert!(first.invoke_read((&second.0.capture0,)));
 assert!(second.invoke_read((&first.0.capture0,)));
 drop(first);
 assert!(second.invoke_read((&second.0.capture0,)));
 assert!(!second.invoke_read((&NumericArray::from([1i64,2i64]),)));
 println!("shared identity preserved");
}
"#);
    let directory=support::TempDirectory::new();let path=directory.0.join("shared.rs");
    let binary=directory.0.join("shared");std::fs::write(&path,rust).unwrap();
    let output=support::run(support::rustc().args(["--edition=2024","-A","warnings"]).arg(&path).arg("-o").arg(&binary),60);
    support::success(&output);
    let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);
    assert_eq!(output.stdout,b"shared identity preserved\n");
}
