//! Emit checked deque operations against the owned VecDeque backend.
use super::*;
impl Emitter<'_> {
    pub(super) fn deque_method_expression(&self, function: &CheckedFunction, expression: &Expr, callee: &Expr, arguments: &[Expr], method: DequeMethod) -> String {
        let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
        let Type::Record(id) = base.ty.unwrap() else { unreachable!() };
        let element = self.program.records[id].native_element().unwrap();
        let read = matches!(method, DequeMethod::Front | DequeMethod::Back | DequeMethod::At);
        let receiver = self.expr(function, base, if read { Mode::Read } else { Mode::Mutate });
        let argument = arguments.first().map(|argument| self.expr(function, argument, if method == DequeMethod::At { Mode::Read } else { Mode::Consume })).unwrap_or_default();
        let operation = match method {
            DequeMethod::PushFront => "push_front", DequeMethod::PushBack => "push_back",
            DequeMethod::PopFront => "pop_front", DequeMethod::PopBack => "pop_back",
            DequeMethod::Front => "front", DequeMethod::Back => "back",
            DequeMethod::At => "at", DequeMethod::Clear => "clear",
        };
        let selected = format!("deque_receiver.values.{operation}({argument}){}", if read && element.is_copy() { ".copied()" } else { "" });
        let result = match expression.ty.unwrap() {
            Type::Void => selected,
            Type::Undefined => format!("{{let _={selected};TsrUndefined}}"),
            Type::Union(id) | Type::BorrowedUnion(id) => {
                let borrowed = matches!(expression.ty, Some(Type::BorrowedUnion(_)));
                let absent = self.program.unions[id.index()].members.iter().position(|ty| *ty == Type::Undefined).unwrap();
                let present = self.optional_element_present(id, element, borrowed, "payload");
                let tag = if borrowed { "ViewUnion" } else { "Union" };
                let nil = if borrowed { "&TsrUndefined" } else { "TsrUndefined" };
                format!("match {selected}{{Some(payload)=>{present},None=>{tag}{}::V{absent}({nil})}}", id.index())
            }
            _ => unreachable!("checked deque result"),
        };
        format!("{{let deque_receiver={receiver};{result}}}")
    }
}
