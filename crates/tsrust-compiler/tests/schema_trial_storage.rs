mod support;

#[test]
fn sequence_storage_preserves_rollback_commitment_and_cleanup_at_o0_o3() {
    let allocator = include_str!("../../../benchmarks/runtime/allocation-driver.rs")
        .split("fn main()").next().unwrap()
        .replace("fn grow(bytes: usize) {", "static WATCHED: AtomicUsize = AtomicUsize::new(0);\nstatic ROLLBACK: AtomicUsize = AtomicUsize::new(0);\nfn grow(bytes: usize) { if bytes != 0 && bytes == WATCHED.load(Relaxed) { ROLLBACK.fetch_add(1, Relaxed); }");
    let harness = r#"
use super::trial::{sequence, Restore, Trial};
use std::cell::Cell;
struct Undo<'a> { tag: [usize; 8], restored: &'a Cell<usize>, dropped: &'a Cell<usize> }
impl<T> Restore<T,T> for Undo<'_> {
    fn restore(self, value:T)->T { self.restored.set(self.restored.get()+1); value }
}
impl Drop for Undo<'_> { fn drop(&mut self) { self.dropped.set(self.dropped.get()+1); } }
fn measure<T>(bytes:usize, action:impl FnOnce()->T)->(T,usize,usize,usize) {
    WATCHED.store(bytes,Relaxed); ROLLBACK.store(0,Relaxed);
    ALLOCS.store(0,Relaxed); REALLOCS.store(0,Relaxed);
    let value=std::hint::black_box(action());
    (value,ALLOCS.load(Relaxed),REALLOCS.load(Relaxed),ROLLBACK.load(Relaxed))
}
pub(crate) fn run() {
    let hook=std::panic::take_hook(); std::panic::set_hook(Box::new(|info|{if info.payload().downcast_ref::<&str>()!=Some(&"sequence storage unwind"){eprintln!("{info}");}}));
    for count in [0,1,4,5,17] {
        for prepared in [false,true] {
            let baseline=LIVE.load(Relaxed);
            let restored=Cell::new(0); let dropped=Cell::new(0);
            let input=(0..count).collect::<Vec<usize>>();
            let (trial,allocations,reallocations,rollback)=measure(std::mem::size_of::<Undo<'_>>()*count,|| {
                sequence(std::hint::black_box(input),|_,value|->Trial<usize,usize,Undo<'_>,()> {
                    if std::hint::black_box(prepared) {
                        Trial::Prepared{value,restoration:Undo{tag:[0;8],restored:&restored,dropped:&dropped}}
                    } else { Trial::Committed(Some(value)) }
                })
            });
            assert_eq!(rollback,usize::from(prepared && count>0));
            assert_eq!(allocations,usize::from(count>0)*(1+usize::from(prepared)));
            assert_eq!(reallocations,0,"reserve once at inline/heap boundaries");
            let values=match trial {
                Trial::Prepared{value,restoration}=>restoration.restore(value),
                Trial::Committed(Some(value))=>value,
                _=>panic!("valid sequence rejected"),
            };
            assert_eq!(values,(0..count).collect::<Vec<_>>());
            assert_eq!(restored.get(),if prepared{count}else{0});
            assert_eq!(dropped.get(),if prepared{count}else{0});
            drop(values); assert_eq!(LIVE.load(Relaxed),baseline);
        }
        if count>0 {
            let baseline=LIVE.load(Relaxed);let restored=Cell::new(0);let dropped=Cell::new(0);let calls=Cell::new(0);
            let input=(0..count).collect::<Vec<usize>>();
            let (trial,_,_,rollback)=measure(std::mem::size_of::<Undo<'_>>()*count,|| {
                sequence(std::hint::black_box(input),|index,value|->Trial<usize,usize,Undo<'_>,()> {
                    calls.set(calls.get()+1);
                    if std::hint::black_box(index==0){Trial::Rejected(value)}else{Trial::Prepared{value,restoration:Undo{tag:[0;8],restored:&restored,dropped:&dropped}}}
                })
            });
            assert_eq!(rollback,0);assert_eq!(calls.get(),1);assert_eq!(restored.get(),0);assert_eq!(dropped.get(),0);
            let Trial::Rejected(values)=trial else{panic!("rejection committed")};
            assert_eq!(values,(0..count).collect::<Vec<_>>());drop(values);assert_eq!(LIVE.load(Relaxed),baseline);
        }
        let baseline=LIVE.load(Relaxed);let input=(0..count).collect::<Vec<usize>>();
        let (trial,allocations,reallocations,_)=measure(0,||sequence::<_,_,_,()>(std::hint::black_box(input),|_,value|Trial::Prepared{value,restoration:|value|value}));
        assert_eq!(allocations,usize::from(count>0));assert_eq!(reallocations,0);
        let Trial::Prepared{value,restoration}=trial else{panic!("zero-sized restoration committed")};
        let values=restoration.restore(value);assert_eq!(values,(0..count).collect::<Vec<_>>());drop(values);assert_eq!(LIVE.load(Relaxed),baseline);
    }
    struct Item<'a>{id:usize,drops:&'a Cell<usize>}
    impl Drop for Item<'_>{fn drop(&mut self){self.drops.set(self.drops.get()+1);}}
    for mode in 0..4 {
        let baseline=LIVE.load(Relaxed);let data_drops=Cell::new(0);let undo_drops=Cell::new(0);let restored=Cell::new(0);let calls=Cell::new(0);
        let input=(0..4).map(|id|Item{id,drops:&data_drops}).collect::<Vec<_>>();
        let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sequence(std::hint::black_box(input),|index,value|->Trial<Item<'_>,Item<'_>,Undo<'_>,()> {
                calls.set(calls.get()+1);
                if index==1 {
                    match std::hint::black_box(mode) {
                        0=>return Trial::Rejected(value),
                        1=>return Trial::Committed(Some(value)),
                        2=>{drop(value);return Trial::Aborted(());},
                        _=>panic!("sequence storage unwind"),
                    }
                }
                if index==2 && mode==1 {return Trial::Rejected(value);}
                Trial::Prepared{value,restoration:Undo{tag:[0;8],restored:&restored,dropped:&undo_drops}}
            })
        }));
        match (&outcome,mode) {
            (Ok(Trial::Rejected(values)),0)=>assert!(values.iter().map(|item|item.id).eq(0..4)),
            (Ok(Trial::Committed(None)),1)|(Ok(Trial::Aborted(())),2)|(Err(_),3)=>{},
            _=>panic!("wrong sequence state"),
        }
        drop(outcome);assert_eq!(data_drops.get(),4);assert_eq!(undo_drops.get(),1);assert_eq!(restored.get(),usize::from(mode==0));assert_eq!(calls.get(),if mode==1{3}else{2});assert_eq!(LIVE.load(Relaxed),baseline);
    }
    use super::issues::{Code,Issues,Kind,OwnedSegment,Segment};
    for label in ["", "count", "é😀secret", "\0é😀", "a long dynamic field name"] {
        for budget in [0,1,2,3,4,5,6,7,usize::MAX] {
            let baseline=LIVE.load(Relaxed);
            let mut original=label.to_owned();
            let mut remaining=budget;
            let expected=label.chars().take_while(|ch|{if ch.len_utf8()<=remaining{remaining-=ch.len_utf8();true}else{false}}).collect::<String>();
            let mut issues=Issues::new(1,8,budget);
            issues.push(&[Segment::Field(&original),Segment::Index(7)],Code::Missing,Kind::Integer,Kind::Missing);
            original.clear();drop(original);
            let issue=issues.items.pop().unwrap();
            assert_eq!(issue.path_truncated,expected.len()<label.len());
            assert_eq!(issue.path.len(),if expected.len()<label.len(){1}else{2});
            let mut path=issue.path.into_iter();
            let OwnedSegment::Field(name)=path.next().unwrap() else{panic!("field path changed")};
            let pointer=name.as_ptr();ALLOCS.store(0,Relaxed);REALLOCS.store(0,Relaxed);
            let public:std::string::String=std::hint::black_box(name).into();
            assert_eq!(ALLOCS.load(Relaxed),0,"public path conversion must reuse the label allocation");
            assert_eq!(REALLOCS.load(Relaxed),0);assert_eq!(public.as_ptr(),pointer);assert_eq!(public,expected);
            if expected.len()==label.len(){assert!(matches!(path.next(),Some(OwnedSegment::Index(7))));}
            drop(path);drop(public);drop(expected);drop(issues);assert_eq!(LIVE.load(Relaxed),baseline);
        }
    }
    let baseline=LIVE.load(Relaxed);
    let mut original=String::from("dynamic child");let mut incoming=Issues::new(2,8,64);
    incoming.push(&[Segment::Field(&original)],Code::Refinement,Kind::String,Kind::String);
    original.clear();drop(original);
    let mut parent=Issues::new(2,8,64);parent.append_relative(&[Segment::Field("parent"),Segment::Index(2)],incoming);
    assert_eq!(parent.items[0].path,[OwnedSegment::Field("parent".into()),OwnedSegment::Index(2),OwnedSegment::Field("dynamic child".into())]);
    drop(parent);assert_eq!(LIVE.load(Relaxed),baseline);
    std::panic::set_hook(hook);
}
"#;
    let runtime = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stdlib/schema/runtime");
    let source = format!("mod stdlib {{ pub(crate) mod schema {{ #[path={:?}] mod issues; #[path={:?}] mod union; #[path={:?}] mod trial; #[cfg(not(test))] pub(crate) mod allocation_witness {{ {allocator}\n{harness} }} }} }}\n#[cfg(not(test))] fn main() {{ stdlib::schema::allocation_witness::run(); }}",runtime.join("issues.rs"),runtime.join("union.rs"),runtime.join("trial.rs"));
    let directory = support::TempDirectory::new();
    let input = directory.0.join("schema-sequence.rs");
    std::fs::write(&input, source).unwrap();
    for level in ["0", "3"] {
        for unit_tests in [false, true] {
            let binary = directory.0.join(format!("schema-sequence-{level}-{unit_tests}"));
            let mut compiler = support::rustc();
            compiler.args(["--edition=2024", "-A", "warnings", "-C", &format!("opt-level={level}"), "-C", "overflow-checks=on"]);
            if unit_tests { compiler.args(["--test", "-F", "unsafe_code"]); }
            support::success(&support::run(compiler.arg(&input).arg("-o").arg(&binary), 60));
            let mut command = std::process::Command::new(binary);
            if unit_tests { command.arg("--test-threads=3"); }
            support::success(&support::run(&mut command, 30));
        }
    }
}
