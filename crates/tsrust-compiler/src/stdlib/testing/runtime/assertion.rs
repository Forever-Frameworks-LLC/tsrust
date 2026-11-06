// Synchronous candidate invocation provenance. Erased callbacks may invoke an
// assertion: save/restore one TLS pointer, without allocation or a carrier ABI.
struct TsrAssertionSite { source:&'static str,start:usize,end:usize,line:usize,column:usize }
std::thread_local! {static TSR_ASSERTION_SITE:std::cell::Cell<Option<&'static TsrAssertionSite>>=const{std::cell::Cell::new(None)};}
struct TsrAssertionGuard(Option<&'static TsrAssertionSite>);
impl TsrAssertionGuard {fn new(site:&'static TsrAssertionSite)->Self{Self(TSR_ASSERTION_SITE.with(|slot|slot.replace(Some(site))))}}
impl Drop for TsrAssertionGuard {fn drop(&mut self){TSR_ASSERTION_SITE.with(|slot|slot.set(self.0));}}
fn tsrust_testing_assertion_site()->Option<&'static TsrAssertionSite>{TSR_ASSERTION_SITE.with(std::cell::Cell::get)}
fn tsrust_testing_check_assertion(condition: bool, message: &String) {
    if !condition {
        if let Some(site)=tsrust_testing_assertion_site(){panic!("AssertionError: {} at {}:{}:{}",message.as_ref(),site.source,site.line,site.column);}
        panic!("AssertionError: {}", message.as_ref());
    }
}
