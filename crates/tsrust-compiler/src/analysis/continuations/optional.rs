//! Optional branch state separates receiver checking from lazy continuation checking.
use super::super::*;
use crate::syntax::{NullishBranch,SelectedSource};
pub(super) struct Base{
    pub(super) value:Expr,pub(super) mode:Mode,operator:Span,
    place:Option<crate::flow::Place>,possible:Option<Vec<Type>>,named:bool,
}
pub(super) struct Branch{
    base:Expr,operator:Span,place:Option<crate::flow::Place>,union:Option<UnionId>,
    before:Vec<(usize,Availability)>,before_flow:crate::flow::Facts,
    capture_boundary:crate::capture_loan_identity::Boundary,
    branch:NullishBranch,owned:bool,absent:bool,expected:Option<Type>,
}
impl Analyzer{
    pub(super) fn abort_optional_branch(&mut self,context:&mut Context,frame:Branch){
        frame.capture_boundary.unwind(&mut context.capture_loans);
        context.selected_access.remove(&frame.operator.start);
        for(id,state)in frame.before{state.restore(&mut context.bindings[id]);}
        context.flow=frame.before_flow;
    }

    pub(super) fn prepare_optional_base(&mut self,context:&mut Context,body:&mut Expr)->Option<Base>{
        let(base,operator)=Self::optional_extract(body)?;
        let place = crate::flow::place(&base, &context.names);
        let possible = place.as_ref().filter(|place|place.fields.is_empty()).and_then(|place|context.constant_absence.get(&place.root)).map(|ty|vec![*ty]).or_else(||place
            .as_ref()
            .and_then(|place| context.flow.get(place))
            .map(|fact| fact.members.clone()));
        if let Some(place) = &place {
            context.suppressed_flow.insert(place.clone());
        }
        let named = Self::named_storage(&base)
            || self.optional_source_named(context, &base)
            || self.borrowed_place_hint(context, &base);
        Some(Base{value:base,mode:if named{Mode::Read}else{Mode::Consume},operator,place,possible,named})
    }
    pub(super) fn prepare_optional_branch(&mut self,context:&mut Context,body:&mut Expr,frame:Base,expected:Option<Type>,checked:Result<Type,Diagnostic>)->Result<Branch,Diagnostic>{
        let Base{value:base,operator,place,possible,named,..}=frame;
        if let Some(place)=&place{context.suppressed_flow.remove(place);}
        let actual=checked?;
        let union = actual.union();
        let members = union
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![actual]);
        let possible = possible.unwrap_or_else(|| members.clone());
        let present_types = possible.iter().copied().filter(|ty| !matches!(ty,Type::Null|Type::Undefined)).collect::<Vec<_>>();
        let mut payloads = present_types.iter().copied();
        let payload = payloads.next().or_else(|| self.unions.payload(actual.owning())).ok_or_else(|| Diagnostic::new("type", "An optional member or call requires a declared non-null payload type.", operator))?;
        if payloads.next().is_some() { return Err(Diagnostic::new("unsupported", "Optional member access requires one proven payload type; narrow the union first.", operator)); }
        let absent = possible
            .iter()
            .any(|ty| matches!(ty, Type::Null | Type::Undefined));
        let present = possible
            .iter()
            .any(|ty| !matches!(ty, Type::Null | Type::Undefined));
        let branch = if !absent {
            NullishBranch::Present
        } else if !present {
            NullishBranch::Absent
        } else {
            NullishBranch::Dynamic
        };
        let owned = actual.is_copy() || (!named && !matches!(actual, Type::BorrowedUnion(_)));
        let mut roots = Self::loan_roots(&base, Mode::Read);
        for root in roots.clone() {
            for source in &context.bindings[root].view_roots {
                if !roots.contains(source) {
                    roots.push(*source);
                }
            }
        }
        let source = SelectedSource {
            readonly: matches!(actual, Type::BorrowedUnion(_))
                || self.borrowed_place_hint(context, &base),
            roots,
            place: place.as_ref().map(|p| (p.root, p.fields.clone())),
            span: base.span,
        };
        Self::optional_selected(body, operator.start, payload, &source, owned);
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let before_flow = context.flow.clone();
        if let Some(place) = &place {
            context.flow.insert(
                place.clone(),
                crate::flow::Fact {
                    members: vec![payload],
                    origin: Some((operator.start, operator.end)),
                },
            );
        }
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        Ok(Branch{base,operator,place,union,before,before_flow,capture_boundary,branch,owned,absent,expected})
    }
    pub(super) fn finish_optional_branch(&mut self,context:&mut Context,body:&mut Expr,frame:Branch,body_ty:Type)->Result<Type,Diagnostic>{
        let Branch{base,operator,place,union,before,before_flow,capture_boundary,branch,owned,absent,expected}=frame;
        capture_boundary.unwind(&mut context.capture_loans);
        let access = context
            .selected_access
            .remove(&operator.start)
            .unwrap_or(Mode::Read);
        if branch != NullishBranch::Present {
            for (id, state) in before {
                let after = Availability::of(&context.bindings[id]);
                if branch == NullishBranch::Absent {
                    state.restore(&mut context.bindings[id]);
                } else {
                    state.join(after).restore(&mut context.bindings[id]);
                }
            }
            context.flow = if branch == NullishBranch::Absent {
                before_flow
            } else {
                crate::flow::join(&before_flow, &context.flow)
            };
        } else if let Some(place) = &place {
            // Selection facts are branch-local; preserve unrelated successful effects.
            if let Some(fact) = before_flow.get(place) {
                context.flow.insert(place.clone(), fact.clone());
            } else {
                context.flow.remove(place);
            }
        }
        let value_type = if body_ty == Type::Void {
            Type::Undefined
        } else {
            body_ty.owning()
        };
        let joined = if absent {
            self.unions
                .intern([value_type, Type::Undefined], body.span)?
        } else {
            body_ty
        };
        let borrowed = !joined.is_copy()
            && (matches!(body_ty, Type::BorrowedUnion(_))
                || self.borrowed_place_hint(context, body));
        let result = if borrowed {
            joined.union().map(Type::BorrowedUnion).unwrap_or(joined)
        } else {
            joined
        };
        self.require(result, expected, body.span)?;
        let checked = std::mem::replace(body, Expr::checked(ExprKind::Null, body.span, None));
        *body = Expr::checked(
            ExprKind::Optional {
                base: Box::new(base),
                body: Box::new(checked),
                slot: operator.start,
                union,
                access,
                branch,
                owned,
            },
            body.span,
            Some(result),
        );
        Ok(result)
    }
}
