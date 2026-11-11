// Synchronous candidate invocation provenance. Erased callbacks may invoke an
// assertion: save/restore one TLS pointer, without allocation or a carrier ABI.
struct TsrAssertionSite { source:&'static str,start:usize,end:usize,line:usize,column:usize }
std::thread_local! {static TSR_ASSERTION_SITE:std::cell::Cell<Option<&'static TsrAssertionSite>>=const{std::cell::Cell::new(None)};}
struct TsrAssertionGuard(Option<&'static TsrAssertionSite>);
impl TsrAssertionGuard {fn new(site:&'static TsrAssertionSite)->Self{Self(TSR_ASSERTION_SITE.with(|slot|slot.replace(Some(site))))}}
impl Drop for TsrAssertionGuard {fn drop(&mut self){TSR_ASSERTION_SITE.with(|slot|slot.set(self.0));}}
// Report sources relative to the working directory when they are inside it.
fn tsrust_testing_site_path(source:&'static str)->std::borrow::Cow<'static,str>{
    let path=std::path::Path::new(source);
    std::env::current_dir().ok().into_iter().flat_map(|directory|[directory.clone(),directory.canonicalize().unwrap_or(directory)])
        .find_map(|directory|path.strip_prefix(&directory).ok().map(|relative|std::borrow::Cow::Owned(relative.display().to_string())))
        .unwrap_or(std::borrow::Cow::Borrowed(source))
}
fn tsrust_testing_assertion_site()->Option<&'static TsrAssertionSite>{TSR_ASSERTION_SITE.with(std::cell::Cell::get)}
fn tsrust_testing_check_assertion(condition: bool, message: &String) {
    if !condition {
        if let Some(site)=tsrust_testing_assertion_site(){panic!("AssertionError: {} at {}:{}:{}",message.as_ref(),tsrust_testing_site_path(site.source),site.line,site.column);}
        panic!("AssertionError: {}", message.as_ref());
    }
}
fn tsrust_testing_begin_output_capture(){tsr_console_output_begin();}
fn tsrust_testing_end_output_capture()->String{String::Owned(tsr_console_output_end())}
