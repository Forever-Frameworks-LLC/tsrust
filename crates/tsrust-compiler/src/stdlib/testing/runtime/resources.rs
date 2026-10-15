// The audit retains only IDs. Its process lifetime is deliberate test instrumentation.
#[derive(Default)]
struct TsrResourceAudit { created: i64, releases: Vec<i64> }
static TSR_RESOURCE_AUDIT: std::sync::Mutex<TsrResourceAudit> = std::sync::Mutex::new(TsrResourceAudit { created: 0, releases: Vec::new() });
struct TsrTrackedResource {
    id: i64,
    storage: Option<Box<u8>>,
    affinity: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl Drop for TsrTrackedResource {
    fn drop(&mut self) {
        drop(self.storage.take());
        TSR_RESOURCE_AUDIT.lock().unwrap_or_else(|e| e.into_inner()).releases.push(self.id);
    }
}
fn tsr_resource_create() -> TsrTrackedResource {
    let mut audit = TSR_RESOURCE_AUDIT.lock().unwrap_or_else(|e| e.into_inner());
    audit.created = audit.created.checked_add(1).expect("Tracked resource identity exhausted");
    TsrTrackedResource { id: audit.created, storage: Some(Box::new(0)), affinity: std::marker::PhantomData }
}
fn tsr_resource_id(resource: &TsrTrackedResource) -> i64 { resource.id }
fn tsr_resource_close(resource: TsrTrackedResource) { drop(resource); }
fn tsr_resource_release_count(id: i64) -> i64 {
    TSR_RESOURCE_AUDIT.lock().unwrap_or_else(|e| e.into_inner()).releases.iter().filter(|&&value| value == id).count() as i64
}
fn tsr_resource_release_order(id: i64) -> i64 {
    TSR_RESOURCE_AUDIT.lock().unwrap_or_else(|e| e.into_inner()).releases.iter().position(|&value| value == id).map_or(0, |index| index as i64 + 1)
}
fn tsr_resource_live_count() -> i64 {
    let audit = TSR_RESOURCE_AUDIT.lock().unwrap_or_else(|e| e.into_inner());
    audit.created - audit.releases.len() as i64
}
