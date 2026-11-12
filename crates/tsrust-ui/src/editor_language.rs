//! Window-owned pending language reads. A dropped native task cancels its read.
#![cfg_attr(not(feature="gpui"), allow(dead_code))]
use std::{cell::{Cell,RefCell}, collections::BTreeMap, future::Future, pin::Pin, rc::{Rc, Weak}, task::{Context, Poll, Waker}, time::{Duration, Instant}};
use serde_json::Value;

thread_local! {static NEXT_REQUEST: Cell<u64> = const {Cell::new(0)};}

#[derive(Default)]
pub(crate) struct Requests {
    pending: BTreeMap<u64, Pending>,
    cancelled: Vec<(u64, u64)>,
}
struct Pending {
    target: u64,
    method: String,
    text: String,
    deadline: Instant,
    slot: Rc<RefCell<Slot>>,
}
#[derive(Default)]
struct Slot { result: Option<Value>, waker: Option<Waker> }
impl Slot {
    fn finish(&mut self, result: Value) {
        self.result = Some(result);
        if let Some(waker) = self.waker.take() { waker.wake(); }
    }
}
pub(crate) struct Reply {
    id: u64,
    requests: Weak<RefCell<Requests>>,
    slot: Rc<RefCell<Slot>>,
    pub(crate) on_cancel: Option<Box<dyn Fn()>>,
}
impl Future for Reply {
    type Output = Value;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Value> {
        let mut slot = self.slot.borrow_mut();
        match slot.result.take() {
            Some(value) => Poll::Ready(value),
            None => {slot.waker = Some(cx.waker().clone()); Poll::Pending}
        }
    }
}
impl Drop for Reply {
    fn drop(&mut self) {
        if let Some(requests) = self.requests.upgrade() {
            let pending = requests.borrow().pending.contains_key(&self.id);
            requests.borrow_mut().cancel(self.id);
            if pending { if let Some(notify) = &self.on_cancel {notify();} }
        }
    }
}
impl Requests {
    pub(crate) fn begin(requests: &Rc<RefCell<Self>>, target: u64, method: &str, text: String) -> (u64, Reply) {
        let mut state = requests.borrow_mut();
        let previous = state.pending.iter().filter(|(_, p)|p.target == target && p.method == method).map(|(&id,_)|id).collect::<Vec<_>>();
        for id in previous {state.cancel(id);}
        let id=NEXT_REQUEST.with(|next| {let id=next.get().checked_add(1).filter(|id|*id<=i64::MAX as u64).expect("language request identities exhausted");next.set(id);id});
        let slot = Rc::new(RefCell::new(Slot::default()));
        state.pending.insert(id, Pending {target, method:method.into(), text, deadline:Instant::now()+Duration::from_secs(5), slot:slot.clone()});
        (id, Reply {id, requests:Rc::downgrade(requests), slot, on_cancel:None})
    }
    fn cancel(&mut self, id: u64) {
        if let Some(pending) = self.pending.remove(&id) {
            pending.slot.borrow_mut().finish(Value::Null);
            self.cancelled.push((pending.target, id));
        }
    }
    pub(crate) fn target(&self, id: u64) -> Option<u64> {self.pending.get(&id).map(|p|p.target)}
    pub(crate) fn resolve(&mut self, id: u64, text: Option<&str>, mut result: Value) -> bool {
        let Some(pending) = self.pending.remove(&id) else {return false};
        let current = text == Some(pending.text.as_str()) && native_ranges(&pending.method, &mut result, &pending.text);
        pending.slot.borrow_mut().finish(if current {result} else {Value::Null});
        current
    }
    pub(crate) fn cancellations(&mut self, mounted: impl Fn(u64)->bool) -> Vec<(u64,u64)> {
        let expired = self.pending.iter().filter(|(_,p)|p.deadline <= Instant::now() || !mounted(p.target)).map(|(&id,_)|id).collect::<Vec<_>>();
        for id in expired {self.cancel(id);}
        std::mem::take(&mut self.cancelled)
    }
}

/// GPUI positions count scalars; the server uses UTF-16 code units.
pub(crate) fn position(text: &str, offset: usize) -> Option<Value> {
    let prefix=text.get(..offset)?;
    Some(serde_json::json!({"line":prefix.bytes().filter(|&b|b==b'\n').count(),"character":prefix.rsplit('\n').next()?.encode_utf16().count()}))
}
fn native_ranges(method: &str, value: &mut Value, text: &str) -> bool {
    let lines=text.split('\n').collect::<Vec<_>>();
    fn range(value: &mut Value, lines: &[&str]) -> bool {
        for end in ["start","end"] {
            let Some(position)=value.get_mut(end) else{return false};
            let Some(line)=position.get("line").and_then(Value::as_u64).and_then(|row|lines.get(row as usize).copied()) else{return false};
            let Some(units)=position.get("character").and_then(Value::as_u64) else{return false};
            let (mut utf16,mut scalars)=(0u64,0u64);
            for ch in line.chars() {if utf16>=units {break;} utf16+=ch.len_utf16() as u64;scalars+=1;}
            if utf16!=units {return false;}
            position["character"]=scalars.into();
        }
        true
    }
    fn item(value: &mut Value, lines: &[&str]) -> bool {
        if let Some(edit)=value.get_mut("textEdit") {
            for field in ["range","insert","replace"] {if let Some(value)=edit.get_mut(field) {if !range(value,lines){return false;}}}
        }
        if let Some(edits)=value.get_mut("additionalTextEdits").and_then(Value::as_array_mut) {
            for edit in edits {if !edit.get_mut("range").is_some_and(|value|range(value,lines)){return false;}}
        }
        true
    }
    if method=="textDocument/hover" {return value.get_mut("range").is_none_or(|value|range(value,&lines));}
    if method=="textDocument/completion" {
        if let Some(items)=value.as_array_mut(){return items.iter_mut().all(|value|item(value,&lines));}
        if let Some(items)=value.get_mut("items").and_then(Value::as_array_mut){return items.iter_mut().all(|value|item(value,&lines));}
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::Wake;
    struct Noop;
    impl Wake for Noop {fn wake(self:std::sync::Arc<Self>) {}}
    fn poll(reply: &mut Reply) -> Poll<Value> {
        let waker = Waker::from(std::sync::Arc::new(Noop));
        Pin::new(reply).poll(&mut Context::from_waker(&waker))
    }
    #[test]
    fn wire_positions_use_utf16_and_native_ranges_preserve_opaque_completion_data() {
        let text="/* 😀 */row.value";
        assert_eq!(position(text,"/* 😀 */row.".len()).unwrap()["character"],12);
        let mut result=serde_json::json!({"items":[{"label":"value","textEdit":{"range":{"start":{"line":0,"character":12},"end":{"line":0,"character":17}},"newText":"value"},"data":{"line":0,"character":12}}]});
        assert!(native_ranges("textDocument/completion",&mut result,text));
        assert_eq!(result["items"][0]["textEdit"]["range"]["start"]["character"],11);
        assert_eq!(result["items"][0]["data"]["character"],12);
        let mut hover=serde_json::json!({"range":{"start":{"line":0,"character":4},"end":{"line":0,"character":5}}});
        assert!(!native_ranges("textDocument/hover",&mut hover,text),"surrogate interior must be rejected");
    }
    #[test]
    fn replies_require_current_text_and_cancel_on_replace_drop_timeout_and_unmount() {
        let requests = Rc::new(RefCell::new(Requests::default()));
        let (old, mut first) = Requests::begin(&requests, 1, "hover", "😀first".into());
        assert!(poll(&mut first).is_pending());
        let (id, mut second) = Requests::begin(&requests, 1, "hover", "😀second".into());
        let other_window=Rc::new(RefCell::new(Requests::default()));
        let (other_id,_other_reply)=Requests::begin(&other_window,1,"hover","😀second".into());
        assert_ne!(id,other_id);
        assert!(!other_window.borrow_mut().resolve(id,Some("😀second"),Value::Bool(true)));
        assert_eq!(poll(&mut first), Poll::Ready(Value::Null));
        assert_eq!(requests.borrow_mut().cancellations(|_|true), vec![(1,old)]);
        assert!(!requests.borrow_mut().resolve(old, Some("😀first"), Value::Bool(true)));
        assert!(!requests.borrow_mut().resolve(id, Some("changed"), Value::Bool(true)));
        assert_eq!(poll(&mut second), Poll::Ready(Value::Null));
        let (id, mut reply) = Requests::begin(&requests, 1, "completion", "current".into());
        assert!(requests.borrow_mut().resolve(id, Some("current"), Value::Bool(true)));
        assert_eq!(poll(&mut reply), Poll::Ready(Value::Bool(true)));
        let (id, reply) = Requests::begin(&requests, 1, "hover", "current".into());
        drop(reply);
        assert_eq!(requests.borrow_mut().cancellations(|_|true), vec![(1,id)]);
        let (id, mut reply) = Requests::begin(&requests, 2, "hover", "current".into());
        requests.borrow_mut().pending.get_mut(&id).unwrap().deadline = Instant::now();
        assert_eq!(requests.borrow_mut().cancellations(|_|true), vec![(2,id)]);
        assert_eq!(poll(&mut reply), Poll::Ready(Value::Null));
        let (id, mut reply) = Requests::begin(&requests, 3, "hover", "current".into());
        assert_eq!(requests.borrow_mut().cancellations(|_|false), vec![(3,id)]);
        assert_eq!(poll(&mut reply), Poll::Ready(Value::Null));
        assert!(requests.borrow().pending.is_empty());
    }
}
