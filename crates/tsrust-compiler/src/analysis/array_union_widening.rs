//! Array widening moves each element into an existing tagged alternative.
use super::*;

impl Analyzer {
    fn array_union_target(&self, target: Type) -> Option<Type> {
        if !matches!(target, Type::Array(_) | Type::Union(_)) { return None; }
        let members = target.union().map_or_else(|| vec![target], |id| self.unions.members(id));
        let mut array = None;
        for member in members {
            match member {
                Type::Array(element) if matches!(element.ty(), Type::Union(_)) && array.is_none() => array = Some(member),
                Type::Null | Type::Undefined => {},
                _ => return None,
            }
        }
        array
    }

    fn array_union_widenable(&self, source: Type, target: Type, array: Type) -> bool {
        if !matches!(source, Type::Array(_) | Type::Union(_)) { return false; }
        let Type::Array(destination) = array else { return false; };
        let Type::Union(elements) = destination.ty() else { return false; };
        let targets = self.unions.members(elements);
        let sources = source.union().map_or_else(|| vec![source], |id| self.unions.members(id));
        let target_members = target.union().map_or_else(|| vec![target], |id| self.unions.members(id));
        let mut found_array = false;
        for member in sources {
            match member {
                Type::Array(element) => {
                    found_array = true;
                    let members = element.ty().union().map_or_else(|| vec![element.ty()], |id| self.unions.members(id));
                    if !members.iter().all(|&member| targets.iter().any(|candidate| candidate.accepts_scalar_value(member))) {
                        return false;
                    }
                }
                Type::Null | Type::Undefined if target_members.contains(&member) => {},
                _ => return false,
            }
        }
        found_array
    }

    pub(super) fn finish_array_union_widening(&self, expr: &mut Expr, target: Type, actual: Type) -> Result<Type, Diagnostic> {
        if !self.array_union_target(target).is_some_and(|array| self.array_union_widenable(actual, target, array)) {
            return Err(Diagnostic::new("array-widening", "Array elements must match destination union alternatives exactly.", expr.span));
        }
        expr.wrap_record_carrier(target);
        Ok(target)
    }

    pub(super) fn prepare_array_union_widening(
        &mut self, context: &Context, expr: &Expr, mode: Mode, expected: Option<Type>,
    ) -> Result<Option<continuations::prelude::Prelude>, Diagnostic> {
        // Literals already receive their element context. Borrowed arrays cannot
        // gain writable alternatives or silently acquire an independent owner.
        if matches!(expr.kind, ExprKind::Array(_)) { return Ok(None); }
        let Some(target) = expected else { return Ok(None); };
        let Some(array) = self.array_union_target(target) else { return Ok(None); };
        let Some(source) = self.expression_type_hint(context, expr) else { return Ok(None); };
        if source == target || !self.array_union_widenable(source, target, array) { return Ok(None); }
        if mode != Mode::Consume {
            return Err(Diagnostic::new("array-widening", "Widening array elements requires an owning move; convert the owned array before borrowing it.", expr.span));
        }
        Ok(Some(continuations::prelude::Prelude::Child {
            mode: Mode::Consume, expected: Some(source),
            frame: continuations::prelude::Frame::ArrayUnionWiden(target),
        }))
    }
}
