//! Statement and control-flow grammar steps of the parser scheduler.
use super::match_syntax::{ParsedArm, ParsedMatch};
use super::scheduler::{DeclarationFrame, ForFrame, ForOfFrame, IfFrame, Machine, MatchFrame, StatementExpression, SwitchClause, SwitchFrame, Work};
use super::*;

fn statement(kind: StmtKind, span: Span) -> Stmt {
    Stmt { labels: Vec::new(), control_target: None, kind, span }
}

impl Parser<'_> {
    fn block_limit(span: Span) -> Diagnostic {
        Diagnostic::new("unsupported", "Block nesting exceeds the prototype limit of 128.", span)
    }

    pub(super) fn statement_step(&mut self, task: Work, machine: &mut Machine) -> Result<(), Diagnostic> {
        match task {
            Work::SwitchNext(frame) => self.switch_next(frame, machine)?,
            Work::Block => {
                if self.policy.reached(self.depth) {
                    return Err(Self::block_limit(self.peek().span));
                }
                self.expect("{")?;
                self.depth += 1;
                // Declarations pending for the enclosing statement list, such
                // as the hidden declaration of a named function expression in
                // an `if` condition, do not belong to this block.
                let outer = std::mem::take(&mut self.pending);
                machine.work.push(Work::BlockNext { body: Vec::new(), outer });
            }
            Work::BlockNext { body, outer } => {
                if self.accept("}") {
                    self.depth -= 1;
                    self.pending = outer;
                    machine.bodies.push(body);
                } else {
                    if self.peek().kind == TokenKind::End
                        && let Err(error) = self.expect("}")
                    {
                        self.pending = outer;
                        return Err(error);
                    }
                    let recovery = self.recovery(machine);
                    machine.request(Work::BlockAfter { body, outer, recovery }, Work::Statement);
                }
            }
            Work::BlockAfter { mut body, outer, .. } => {
                body.push(machine.statement());
                body.append(&mut self.pending);
                machine.work.push(Work::BlockNext { body, outer });
            }
            // The body of if, else, while, do, for or for...of: a block, or
            // one statement that forms a block of its own.
            Work::Body => {
                if self.is("{") {
                    machine.work.push(Work::Block);
                    return Ok(());
                }
                if self.policy.reached(self.depth) {
                    return Err(Self::block_limit(self.peek().span));
                }
                if self.is(";") {
                    return Err(Diagnostic::new("syntax", "An empty statement cannot be a body; write {} for an empty body.", self.peek().span));
                }
                if self.is("const") || self.is("let") || self.is("function") {
                    return Err(Diagnostic::new("syntax", "A declaration cannot be a body without braces; put it in a block.", self.peek().span));
                }
                self.depth += 1;
                let outer = std::mem::take(&mut self.pending);
                machine.request(Work::BodyStatement { outer }, Work::Statement);
            }
            Work::BodyStatement { outer } => {
                self.depth -= 1;
                let mut body = vec![machine.statement()];
                body.append(&mut self.pending);
                self.pending = outer;
                machine.bodies.push(body);
            }
            Work::Statement => self.statement(machine)?,
            Work::StatementBlock(span) => {
                let body = machine.body();
                machine.statements.push(statement(StmtKind::Block(body), span));
            }
            Work::StatementExpression { span, kind } => {
                let expression = machine.value();
                let kind = match kind {
                    StatementExpression::Return => StmtKind::Return(Some(expression)),
                    StatementExpression::Throw => StmtKind::Throw(expression),
                    StatementExpression::Discard => {
                        if matches!(&expression.kind, ExprKind::Assign { target, .. } if matches!(target.kind, ExprKind::Array(_) | ExprKind::Record(_))) {
                            self.destructuring_assignment(expression)?
                        } else {
                            Self::expression_statement(expression)
                        }
                    }
                };
                self.expect(";")?;
                machine.statements.push(statement(kind, span));
            }
            Work::Labeled { labels, classic_for } => {
                let mut labeled = machine.statement();
                if classic_for {
                    if let StmtKind::Block(body) = &mut labeled.kind {
                        body.last_mut().expect("classic-for loop").labels = labels;
                    }
                } else if matches!(labeled.kind, StmtKind::While { .. } | StmtKind::Loop { .. } | StmtKind::ForOf { .. } | StmtKind::Block(_)) {
                    labeled.labels = labels;
                } else {
                    let span = labeled.span;
                    labeled = Stmt { labels, control_target: None, kind: StmtKind::Block(vec![labeled]), span };
                }
                machine.statements.push(labeled);
            }
            Work::If => {
                let span = self.expect("if")?.span;
                self.expect("(")?;
                machine.request_comma(Work::IfCondition(span));
            }
            Work::IfCondition(span) => {
                let condition = machine.value();
                self.expect(")")?;
                machine.request(Work::IfThen { span, condition: Box::new(condition) }, Work::Body);
            }
            Work::IfThen { span, condition } => {
                let then_body = machine.body();
                if !self.accept("else") {
                    machine.statements.push(Self::conditional_statement(span, *condition, then_body, Vec::new()));
                    return Ok(());
                }
                let frame = Box::new(IfFrame { span, condition: *condition, then_body });
                if !self.is("if") {
                    machine.request(Work::IfElse(frame), Work::Body);
                    return Ok(());
                }
                if self.policy.reached(self.depth) {
                    return Err(Diagnostic::new("unsupported", "Conditional nesting exceeds the prototype limit of 128.", self.peek().span));
                }
                self.depth += 1;
                machine.request(Work::IfElseIf(frame), Work::If);
            }
            Work::IfElse(frame) => {
                let else_body = machine.body();
                machine.statements.push(Self::conditional_statement(frame.span, frame.condition, frame.then_body, else_body));
            }
            Work::IfElseIf(frame) => {
                self.depth -= 1;
                let nested = machine.statement();
                machine.statements.push(Self::conditional_statement(frame.span, frame.condition, frame.then_body, vec![nested]));
            }
            Work::WhileCondition(span) => {
                let condition = machine.value();
                self.expect(")")?;
                machine.request(Work::WhileBody { span, condition: Box::new(condition) }, Work::Body);
            }
            Work::WhileBody { span, condition } => {
                let body = machine.body();
                machine.statements.push(statement(StmtKind::While { condition: *condition, body }, span));
            }
            Work::DoBody(span) => {
                let body = machine.body();
                self.expect("while")?;
                self.expect("(")?;
                machine.request_comma(Work::DoCondition { span, body });
            }
            Work::DoCondition { span, body } => {
                let condition = machine.value();
                self.expect(")")?;
                self.expect(";")?;
                machine.statements.push(statement(
                    StmtKind::Loop { condition: Some(condition), body, update: Vec::new(), post_test: true, tail_reachable: true },
                    span,
                ));
            }
            Work::TryBody(start) => {
                let body = machine.body();
                if self.accept("catch") {
                    let name = if self.accept("(") {
                        let name = self.name()?;
                        self.expect(")")?;
                        Some(name)
                    } else {
                        None
                    };
                    machine.request(Work::TryCatch { start, body, name: name.map(Box::new) }, Work::Block);
                } else {
                    self.try_finally(start, body, None, machine)?;
                }
            }
            Work::TryCatch { start, body, name } => {
                let catch = CatchClause { error_type: None, error_contract: None, name: name.map(|name| *name), binding: None, body: machine.body() };
                self.try_finally(start, body, Some(catch), machine)?;
            }
            Work::TryFinally { start, body, catch } => {
                let finally = Some(machine.body());
                self.try_finish(start, body, catch.map(|catch| *catch), finally, machine)?;
            }
            Work::ForOfPattern { span, whole } => {
                let positions = machine.body();
                self.for_of_binding(span, *whole, false, positions, machine)?;
            }
            Work::ForOfIterable(frame) => {
                let iterable = machine.value();
                let iterable = if frame.keys { Self::object_key_call("keys#in", vec![iterable], frame.span) } else { iterable };
                self.expect(")")?;
                machine.request(Work::ForOfBody(Box::new((frame.name, frame.rebindable, iterable, frame.positions, frame.span))), Work::Body);
            }
            Work::ForOfBody(header) => {
                let body = machine.body();
                machine.statements.push(Self::for_of_loop(header, body));
            }
            Work::ForInitializerList(span) => {
                let scoped = machine.body();
                self.for_condition(span, scoped, machine)?;
            }
            Work::ForInitializerDeclaration(span) => {
                let initializer = machine.statement();
                if !matches!(initializer.kind, StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_)) {
                    return Err(Diagnostic::new(
                        "syntax",
                        "A for initializer must be a declaration, assignment, or expression.",
                        initializer.span,
                    ));
                }
                let mut scoped = vec![initializer];
                scoped.append(&mut self.pending);
                self.for_condition(span, scoped, machine)?;
            }
            Work::ForCondition { span, mut scoped } => {
                // The loop tests its condition inside the scoped block.
                scoped.append(&mut self.pending);
                let condition = Some(Box::new(machine.value()));
                self.for_update(span, scoped, condition, machine)?;
            }
            Work::ForUpdate { span, scoped, condition } => {
                let update = machine.body();
                self.for_body(span, scoped, condition, update, machine)?;
            }
            Work::ForBody(frame) => {
                let ForFrame { span, mut scoped, condition, update } = *frame;
                let body = machine.body();
                scoped.push(statement(
                    StmtKind::Loop { condition: condition.map(|condition| *condition), body, update, post_test: false, tail_reachable: true },
                    span,
                ));
                machine.statements.push(statement(StmtKind::Block(scoped), span));
            }
            Work::ExpressionList(list) => {
                if matches!(self.peek().text.as_str(), "let" | "const" | "var") {
                    return Err(Diagnostic::new("syntax", "A declaration cannot be a comma operand.", self.peek().span));
                }
                machine.request(Work::ExpressionListItem(list), Work::Parse(0));
            }
            Work::ExpressionListItem(mut list) => {
                let expression = machine.value();
                let span = expression.span;
                let pattern = matches!(&expression.kind, ExprKind::Assign { target, .. } if matches!(target.kind, ExprKind::Array(_) | ExprKind::Record(_)));
                let kind = if pattern {
                    self.destructuring_assignment(expression)?
                } else if matches!(expression.kind, ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. }) {
                    Self::expression_statement(expression)
                } else {
                    StmtKind::Expr(Expr::new(ExprKind::Sequence(vec![expression]), span))
                };
                list.push(statement(kind, span));
                list.append(&mut self.pending);
                if self.accept(",") {
                    machine.work.push(Work::ExpressionList(list));
                } else {
                    machine.bodies.push(list);
                }
            }
            Work::SwitchDiscriminant(span) => {
                let discriminant = machine.value();
                self.expect(")")?;
                self.expect("{")?;
                if self.policy.reached(self.depth) {
                    return Err(Self::block_limit(span));
                }
                self.depth += 1;
                let outer = std::mem::take(&mut self.pending);
                self.switch_next(Box::new(SwitchFrame { span, discriminant, outer, clauses: Vec::new(), default_seen: false }), machine)?;
            }
            Work::SwitchCaseValue { mut frame, mut clause } => {
                // A case value is tested outside its clause body.
                frame.outer.append(&mut self.pending);
                clause.values.push(machine.value());
                self.expect(":")?;
                self.switch_labels(frame, clause, machine)?;
            }
            Work::SwitchBodyAfter { frame, mut clause } => {
                clause.body.push(machine.statement());
                clause.body.append(&mut self.pending);
                self.switch_body(frame, clause, machine)?;
            }
            Work::MatchSyntax => {
                let start = self.expect("match")?.span.start;
                self.expect("(")?;
                machine.request_comma(Work::MatchScrutinee(start));
            }
            Work::MatchScrutinee(start) => {
                let scrutinee = machine.value();
                self.expect(")")?;
                self.expect("{")?;
                machine.work.push(Work::MatchNext(Box::new(MatchFrame { start, scrutinee, arms: Vec::new() })));
            }
            Work::MatchNext(frame) => {
                if self.accept("}") {
                    let MatchFrame { start, scrutinee, arms } = *frame;
                    machine.matches.push(ParsedMatch { scrutinee, arms, span: Span::new(start, self.tokens[self.i - 1].span.end) });
                    return Ok(());
                }
                if self.peek().kind == TokenKind::End {
                    self.expect("}")?;
                }
                if self.policy.reached(self.depth + frame.arms.len()) {
                    return Err(Diagnostic::new("unsupported", "Match arm nesting exceeds the prototype limit of 128.", self.peek().span));
                }
                let arm_start = self.peek().span.start;
                let pattern = self.match_pattern()?;
                self.expect("=>")?;
                let body_start = self.peek().span.start;
                if self.is("{") {
                    machine.request(Work::MatchArm { frame, pattern: Box::new(pattern), arm_start, body_start }, Work::Block);
                } else {
                    machine.request(Work::MatchArmValue { frame, pattern: Box::new(pattern), arm_start, body_start }, Work::Parse(0));
                }
            }
            Work::MatchArm { mut frame, pattern, arm_start, body_start } => {
                let body = machine.body();
                let end = self.tokens[self.i - 1].span.end;
                frame.arms.push(ParsedArm { pattern: *pattern, body, value: None, body_span: Span::new(body_start, end), span: Span::new(arm_start, end) });
                // Rust-style block arms may omit their separator; a trailing comma
                // is accepted. A doubled comma remains a malformed next pattern.
                self.accept(",");
                machine.work.push(Work::MatchNext(frame));
            }
            Work::MatchArmValue { mut frame, pattern, arm_start, body_start } => {
                let value = machine.value();
                let end = value.span.end;
                frame.arms.push(ParsedArm { pattern: *pattern, body: Vec::new(), value: Some(value), body_span: Span::new(body_start, end), span: Span::new(arm_start, end) });
                if !self.is("}") { self.expect(",")?; }
                machine.work.push(Work::MatchNext(frame));
            }
            Work::LowerMatchValue(holder) => {
                let parsed = machine.matches.pop().expect("completed match expression");
                machine.values.push(self.lower_match_value(*holder, parsed)?);
                machine.work.push(Work::Postfix { optional: false });
            }
            Work::LowerMatch(holder) => {
                let parsed = machine.matches.pop().expect("completed match statement");
                let lowered = self.lower_match_statement(*holder, parsed);
                machine.statements.push(lowered);
            }
            _ => unreachable!("statement work"),
        }
        Ok(())
    }

    /// Select one statement form. Every form completes with one statement.
    fn statement(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        if self.label_follows() {
            return self.labeled_statement(machine);
        }
        let span = self.peek().span;
        if self.is("function") {
            self.local_function(machine)
        } else if self.match_statement_header() {
            let mut holder = self.pattern_holder();
            holder.text = holder.text.replace("pattern#", "match#");
            machine.request(Work::LowerMatch(Box::new(holder)), Work::MatchSyntax);
            Ok(())
        } else if self.is("try") {
            let start = self.expect("try")?.span;
            machine.request(Work::TryBody(start), Work::Block);
            Ok(())
        } else if self.is("for") {
            if self.for_of_header() { self.for_of_statement(machine) } else { self.for_statement(machine) }
        } else if self.is("if") {
            machine.work.push(Work::If);
            Ok(())
        } else if self.is("while") {
            let span = self.expect("while")?.span;
            self.expect("(")?;
            machine.request_comma(Work::WhileCondition(span));
            Ok(())
        } else if self.is("do") {
            let span = self.expect("do")?.span;
            machine.request(Work::DoBody(span), Work::Body);
            Ok(())
        } else if self.is("switch") && self.tokens.get(self.i + 1).is_some_and(|token| token.text == "(") {
            let span = self.expect("switch")?.span;
            self.expect("(")?;
            machine.request_comma(Work::SwitchDiscriminant(span));
            Ok(())
        } else if self.is("{") {
            machine.request(Work::StatementBlock(span), Work::Block);
            Ok(())
        } else {
            self.simple_statement(true, machine)
        }
    }

    /// A declaration, a jump or an expression statement.
    fn simple_statement(&mut self, terminated: bool, machine: &mut Machine) -> Result<(), Diagnostic> {
        let span = self.peek().span;
        if self.is("const") || self.is("let") {
            let rebindable = self.take().text == "let";
            let frame = Box::new(DeclarationFrame { span, rebindable, terminated, first: None, listed: None });
            return self.declarator(frame, machine);
        }
        if self.is("break") || self.is("continue") {
            let kind = if self.take().text == "break" { StmtKind::Break } else { StmtKind::Continue };
            let label = if self.peek().kind == TokenKind::Name { Some(self.name()?) } else { None };
            self.expect(";")?;
            machine.statements.push(Stmt { labels: label.into_iter().collect(), control_target: None, kind, span });
            return Ok(());
        }
        if self.accept("throw") {
            if self.source[span.end..self.peek().span.start].contains(['\n', '\r']) {
                return Err(Diagnostic::new("syntax", "A thrown value must begin on the same line as throw.", span));
            }
            machine.request_comma(Work::StatementExpression { span, kind: StatementExpression::Throw });
        } else if self.accept("return") {
            if self.accept(";") {
                machine.statements.push(statement(StmtKind::Return(None), span));
            } else {
                machine.request_comma(Work::StatementExpression { span, kind: StatementExpression::Return });
            }
        } else {
            machine.request_comma(Work::StatementExpression { span, kind: StatementExpression::Discard });
        }
        Ok(())
    }

    /// `name:` starts a labeled statement. `return` and `throw` always start
    /// their own statement, in every body.
    fn label_follows(&self) -> bool {
        self.peek().kind == TokenKind::Name
            && !matches!(self.peek().text.as_str(), "return" | "throw")
            && self.tokens.get(self.i + 1).is_some_and(|token| token.text == ":")
    }

    fn labeled_statement(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        let mut labels = Vec::new();
        while self.label_follows() {
            labels.push(self.name()?);
            self.expect(":")?;
            if labels.len() > 32 {
                return Err(Diagnostic::new("unsupported", "A statement supports at most 32 labels.", labels[0].span));
            }
        }
        if matches!(self.peek().text.as_str(), "let" | "const" | "type" | "function") {
            return Err(Diagnostic::new("syntax", "A label must precede a statement, not a declaration.", self.peek().span));
        }
        if self.is(";") {
            let span = self.take().span;
            machine.statements.push(Stmt { labels, control_target: None, kind: StmtKind::Block(Vec::new()), span });
            return Ok(());
        }
        let classic_for = self.is("for") && !self.for_of_header();
        machine.request(Work::Labeled { labels, classic_for }, Work::Statement);
        Ok(())
    }

    fn try_finally(&mut self, start: Span, body: Vec<Stmt>, catch: Option<CatchClause>, machine: &mut Machine) -> Result<(), Diagnostic> {
        if self.accept("finally") {
            machine.request(Work::TryFinally { start, body, catch: catch.map(Box::new) }, Work::Block);
            Ok(())
        } else {
            self.try_finish(start, body, catch, None, machine)
        }
    }

    fn try_finish(&mut self, start: Span, body: Vec<Stmt>, catch: Option<CatchClause>, finally: Option<Vec<Stmt>>, machine: &mut Machine) -> Result<(), Diagnostic> {
        if catch.is_none() && finally.is_none() {
            return Err(Diagnostic::new("syntax", "A try block requires catch, finally, or both.", start));
        }
        let span = Span::new(start.start, self.tokens[self.i - 1].span.end);
        machine.statements.push(statement(StmtKind::Try(Box::new(ExceptionRegion { body, catch, finally })), span));
        Ok(())
    }

    fn for_of_statement(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        let span = self.expect("for")?.span;
        self.expect("(")?;
        if !self.is("const") && !self.is("let") {
            return Err(Diagnostic::new("unsupported", "for...of requires a const or let iteration binding.", self.peek().span));
        }
        let rebindable = self.take().text == "let";
        // A pattern iterates a hidden whole element; its names are declared
        // first in the body and borrow the element's positions.
        if matches!(self.peek().text.as_str(), "[" | "{") {
            let whole = self.pattern_holder();
            let pattern = Work::Pattern { whole: whole.text.clone(), rebindable, depth: 0, output: Vec::new() };
            machine.request(Work::ForOfPattern { span, whole: Box::new(whole) }, pattern);
            return Ok(());
        }
        let name = self.name()?;
        self.for_of_binding(span, name, rebindable, Vec::new(), machine)
    }

    fn for_of_binding(&mut self, span: Span, name: Token, rebindable: bool, positions: Vec<Stmt>, machine: &mut Machine) -> Result<(), Diagnostic> {
        let keys = self.is("in");
        if keys && !positions.is_empty() {
            return Err(Diagnostic::new("unsupported", "for...in requires a named binding.", name.span));
        }
        if keys {
            self.expect("in")?;
        } else {
            self.expect("of")?;
        }
        let parent = Work::ForOfIterable(Box::new(ForOfFrame { span, name, rebindable, positions, keys }));
        if keys {
            machine.request_comma(parent);
        } else {
            machine.request(parent, Work::Parse(0));
        }
        Ok(())
    }

    fn for_statement(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        let span = self.expect("for")?.span;
        self.expect("(")?;
        if !self.is(";") && !matches!(self.peek().text.as_str(), "let" | "const") {
            machine.request(Work::ForInitializerList(span), Work::ExpressionList(Vec::new()));
            Ok(())
        } else if !self.is(";") {
            machine.work.push(Work::ForInitializerDeclaration(span));
            self.simple_statement(false, machine)
        } else {
            self.for_condition(span, Vec::new(), machine)
        }
    }

    fn for_condition(&mut self, span: Span, scoped: Vec<Stmt>, machine: &mut Machine) -> Result<(), Diagnostic> {
        self.expect(";")?;
        if self.is(";") {
            self.for_update(span, scoped, None, machine)
        } else {
            machine.request_comma(Work::ForCondition { span, scoped });
            Ok(())
        }
    }

    /// The update clause: one or more assignments or expressions separated
    /// by commas, run in source order on every continuing edge.
    fn for_update(&mut self, span: Span, scoped: Vec<Stmt>, condition: Option<Box<Expr>>, machine: &mut Machine) -> Result<(), Diagnostic> {
        self.expect(";")?;
        if self.is(")") {
            self.for_body(span, scoped, condition, Vec::new(), machine)
        } else {
            machine.request(Work::ForUpdate { span, scoped, condition }, Work::ExpressionList(Vec::new()));
            Ok(())
        }
    }

    fn for_body(&mut self, span: Span, scoped: Vec<Stmt>, condition: Option<Box<Expr>>, update: Vec<Stmt>, machine: &mut Machine) -> Result<(), Diagnostic> {
        self.expect(")")?;
        machine.request(Work::ForBody(Box::new(ForFrame { span, scoped, condition, update })), Work::Body);
        Ok(())
    }

    /// `switch (value) { case a: ... default: ... }` becomes an if chain over
    /// strict equality, tested in source order. The discriminant is evaluated
    /// once. Each case ends with break, return or continue, or groups with the
    /// next case; running from one case body into the next is not supported.
    fn switch_next(&mut self, frame: Box<SwitchFrame>, machine: &mut Machine) -> Result<(), Diagnostic> {
        if self.accept("}") {
            return self.switch_finish(*frame, machine);
        }
        let at = self.peek().span;
        if !self.is("case") && !self.is("default") {
            return Err(Diagnostic::new("syntax", "Expected case or default in a switch body.", at));
        }
        self.switch_labels(frame, Box::new(SwitchClause { at, values: Vec::new(), default: false, body: Vec::new() }), machine)
    }

    // Adjacent labels share one body.
    fn switch_labels(&mut self, mut frame: Box<SwitchFrame>, mut clause: Box<SwitchClause>, machine: &mut Machine) -> Result<(), Diagnostic> {
        while self.is("case") || self.is("default") {
            if self.take().text == "default" {
                if frame.default_seen {
                    return Err(Diagnostic::new("syntax", "A switch can have one default clause.", clause.at));
                }
                frame.default_seen = true;
                clause.default = true;
            } else {
                machine.request_comma(Work::SwitchCaseValue { frame, clause });
                return Ok(());
            }
            self.expect(":")?;
        }
        self.switch_body(frame, clause, machine)
    }

    fn switch_body(&mut self, mut frame: Box<SwitchFrame>, clause: Box<SwitchClause>, machine: &mut Machine) -> Result<(), Diagnostic> {
        if !self.is("case") && !self.is("default") && !self.is("}") {
            if self.peek().kind == TokenKind::End {
                self.expect("}")?;
            }
            machine.request(Work::SwitchBodyAfter { frame, clause }, Work::Statement);
            return Ok(());
        }
        let clause = *clause;
        frame.clauses.push((clause.values, clause.default, clause.body, clause.at));
        machine.work.push(Work::SwitchNext(frame));
        Ok(())
    }

    fn switch_finish(&mut self, frame: SwitchFrame, machine: &mut Machine) -> Result<(), Diagnostic> {
        let SwitchFrame { span, discriminant, mut clauses, outer, .. } = frame;
        self.depth -= 1;
        self.pending = outer;
        if self.policy.bounded() && clauses.len() > 120 {
            return Err(Diagnostic::new("unsupported", "A switch supports at most 120 clauses.", span));
        }
        let last = clauses.len().saturating_sub(1);
        for (index, (_, _, body, at)) in clauses.iter_mut().enumerate() {
            let ended = Self::end_switch_clause(body);
            if !ended && index != last && !body.is_empty() {
                return Err(Diagnostic::new(
                    "unsupported",
                    "This case runs into the next one. End it with break, return or continue, or leave it empty to share the next body.",
                    *at,
                ));
            }
            if let Some(nested) = self.switch_break(body) {
                return Err(Diagnostic::new(
                    "unsupported",
                    "A break inside a nested statement of a case is not supported; make break the last statement of the case, or restructure with if and else.",
                    nested,
                ));
            }
        }
        // A named discriminant is compared in place; any other value is held once.
        let mut held = None;
        let subject = if matches!(discriminant.kind, ExprKind::Name(_)) {
            discriminant
        } else {
            let holder = self.pattern_holder();
            let name = Expr::new(ExprKind::Name(holder.text.clone()), discriminant.span);
            held = Some(statement(StmtKind::Let { rebindable: false, name: holder, annotation: None, value: discriminant, binding: None }, span));
            name
        };
        // Build the chain from the last clause back; default is the final else.
        let mut default_body = None;
        let mut chain: Vec<Stmt> = Vec::new();
        let mut tested = Vec::new();
        for (values, default, body, at) in clauses {
            if default {
                default_body = Some(body);
            } else {
                tested.push((values, body, at));
            }
        }
        if let Some(body) = default_body {
            chain = body;
        }
        for (values, body, at) in tested.into_iter().rev() {
            let mut condition: Option<Expr> = None;
            for value in values {
                let value_span = value.span;
                let equal = Expr::new(
                    ExprKind::Binary {
                        op: BinaryOp::Equal,
                        operator: value_span,
                        left: Box::new(Expr::new(subject.kind.clone(), subject.span)),
                        right: Box::new(value),
                    },
                    value_span,
                );
                condition = Some(match condition {
                    None => equal,
                    Some(previous) => Expr::new(
                        ExprKind::Binary { op: BinaryOp::Or, operator: value_span, left: Box::new(previous), right: Box::new(equal) },
                        value_span,
                    ),
                });
            }
            let condition = condition.expect("case clause has a value");
            chain = vec![statement(StmtKind::If { condition, then_body: body, else_body: chain }, at)];
        }
        // The statements built here need no second break check when an
        // enclosing switch validates its own clause.
        self.validated_switches.insert(span.start);
        self.validated_switches.extend(chain.iter().filter(|lowered| matches!(lowered.kind, StmtKind::If { .. })).map(|lowered| lowered.span.start));
        machine.statements.push(match held {
            Some(first) => {
                self.pending.append(&mut chain);
                first
            }
            None => statement(StmtKind::Block(chain), span),
        });
        Ok(())
    }

    /// Remove the break that ends a clause, also when it ends the clause's
    /// final block. Report whether the clause ends explicitly.
    fn end_switch_clause(mut body: &mut Vec<Stmt>) -> bool {
        loop {
            if body.last().is_some_and(|statement| matches!(statement.kind, StmtKind::Break) && !statement.labels.is_empty()) {
                return true;
            }
            match body.last().map(|statement| &statement.kind) {
                Some(StmtKind::Break) => {
                    body.pop();
                    return true;
                }
                Some(StmtKind::Return(_) | StmtKind::Continue) => return true,
                Some(StmtKind::Block(_)) => {}
                _ => return false,
            }
            let Some(StmtKind::Block(inner)) = body.last_mut().map(|statement| &mut statement.kind) else {
                unreachable!("final block of a switch clause")
            };
            body = inner;
        }
    }

    /// A break that would leave the switch from inside a nested statement.
    /// Loops own their breaks.
    fn switch_break(&self, body: &[Stmt]) -> Option<Span> {
        let mut pending = vec![body.iter()];
        while let Some(statements) = pending.last_mut() {
            let Some(statement) = statements.next() else {
                pending.pop();
                continue;
            };
            match &statement.kind {
                StmtKind::Break if statement.labels.is_empty() => return Some(statement.span),
                StmtKind::Block(_) | StmtKind::If { .. } if self.validated_switches.contains(&statement.span.start) => {}
                StmtKind::Block(inner) => pending.push(inner.iter()),
                StmtKind::If { then_body, else_body, .. } => {
                    pending.push(else_body.iter());
                    pending.push(then_body.iter());
                }
                _ => {}
            }
        }
        None
    }
}
