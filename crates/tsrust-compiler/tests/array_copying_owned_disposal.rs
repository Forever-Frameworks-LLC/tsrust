mod support;
use tsrust_compiler::check;

#[test]
fn owned_live_prefixes_release_bytes_on_normal_and_unwind_paths_without_cloning_overwritten_values()
{
    let source = r#"
        type Small={name:string;};
        type Node={name:string;children:Node[];};
        function reverseWords(values:string[]):string[]{return values.toReversed();}
        function setWords(values:string[],index:int,value:string):string[]{return values.with(index,value);}
        function reverseRows(values:Small[]):Small[]{return values.toReversed();}
        function setRows(values:Small[],index:int,value:Small):Small[]{return values.with(index,value);}
        function reverseTrees(values:Node[]):Node[]{return values.toReversed();}
        function setTrees(values:Node[],index:int,value:Node):Node[]{return values.with(index,value);}
        function main(){}
    "#;
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("owned.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()") + ALLOCATOR + WITNESS;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("copying-owned.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("copying-owned-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(&executable), 10);
        support::success(&output);
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

// Test-only System allocator forwarding. No allocator or unsafe code enters
// compiler/runtime output. Bytes include backing vectors, strings, panic payloads
// and recursive clone/drop worklists; baseline excludes process initialization.
const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE_BYTES:AtomicIsize=AtomicIsize::new(0);
static SELECTED_ALLOCS:AtomicUsize=AtomicUsize::new(0);
const SELECTED_SIZE:usize=4093;
struct Observer;
unsafe impl GlobalAlloc for Observer {
    unsafe fn alloc(&self,layout:Layout)->*mut u8 {
        let pointer=unsafe{System.alloc(layout)};
        if !pointer.is_null(){LIVE_BYTES.fetch_add(layout.size() as isize,Ordering::SeqCst);if layout.size()==SELECTED_SIZE{SELECTED_ALLOCS.fetch_add(1,Ordering::SeqCst);}}
        pointer
    }
    unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){LIVE_BYTES.fetch_sub(layout.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(pointer,layout)}}
    unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8 {
        let result=unsafe{System.realloc(pointer,layout,size)};
        if !result.is_null(){LIVE_BYTES.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);if size==SELECTED_SIZE{SELECTED_ALLOCS.fetch_add(1,Ordering::SeqCst);}}
        result
    }
}
#[global_allocator]static GLOBAL:Observer=Observer;
"#;

const WITNESS: &str = r#"
fn owned(size:usize)->String{String::Owned("x".repeat(size))}
fn small(size:usize)->Record0{Record0{field0:owned(size)}}
fn node(size:usize)->Record1{Record1{field0:owned(size),field1:RecordArray1::from([Record1{field0:owned(79),field1:RecordArray1::from([])}])}}
fn live()->isize{LIVE_BYTES.load(Ordering::SeqCst)}
fn main(){
    source_main();let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
    // Warm panic bookkeeping, then require every owner-created byte to be freed.
    assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=live();
    for len in [0,1,4,5] { for spare in [false,true] {
        // Direct construction includes a full inline prefix. A second pass
        // adds/removes one owner, checking stale slots and spare heap capacity.
        let mut words=Array::from([]);let mut rows=Array::from([]);let mut trees=RecordArray1::from([]);
        for index in 0..len {
            let size=if index+1==len{SELECTED_SIZE}else{97};
            words.push(owned(size));rows.push(small(size));trees.push(node(size));
        }
        if spare { words.push(owned(97));rows.push(small(97));trees.push(node(97));
            drop(words.remove(len));drop(rows.remove(len));drop(trees.remove(len)); }
        let inputs_live=live();
        let reversed=function0(&words);let reversed_rows=function2(&rows);let reversed_trees=function4(&trees);
        assert_eq!(reversed.len(),len);assert_eq!(reversed_rows.len(),len);assert_eq!(reversed_trees.len(),len);
        for index in 0..len {
            let from=len-1-index;
            assert_eq!(reversed.get(index).unwrap(),words.get(from).unwrap());
            assert_ne!(reversed.get(index).unwrap().as_ptr(),words.get(from).unwrap().as_ptr());
            assert_eq!(reversed_rows.get(index).unwrap().field0,rows.get(from).unwrap().field0);
            assert_ne!(reversed_rows.get(index).unwrap().field0.as_ptr(),rows.get(from).unwrap().field0.as_ptr());
            assert_eq!(reversed_trees[index].field0,trees[from].field0);
            assert_ne!(reversed_trees[index].field0.as_ptr(),trees[from].field0.as_ptr());
            assert_ne!(reversed_trees[index].field1[0].field0.as_ptr(),trees[from].field1[0].field0.as_ptr());
        }
        drop(reversed);drop(reversed_rows);drop(reversed_trees);assert_eq!(live()-inputs_live,0);
        if len>0 {
            let word=owned(131);let row=small(131);let tree=node(131);
            let word_pointer=word.as_ptr();let row_pointer=row.field0.as_ptr();let tree_pointer=tree.field0.as_ptr();
            let source_pointer=words.get(len-1).unwrap().as_ptr();
            SELECTED_ALLOCS.store(0,Ordering::SeqCst);
            let mut copied=function1(&words,-1,word);let mut copied_rows=function3(&rows,-1,row);let mut copied_trees=function5(&trees,-1,tree);
            // Allocation size identifies the overwritten source's deep payload.
            // Any redundant clone is observed even if the clone is then dropped.
            assert_eq!(SELECTED_ALLOCS.load(Ordering::SeqCst),0,"overwritten owner was cloned");
            assert_eq!(copied.len(),len);assert_eq!(copied_rows.len(),len);assert_eq!(copied_trees.len(),len);
            assert_eq!(copied.get(len-1).unwrap().as_ptr(),word_pointer);
            assert_eq!(copied_rows.get(len-1).unwrap().field0.as_ptr(),row_pointer);
            assert_eq!(copied_trees[len-1].field0.as_ptr(),tree_pointer);
            assert_eq!(words.get(len-1).unwrap().as_ptr(),source_pointer);
            for index in 0..len-1 {
                assert_eq!(copied.get(index).unwrap(),words.get(index).unwrap());
                assert_ne!(copied.get(index).unwrap().as_ptr(),words.get(index).unwrap().as_ptr());
                assert_ne!(copied_rows.get(index).unwrap().field0.as_ptr(),rows.get(index).unwrap().field0.as_ptr());
                assert_ne!(copied_trees[index].field1[0].field0.as_ptr(),trees[index].field1[0].field0.as_ptr());
            }
            *copied.get_mut(len-1).unwrap()=String::Borrowed("changed");
            copied_rows.get_mut(len-1).unwrap().field0=String::Borrowed("changed");
            copied_trees[len-1].field1[0].field0=String::Borrowed("changed");
            assert_eq!(trees[len-1].field1[0].field0.len(),79);
            drop(copied);drop(copied_rows);drop(copied_trees);assert_eq!(live()-inputs_live,0);
        }
        for index in [i64::MIN,i64::MAX,len as i64] {
            assert!(std::panic::catch_unwind(||function1(&words,index,owned(131))).is_err());
            assert!(std::panic::catch_unwind(||function3(&rows,index,small(131))).is_err());
            assert!(std::panic::catch_unwind(||function5(&trees,index,node(131))).is_err());
            assert_eq!(live()-inputs_live,0,"replacement leaked during bounds unwind");
        }
        assert!(std::panic::catch_unwind(||{
            let copied=function0(&words);let copied_rows=function2(&rows);let copied_trees=function4(&trees);
            std::hint::black_box((&copied,&copied_rows,&copied_trees));panic!("dispose reversed results");
        }).is_err());assert_eq!(live()-inputs_live,0);
        if len>0 {
            assert!(std::panic::catch_unwind(||{
                let copied=function1(&words,0,owned(131));let copied_rows=function3(&rows,0,small(131));let copied_trees=function5(&trees,0,node(131));
                std::hint::black_box((&copied,&copied_rows,&copied_trees));panic!("dispose replaced results");
            }).is_err());assert_eq!(live()-inputs_live,0);
        }
        assert!(std::panic::catch_unwind(||{
            // These source owners are local to the unwinding scope too.
            let local_words=words.clone();let local_rows=rows.clone();let local_trees=trees.clone();
            let reverse_words=function0(&local_words);let reverse_rows=function2(&local_rows);let reverse_trees=function4(&local_trees);
            std::hint::black_box((&reverse_words,&reverse_rows,&reverse_trees));
            if len>0 {
                let set_words=function1(&local_words,0,owned(131));let set_rows=function3(&local_rows,0,small(131));let set_trees=function5(&local_trees,0,node(131));
                std::hint::black_box((&set_words,&set_rows,&set_trees));panic!("dispose local sources and copied results");
            }
            panic!("dispose empty local sources and copied results");
        }).is_err());assert_eq!(live()-inputs_live,0);
        drop(words);drop(rows);drop(trees);assert_eq!(live()-baseline,0,"owned live bytes after disposal");
    }}
    std::panic::set_hook(hook);
}
"#;
