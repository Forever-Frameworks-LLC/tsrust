//! Native builtin adapters retain actual arity after once-only operand staging.
use super::*;

impl Emitter<'_> {
    pub(super) fn spread_native_string_rest(&self, plan: &CheckedSpreadCall) -> String {
        let CallTarget::NativeBinding(id)=plan.call.target.expect("checked native spread target") else {unreachable!("native rest target")};
        self.native_bindings.borrow_mut().insert(id);
        let crate::native_bindings::NativeBackend::Bundled{symbol}=crate::native_bindings::BINDINGS[id].backend;
        let mut arguments="std::iter::empty::<&String>()".to_string();
        for (index,segment) in plan.segments.iter().enumerate() {
            let copied=if segment.spread && segment.value.ty==Some(Type::String) {""} else {".copied()"};
            write!(arguments,".chain(spread_segment{}_{index}.iter(){copied})",plan.slot).unwrap();
        }
        format!("{symbol}(&{arguments}.map(|value|value.as_ref()).collect::<Vec<_>>())")
    }

    pub(super) fn spread_builtin_math_call(&self, plan: &CheckedSpreadCall) -> String {
        let mut values = "std::iter::empty::<f64>()".to_string();
        for index in 0..plan.segments.len() {
            write!(
                values,
                ".chain(spread_segment{}_{index}.iter().copied())",
                plan.slot
            )
            .unwrap();
        }
        match plan.call.target.expect("checked Math spread target") {
            CallTarget::MathMin => format!("tsr_math_extreme_iter({values}, false)"),
            CallTarget::MathMax => format!("tsr_math_extreme_iter({values}, true)"),
            CallTarget::MathHypot => format!("tsr_math_hypot_iter({values})"),
            _ => unreachable!("checked variadic Math spread target"),
        }
    }

    pub(super) fn spread_builtin_string_concat(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
    ) -> String {
        let ExprKind::Call(callee, _) = &plan.call.kind else {
            unreachable!("checked string concat spread call");
        };
        let ExprKind::Field(base, _) = &callee.kind else {
            unreachable!("checked string concat receiver");
        };
        let receiver = self.expr(fun, base, Mode::Read);
        let mut arguments = "std::iter::empty::<&String>()".to_string();
        for (index,segment) in plan.segments.iter().enumerate() {
            let copied=if segment.spread && segment.value.ty==Some(Type::String) {""}else{".copied()"};
            write!(arguments,".chain(spread_segment{}_{index}.iter(){copied})",plan.slot).unwrap();
        }
        let failure = format!(
            "string concat capacity or allocation failure at source bytes {}..{}",
            plan.call.span.start, plan.call.span.end
        );
        format!("tsr_string_concat_iter({receiver}, {arguments}, {failure:?})")
    }

    pub(super) fn spread_builtin_console_call(&self, plan: &CheckedSpreadCall) -> String {
        let ExprKind::Call(callee, _) = &plan.call.kind else {
            unreachable!("checked console spread call");
        };
        let ExprKind::Field(_, method) = &callee.kind else {
            unreachable!("checked console spread method");
        };
        let mut output = format!(
            "{{ let mut spread_console_values = Vec::with_capacity(spread_count{}); ",
            plan.slot
        );
        for (index, segment) in plan.segments.iter().enumerate() {
            let value = if segment.element.is_copy() {
                "spread_console_value"
            } else {
                "*spread_console_value"
            };
            let inspected = self.console_value(value, segment.element, "0", false);
            write!(output, "for spread_console_value in spread_segment{}_{index}.iter() {{ spread_console_values.push({inspected}); }} ", plan.slot).unwrap();
        }
        write!(
            output,
            "tsr_console_call({method:?}, &spread_console_values); }}"
        )
        .unwrap();
        output
    }

    pub(super) fn spread_builtin_observe_call(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
        mode: Mode,
    ) -> String {
        let mut output = String::new();
        for variant in &plan.variants {
            let SpreadVariantSelection::FirstSegment(index) = variant.selection else {
                continue;
            };
            let first=if plan.segments[index].spread && plan.segments[index].value.ty==Some(Type::String){"spread_first"}else{"*spread_first"};
            write!(output, "if let Some(spread_first) = spread_segment{}_{index}.iter().next() {{ let {} = {first}; ",
                plan.slot, SpreadSelection::Argument(0).native_name(plan.slot)).unwrap();
            output.push_str(&self.expr(fun, &variant.call, mode));
            output.push_str(" } else ");
        }
        output.push_str("{ ");
        if let Some(empty) = plan
            .variants
            .iter()
            .find(|variant| variant.selection == SpreadVariantSelection::Empty)
        {
            output.push_str(&self.expr(fun, &empty.call, mode));
        } else {
            output.push_str("unreachable!(\"checked spread has a first argument\")");
        }
        output.push_str(" }");
        output
    }

    pub(super) fn spread_builtin_unshift_call(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
    ) -> String {
        let ExprKind::Call(callee, _) = &plan.call.kind else {
            unreachable!("checked unshift spread call");
        };
        let ExprKind::Field(base, _) = &callee.kind else {
            unreachable!("checked unshift spread receiver");
        };
        let failure = format!(
            "array length does not fit int at source bytes {}..{}",
            plan.call.span.start, plan.call.span.end
        );
        if plan
            .segments
            .iter()
            .all(|segment| segment.spread && segment.length == Some(0))
        {
            let receiver = self.expr(fun, base, Mode::Read);
            return format!("i64::try_from(({receiver}).len()).expect({failure:?})");
        }
        let Type::Array(element) = base.ty.expect("checked unshift array") else {
            unreachable!("checked unshift array receiver");
        };
        let receiver = self.expr(fun, base, Mode::Mutate);
        let mut added = format!("std::iter::empty::<{}>()", self.native_type(element.ty()));
        for index in 0..plan.segments.len() {
            write!(
                added,
                ".chain(spread_segment{}_{index}.into_iter())",
                plan.slot
            )
            .unwrap();
        }
        format!(
            "({receiver}).tsr_unshift_iter({added}, spread_count{}, {failure:?})",
            plan.slot
        )
    }

    pub(super) fn spread_builtin_splice_call(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
    ) -> String {
        let ExprKind::Call(callee, _) = &plan.call.kind else {
            unreachable!("checked splice spread call");
        };
        let ExprKind::Field(base, _) = &callee.kind else {
            unreachable!("checked splice receiver");
        };
        let array_type = base.ty.expect("checked splice array");
        let empty = format!(
            "{{ let _splice_receiver = {}; let splice_empty: {} = [].into(); splice_empty }}",
            self.expr(fun, base, Mode::Read),
            self.native_type(array_type)
        );
        if plan
            .segments
            .iter()
            .all(|segment| segment.spread && segment.length == Some(0))
        {
            return empty;
        }
        let receiver = self.expr(fun, base, Mode::Mutate);
        let start = SpreadSelection::Argument(0).native_name(plan.slot);
        let delete = SpreadSelection::Argument(1).native_name(plan.slot);
        let rest = SpreadSelection::Rest.native_name(plan.slot);
        let failure = format!(
            "splice capacity or int length overflow at source bytes {}..{}",
            plan.call.span.start, plan.call.span.end
        );
        let result = if let Type::Array(ArrayElement::Record(record, 0)) = array_type
            && self.record_runtime.recursive(record)
        {
            format!("RecordArray{record}(spread_removed)")
        } else {
            "spread_removed".to_owned()
        };
        format!(
            "if spread_count{} == 0 {{ {empty} }} else {{ let spread_start = {start}.take().map(|slot| *slot.tsr_read()).expect(\"checked spread start\"); let spread_delete = {delete}.take().map(|slot| *slot.tsr_read()); let spread_added_count = {rest}.len(); let spread_removed = ({receiver}).tsr_splice_iter(spread_start, spread_delete, {rest}.tsr_owned_items(), spread_added_count, {failure:?}); {result} }}",
            plan.slot
        )
    }

    pub(super) fn spread_builtin_to_spliced_call(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
    ) -> String {
        let ExprKind::Call(callee, _) = &plan.call.kind else {
            unreachable!("checked toSpliced spread call");
        };
        let ExprKind::Field(base, _) = &callee.kind else {
            unreachable!("checked toSpliced receiver");
        };
        let array_type = base.ty.expect("checked toSpliced array");
        let Type::Array(element) = array_type else {
            unreachable!("checked toSpliced array receiver");
        };
        let receiver = self.expr(fun, base, Mode::Read);
        let start = SpreadSelection::Argument(0).native_name(plan.slot);
        let delete = SpreadSelection::Argument(1).native_name(plan.slot);
        let rest = SpreadSelection::Rest.native_name(plan.slot);
        let failure = format!(
            "toSpliced capacity overflow at source bytes {}..{}",
            plan.call.span.start, plan.call.span.end
        );
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        let prefix = if literal {
            "spread_copy_receiver.values.iter().take(spread_copy_start).map(|value| String::Borrowed(value))"
        } else {
            "spread_copy_receiver.iter().take(spread_copy_start).cloned()"
        };
        let suffix = if literal {
            "spread_copy_receiver.values.iter().skip(spread_copy_end).map(|value| String::Borrowed(value))"
        } else {
            "spread_copy_receiver.iter().skip(spread_copy_end).cloned()"
        };
        let result = self.flat_array_result(array_type, "spread_copy_values");
        format!(
            "{{ let spread_copy_receiver = {receiver}; let spread_copy_len = spread_copy_receiver.len(); let spread_copy_start = tsr_search_start(spread_copy_len, {start}.take().map(|slot| *slot.tsr_read()).unwrap_or(0_i64)); let spread_copy_deleted = if spread_count{slot} == 0 {{ 0 }} else {{ {delete}.take().map(|slot| *slot.tsr_read()).map_or(spread_copy_len-spread_copy_start, |count| usize::try_from(count.max(0)).unwrap_or(usize::MAX).min(spread_copy_len-spread_copy_start)) }}; let spread_copy_end = spread_copy_start+spread_copy_deleted; let spread_copy_capacity = (spread_copy_len-spread_copy_deleted).checked_add({rest}.len()).expect({failure:?}); let mut spread_copy_values: Vec<{item}> = Vec::with_capacity(spread_copy_capacity); spread_copy_values.extend({prefix}); spread_copy_values.extend({rest}.tsr_owned_items()); spread_copy_values.extend({suffix}); {result} }}",
            slot = plan.slot,
            item = self.native_type(element.ty())
        )
    }

    pub(super) fn spread_builtin_array_concat_call(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
    ) -> String {
        let ExprKind::Call(callee, _) = &plan.call.kind else {
            unreachable!("checked concat spread call");
        };
        let ExprKind::Field(base, _) = &callee.kind else {
            unreachable!("checked concat receiver");
        };
        let array_type = base.ty.expect("checked concat array");
        let Type::Array(element) = array_type else {
            unreachable!("checked concat array receiver");
        };
        let receiver = self.expr(fun, base, Mode::Read);
        let failure = format!(
            "array concat capacity overflow at source bytes {}..{}",
            plan.call.span.start, plan.call.span.end
        );
        let mut output = format!(
            "{{ let spread_concat_receiver = {receiver}; let mut spread_concat_capacity = spread_concat_receiver.len(); "
        );
        for (index, segment) in plan.segments.iter().enumerate() {
            if segment.element == array_type {
                write!(output, "for value in spread_segment{}_{index}.iter() {{ spread_concat_capacity = spread_concat_capacity.checked_add(value.len()).expect({failure:?}); }} ", plan.slot).unwrap();
            } else {
                let added_count = if segment.spread {
                    format!("spread_segment{}_{index}.len()", plan.slot)
                } else {
                    format!("usize::from(spread_segment{}_{index}.is_some())", plan.slot)
                };
                write!(output, "spread_concat_capacity = spread_concat_capacity.checked_add({added_count}).expect({failure:?}); ").unwrap();
            }
        }
        write!(
            output,
            "let mut spread_concat_values: Vec<{}> = Vec::with_capacity(spread_concat_capacity); ",
            self.native_type(element.ty())
        )
        .unwrap();
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        let receiver_items = if literal {
            "spread_concat_receiver.values.iter().map(|value| String::Borrowed(value))"
        } else if element.ty().is_copy() {
            "spread_concat_receiver.iter().copied()"
        } else {
            "spread_concat_receiver.iter().cloned()"
        };
        write!(output, "spread_concat_values.extend({receiver_items}); ").unwrap();
        for (index, segment) in plan.segments.iter().enumerate() {
            let copied = element.ty().is_copy();
            if segment.element == array_type {
                let operation = if copied { "copied" } else { "cloned" };
                write!(output,"for value in spread_segment{}_{index}.iter() {{ spread_concat_values.extend(value.iter().{operation}()); }} ",plan.slot).unwrap();
            } else {
                let value = if copied {
                    "*value".to_owned()
                } else {
                    format!(
                        "<{} as Clone>::clone(*value)",
                        self.native_type(element.ty())
                    )
                };
                write!(output,"for value in spread_segment{}_{index}.iter() {{ spread_concat_values.push({value}); }} ",plan.slot).unwrap();
            }
        }
        output.push_str(&self.flat_array_result(array_type, "spread_concat_values"));
        output.push_str(" }");
        output
    }

    pub(super) fn spread_builtin_fixed_call(
        &self,
        fun: &CheckedFunction,
        plan: &CheckedSpreadCall,
        mode: Mode,
    ) -> String {
        let mut output = format!("match spread_count{} {{ ", plan.slot);
        for variant in &plan.variants {
            let SpreadVariantSelection::Count(count) = variant.selection else {
                unreachable!("checked fixed spread arity variant");
            };
            assert_eq!(count, variant.count, "checked spread count identity");
            let call = &variant.call;
            write!(output, "{count} => {{ ").unwrap();
            for position in 0..count {
                let local = SpreadSelection::Argument(position).native_name(plan.slot);
                // Native Options describe physical slot presence. They do not
                // substitute language Undefined or run an omitted default.
                write!(
                    output,
                    "let mut {local} = {local}.take().expect(\"checked spread argument presence\"); "
                )
                .unwrap();
            }
            output.push_str(&self.expr(fun, call, mode));
            output.push_str(" }, ");
        }
        // The wrapper rejects invalid counts after all argument evaluations and
        // before invoking the builtin, so this branch is an internal invariant.
        output.push_str("_ => unreachable!(\"checked builtin spread arity\"), }");
        output
    }
}
