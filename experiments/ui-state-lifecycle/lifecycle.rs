//! Experimental synchronous ownership model; not a renderer or public UI API.
#![forbid(unsafe_code)]
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::{Rc, Weak};

type Audit = Rc<RefCell<Vec<String>>>;
struct Tracked {
    name: String,
    audit: Audit,
}
impl Tracked {
    fn new(name: String, audit: &Audit) -> Self {
        audit.borrow_mut().push(format!("+{name}"));
        Self {
            name,
            audit: audit.clone(),
        }
    }
}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.audit.borrow_mut().push(format!("-{}", self.name));
    }
}
#[derive(Clone, Debug)]
struct Handle {
    owner: Weak<()>,
    id: u64,
}
#[derive(Debug, PartialEq, Eq)]
enum Error {
    Stale,
    InvalidKey,
    InvalidOrder,
    Exhausted,
    Reentrant,
}
type Result<T> = std::result::Result<T, Error>;
type Handler = Box<dyn FnMut(&mut i64, &mut Owner, Handle, i64)>;
struct Resources {
    value: i64,
    handler: Handler,
    _state: Tracked,
}
struct Node {
    key: String,
    parent: Option<u64>,
    children: Vec<u64>,
    order: Vec<u64>,
    live: bool,
    resources: Option<Resources>,
}
struct Event {
    target: Handle,
    delta: i64,
    _payload: Tracked,
}
struct Owner {
    identity: Rc<()>,
    next: u64,
    nodes: BTreeMap<u64, Node>,
    queue: VecDeque<Event>,
    retired: Vec<u64>,
    delivering: bool,
    audit: Audit,
}
impl Owner {
    fn new(audit: Audit) -> Self {
        let mut owner = Self {
            identity: Rc::new(()),
            next: 1,
            nodes: BTreeMap::new(),
            queue: VecDeque::new(),
            retired: vec![],
            delivering: false,
            audit,
        };
        let resources = owner.resources(0, 0, |_, _, _, _| {});
        owner.nodes.insert(
            0,
            Node {
                key: "root".into(),
                parent: None,
                children: vec![],
                order: vec![],
                live: true,
                resources: Some(resources),
            },
        );
        owner
    }
    fn handle(&self, id: u64) -> Handle {
        Handle {
            owner: Rc::downgrade(&self.identity),
            id,
        }
    }
    fn root(&self) -> Handle {
        self.handle(0)
    }
    fn validate(&self, handle: &Handle) -> Result<()> {
        if !Weak::ptr_eq(&handle.owner, &Rc::downgrade(&self.identity))
            || !self.nodes.get(&handle.id).is_some_and(|node| node.live)
        {
            return Err(Error::Stale);
        }
        Ok(())
    }
    fn resources(
        &self,
        id: u64,
        value: i64,
        mut callback: impl FnMut(&mut i64, &mut Owner, Handle, i64) + 'static,
    ) -> Resources {
        let capture = Tracked::new(format!("handler:{id}"), &self.audit);
        let handler = Box::new(
            move |state: &mut i64, owner: &mut Owner, handle: Handle, delta: i64| {
                let _keep_capture = &capture;
                callback(state, owner, handle, delta);
            },
        );
        Resources {
            value,
            handler,
            _state: Tracked::new(format!("state:{id}"), &self.audit),
        }
    }
    fn mount(
        &mut self,
        parent: &Handle,
        key: &str,
        value: i64,
        callback: impl FnMut(&mut i64, &mut Owner, Handle, i64) + 'static,
    ) -> Result<Handle> {
        self.validate(parent)?;
        if key.is_empty()
            || self.nodes[&parent.id]
                .children
                .iter()
                .any(|id| self.nodes[id].live && self.nodes[id].key == key)
        {
            return Err(Error::InvalidKey);
        }
        let id = self.next;
        let next = id.checked_add(1).ok_or(Error::Exhausted)?;
        let resources = self.resources(id, value, callback);
        self.nodes.insert(
            id,
            Node {
                key: key.into(),
                parent: Some(parent.id),
                children: vec![],
                order: vec![],
                live: true,
                resources: Some(resources),
            },
        );
        let parent = self.nodes.get_mut(&parent.id).unwrap();
        parent.children.push(id);
        parent.order.push(id);
        self.next = next;
        Ok(self.handle(id))
    }
    fn counter(&mut self, parent: &Handle, key: &str, value: i64) -> Result<Handle> {
        self.mount(parent, key, value, |state, _, _, delta| *state += delta)
    }
    fn read(&self, handle: &Handle) -> Result<i64> {
        self.validate(handle)?;
        // Active state is only accessible through the current callback's mutable argument.
        self.nodes[&handle.id]
            .resources
            .as_ref()
            .map(|r| r.value)
            .ok_or(Error::Reentrant)
    }
    fn values(&self, parent: &Handle) -> Result<Vec<(String, i64)>> {
        self.validate(parent)?;
        self.nodes[&parent.id]
            .order
            .iter()
            .map(|id| Ok((self.nodes[id].key.clone(), self.read(&self.handle(*id))?)))
            .collect()
    }
    fn reorder(&mut self, parent: &Handle, keys: &[&str]) -> Result<()> {
        self.validate(parent)?;
        let children = &self.nodes[&parent.id].order;
        if keys.len() != children.len() {
            return Err(Error::InvalidOrder);
        }
        let mut order = Vec::new();
        for key in keys {
            let id = children
                .iter()
                .find(|id| self.nodes[id].key == *key)
                .ok_or(Error::InvalidOrder)?;
            if order.contains(id) {
                return Err(Error::InvalidOrder);
            }
            order.push(*id);
        }
        self.nodes.get_mut(&parent.id).unwrap().order = order;
        Ok(())
    }
    fn enqueue(&mut self, target: &Handle, delta: i64) -> Result<()> {
        self.validate(target)?;
        self.queue.push_back(Event {
            target: target.clone(),
            delta,
            _payload: Tracked::new(format!("payload:{}", target.id), &self.audit),
        });
        Ok(())
    }
    fn retire(&mut self, id: u64) {
        if !self.nodes[&id].live {
            return;
        }
        self.nodes.get_mut(&id).unwrap().live = false;
        let children = self.nodes[&id].children.clone();
        for child in children {
            self.retire(child);
        }
        self.retired.push(id);
    }
    fn remove(&mut self, handle: &Handle) -> Result<()> {
        if !Weak::ptr_eq(&handle.owner, &Rc::downgrade(&self.identity)) {
            return Err(Error::Stale);
        }
        if !self.nodes.get(&handle.id).is_some_and(|node| node.live) {
            return Ok(());
        }
        self.retire(handle.id);
        if !self.delivering {
            self.cleanup();
        }
        Ok(())
    }
    fn cleanup(&mut self) {
        for id in std::mem::take(&mut self.retired) {
            if let Some(node) = self.nodes.remove(&id) {
                if let Some(parent) = node.parent.and_then(|id| self.nodes.get_mut(&id)) {
                    parent.children.retain(|child| *child != id);
                    parent.order.retain(|child| *child != id);
                }
                drop(node);
            }
        }
    }
    fn drain(&mut self) -> Result<Vec<Result<()>>> {
        if self.delivering {
            return Err(Error::Reentrant);
        }
        let mut results = vec![];
        while let Some(event) = self.queue.pop_front() {
            if let Err(error) = self.validate(&event.target) {
                results.push(Err(error));
                continue;
            }
            self.delivering = true;
            let mut resources = self
                .nodes
                .get_mut(&event.target.id)
                .unwrap()
                .resources
                .take()
                .unwrap();
            (resources.handler)(
                &mut resources.value,
                self,
                event.target.clone(),
                event.delta,
            );
            self.nodes.get_mut(&event.target.id).unwrap().resources = Some(resources);
            self.delivering = false;
            self.cleanup();
            results.push(Ok(()));
        }
        Ok(results)
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        if self.nodes.contains_key(&0) {
            self.retire(0);
        }
        self.cleanup();
    }
}
fn balance(audit: &Audit) -> i64 {
    audit
        .borrow()
        .iter()
        .map(|entry| if entry.starts_with('+') { 1 } else { -1 })
        .sum()
}
fn demo() -> String {
    let audit = Audit::default();
    let mut owner = Owner::new(audit.clone());
    let root = owner.root();
    let a = owner.counter(&root, "a", 0).unwrap();
    let b = owner.counter(&root, "b", 0).unwrap();
    owner.enqueue(&a, 1).unwrap();
    owner.enqueue(&a, 1).unwrap();
    owner.enqueue(&b, 1).unwrap();
    owner.drain().unwrap();
    let mut output = format!(
        "counters: a={}, b={}\n",
        owner.read(&a).unwrap(),
        owner.read(&b).unwrap()
    );
    let list = owner.counter(&root, "list", 0).unwrap();
    let _one = owner.counter(&list, "1", 10).unwrap();
    let two = owner.counter(&list, "2", 20).unwrap();
    let _four = owner.counter(&list, "4", 40).unwrap();
    owner.enqueue(&two, 3).unwrap();
    owner.drain().unwrap();
    owner.reorder(&list, &["4", "2", "1"]).unwrap();
    output += &format!("reordered: {:?}\n", owner.values(&list).unwrap());
    owner.enqueue(&two, 99).unwrap();
    owner.remove(&two).unwrap();
    let fresh = owner.counter(&list, "2", 0).unwrap();
    output += &format!(
        "removed event: {:?}; old handle: {:?}\n",
        owner.drain().unwrap(),
        owner.read(&two)
    );
    owner.enqueue(&fresh, 1).unwrap();
    owner.drain().unwrap();
    output += &format!("reinserted: {}\n", owner.read(&fresh).unwrap());
    owner.remove(&root).unwrap();
    output += &format!("tracked live resources: {}\n", balance(&audit));
    assert_eq!(balance(&audit), 0);
    output
}
fn main() {
    print!("{}", demo());
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Owner, Handle, Audit) {
        let audit = Audit::default();
        let owner = Owner::new(audit.clone());
        let root = owner.root();
        (owner, root, audit)
    }
    fn send(owner: &mut Owner, handle: &Handle, n: i64) {
        owner.enqueue(handle, n).unwrap();
        assert_eq!(owner.drain(), Ok(vec![Ok(())]));
    }
    fn drops(audit: &Audit) -> Vec<String> {
        audit
            .borrow()
            .iter()
            .filter(|s| s.starts_with("-state:"))
            .cloned()
            .collect()
    }
    #[test]
    fn l01_independent_counters() {
        let (mut o, r, _) = setup();
        let a = o.counter(&r, "a", 0).unwrap();
        let b = o.counter(&r, "b", 0).unwrap();
        send(&mut o, &a, 1);
        send(&mut o, &a, 1);
        send(&mut o, &b, 1);
        assert_eq!((o.read(&a), o.read(&b)), (Ok(2), Ok(1)));
    }
    #[test]
    fn l02_retained_capture_current_state() {
        fn construct(o: &mut Owner, r: &Handle) -> Handle {
            let step = 3;
            o.mount(r, "a", 2, move |v, _, _, delta| *v = *v * step + delta)
                .unwrap()
        }
        let (mut o, r, _) = setup();
        let a = construct(&mut o, &r);
        send(&mut o, &a, 1);
        assert_eq!(o.read(&a), Ok(7));
        send(&mut o, &a, 2);
        assert_eq!(o.read(&a), Ok(23));
    }
    #[test]
    fn l03_cross_owner_handles() {
        let (mut a, ar, _) = setup();
        let (mut b, br, _) = setup();
        let x = a.counter(&ar, "1", 5).unwrap();
        let y = b.counter(&br, "1", 8).unwrap();
        assert_eq!(b.enqueue(&x, 1), Err(Error::Stale));
        assert_eq!(b.read(&x), Err(Error::Stale));
        assert_eq!(b.remove(&x), Err(Error::Stale));
        assert_eq!(a.read(&y), Err(Error::Stale));
        assert_eq!(b.read(&y), Ok(8));
    }
    #[test]
    fn l04_reorder_preserves_state_and_mounts() {
        let (mut o, r, audit) = setup();
        let one = o.counter(&r, "1", 10).unwrap();
        let two = o.counter(&r, "2", 20).unwrap();
        o.counter(&r, "4", 40).unwrap();
        send(&mut o, &two, 3);
        let before = audit.borrow().clone();
        o.reorder(&r, &["4", "2", "1"]).unwrap();
        assert_eq!(
            o.values(&r).unwrap(),
            vec![("4".into(), 40), ("2".into(), 23), ("1".into(), 10)]
        );
        assert_eq!(*audit.borrow(), before);
        assert_eq!(o.read(&one), Ok(10));
    }
    #[test]
    fn l05_invalid_keys_and_order_are_atomic() {
        let (mut o, r, audit) = setup();
        o.counter(&r, "1", 10).unwrap();
        let before = audit.borrow().clone();
        assert_eq!(o.counter(&r, "1", 99).unwrap_err(), Error::InvalidKey);
        assert_eq!(o.counter(&r, "", 99).unwrap_err(), Error::InvalidKey);
        for keys in [vec![], vec!["missing"], vec!["1", "1"]] {
            assert_eq!(o.reorder(&r, &keys), Err(Error::InvalidOrder));
        }
        assert_eq!(o.values(&r).unwrap(), vec![("1".into(), 10)]);
        assert_eq!(*audit.borrow(), before);
    }
    #[test]
    fn l06_removed_pending_event_is_stale() {
        let (mut o, r, _) = setup();
        let a = o.counter(&r, "1", 10).unwrap();
        let b = o.counter(&r, "2", 20).unwrap();
        o.enqueue(&b, 5).unwrap();
        o.remove(&b).unwrap();
        assert_eq!(o.enqueue(&b, 1), Err(Error::Stale));
        assert_eq!(o.read(&b), Err(Error::Stale));
        assert_eq!(o.drain(), Ok(vec![Err(Error::Stale)]));
        assert_eq!(o.read(&a), Ok(10));
    }
    #[test]
    fn l07_reinsert_does_not_revive_handle() {
        let (mut o, r, _) = setup();
        let old = o.counter(&r, "2", 20).unwrap();
        o.enqueue(&old, 90).unwrap();
        o.remove(&old).unwrap();
        let fresh = o.counter(&r, "2", 0).unwrap();
        assert_ne!(old.id, fresh.id);
        assert_eq!(o.drain(), Ok(vec![Err(Error::Stale)]));
        assert_eq!(o.read(&old), Err(Error::Stale));
        send(&mut o, &fresh, 1);
        assert_eq!(o.read(&fresh), Ok(1));
    }
    #[test]
    fn l08_child_first_idempotent_disposal() {
        let (mut o, r, audit) = setup();
        let a = o.counter(&r, "a", 0).unwrap();
        let b = o.counter(&r, "b", 0).unwrap();
        let c = o.counter(&a, "c", 0).unwrap();
        o.reorder(&r, &["b", "a"]).unwrap();
        o.remove(&c).unwrap();
        o.remove(&c).unwrap();
        send(&mut o, &a, 1);
        o.remove(&r).unwrap();
        o.remove(&r).unwrap();
        assert_eq!(
            drops(&audit),
            vec![
                format!("-state:{}", c.id),
                format!("-state:{}", a.id),
                format!("-state:{}", b.id),
                "-state:0".into()
            ]
        );
        assert_eq!(balance(&audit), 0);
    }
    #[test]
    fn l09_fifo_and_completed_mutations() {
        let (mut o, r, _) = setup();
        let trace = Rc::new(RefCell::new(vec![]));
        let t = trace.clone();
        let a = o
            .mount(&r, "a", 0, move |v, o, h, d| {
                *v += d;
                t.borrow_mut().push(*v);
                if d == 1 {
                    o.enqueue(&h, 3).unwrap();
                }
            })
            .unwrap();
        o.enqueue(&a, 1).unwrap();
        o.enqueue(&a, 2).unwrap();
        assert_eq!(o.drain(), Ok(vec![Ok(()), Ok(()), Ok(())]));
        assert_eq!(*trace.borrow(), vec![1, 3, 6]);
    }
    #[test]
    fn l10_recursive_drain_rejected() {
        let (mut o, r, _) = setup();
        let a = o
            .mount(&r, "a", 0, |v, o, h, d| {
                *v += d;
                assert_eq!(o.read(&h), Err(Error::Reentrant));
                assert_eq!(o.drain(), Err(Error::Reentrant));
            })
            .unwrap();
        o.enqueue(&a, 1).unwrap();
        o.enqueue(&a, 2).unwrap();
        assert_eq!(o.drain(), Ok(vec![Ok(()), Ok(())]));
        assert_eq!(o.read(&a), Ok(3));
    }
    #[test]
    fn l11_self_and_sibling_removal() {
        let (mut o, r, audit) = setup();
        let sibling = o.counter(&r, "sibling", 0).unwrap();
        let s = sibling.clone();
        let a = o
            .mount(&r, "a", 0, move |v, o, h, _| {
                o.remove(&s).unwrap();
                o.remove(&h).unwrap();
                assert_eq!(o.read(&h), Err(Error::Stale));
                assert_eq!(o.enqueue(&h, 1), Err(Error::Stale));
                *v = 42;
            })
            .unwrap();
        o.enqueue(&a, 1).unwrap();
        o.enqueue(&a, 1).unwrap();
        o.enqueue(&sibling, 1).unwrap();
        assert_eq!(
            o.drain(),
            Ok(vec![Ok(()), Err(Error::Stale), Err(Error::Stale)])
        );
        assert_eq!(
            drops(&audit),
            vec![format!("-state:{}", sibling.id), format!("-state:{}", a.id)]
        );
    }
    #[test]
    fn l11_active_child_disposes_root_child_first() {
        let (mut o, r, audit) = setup();
        let root = r.clone();
        let a = o
            .mount(&r, "a", 0, move |v, o, _, _| {
                o.remove(&root).unwrap();
                *v += 1;
            })
            .unwrap();
        let b = o.counter(&r, "b", 0).unwrap();
        o.enqueue(&a, 1).unwrap();
        o.drain().unwrap();
        assert_eq!(
            drops(&audit),
            vec![
                format!("-state:{}", a.id),
                format!("-state:{}", b.id),
                "-state:0".into()
            ]
        );
        assert_eq!(balance(&audit), 0);
    }
    #[test]
    fn l12_disposed_resources_do_not_wait_for_handles_or_payloads() {
        for clear in [false, true] {
            let (mut o, r, audit) = setup();
            let a = o.counter(&r, "a", 0).unwrap();
            o.enqueue(&a, 1).unwrap();
            o.remove(&r).unwrap();
            assert_eq!(balance(&audit), 1);
            assert_eq!(o.read(&a), Err(Error::Stale));
            if clear {
                o.queue.clear();
            } else {
                assert_eq!(o.drain(), Ok(vec![Err(Error::Stale)]));
            }
            assert_eq!(balance(&audit), 0);
        }
    }
    #[test]
    fn l13_owner_drop_releases_queue_and_captures() {
        let (mut o, r, audit) = setup();
        let a = o.counter(&r, "a", 0).unwrap();
        o.enqueue(&a, 1).unwrap();
        drop(o);
        assert!(a.owner.upgrade().is_none());
        assert_eq!(balance(&audit), 0);
        let (other, _, _) = setup();
        assert_eq!(other.read(&a), Err(Error::Stale));
    }
    #[test]
    fn l14_thousand_cycles_return_to_baseline() {
        let (mut o, r, audit) = setup();
        let baseline = balance(&audit);
        for _ in 0..1000 {
            let a = o.counter(&r, "1", 10).unwrap();
            let b = o.counter(&r, "2", 20).unwrap();
            let c = o.counter(&r, "4", 40).unwrap();
            send(&mut o, &b, 3);
            o.reorder(&r, &["4", "2", "1"]).unwrap();
            o.enqueue(&b, 99).unwrap();
            for h in [&a, &b, &c] {
                o.remove(h).unwrap();
            }
            assert_eq!(o.drain(), Ok(vec![Err(Error::Stale)]));
            assert_eq!(balance(&audit), baseline);
            assert_eq!(o.nodes.len(), 1);
            assert!(o.values(&r).unwrap().is_empty());
        }
        o.remove(&r).unwrap();
        assert_eq!(balance(&audit), 0);
    }
    #[test]
    fn l17_checked_identity_exhaustion() {
        let (mut o, r, audit) = setup();
        let old = o.counter(&r, "a", 1).unwrap();
        o.remove(&old).unwrap();
        o.next = u64::MAX - 1;
        let last = o.counter(&r, "last", 2).unwrap();
        let before = audit.borrow().clone();
        assert_eq!(o.counter(&r, "overflow", 0).unwrap_err(), Error::Exhausted);
        assert_eq!(*audit.borrow(), before);
        assert_eq!(o.read(&old), Err(Error::Stale));
        assert_eq!(o.read(&last), Ok(2));
    }
    #[test]
    fn l18_demo_exact_output() {
        assert_eq!(
            demo(),
            "counters: a=2, b=1\nreordered: [(\"4\", 40), (\"2\", 23), (\"1\", 10)]\nremoved event: [Err(Stale)]; old handle: Err(Stale)\nreinserted: 1\ntracked live resources: 0\n"
        );
    }
}
