//! Explicit input lifetimes for callbacks retained through mutable records.
use super::*;

impl Emitter<'_> {
    pub(super) fn record_borrow_inputs(&self, function: usize) -> Vec<usize> {
        let mut inputs = self
            .program
            .record_parameter_borrows
            .get(&function)
            .into_iter()
            .flatten()
            .map(|borrow| borrow.source)
            .collect::<Vec<_>>();
        inputs.sort_unstable();
        inputs.dedup();
        inputs
    }

    pub(super) fn record_borrow_replacement_type(&self, function: usize, ty: Type, parameter: usize, path: &[String]) -> String {
        match ty {
            Type::Closure(origin) => {
                let closure = &self.program.closures[origin];
                let (mut lifetimes, fields) = self.capture_layout(closure);
                for borrow in self
                    .program
                    .record_parameter_borrows
                    .get(&function)
                    .into_iter()
                    .flatten()
                    .filter(|borrow| borrow.origin == origin && borrow.target.binding == parameter && borrow.target.fields == path)
                {
                    let field = &fields[borrow.capture];
                    for (slot, lifetime) in lifetimes.iter_mut().enumerate() {
                        let token = format!("'c{slot}");
                        if field
                            .split(|ch: char| !ch.is_alphanumeric() && ch != '\'' && ch != '_')
                            .any(|word| word == token)
                        {
                            *lifetime = format!("'record_input{}", borrow.source);
                        }
                    }
                }
                self.named_record_environment_type(ty, &lifetimes)
            }
            Type::BorrowedCallable(family) => {
                let arguments = self
                    .program
                    .stored_contracts
                    .origins(family)
                    .expect("checked origins")
                    .iter()
                    .map(|&origin| {
                        self.record_borrow_replacement_type(function, Type::Closure(origin), parameter, path)
                    })
                    .collect::<Vec<_>>();
                format!("BorrowedCallable{family}<{}>", arguments.join(", "))
            }
            _ => self.native_type(ty),
        }
    }
}
