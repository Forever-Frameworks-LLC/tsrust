//! A checked constant-label traversal can construct its result directly.
use super::*;
use crate::fixed_storage::{visit_expression, visit_expressions};

fn named(value: &Expr, binding: usize) -> bool {
    matches!(value.kind, ExprKind::Name(_)) && value.binding == Some(binding)
}
fn integer(value: &Expr, text: &str) -> bool { matches!(&value.kind, ExprKind::Integer(value) if value == text) }
fn mentions(statement: &Stmt, binding: usize) -> bool {
    let mut found=false;
    visit_expressions(std::slice::from_ref(statement), &mut |value| {
        found |= named(value,binding) || matches!(&value.kind, ExprKind::SpreadValue(selected) if selected.source.roots.contains(&binding));
    });
    found
}
fn fresh(prior: &[Stmt], binding: usize, counter: bool) -> bool {
    for statement in prior.iter().rev() {
        if let StmtKind::Let { binding:Some(id),value,.. } = &statement.kind && *id==binding {
            return if counter {integer(value,"0")} else {matches!(&value.kind, ExprKind::Array(items) if items.is_empty())};
        }
        if counter && let StmtKind::Assign {target,value,operator:None} = &statement.kind && named(target,binding) { return integer(value,"0"); }
        if mentions(statement,binding) { return false; }
    }
    false
}
fn step(statement: &Stmt, counter: usize) -> bool {
    let StmtKind::Assign {target,value,operator}=&statement.kind else {return false;};
    if !named(target,counter) {return false;}
    if matches!(operator,Some((BinaryOp::Add,_))) {return integer(value,"1");}
    operator.is_none() && matches!(&value.kind,ExprKind::Binary {op:BinaryOp::Add,left,right,..} if named(left,counter) && integer(right,"1"))
}

impl Emitter<'_> {
    fn builder_label(&self, fun: &CheckedFunction, value: &Expr) -> Option<String> {
        if value.target==Some(CallTarget::StringConcat) && let ExprKind::Call(callee,args)=&value.kind
            && let ExprKind::Field(base,_)=&callee.kind { return self.constant_strings.concat(fun,base,args); }
        self.constant_strings.concat(fun,value,&[])
    }
    fn builder_evaluation(&self, fun: &CheckedFunction, value: &Expr, source: usize, counter: usize) -> bool {
        let mut valid=true;
        visit_expression(value,&mut |value| {
            valid &= match &value.kind {
                ExprKind::Name(_) => value.binding.is_none_or(|id| id==source || id==counter || self.constant_strings.concat(fun,value,&[]).is_some()),
                ExprKind::String(_) | ExprKind::Null | ExprKind::Undefined => true,
                ExprKind::Index(base,index) => named(base,source) && named(index,counter),
                ExprKind::Unary {op:UnaryOp::Typeof,..} => true,
                ExprKind::Call(_,_) => matches!(value.target,Some(CallTarget::StringConcat | CallTarget::StringConversion)) && self.builder_label(fun,value).is_some(),
                ExprKind::Field(base,name) => name=="concat" && base.ty.map(Type::scalar_base)==Some(Type::String),
                _ => false,
            };
        });
        valid
    }
    pub(super) fn counted_builder(&self, fun: &CheckedFunction, parent: &[Stmt], statement: &Stmt) -> Option<String> {
        let StmtKind::While {condition,body}=&statement.kind else {return None;};
        if !statement.labels.is_empty() || self.exception_regions.carries_storage()
            || !fun.storage_exits.array_epochs(statement.span).is_empty() || !fun.storage_exits.map_entry_epochs(statement.span).is_empty() {return None;}
        let ExprKind::Binary {op:BinaryOp::Less,left,right,..}=&condition.kind else {return None;};
        let counter=left.binding.filter(|id|named(left,*id))?;
        let ExprKind::Field(input,property)=&right.kind else {return None;};
        if property!="length" || !matches!(input.ty,Some(Type::Array(_))) {return None;}
        let source=input.binding.filter(|id|named(input,*id))?;
        let (last,items)=body.split_last()?;
        if !step(last,counter) {return None;}
        let mut result=None;
        for item in items {
            match &item.kind {
                StmtKind::Let {binding:Some(id),value,..} => {
                    let binding=&fun.bindings[*id];
                    if binding.rebindable || binding.mutable || self.builder_label(fun,value).is_none()
                        || !self.builder_evaluation(fun,value,source,counter) {return None;}
                }
                StmtKind::Expr(value) if value.target==Some(CallTarget::ArrayPush) && result.is_none() => {
                    let ExprKind::Call(callee,args)=&value.kind else {return None;};
                    let ExprKind::Field(base,_)=&callee.kind else {return None;};
                    let [argument]=args.as_slice() else {return None;};
                    let destination=base.binding.filter(|id|named(base,*id))?;
                    let label=self.builder_label(fun,argument)?;
                    if !self.builder_evaluation(fun,argument,source,counter) {return None;}
                    result=Some((destination,label,base));
                }
                _=>return None,
            }
        }
        let (destination,label,base)=result?;
        if destination==source || destination==counter || source==counter || self.expression_shape(base).is_some() {return None;}
        let output=&fun.bindings[destination];let index=&fun.bindings[counter];let input_binding=&fun.bindings[source];
        if !matches!(output.ty,Type::Array(element) if element.ty().scalar_base()==Type::String)
            || output.parameter || output.rebindable || output.capture_reference || output.borrowed_place.is_some() || output.mutable_place
            || index.ty!=Type::Int || index.parameter || index.capture_reference || index.borrowed_place.is_some() || index.mutable_place
            || input_binding.mutable || input_binding.rebindable || input_binding.capture_reference || input_binding.borrowed_place.is_some()
            || self.finally_moves.contains(fun,destination) || crate::analysis::has_exception_syntax(&fun.body) {return None;}
        let captured=receiver_moves::captured(fun,&self.program.closures)?;
        if [source,counter,destination].iter().any(|id|captured.contains(id)) {return None;}
        let mut hidden_capture=false;
        crate::fixed_storage::visit_function_expressions(fun,&mut |value| {
            if let Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=value.target {
                hidden_capture |= self.program.named_capture_access.get(&id).is_some_and(|captures|!captures.is_empty());
            }
        });
        if hidden_capture {return None;}
        let position=parent.iter().position(|item|std::ptr::eq(item,statement))?;
        if !fresh(&parent[..position],destination,false) || !fresh(&parent[..position],counter,true) {return None;}
        let length=self.expr(fun,right,Mode::Copy);let text=self.string_literal(&label);
        // Preserve the original checked bound before allocation; all removed
        // reads are in range and the unit increment cannot overflow this bound.
        Some(format!("{{ let counted_length = {length}; let counted_count = usize::try_from(counted_length - value{counter}).expect(\"checked array length\") + value{destination}.len(); value{destination} = if counted_count > 4 {{ Array::Heap(vec![{text}; counted_count]) }} else {{ let mut values = [const {{ None }}; 4]; for slot in values.iter_mut().take(counted_count) {{ *slot = Some({text}); }} Array::Small {{ len: counted_count, values }} }}; value{counter} = counted_length; let _ = value{counter} < counted_length; }}"))
    }
}
