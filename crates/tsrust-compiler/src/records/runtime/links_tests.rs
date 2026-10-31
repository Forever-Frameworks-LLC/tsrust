//! Native ownership witnesses for nullable links before source publication.
use super::*;
use crate::{cloneability::tests::record, syntax::UnionId, unions::UnionType};

#[test]
fn nullable_links_clone_move_and_drop_iteratively() {
    let mut source = String::from("#![allow(dead_code,unused_variables,unused_mut)]\n");
    source.push_str(ALLOCATION_OBSERVER);
    for (name, mutual) in [("self_link", false), ("mutual_link", true)] {
        let target = usize::from(mutual);
        let union = UnionId { index: 0, copy: false, scalar_members:false };
        let records = [
            record(&[Type::Union(union), Type::Record(2)]),
            record(&[if mutual { Type::Record(0) } else { Type::Int }]),
            record(&[Type::Int]),
            record(&[Type::Array(ArrayElement::Record(3,0)),Type::Union(union)]),
            record(&[Type::Array(ArrayElement::Record(4,0)),Type::Record(2)]),
        ];
        let unions = [UnionType { recursive:false,recursive_declaration:None, members: vec![Type::Record(target), Type::Undefined] }];
        let runtime = RecordRuntime::new(&records, &unions).emit(&records);
        let other = if mutual { "struct Record1 { field0:Record0 }" } else { "" };
        let link_value = if mutual { "Record1 { field0: root }" } else { "root" };
        let child = if mutual { "&next.field0" } else { "next" };
        source.push_str(&format!(r#"
mod {name} {{
use std::sync::atomic::{{AtomicUsize,Ordering}};
static CREATED:AtomicUsize=AtomicUsize::new(0);
static DROPPED:AtomicUsize=AtomicUsize::new(0);
static DROP_ORDER:[AtomicUsize;20002]=[const{{AtomicUsize::new(0)}};20002];
struct Record2 {{ value:usize }}
impl Record2 {{ fn new(value:usize)->Self {{ CREATED.fetch_add(1,Ordering::SeqCst); Self{{value}} }} }}
impl Clone for Record2 {{ fn clone(&self)->Self {{ Self::new(self.value) }} }}
impl Drop for Record2 {{ fn drop(&mut self) {{ let slot=DROPPED.fetch_add(1,Ordering::SeqCst);DROP_ORDER[slot].store(self.value,Ordering::SeqCst); }} }}
#[derive(Clone)] struct TsrUndefined;
enum Union0 {{ V0(RecordLink{target}), V1(TsrUndefined) }}
struct Record0 {{ field0:Union0,field1:Record2 }}
struct Record3 {{ field0:RecordArray3,field1:Union0 }}
struct Record4 {{ field0:RecordArray4,field1:Record2 }}
{other}
{runtime}
pub fn run() {{
    let baseline=crate::LIVE.load(Ordering::SeqCst);
    let allocations=crate::ALLOCATIONS.load(Ordering::SeqCst);
    let mut root=Record0{{field0:Union0::V1(TsrUndefined),field1:Record2::new(0)}};
    for value in 1..10001 {{ root=Record0{{field0:Union0::V0(RecordLink{target}::new({link_value})),field1:Record2::new(value)}}; }}
    let bytes=10000*std::mem::size_of::<Record{target}>();
    assert_eq!(crate::ALLOCATIONS.load(Ordering::SeqCst)-allocations,10000);
    assert_eq!(crate::LIVE.load(Ordering::SeqCst)-baseline,bytes);
    let mut copied=root.clone();
    assert_eq!(crate::LIVE.load(Ordering::SeqCst)-baseline,2*bytes);
    copied.field1.value=99999;
    assert_eq!(root.field1.value,10000);
    let (Union0::V0(original),Union0::V0(copy))=(&root.field0,&copied.field0) else {{panic!("present links")}};
    assert_ne!(std::ptr::from_ref(&**original),std::ptr::from_ref(&**copy));
    let mut cursor=&root;let mut expected=10000;
    loop {{ assert_eq!(cursor.field1.value,expected);match &cursor.field0 {{ Union0::V0(next)=>{{cursor={child};expected-=1;}},Union0::V1(_)=>break }} }}
    assert_eq!(expected,0);
    let moved=match root.field0 {{ Union0::V0(link)=>link.into_inner(), _=>unreachable!() }};
    drop(moved);drop(root.field1);drop(copied);
    assert_eq!(CREATED.load(Ordering::SeqCst),20002);
    assert_eq!(DROPPED.load(Ordering::SeqCst),20002);
    assert_eq!(crate::LIVE.load(Ordering::SeqCst),baseline);
    // Links retain the existing recursive-array disposal order under the same
    // partial move, deep clone and root-field mutation sequence.
    let link_order=DROP_ORDER.iter().map(|slot|slot.load(Ordering::SeqCst)).collect::<Vec<_>>();
    DROPPED.store(0,Ordering::SeqCst);
    let mut array_root=Record4{{field0:RecordArray4::from([]),field1:Record2::new(0)}};
    for value in 1..10001{{array_root=Record4{{field0:RecordArray4::from([array_root]),field1:Record2::new(value)}};}}
    let mut array_copy=array_root.clone();array_copy.field1.value=99999;
    let array_child=array_root.field0.pop().unwrap();drop(array_root.field0);drop(array_child);drop(array_root.field1);drop(array_copy);
    assert_eq!(DROPPED.load(Ordering::SeqCst),20002);
    assert!(link_order.iter().zip(DROP_ORDER.iter()).all(|(expected,actual)|*expected==actual.load(Ordering::SeqCst)));
    drop(link_order);assert_eq!(crate::LIVE.load(Ordering::SeqCst),baseline);
}}
}}
"#));
    }
    for (name, mutable) in [("shared_views",false),("exclusive_views",true)] {
        let union=UnionId{index:0,copy:false, scalar_members:false};
        let mut view=record(&[]);
        view.native_type=Some(if mutable {crate::native_signatures::NativeTypeId::ByteViewMut}else{crate::native_signatures::NativeTypeId::ByteView});
        let records=[record(&[Type::Union(union),Type::Record(1)]),view];
        let unions=[UnionType{recursive:false,recursive_declaration:None,members:vec![Type::Record(0),Type::Undefined]}];
        let runtime=RecordRuntime::new(&records,&unions).emit(&records);
        assert!(!runtime.contains("impl Clone"));
        assert!(!runtime.contains("unsafe"));
        let reference=if mutable {"&mut"} else {"&"};
        let mutable_token=if mutable {"mut "}else{""};
        let change=if mutable {"returned.bytes[0]=13;"} else {""};
        let expected=if mutable {13} else {7};
        source.push_str(&format!(r#"
mod {name} {{
use std::sync::atomic::{{AtomicUsize,Ordering}};
static DROPPED:AtomicUsize=AtomicUsize::new(0);
struct Record1<'view>{{bytes:&'view {mutable_token}[u8]}}
impl Drop for Record1<'_>{{fn drop(&mut self){{DROPPED.fetch_add(1,Ordering::SeqCst);}}}}
struct TsrUndefined;
enum Union0<'view>{{V0(RecordLink0<'view>),V1(TsrUndefined)}}
struct Record0<'view>{{field0:Union0<'view>,field1:Record1<'view>}}
{runtime}
#[cfg(escape)] fn invalid_escape()->Record0<'static>{{
    let mut owner=[7];Record0{{field0:Union0::V1(TsrUndefined),field1:Record1{{bytes:{reference} owner}}}}
}}
pub fn run(){{
    let mut owner=vec![7u8;10001];let pointer=owner.as_ptr();
    let baseline=crate::LIVE.load(Ordering::SeqCst);
    {{
    let mut cells=owner.chunks_mut(1);
    let mut root=Record0{{field0:Union0::V1(TsrUndefined),field1:Record1{{bytes:cells.next().unwrap()}}}};
    for cell in cells{{root=Record0{{field0:Union0::V0(RecordLink0::new(root)),field1:Record1{{bytes:cell}}}};}}
    assert_eq!(crate::LIVE.load(Ordering::SeqCst)-baseline,10000*std::mem::size_of::<Record0<'_>>());
    let mut cursor=&root;let mut count=1;
    while let Union0::V0(link)=&cursor.field0{{cursor=link;count+=1;}}
    assert_eq!(count,10001);assert_eq!(cursor.field1.bytes.as_ptr(),pointer);
    let mut returned=root.field1;
    {change}
    assert_eq!(returned.bytes.as_ptr(),pointer.wrapping_add(10000));
    drop(root.field0);assert_eq!(DROPPED.load(Ordering::SeqCst),10000);
    drop(returned);assert_eq!(DROPPED.load(Ordering::SeqCst),10001);
    assert_eq!(crate::LIVE.load(Ordering::SeqCst),baseline);
    }}
    assert_eq!(owner[10000],{expected});owner[0]=9;
}}
}}
"#));
    }
    source.push_str("fn main(){std::thread::Builder::new().stack_size(128*1024).spawn(||{self_link::run();mutual_link::run();shared_views::run();exclusive_views::run();}).unwrap().join().unwrap();}");
    let directory=std::env::temp_dir().join(format!("tsrust-nullable-link-runtime-{}",std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    let _cleanup=Cleanup(directory.clone());
    let input=directory.join("main.rs");std::fs::write(&input,source).unwrap();
    for level in [0,3] {
        let output=directory.join(format!("run-{level}"));
        let built=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024",&format!("-Copt-level={level}")]).arg(&input).arg("-o").arg(&output).output().unwrap();
        assert!(built.status.success(),"{}",String::from_utf8_lossy(&built.stderr));
        let run=std::process::Command::new(output).output().unwrap();
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    }
    let rejected=std::process::Command::new("rustup").args(["run",crate::RUST_BACKEND_VERSION,"rustc","--edition=2024","--cfg","escape"]).arg(&input).arg("-o").arg(directory.join("rejected")).output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("cannot return value referencing local variable"),"{}",String::from_utf8_lossy(&rejected.stderr));
}


// Instrumentation only: generated ownership support contains no unsafe code.
const ALLOCATION_OBSERVER:&str=r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering};
static LIVE:AtomicUsize=AtomicUsize::new(0);
static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);
struct Observer;
unsafe impl GlobalAlloc for Observer {
    unsafe fn alloc(&self,layout:Layout)->*mut u8 {
        let pointer=unsafe{System.alloc(layout)};
        if !pointer.is_null(){LIVE.fetch_add(layout.size(),Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}
        pointer
    }
    unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(layout.size(),Ordering::SeqCst);}
    unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8 {
        let pointer=unsafe{System.realloc(pointer,layout,size)};
        if !pointer.is_null(){LIVE.fetch_add(size,Ordering::SeqCst);LIVE.fetch_sub(layout.size(),Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}
        pointer
    }
}
#[global_allocator]static ALLOCATOR:Observer=Observer;
"#;
