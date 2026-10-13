//! Declaration grammar steps of the parser scheduler: declarators, binding
//! patterns, parameter lists with defaults, and function and closure bodies.
use super::scheduler::{ClosureFrame, DeclarationFrame, Machine, ParameterHead, ParametersFrame, PatternElement, PatternFrame, Work};
use super::*;

fn statement(kind: StmtKind, span: Span) -> Stmt {
    Stmt { labels: Vec::new(), control_target: None, kind, span }
}

impl Parser<'_> {
    pub(super) fn declaration_step(&mut self, task: Work, machine: &mut Machine) -> Result<(), Diagnostic> {
        match task {
            Work::DeclaratorValue { frame, name } => {
                let (name, annotation) = *name;
                let value = machine.value();
                let kind = StmtKind::Let { rebindable: frame.rebindable, name, annotation, value, binding: None };
                self.declarator_finish(frame, kind, None, machine)?;
            }
            Work::DeclaratorPattern { frame, whole, at } => {
                let positions = machine.body();
                let annotation = if self.accept(":") { Some(self.value_annotation()?) } else { None };
                self.expect("=")?;
                machine.request(Work::DeclaratorPatternValue { frame, head: Box::new((*whole, annotation)), positions, at }, Work::Parse(0));
            }
            Work::DeclaratorPatternValue { frame, head, mut positions, at } => {
                let (whole, annotation) = *head;
                let value = machine.value();
                // A named source needs no holder: each name takes its position
                // from the source itself, so copied positions leave it usable.
                let direct = match (&value.kind, &annotation) {
                    (ExprKind::Name(source), None) if !positions.is_empty() => Some(source.clone()),
                    _ => None,
                };
                let kind = if let Some(source) = direct {
                    for position in &mut positions {
                        if let StmtKind::Let { value: selected, .. } = &mut position.kind {
                            Self::replace_pattern_source(selected, &whole.text, &source, value.span);
                        }
                    }
                    let first = positions.remove(0);
                    self.pending.append(&mut positions);
                    first.into_kind()
                } else {
                    self.pending.append(&mut positions);
                    StmtKind::Let { rebindable: false, name: whole, annotation, value, binding: None }
                };
                self.declarator_finish(frame, kind, Some(at), machine)?;
            }
            Work::Pattern { whole, rebindable, depth, output } => {
                if self.policy.bounded() && depth > 32 {
                    return Err(Diagnostic::new("unsupported", "Destructuring pattern nesting exceeds the limit of 32.", self.peek().span));
                }
                let array = self.peek().text == "[";
                self.take();
                let frame = PatternFrame { whole, rebindable, array, position: 0, excluded: Vec::new(), output, depth };
                self.pattern_next(Box::new(frame), machine)?;
            }
            Work::PatternNext(frame) => self.pattern_next(frame, machine)?,
            Work::PatternNested { mut frame, element } => {
                frame.output = machine.body();
                if self.is("=") {
                    let operator = self.take().span;
                    machine.request(Work::PatternNestedDefault { frame, element, operator }, Work::Parse(0));
                } else {
                    let PatternElement { start, selected, name, slot } = *element;
                    self.pattern_nested(frame, start, selected, name, slot, machine)?;
                }
            }
            Work::PatternNestedDefault { frame, element, operator } => {
                let PatternElement { start, selected, name, slot } = *element;
                let selected = Self::pattern_default(selected, operator, machine.value());
                self.pattern_nested(frame, start, selected, name, slot, machine)?;
            }
            Work::PatternNameDefault { mut frame, element, operator } => {
                let PatternElement { start, selected, name, .. } = *element;
                let selected = Self::pattern_default(selected, operator, machine.value());
                self.pattern_name(&mut frame, start, name, selected)?;
                machine.work.push(Work::PatternNext(frame));
            }
            Work::PatternRest { mut frame, element } => {
                frame.output = machine.body();
                let PatternElement { start, name, slot, .. } = *element;
                self.array_rest_pattern(&mut frame, start, name, Some(slot))?;
                self.expect("]")?;
                machine.bodies.push(frame.output);
            }
            Work::Parameters(frame) => {
                if self.is(")") {
                    self.parameters_finish(frame, machine)?;
                } else {
                    self.parameter(frame, machine)?;
                }
            }
            Work::ParameterPattern { frame, rest, mut whole } => {
                let mut positions = machine.body();
                // Retain the parameter pattern boundary for initialization scheduling.
                whole.span.end = self.tokens[self.i - 1].span.end;
                self.parameter_patterns.append(&mut positions);
                self.parameter_named(frame, rest, *whole, true, machine)?;
            }
            Work::ParameterDefault { frame, head } => {
                let value = machine.value();
                let ParameterHead { rest, name, ty, access, operator, declared } = *head;
                // A default runs before the body block exists, so there is no block
                // to hold the private declaration of a named function expression.
                if let Some(local) = self.pending[declared..].iter().find(|local| matches!(local.kind, StmtKind::LocalFunction(_))) {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "A named function expression cannot be a parameter default yet. Use an arrow function, or declare the function and write its name here.",
                        local.span,
                    ));
                }
                let initializer = ParameterInitializer::Default { span: Span::new(operator.start, value.span.end), value: Box::new(value) };
                self.parameter_finish(frame, CallableParameter { rest, name, ty, access, initializer }, machine)?;
            }
            Work::NamedFunction => {
                let name = self.name()?;
                let type_parameters = self.type_parameters()?;
                self.expect("(")?;
                machine.request(
                    Work::NamedFunctionParameters(Box::new((name, type_parameters))),
                    Work::Parameters(Box::new(ParametersFrame { params: Vec::new(), declared: true })),
                );
            }
            Work::NamedFunctionParameters(header) => {
                let (name, type_parameters) = *header;
                let declared = machine.parameters.pop().expect("completed parameter list");
                let mut params = Vec::new();
                let mut parameter_initializers = Vec::new();
                let mut parameter_access = Vec::new();
                let mut rest_parameter = false;
                for parameter in declared {
                    rest_parameter = parameter.rest;
                    parameter_access.push(parameter.access.map(|token| {
                        (
                            match token.text.as_str() {
                                "read" => crate::Mode::Read,
                                "mutate" => crate::Mode::Mutate,
                                _ => crate::Mode::Consume,
                            },
                            token.span,
                        )
                    }));
                    let (ty, callback) = match parameter.ty {
                        ParameterAnnotation::Named(ty) => (*ty, None),
                        ParameterAnnotation::Callback(callback) => (callback.value_annotation(), Some(*callback)),
                        ParameterAnnotation::Inferred(_) => unreachable!("declared parameter has an annotation"),
                    };
                    parameter_initializers.push(parameter.initializer);
                    params.push((parameter.name, ty, callback));
                }
                let result = if self.accept(":") { Some(self.result_annotation()?) } else { None };
                let patterns = std::mem::take(&mut self.parameter_patterns);
                let mut body = Vec::new();
                Self::open_with_patterns(patterns, &mut body);
                let scope_span = name.span;
                let function = Function {
                    application: None,
                    scope_span,
                    name,
                    type_parameters,
                    params,
                    parameter_initializers,
                    parameter_access,
                    rest_parameter,
                    result,
                    body,
                };
                machine.request(Work::NamedFunctionBody(Box::new(function)), Work::Block);
            }
            Work::NamedFunctionBody(mut function) => {
                function.body.append(&mut machine.body());
                function.scope_span.end = self.tokens[self.i - 1].span.end;
                machine.functions.push(*function);
            }
            Work::LocalFunction(start) => {
                let function = machine.functions.pop().expect("completed local function");
                let span = Span::new(start, function.scope_span.end);
                machine.statements.push(statement(StmtKind::LocalFunction(Box::new(function)), span));
            }
            // `function name(...) { ... }` in expression position. The function is
            // declared in the enclosing block under a name that source cannot spell,
            // so only this expression refers to it; the lexical resolver makes the
            // written name visible inside the function body for recursion.
            Work::NamedFunctionExpression { outer } => {
                let mut inner = std::mem::replace(&mut self.pending, outer);
                let mut declaration = machine.statement();
                let StmtKind::LocalFunction(function) = &mut declaration.kind else {
                    unreachable!("local function statement")
                };
                function.body.append(&mut inner);
                let name = function.name.clone();
                function.name.text = format!("{}{}{}", name.text, crate::local_functions::EXPRESSION_MARK, name.span.start);
                let reference = Expr::new(ExprKind::Name(function.name.text.clone()), name.span);
                self.pending.push(declaration);
                machine.values.push(reference);
                machine.work.push(Work::Postfix { optional: false });
            }
            Work::ClosureParameters { start, outer_patterns } => {
                let params = machine.parameters.pop().expect("completed parameter list");
                self.closure_body(start, outer_patterns, params, machine)?;
            }
            Work::FunctionParameters(start) => {
                let params = machine.parameters.pop().expect("completed parameter list");
                let result = if self.accept(":") { self.result_annotation()? } else { ResultAnnotation::Inferred(start) };
                if !self.is("{") {
                    return Err(Diagnostic::new(
                        "syntax",
                        "A function expression needs a block body; use an arrow function for an expression body.",
                        self.peek().span,
                    ));
                }
                let id = self.next_closure;
                self.next_closure += 1;
                let patterns = std::mem::take(&mut self.parameter_patterns);
                machine.request(Work::ClosureBlock(Box::new(ClosureFrame { id, start, header: Box::new((params, result)), patterns })), Work::Block);
            }
            Work::ClosureBlock(frame) => {
                let frame = *frame;
                let mut body = machine.body();
                Self::open_with_patterns(frame.patterns, &mut body);
                let span = Span::new(frame.start.start, self.tokens[self.i - 1].span.end);
                machine.values.push(Self::closure_expression(frame.id, frame.header, body, span));
                machine.work.push(Work::Postfix { optional: false });
            }
            Work::ClosureValue { frame, outer } => {
                let frame = *frame;
                let value = machine.value();
                let mut body = std::mem::replace(&mut self.pending, outer);
                body.append(&mut Self::returned(Box::new(value)));
                Self::open_with_patterns(frame.patterns, &mut body);
                let span = Span::new(frame.start.start, self.tokens[self.i - 1].span.end);
                machine.values.push(Self::closure_expression(frame.id, frame.header, body, span));
                machine.work.push(Work::Postfix { optional: false });
            }
            _ => unreachable!("declaration work"),
        }
        Ok(())
    }

    /// A `function name(...) { ... }` declaration statement.
    pub(super) fn local_function(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        let keyword = self.expect("function")?;
        if self.unresolved_local_function.is_none() {
            self.unresolved_local_function = Some(self.peek().clone());
        }
        machine.request(Work::LocalFunction(keyword.span.start), Work::NamedFunction);
        Ok(())
    }

    /// One declarator after `const`, `let` or a comma: a name, or a pattern
    /// whose position declarations are appended to `pending`.
    pub(super) fn declarator(&mut self, frame: Box<DeclarationFrame>, machine: &mut Machine) -> Result<(), Diagnostic> {
        if matches!(self.peek().text.as_str(), "[" | "{") {
            // `const [a, b] = value;` binds the whole value once, then each
            // name takes its position under the ordinary binding rules.
            let at = self.pending.len();
            let whole = self.pattern_holder();
            let pattern = Work::Pattern { whole: whole.text.clone(), rebindable: frame.rebindable, depth: 0, output: Vec::new() };
            machine.request(Work::DeclaratorPattern { frame, whole: Box::new(whole), at }, pattern);
            return Ok(());
        }
        let name = self.name()?;
        let annotation = if self.accept(":") { Some(self.value_annotation()?) } else { None };
        // `let name: Type;` has no initializer and gets the deferred
        // placeholder; the variable is assigned later.
        if frame.rebindable && (self.is(";") || self.is(",")) {
            if annotation.is_none() {
                return Err(Diagnostic::new(
                    "annotation-required",
                    format!("Declare the type of {:?} when it has no initializer, for example `let {}: int;`.", name.text, name.text),
                    name.span,
                ));
            }
            let value = Expr::deferred(self.tokens[self.i - 1].span);
            let kind = StmtKind::Let { rebindable: frame.rebindable, name, annotation, value, binding: None };
            return self.declarator_finish(frame, kind, None, machine);
        }
        self.expect("=")?;
        machine.request(Work::DeclaratorValue { frame, name: Box::new((name, annotation)) }, Work::Parse(0));
        Ok(())
    }

    /// Place one completed declarator, then continue the list or end the
    /// statement. A listed pattern's whole-value declaration precedes the
    /// position declarations it already appended at `at`.
    fn declarator_finish(&mut self, mut frame: Box<DeclarationFrame>, kind: StmtKind, at: Option<usize>, machine: &mut Machine) -> Result<(), Diagnostic> {
        match frame.listed {
            None => frame.first = Some(kind),
            Some(span) => match at {
                Some(at) => self.pending.insert(at, statement(kind, span)),
                None => self.pending.push(statement(kind, span)),
            },
        }
        if self.accept(",") {
            frame.listed = Some(self.peek().span);
            return self.declarator(frame, machine);
        }
        if frame.terminated {
            self.expect(";")?;
        }
        let kind = frame.first.take().expect("first declarator");
        machine.statements.push(statement(kind, frame.span));
        Ok(())
    }

    /// Continue `[a, , b]` or `{ x, label: text }` after its opening bracket or
    /// after one completed element. One declaration is appended per name; a
    /// nested pattern first binds its own position to a hidden name.
    fn pattern_next(&mut self, mut frame: Box<PatternFrame>, machine: &mut Machine) -> Result<(), Diagnostic> {
        let close = if frame.array { "]" } else { "}" };
        while !self.accept(close) {
            if frame.array && self.accept(",") {
                // An empty slot skips one position.
                frame.position += 1;
                continue;
            }
            if self.is("...") {
                if frame.array {
                    let start = self.expect("...")?.span;
                    if matches!(self.peek().text.as_str(), "[" | "{") {
                        let holder = self.pattern_holder();
                        let (pattern, slot) = Self::nested_pattern(&mut frame, &holder);
                        let selected = Expr::new(ExprKind::Undefined, start);
                        machine.request(Work::PatternRest { frame, element: Box::new(PatternElement { start, selected, name: holder, slot }) }, pattern);
                        return Ok(());
                    }
                    let name = self.name()?;
                    self.array_rest_pattern(&mut frame, start, name, None)?;
                } else {
                    self.record_rest_pattern(&frame.whole, frame.rebindable, &mut frame.output, &frame.excluded)?;
                }
                self.expect(close)?;
                break;
            }
            let start = self.peek().span;
            let source = |span: Span| Expr::new(ExprKind::Name(frame.whole.clone()), span);
            // Record patterns name a field, optionally followed by a target.
            let (selected, shorthand) = if frame.array {
                let index = Expr::new(ExprKind::Integer(frame.position.to_string()), start);
                (Expr::new(ExprKind::Index(Box::new(source(start)), Box::new(index)), start), None)
            } else {
                let field = self.name()?;
                let selected = Expr::new(ExprKind::Field(Box::new(source(field.span)), field.text.clone()), field.span);
                frame.excluded.push(field.text.clone());
                if self.accept(":") { (selected, None) } else { (selected, Some(field)) }
            };
            let name = match shorthand {
                Some(field) => field,
                None if matches!(self.peek().text.as_str(), "[" | "{") => {
                    let holder = self.pattern_holder();
                    let (pattern, slot) = Self::nested_pattern(&mut frame, &holder);
                    machine.request(Work::PatternNested { frame, element: Box::new(PatternElement { start, selected, name: holder, slot }) }, pattern);
                    return Ok(());
                }
                None => self.name()?,
            };
            if self.is("=") {
                let operator = self.take().span;
                machine.request(Work::PatternNameDefault { frame, element: Box::new(PatternElement { start, selected, name, slot: 0 }), operator }, Work::Parse(0));
                return Ok(());
            }
            self.pattern_name(&mut frame, start, name, selected)?;
        }
        machine.bodies.push(frame.output);
        Ok(())
    }

    fn pattern_name(&mut self, frame: &mut PatternFrame, start: Span, name: Token, mut selected: Expr) -> Result<(), Diagnostic> {
        // The selection carries the span of its name, which marks the
        // declaration as a pattern position for editor type facts.
        selected.span = name.span;
        frame.output.push(statement(
            StmtKind::Let { rebindable: frame.rebindable, name, annotation: None, value: selected, binding: None },
            start,
        ));
        self.pattern_element_end(frame)
    }

    /// Request a nested pattern. It appends to the same declaration list,
    /// after one reserved position for the declaration of its hidden holder.
    /// The holder's value is complete only after the nested pattern.
    fn nested_pattern(frame: &mut PatternFrame, holder: &Token) -> (Work, usize) {
        let mut output = std::mem::take(&mut frame.output);
        let slot = output.len();
        output.push(statement(StmtKind::Break, holder.span));
        (Work::Pattern { whole: holder.text.clone(), rebindable: frame.rebindable, depth: frame.depth + 1, output }, slot)
    }

    fn pattern_nested(&mut self, mut frame: Box<PatternFrame>, start: Span, selected: Expr, holder: Token, slot: usize, machine: &mut Machine) -> Result<(), Diagnostic> {
        frame.output[slot] = statement(
            StmtKind::Let { rebindable: false, name: holder, annotation: None, value: selected, binding: None },
            start,
        );
        self.pattern_element_end(&mut frame)?;
        machine.work.push(Work::PatternNext(frame));
        Ok(())
    }

    fn pattern_element_end(&mut self, frame: &mut PatternFrame) -> Result<(), Diagnostic> {
        frame.position += 1;
        let close = if frame.array { "]" } else { "}" };
        if !self.is(close) {
            self.expect(",")?;
        }
        Ok(())
    }

    /// `selected = fallback` inside a pattern: the fallback replaces undefined only.
    fn pattern_default(selected: Expr, operator: Span, fallback: Expr) -> Expr {
        let span = selected.span;
        let selected = Expr::new(ExprKind::Unary { op: UnaryOp::PatternSelection, operator, value: Box::new(selected) }, span);
        Expr::new(ExprKind::Binary { op: BinaryOp::PatternDefault, operator, left: Box::new(selected), right: Box::new(fallback) }, span)
    }

    /// Rest syntax explicitly requests the independent copy provided by slice.
    /// A nested rest pattern has already appended its declarations after `slot`.
    fn array_rest_pattern(&mut self, frame: &mut PatternFrame, start: Span, name: Token, slot: Option<usize>) -> Result<(), Diagnostic> {
        if !self.is("]") {
            return Err(Diagnostic::new("syntax", "A rest binding must be last and cannot have a default or trailing comma.", self.peek().span));
        }
        let source = Expr::new(ExprKind::Name(frame.whole.clone()), name.span);
        let method = Expr::new(ExprKind::Field(Box::new(source), "pattern#rest".into()), name.span);
        let index = Expr::new(ExprKind::Integer(frame.position.to_string()), name.span);
        let value = Expr::new(ExprKind::Call(Box::new(method), vec![index]), name.span);
        let childless = slot.is_none_or(|slot| slot + 1 == frame.output.len());
        let rest = statement(
            StmtKind::Let { rebindable: frame.rebindable && childless, name, annotation: None, value, binding: None },
            start,
        );
        match slot {
            Some(slot) => frame.output[slot] = rest,
            None => frame.output.push(rest),
        }
        Ok(())
    }

    /// One parameter up to its name. A destructuring pattern is held under a
    /// hidden name; its positions open the body.
    fn parameter(&mut self, frame: Box<ParametersFrame>, machine: &mut Machine) -> Result<(), Diagnostic> {
        let rest = self.accept("...");
        if matches!(self.peek().text.as_str(), "[" | "{") {
            let whole = self.pattern_holder();
            let pattern = Work::Pattern { whole: whole.text.clone(), rebindable: false, depth: 0, output: Vec::new() };
            machine.request(Work::ParameterPattern { frame, rest, whole: Box::new(whole) }, pattern);
            return Ok(());
        }
        let name = self.name()?;
        self.parameter_named(frame, rest, name, false, machine)
    }

    fn parameter_named(&mut self, frame: Box<ParametersFrame>, rest: bool, name: Token, pattern: bool, machine: &mut Machine) -> Result<(), Diagnostic> {
        let optional = if self.accept("?") { Some(self.tokens[self.i - 1].span) } else { None };
        let annotated = if frame.declared {
            self.expect(":")?;
            true
        } else {
            self.accept(":")
        };
        let (ty, mut access) = if !annotated {
            (ParameterAnnotation::Inferred(name.clone()), None)
        } else if self.access_wrapper_start() {
            self.wrapped_parameter_annotation()?
        } else {
            (self.parameter_annotation()?, None)
        };
        // A destructured parameter without a stated access is read: its names
        // borrow the argument's positions. `Owned<T>` lets them move out.
        if pattern && access.is_none() {
            access = Some(Self::pattern_access(&name));
        }
        if self.accept("=") {
            let operator = self.tokens[self.i - 1].span;
            if optional.is_some() {
                return Err(Diagnostic::new("parameter-initializer", "An optional parameter cannot also have a default initializer.", operator));
            }
            let declared = self.pending.len();
            let head = Box::new(ParameterHead { rest, name, ty, access, operator, declared });
            machine.request(Work::ParameterDefault { frame, head }, Work::Parse(0));
            return Ok(());
        }
        let initializer = optional.map_or(ParameterInitializer::Required, ParameterInitializer::Optional);
        self.parameter_finish(frame, CallableParameter { rest, name, ty, access, initializer }, machine)
    }

    fn parameter_finish(&mut self, mut frame: Box<ParametersFrame>, parameter: CallableParameter, machine: &mut Machine) -> Result<(), Diagnostic> {
        Self::rest_initializer(parameter.rest, &parameter.initializer, parameter.name.span)?;
        let rest = parameter.rest;
        frame.params.push(parameter);
        if rest && self.is(",") {
            return Err(Diagnostic::new("rest-parameter", "A rest parameter must be last.", self.peek().span));
        }
        if self.accept(",") {
            self.parameter(frame, machine)
        } else {
            self.parameters_finish(frame, machine)
        }
    }

    fn parameters_finish(&mut self, frame: Box<ParametersFrame>, machine: &mut Machine) -> Result<(), Diagnostic> {
        self.expect(")")?;
        Self::parameter_order(frame.params.iter().filter(|parameter| !parameter.rest).map(|parameter| &parameter.initializer), frame.params.iter().map(|parameter| parameter.name.span))?;
        machine.parameters.push(frame.params);
        Ok(())
    }
}
