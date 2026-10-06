//! Finite capture-access coordinates, separate from supplied source arguments.
use super::*;
use crate::local_capture_plan::BindingId;

#[derive(Clone, Debug)]
pub(crate) struct Access {
    pub binding: BindingId,
    pub slot: usize,
    pub mode: Mode,
}
#[derive(Clone, Default)]
pub(super) struct Lattice {
    modes: HashMap<(usize, BindingId), Mode>,
}
impl Lattice {
    pub(super) fn mode(&self, function: usize, binding: BindingId, initial: Mode) -> Mode {
        self.modes.get(&(function, binding)).copied().unwrap_or(initial)
    }
    /// Each discovered coordinate can rise at most twice beyond its initial mode.
    pub(super) fn observe(&mut self, summaries: &HashMap<usize, Vec<Access>>) -> (bool, usize) {
        let mut changed = false;
        let mut discovered = 0;
        for (&function, accesses) in summaries {
            for access in accesses {
                let key = (function, access.binding);
                match self.modes.entry(key) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(access.mode);
                        discovered += 1;
                        changed = true;
                    }
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        let next = (*entry.get()).max(access.mode);
                        changed |= next != *entry.get();
                        entry.insert(next);
                    }
                }
            }
        }
        (changed, discovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_group_access_converges_without_changing_public_parameter_contracts() {
        let source = "function main(){let value=0;function left(n:int):int{if(n===0){value+=1;return value;}return right(n-1);}function right(n:int):int{return left(n);}console.log(right(2));}";
        let program = crate::syntax::parse(source).unwrap();
        let initial = Analyzer::new(program.clone()).unwrap();
        let modes: Vec<Vec<Mode>> = initial.signatures.iter().map(|signature| signature.params.iter().map(|ty| if ty.is_copy(){Mode::Copy}else{Mode::Read}).collect()).collect();
        let mut lattice = Lattice::default();
        let mut converged = false;
        for _ in 0..8 {
            let mut analyzer = Analyzer::new(program.clone()).unwrap();
            analyzer.assumed_modes = Some(modes.clone());
            analyzer.assumed_capture_access = lattice.clone();
            analyzer.effects_only = true;
            analyzer.speculative_access_run = true;
            let checked = analyzer.run().unwrap();
            let (changed, _) = lattice.observe(&checked.named_capture_access);
            if !changed {
                assert_eq!(checked.named_capture_access.len(), 2);
                for (&index, access) in &checked.named_capture_access {
                    assert_eq!(access.len(), 1);
                    assert_eq!(access[0].mode, Mode::Mutate);
                    assert_eq!(access[0].slot, checked.functions[index].parameter_count);
                    assert_eq!(checked.functions[index].bindings[0].name, "n");
                    assert_eq!(checked.functions[index].bindings[0].mode, Mode::Copy);
                }
                assert!(checked.contracts().keys().all(|name| !name.ends_with(".value")));
                converged = true;
                break;
            }
        }
        assert!(converged, "finite capture lattice must converge");
        let error = crate::check(source).err().expect("capture lowering pending");
        assert_eq!(error.code, "unsupported");
    }
}
