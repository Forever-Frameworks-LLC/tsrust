//! B03 admission controls, not production APIs or canonical workload measurements.
//! Build with the pinned rustc; all platform/environment mutation is child-only.
//! Proposed public boundary (not implemented here): parse returns five required
//! owned strings; format accepts optional string fields, dir precedes root and
//! base precedes name/ext, with a dot supplied for nonempty undotted ext.
//! resolve returns Result<string, PathError> for Io/NotUnicode cwd observations;
//! old relative keeps its existing checked error contract. No canonicalization.
//! vars returns an unsorted, strict-Unicode owned name/value snapshot. Directory
//! observations are strict Unicode; only home absence becomes Ok(None).
//! availableParallelism and hostname return checked OS errors without fallbacks.
//! An optional `--cfg hostname_admission --extern hostname=<approved rlib>`
//! includes the exact pinned crate control after dependency admission, not before.
use std::{hint::black_box,path::Path,sync::atomic::{AtomicUsize,Ordering},time::Instant};
#[cfg(allocation_admission)]
use std::alloc::{GlobalAlloc,Layout,System};
#[cfg(allocation_admission)]
struct Meter;
static ALLOCS:AtomicUsize=AtomicUsize::new(0);
static REALLOCS:AtomicUsize=AtomicUsize::new(0);
static LIVE:AtomicUsize=AtomicUsize::new(0);
static PEAK:AtomicUsize=AtomicUsize::new(0);
#[cfg(allocation_admission)]
fn allocated(size:usize){let live=LIVE.fetch_add(size,Ordering::Relaxed)+size;PEAK.fetch_max(live,Ordering::Relaxed);}
// Instrumentation only: delegate each allocation to the standard system allocator.
// Timing binaries omit this allocator entirely.
#[cfg(allocation_admission)]
unsafe impl GlobalAlloc for Meter {
 unsafe fn alloc(&self,layout:Layout)->*mut u8 {let value=unsafe{System.alloc(layout)};if !value.is_null(){ALLOCS.fetch_add(1,Ordering::Relaxed);allocated(layout.size());}value}
 unsafe fn dealloc(&self,value:*mut u8,layout:Layout){unsafe{System.dealloc(value,layout)};LIVE.fetch_sub(layout.size(),Ordering::Relaxed);}
 unsafe fn realloc(&self,value:*mut u8,layout:Layout,size:usize)->*mut u8 {let result=unsafe{System.realloc(value,layout,size)};if !result.is_null(){REALLOCS.fetch_add(1,Ordering::Relaxed);if size>=layout.size(){allocated(size-layout.size());}else{LIVE.fetch_sub(layout.size()-size,Ordering::Relaxed);}}result}
}
#[cfg(allocation_admission)]
#[global_allocator]static METER:Meter=Meter;
#[path="../../../../crates/tsrust-compiler/src/stdlib/path/runtime.rs"]
pub mod lexical;
pub mod existing_resolution {
 include!("../../../../crates/tsrust-compiler/src/stdlib/path/relative.rs");
 pub fn one(path:&str,windows:bool,cwd:&str,drive:&str)->Result<String,String>{resolve_path_checked(path,windows,&mut ||Ok(cwd.into()),&mut |_|Ok(if drive.is_empty(){None}else{Some(drive.into())}))}
 // A narrow prototype over existing lexical resolution, not a new path parser.
 // Windows multi-part drive selection remains a separate required control.
 pub fn posix(parts:&[&str],cwd:&mut impl FnMut()->Result<String,String>)->Result<String,String>{
  let first=parts.iter().rposition(|part|part.starts_with('/')).unwrap_or(0);
  let mut joined=String::new();
  for part in parts[first..].iter().filter(|part|!part.is_empty()) {
   if !joined.is_empty(){joined.push('/');}
   joined.push_str(part);
  }
  resolve_path_checked(&joined,false,cwd,&mut |_|Ok(None))
 }
 pub fn resolve_absolute_without_cwd(observations:&mut usize)->String{resolve_path_checked("/logs",false,&mut ||{*observations+=1;Err("cwd unavailable".into())},&mut |_|Ok(None)).unwrap()}
}
#[derive(Clone,Debug,PartialEq,Eq)]struct Parts {root:String,dir:String,base:String,ext:String,name:String}
const COMMON:&[&str]=&["/srv/releases/file.tar.gz","reports/é😊.txt","alpha/.gitignore","alpha/file."];
// A std baseline only for the disclosed shared POSIX subset. Windows, roots,
// trailing/repeated separators and dot components use the independent corpus.
fn standard_parts(input:&str)->Parts {
 let path=Path::new(input);let base=path.file_name().unwrap().to_str().unwrap();
 Parts{root:if path.is_absolute(){"/".into()}else{String::new()},dir:path.parent().unwrap().to_str().unwrap().into(),base:base.into(),ext:path.extension().map(|ext|format!(".{}",ext.to_str().unwrap())).unwrap_or_default(),name:path.file_stem().unwrap().to_str().unwrap().into()}
}
fn corpus()->Vec<Vec<&'static str>>{include_str!("corpus.tsv").lines().filter(|line|!line.starts_with('#')&&!line.is_empty()).map(|line|line.split('\t').collect()).collect()}
fn checked_vars()->Result<Vec<(String,String)>,&'static str>{std::env::vars_os().map(|(key,value)|Ok((key.into_string().map_err(|_|"NotUnicode")?,value.into_string().map_err(|_|"NotUnicode")?))).collect()}
fn unicode_path(path:std::path::PathBuf)->Result<String,&'static str>{path.into_os_string().into_string().map_err(|_|"NotUnicode")}
fn checked_directory(path:std::io::Result<std::path::PathBuf>)->Result<String,String>{unicode_path(path.map_err(|error|error.to_string())?).map_err(str::to_owned)}
fn optional_unicode_path(path:Option<std::path::PathBuf>)->Result<Option<String>,&'static str>{path.map(unicode_path).transpose()}
fn checked_home()->Result<Option<String>,&'static str>{optional_unicode_path(std::env::home_dir())}
fn checked_parallelism()->Result<i64,String>{let count=std::thread::available_parallelism().map_err(|error|error.to_string())?.get();i64::try_from(count).map_err(|_|"parallelism count exceeds int".into())}
#[cfg(hostname_admission)]
fn checked_hostname()->Result<String,String>{hostname::get().map_err(|error|error.to_string())?.into_string().map_err(|_|"NotUnicode".into())}
// Compare complete owned outputs before choosing the production format adapter.
// Inputs represent either omitted or explicit empty fields by empty strings.
fn format_intermediate(parts:&Parts,windows:bool)->String{
 let directory=if parts.dir.is_empty(){&parts.root}else{&parts.dir};
 let base=if parts.base.is_empty(){format!("{}{}{}",parts.name,if !parts.ext.is_empty()&&!parts.ext.starts_with('.') {"."}else{""},parts.ext)}else{parts.base.clone()};
 if directory.is_empty(){base}else if directory==&parts.root{format!("{directory}{base}")}else{format!("{directory}{}{base}",if windows{'\\'}else{'/'})}
}
fn format_output(parts:&Parts,windows:bool)->String{
 let directory=if parts.dir.is_empty(){&parts.root}else{&parts.dir};
 let mut output=directory.clone();
 if !directory.is_empty()&&directory!=&parts.root{output.push(if windows{'\\'}else{'/'});}
 if parts.base.is_empty(){output.push_str(&parts.name);if !parts.ext.is_empty()&&!parts.ext.starts_with('.') {output.push('.');}output.push_str(&parts.ext);}else{output.push_str(&parts.base);}
 output
}
fn checked_capacity(lengths:&[usize])->Option<usize>{lengths.iter().try_fold(0usize,|total,length|total.checked_add(*length))}
fn format_capacity(parts:&Parts)->Option<usize>{
 let directory=if parts.dir.is_empty(){&parts.root}else{&parts.dir};
 let base=if parts.base.is_empty(){checked_capacity(&[parts.name.len(),usize::from(!parts.ext.is_empty()&&!parts.ext.starts_with('.')),parts.ext.len()])?}else{parts.base.len()};
 checked_capacity(&[directory.len(),usize::from(!directory.is_empty()&&directory!=&parts.root),base])
}
fn format_exact(parts:&Parts,windows:bool)->String{
 let capacity=format_capacity(parts).expect("path format output length overflow");
 let directory=if parts.dir.is_empty(){&parts.root}else{&parts.dir};
 let mut output=String::with_capacity(capacity);
 output.push_str(directory);
 if !directory.is_empty()&&directory!=&parts.root{output.push(if windows{'\\'}else{'/'});}
 if parts.base.is_empty(){output.push_str(&parts.name);if !parts.ext.is_empty()&&!parts.ext.starts_with('.') {output.push('.');}output.push_str(&parts.ext);}else{output.push_str(&parts.base);}
 debug_assert_eq!(output.len(),capacity);
 output
}
fn format_parts(row:&[&str])->Parts{Parts{root:row[3].into(),dir:row[4].into(),base:row[5].into(),ext:row[6].into(),name:row[7].into()}}
fn check_child(){
 assert!(std::env::var_os("TSRUST_B03_MISSING").is_none());
 assert_eq!(checked_home().unwrap(),Some(if cfg!(windows){"C:\\tsrust-b03-home".into()}else{"/tsrust-b03-home".into()}));
 assert_eq!(std::env::var("TSRUST_B03_EMPTY").unwrap(),"");
 assert_eq!(std::env::var("TSRUST_B03_TEXT").unwrap(),"héllo 🦀");
 if std::env::var_os("TSRUST_B03_EXPECT_RAW").is_some(){assert_eq!(checked_vars(),Err("NotUnicode"));return;}
 let mut vars=checked_vars().unwrap();let value=vars.iter_mut().find(|(name,_)|name=="TSRUST_B03_TEXT").unwrap();value.1.clear();
 assert_eq!(std::env::var("TSRUST_B03_TEXT").unwrap(),"héllo 🦀");
 assert!(std::env::current_dir().unwrap().is_absolute());assert!(std::env::current_exe().unwrap().is_absolute());
 assert!(std::thread::available_parallelism().unwrap().get()>0);
 assert!(checked_parallelism().unwrap()>0);
}
fn checks(){
 let rows=corpus();
 for row in rows.iter().filter(|row|row[0]=="parse") {
  let windows=row[1]=="win32";
  assert_eq!(lexical::basename(row[2],None,windows),row[5],"basename {row:?}");
  assert_eq!(lexical::extname(row[2],windows),row[6],"extname {row:?}");
  if row[3].is_empty()&&row[4].is_empty(){assert_eq!(lexical::dirname(row[2],windows),".");}else{assert_eq!(lexical::dirname(row[2],windows),row[4],"dirname {row:?}");}
 }
 if cfg!(unix){for input in COMMON{let row=rows.iter().find(|row|row[0]=="parse"&&row[1]=="posix"&&row[2]==*input).unwrap();let actual=standard_parts(input);assert_eq!([actual.root.as_str(),&actual.dir,&actual.base,&actual.ext,&actual.name],row[3..8]);}}
 for row in rows.iter().filter(|row|row[0]=="resolve"&&row.len()==6){assert_eq!(existing_resolution::one(row[5],row[1]=="win32",row[2],row[3]).unwrap(),row[4]);}
 for row in rows.iter().filter(|row|row[0]=="resolve"&&row[1]=="posix"){assert_eq!(existing_resolution::posix(&row[5..],&mut ||Ok(row[2].into())).unwrap(),row[4],"resolve {row:?}");}
 assert_eq!(existing_resolution::posix(&["relative"],&mut ||Err("Io".into())),Err("Io".into()));
 assert_eq!(existing_resolution::posix(&["relative"],&mut ||Err("NotUnicode".into())),Err("NotUnicode".into()));
 assert_eq!(existing_resolution::posix(&["/absolute"],&mut ||Err("Io".into())),Ok("/absolute".into()));
 for row in rows.iter().filter(|row|row[0]=="format"){let parts=format_parts(row);assert_eq!(format_output(&parts,row[1]=="win32"),row[2],"format {row:?}");assert_eq!(format_intermediate(&parts,row[1]=="win32"),row[2]);assert_eq!(format_exact(&parts,row[1]=="win32"),row[2]);assert_eq!(format_capacity(&parts),Some(row[2].len()));}
 assert_eq!(checked_capacity(&[usize::MAX,1]),None);assert_eq!(checked_capacity(&[usize::MAX,0]),Some(usize::MAX));
 if cfg!(unix){for input in COMMON{let parts=standard_parts(input);assert_eq!(format_exact(&parts,false),*input);assert_eq!(format_capacity(&parts),Some(input.len()));}}
 let mut observations=0;let absolute=existing_resolution::resolve_absolute_without_cwd(&mut observations);assert_eq!(absolute,"/logs");assert_eq!(observations,0);
 assert_eq!(optional_unicode_path(None),Ok(None));
 let executable=std::env::current_exe().unwrap();
 let child=||{let mut command=std::process::Command::new(&executable);command.arg("--child").env_clear().env("TSRUST_B03_EMPTY","").env("TSRUST_B03_TEXT","héllo 🦀");if cfg!(unix){command.env("HOME","/tsrust-b03-home");}else{command.env("USERPROFILE","C:\\tsrust-b03-home");}command};
 assert!(child().status().unwrap().success());
 #[cfg(unix)]{
  use std::os::unix::ffi::OsStringExt;
  let raw=std::ffi::OsString::from_vec(vec![255]);
  assert!(child().env("TSRUST_B03_EXPECT_RAW","1").env("TSRUST_B03_RAW",&raw).status().unwrap().success());
  assert!(child().env("TSRUST_B03_EXPECT_RAW","1").env(raw.clone(),"value").status().unwrap().success());
  assert_eq!(optional_unicode_path(Some(std::path::PathBuf::from(raw))),Err("NotUnicode"));
 }
 #[cfg(hostname_admission)]assert!(checked_hostname().is_ok(),"hostname control needs a successful Unicode OS observation");
}
struct Sample {nanos:u128,allocations:usize,reallocations:usize,retained:usize,peak:usize}
fn sample<T>(operation:impl FnOnce()->T)->Sample {
 let (allocations,reallocations,live)=(ALLOCS.load(Ordering::Relaxed),REALLOCS.load(Ordering::Relaxed),LIVE.load(Ordering::Relaxed));
 PEAK.store(live,Ordering::Relaxed);
 let start=Instant::now();let output=operation();let nanos=start.elapsed().as_nanos();black_box(&output);
 let result=Sample{nanos,allocations:ALLOCS.load(Ordering::Relaxed)-allocations,reallocations:REALLOCS.load(Ordering::Relaxed)-reallocations,retained:LIVE.load(Ordering::Relaxed)-live,peak:PEAK.load(Ordering::Relaxed)-live};
 drop(output);assert_eq!(LIVE.load(Ordering::Relaxed),live,"live-after-drop delta");result
}
fn profile<T>(name:&str,n:usize,mut operation:impl FnMut()->T){
 for _ in 0..8{black_box(operation());}
 for repetition in 0..3{
  let mut samples=(0..31).map(|_|sample(&mut operation)).collect::<Vec<_>>();
  if cfg!(allocation_admission){
   let median=|select:fn(&Sample)->usize|{let mut values=samples.iter().map(select).collect::<Vec<_>>();values.sort_unstable();values[15]};
   // Instrumented elapsed times are deliberately not reported as performance.
   println!("allocations\t{name}\t{n}\t{repetition}\t{}\t{}\t{}\t{}",median(|sample|sample.allocations),median(|sample|sample.reallocations),median(|sample|sample.retained),median(|sample|sample.peak));
  }else{
   samples.sort_by_key(|sample|sample.nanos);println!("timing\t{name}\t{n}\t{repetition}\t{}",samples[15].nanos);
  }
 }
}
fn main(){
 let args=std::env::args().skip(1).collect::<Vec<_>>();
 if args.first().is_some_and(|arg|arg=="--child"){check_child();return;}
 checks();if !args.first().is_some_and(|arg|matches!(arg.as_str(),"--profile"|"--profile-hostname"|"--profile-format")){println!("B03 shared reference controls passed; Windows multi-part resolve remains a separate admission control");return;}
 assert_eq!(std::env::var("TSRUST_B03_PROFILE_FIXTURES").as_deref(),Ok("1"),"profile with the sanitized driver environment");
 let n=args.get(1).map(|value|value.parse::<usize>().expect("iterations must be usize")).unwrap_or(256);
 assert!(n>0,"iterations must be positive");
 if args[0]=="--profile-hostname" {
  #[cfg(hostname_admission)]{
   profile("hostname-unicode-observation",n,||{(0..n).map(|_|checked_hostname()).collect::<Vec<_>>()});
   return;
  }
  #[cfg(not(hostname_admission))]panic!("hostname dependency was not enabled for this control");
 }
 let rows=corpus();
 let parts=rows.iter().filter(|row|row[0]=="format").map(|row|(format_parts(row),row[1]=="win32")).collect::<Vec<_>>();
 if args[0]=="--profile-format" {
  profile("prototype-format-output",n,||{(0..n).map(|i|{let (parts,windows)=&parts[i%parts.len()];format_output(black_box(parts),*windows)}).collect::<Vec<_>>()});
  profile("prototype-format-exact-capacity",n,||{(0..n).map(|i|{let (parts,windows)=&parts[i%parts.len()];format_exact(black_box(parts),*windows)}).collect::<Vec<_>>()});
  return;
 }
 let resolved=rows.iter().filter(|row|row[0]=="resolve"&&row[1]=="posix").collect::<Vec<_>>();
 if cfg!(unix){profile("std-owned-path-parts",n,||{(0..n).map(|i|standard_parts(black_box(COMMON[i%COMMON.len()]))).collect::<Vec<_>>()});}
 profile("std-unicode-vars-snapshot",n,||{(0..n).map(|_|checked_vars().unwrap()).collect::<Vec<_>>()});
 profile("prototype-format-intermediate",n,||{(0..n).map(|i|{let (parts,windows)=&parts[i%parts.len()];format_intermediate(black_box(parts),*windows)}).collect::<Vec<_>>()});
 profile("prototype-format-output",n,||{(0..n).map(|i|{let (parts,windows)=&parts[i%parts.len()];format_output(black_box(parts),*windows)}).collect::<Vec<_>>()});
 profile("existing-posix-resolve-fixed-context",n,||{(0..n).map(|i|{let row=resolved[i%resolved.len()];existing_resolution::posix(black_box(&row[5..]),&mut ||Ok(row[2].into())).unwrap()}).collect::<Vec<_>>()});
 profile("std-read-unicode-env-value",n,||{(0..n).map(|_|std::env::var(black_box("PATH"))).collect::<Vec<_>>()});
 profile("std-unicode-current-dir",n,||{(0..n).map(|_|checked_directory(std::env::current_dir())).collect::<Vec<_>>()});
 profile("std-unicode-current-exe",n,||{(0..n).map(|_|checked_directory(std::env::current_exe())).collect::<Vec<_>>()});
 profile("std-unicode-temp-dir",n,||{(0..n).map(|_|unicode_path(std::env::temp_dir())).collect::<Vec<_>>()});
 profile("std-unicode-home-dir",n,||{(0..n).map(|_|checked_home()).collect::<Vec<_>>()});
 // This estimates observation cost; do not cache or fabricate a speedup for CPU identity.
 profile("std-available-parallelism",n,||{(0..n).map(|_|checked_parallelism()).collect::<Vec<_>>()});
}
