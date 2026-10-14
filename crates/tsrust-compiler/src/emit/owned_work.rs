//! Stable ownership for temporary checked bodies used by borrowed work frames.
//!
//! An append-only binary tree keeps references valid without unsafe pointers or
//! leaking values. Breadth-first positions keep lookup logarithmic in allocations;
//! growth follows actual work, with no source-nesting bound. Destruction uses an
//! explicit worklist rather than the tree's Rust call stack.
use std::cell::{Cell, OnceCell};

pub(super) struct Arena<T> {
    root: Node<T>,
    next: Cell<usize>,
}
struct Node<T> {
    value: OnceCell<T>,
    left: OnceCell<Box<Node<T>>>,
    right: OnceCell<Box<Node<T>>>,
}
impl<T> Default for Node<T> {
    fn default() -> Self { Self { value: OnceCell::new(), left: OnceCell::new(), right: OnceCell::new() } }
}
impl<T> Default for Arena<T> {
    fn default() -> Self { Self { root: Node::default(), next: Cell::new(1) } }
}
impl<T> Arena<T> {
    pub(super) fn alloc(&self, value: T) -> &T {
        let index=self.next.get();
        self.next.set(index.checked_add(1).expect("emission allocation index exhausted"));
        let mut node=&self.root;
        let mut bit=(1usize << (usize::BITS-1-index.leading_zeros())) >> 1;
        while bit != 0 {
            let child=if index & bit == 0 { &node.left } else { &node.right };
            node=child.get_or_init(||Box::new(Node::default()));
            bit >>= 1;
        }
        node.value.set(value).unwrap_or_else(|_|panic!("emission allocation reused"));
        node.value.get().expect("initialized emission value")
    }
}
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        let mut pending=Vec::new();
        pending.extend(self.left.take());pending.extend(self.right.take());
        while let Some(mut node)=pending.pop() {
            pending.extend(node.left.take());pending.extend(node.right.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    #[test]
    fn references_survive_growth_and_every_value_drops_once() {
        struct Value(usize,Rc<Cell<usize>>);
        impl Drop for Value { fn drop(&mut self) {self.1.set(self.1.get()+1);} }
        let drops=Rc::new(Cell::new(0));
        {
            let arena=Arena::default();let mut values=Vec::new();
            for index in 0..65_536 {values.push(arena.alloc(Value(index,drops.clone())));}
            for (index,value) in values.iter().enumerate(){assert_eq!(value.0,index);}
            assert_eq!(drops.get(),0);
        }
        assert_eq!(drops.get(),65_536);
    }
}
