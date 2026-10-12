//! Native capture-slot metadata is indexed once for each checked body.
use crate::analysis::{CheckedFunction,CheckedProgram};
use std::collections::{HashMap,HashSet};
#[derive(Default)]
pub(super) struct Index(HashMap<usize,HashSet<usize>>);
impl Index {
    pub(super) fn new(program:&CheckedProgram)->Self {
        let mut index=Self::default();
        for closure in &program.closures {
            for capture in closure.captures.iter().filter(|capture|capture.restored) {
                index.0.entry(&closure.function as *const CheckedFunction as usize).or_default().insert(capture.inner);
            }
            for method in &closure.methods {
                for (capture,&slot) in closure.captures.iter().zip(&method.captures).filter(|(capture,_)|capture.restored) {
                    let _=capture;
                    index.0.entry(&method.function as *const CheckedFunction as usize).or_default().insert(slot);
                }
            }
        }
        index
    }
    pub(super) fn contains(&self,function:&CheckedFunction,slot:usize)->bool {
        self.0.get(&(function as *const CheckedFunction as usize)).is_some_and(|slots|slots.contains(&slot))
    }
}
