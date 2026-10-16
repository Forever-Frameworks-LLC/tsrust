//! One scheduler for the whole grammar.
//!
//! Every nested construct is a request on one heap work stack: expressions,
//! statements, bodies, declarations, binding patterns, parameter lists and
//! their defaults. A grammar step never calls another grammar step that waits
//! for a child; it pushes its continuation, then the child request, and
//! returns. The native call depth is therefore independent of source nesting.
//! Token position, pending declarations and grammar scopes stay in `Parser`.
use super::match_syntax::ParsedMatch;
use super::*;

pub(super) struct BinaryFrame {
    pub(super) minimum: u8,
    pub(super) left_start: usize,
    pub(super) unary_base: bool,
}

pub(super) enum Callee {
    Value {
        callee: Expr,
        types: Option<Vec<crate::type_syntax::ValueAnnotation>>,
        optional: bool,
    },
    /// `new Name<...>(` up to its opening parenthesis.
    Constructor {
        start: Span,
        native: bool,
        map: bool,
        annotation: Option<Box<crate::type_syntax::ValueAnnotation>>,
    },
}

pub(super) struct CallFrame {
    pub(super) target: Callee,
    pub(super) arguments: Vec<Expr>,
}

pub(super) struct RightFrame {
    pub(super) frame: BinaryFrame,
    pub(super) left: Expr,
    pub(super) op: BinaryOp,
    pub(super) operator: Span,
    pub(super) assignment: bool,
    pub(super) right_start: usize,
}

/// One pattern element whose nested pattern or default is still open.
pub(super) struct PatternElement {
    pub(super) start: Span,
    pub(super) selected: Expr,
    /// The element's own name, or the hidden holder of a nested pattern.
    pub(super) name: Token,
    /// Reserved position of the holder declaration in the shared output.
    pub(super) slot: usize,
}

pub(super) struct ClosureFrame {
    pub(super) asynchronous: bool,
    pub(super) id: usize,
    pub(super) start: Span,
    pub(super) header: Box<(Vec<CallableParameter>, ResultAnnotation)>,
    pub(super) patterns: Vec<Stmt>,
    /// Pattern declarations of the enclosing parameter list. They stay out
    /// of reach until this callable and its body are complete.
    pub(super) outer_patterns: Vec<Stmt>,
}

/// Parser and result-stack extents at the start of one block statement.
pub(super) struct Recovery {
    start: usize,
    depth: usize,
    pending: usize,
    arguments: usize,
    patterns: usize,
    values: usize,
    statements: usize,
    bodies: usize,
    parameters: usize,
    functions: usize,
    matches: usize,
}

pub(super) enum StatementExpression {
    Return,
    Throw,
    Discard,
}

pub(super) struct IfFrame {
    pub(super) span: Span,
    pub(super) condition: Expr,
    pub(super) then_body: Vec<Stmt>,
}

pub(super) struct ForOfFrame {
    pub(super) span: Span,
    pub(super) name: Token,
    pub(super) rebindable: bool,
    pub(super) positions: Vec<Stmt>,
    pub(super) keys: bool,
}

pub(super) struct ForFrame {
    pub(super) span: Span,
    pub(super) scoped: Vec<Stmt>,
    pub(super) condition: Option<Box<Expr>>,
    pub(super) update: Vec<Stmt>,
}

pub(super) struct SwitchFrame {
    pub(super) span: Span,
    pub(super) discriminant: Expr,
    /// Declarations pending for the enclosing statement list, including the
    /// hidden declarations of the discriminant and the case values.
    pub(super) outer: Vec<Stmt>,
    /// Each clause: its case values (none for default) and its body.
    pub(super) clauses: Vec<(Vec<Expr>, bool, Vec<Stmt>, Span)>,
    pub(super) default_seen: bool,
}

pub(super) struct SwitchClause {
    pub(super) at: Span,
    pub(super) values: Vec<Expr>,
    pub(super) default: bool,
    pub(super) body: Vec<Stmt>,
}

pub(super) struct MatchFrame {
    pub(super) start: usize,
    pub(super) scrutinee: Expr,
    pub(super) arms: Vec<super::match_syntax::ParsedArm>,
}

/// `const a = 1, [b, c] = pair;`. The first declarator is the statement;
/// every later one follows it through `Parser::pending`.
pub(super) struct DeclarationFrame {
    pub(super) span: Span,
    pub(super) rebindable: bool,
    pub(super) terminated: bool,
    pub(super) first: Option<StmtKind>,
    /// Start of the listed declarator being parsed; none for the first.
    pub(super) listed: Option<Span>,
}

/// One `[...]` or `{...}` binding pattern and the declarations it produced.
pub(super) struct PatternFrame {
    pub(super) whole: String,
    pub(super) rebindable: bool,
    pub(super) array: bool,
    pub(super) position: usize,
    pub(super) excluded: Vec<String>,
    pub(super) output: Vec<Stmt>,
    pub(super) depth: usize,
}

pub(super) struct ParametersFrame {
    pub(super) params: Vec<CallableParameter>,
    /// A named declaration requires every parameter annotation.
    pub(super) declared: bool,
}

/// A parameter up to its `=`; the default expression is requested next.
pub(super) struct ParameterHead {
    pub(super) rest: bool,
    pub(super) name: Token,
    pub(super) ty: ParameterAnnotation,
    pub(super) access: Option<Token>,
    pub(super) operator: Span,
    pub(super) declared: usize,
}

pub(super) enum Work {
    // Blocks and bodies. Each completes with one entry on `bodies`. `outer`
    // holds the declarations that were pending for the enclosing statement
    // list; they return to `Parser::pending` when the body completes.
    Block,
    BlockNext { body: Vec<Stmt>, outer: Vec<Stmt> },
    BlockAfter { body: Vec<Stmt>, outer: Vec<Stmt>, recovery: Recovery },
    Body,
    BodyStatement { outer: Vec<Stmt> },
    // Statements. Each completes with one entry on `statements`.
    Statement,
    StatementBlock(Span),
    StatementExpression { span: Span, kind: StatementExpression },
    Labeled { labels: Vec<Token>, classic_for: bool },
    If,
    IfCondition(Span),
    IfThen { span: Span, condition: Box<Expr> },
    IfElse(Box<IfFrame>),
    IfElseIf(Box<IfFrame>),
    WhileCondition(Span),
    WhileBody { span: Span, condition: Box<Expr> },
    DoBody(Span),
    DoCondition { span: Span, body: Vec<Stmt> },
    TryBody(Span),
    TryCatch { start: Span, body: Vec<Stmt>, name: Option<Box<Token>> },
    TryFinally { start: Span, body: Vec<Stmt>, catch: Option<Box<CatchClause>> },
    ForOfPattern { span: Span, whole: Box<Token> },
    ForOfIterable(Box<ForOfFrame>),
    ForOfBody(Box<(Token, bool, Expr, Vec<Stmt>, Span)>),
    ForInitializerList(Span),
    ForInitializerDeclaration(Span),
    ForCondition { span: Span, scoped: Vec<Stmt> },
    ForUpdate { span: Span, scoped: Vec<Stmt>, condition: Option<Box<Expr>> },
    ForBody(Box<ForFrame>),
    /// A comma-separated expression list; completes with one `bodies` entry.
    ExpressionList(Vec<Stmt>),
    ExpressionListItem(Vec<Stmt>),
    SwitchDiscriminant(Span),
    SwitchNext(Box<SwitchFrame>),
    SwitchCaseValue { frame: Box<SwitchFrame>, clause: Box<SwitchClause> },
    SwitchBodyAfter { frame: Box<SwitchFrame>, clause: Box<SwitchClause> },
    /// Completes with one entry on `matches`.
    MatchSyntax,
    MatchScrutinee(usize),
    MatchNext(Box<MatchFrame>),
    MatchArm { frame: Box<MatchFrame>, pattern: Box<super::match_syntax::ParsedPattern>, arm_start: usize, body_start: usize },
    LowerMatch(Box<Token>),
    LowerMatchValue(Box<Token>),
    MatchArmValue { frame: Box<MatchFrame>, pattern: Box<super::match_syntax::ParsedPattern>, arm_start: usize, body_start: usize },
    // Declarations.
    Declarator(Box<DeclarationFrame>),
    DeclaratorValue { frame: Box<DeclarationFrame>, name: Box<(Token, Option<crate::type_syntax::ValueAnnotation>)> },
    DeclaratorPattern { frame: Box<DeclarationFrame>, whole: Box<Token>, at: usize },
    DeclaratorPatternValue { frame: Box<DeclarationFrame>, head: Box<(Token, Option<crate::type_syntax::ValueAnnotation>)>, positions: Vec<Stmt>, at: usize },
    /// A binding pattern. It appends its declarations to `output` and
    /// completes with that list on `bodies`.
    Pattern { whole: String, rebindable: bool, depth: usize, output: Vec<Stmt> },
    PatternNext(Box<PatternFrame>),
    PatternNested { frame: Box<PatternFrame>, element: Box<PatternElement> },
    PatternNestedDefault { frame: Box<PatternFrame>, element: Box<PatternElement>, operator: Span },
    PatternNameDefault { frame: Box<PatternFrame>, element: Box<PatternElement>, operator: Span },
    PatternRest { frame: Box<PatternFrame>, element: Box<PatternElement> },
    /// A parameter list after `(`; completes with one entry on `parameters`.
    Parameters(Box<ParametersFrame>),
    Parameter(Box<ParametersFrame>),
    ParameterPattern { frame: Box<ParametersFrame>, rest: bool, whole: Box<Token> },
    ParameterDefault { frame: Box<ParametersFrame>, head: Box<ParameterHead> },
    /// A named function after its keyword; completes with one `functions` entry.
    NamedFunction,
    NamedFunctionParameters { header: Box<(Token, Vec<crate::type_syntax::TypeName>)>, outer_patterns: Vec<Stmt> },
    NamedFunctionBody { function: Box<Function>, outer_patterns: Vec<Stmt> },
    LocalFunction(usize),
    NamedFunctionExpression { outer: Vec<Stmt> },
    ClosureParameters { start: Span, outer_patterns: Vec<Stmt>, asynchronous: bool },
    FunctionParameters { start: Span, outer_patterns: Vec<Stmt> },
    ClosureBlock(Box<ClosureFrame>),
    ClosureValue { frame: Box<ClosureFrame>, outer: Vec<Stmt> },
    // Expressions. Each completes with one entry on `values`.
    Parse(u8),
    LeaveDepth,
    Primary,
    Array { start: usize, items: Vec<Expr>, spread: Option<Span> },
    RecordNext { start: usize, fields: Vec<(Token, Expr)> },
    RecordField { start: usize, fields: Vec<(Token, Expr)>, key: Box<Token> },
    Template { start: usize, parts: Vec<Expr> },
    Unary(Vec<Token>),
    Binary(BinaryFrame),
    Right(Box<RightFrame>),
    In { frame: BinaryFrame, left: Box<Expr> },
    Consequent(Box<Expr>),
    Alternate(Box<(Expr, Expr)>),
    Comma(Vec<Expr>),
    Group,
    Postfix { optional: bool },
    Index { base: Box<Expr>, optional: bool },
    Argument { call: Box<CallFrame>, spread: Option<Span> },
}

/// The work stack and the completed results of its finished requests.
#[derive(Default)]
pub(super) struct Machine {
    pub(super) work: Vec<Work>,
    pub(super) values: Vec<Expr>,
    pub(super) statements: Vec<Stmt>,
    pub(super) bodies: Vec<Vec<Stmt>>,
    pub(super) parameters: Vec<Vec<CallableParameter>>,
    pub(super) functions: Vec<Function>,
    pub(super) matches: Vec<ParsedMatch>,
}

impl Machine {
    /// Queue a parent continuation and the child request it waits for.
    pub(super) fn request(&mut self, parent: Work, child: Work) {
        self.work.push(parent);
        self.work.push(child);
    }
    /// Queue a parent that waits for one comma expression.
    pub(super) fn request_comma(&mut self, parent: Work) {
        self.work.push(parent);
        self.work.push(Work::Comma(Vec::new()));
        self.work.push(Work::Parse(0));
    }
    pub(super) fn value(&mut self) -> Expr {
        self.values.pop().expect("completed expression")
    }
    pub(super) fn statement(&mut self) -> Stmt {
        self.statements.pop().expect("completed statement")
    }
    pub(super) fn body(&mut self) -> Vec<Stmt> {
        self.bodies.pop().expect("completed statement list")
    }
}

impl Parser<'_> {
    /// Run one root request to completion. Entry points are the only callers;
    /// grammar steps must queue work instead of entering a second run.
    pub(super) fn run(&mut self, root: Work) -> Result<Machine, Diagnostic> {
        assert!(!self.scheduling, "grammar work re-entered the parser scheduler");
        self.scheduling = true;
        let depth = self.depth;
        let mut machine = Machine::default();
        machine.work.push(root);
        let result = self.drive(&mut machine);
        // Recovery owns token synchronization. No abandoned request keeps an
        // active grammar-depth contribution after its work stack is dropped.
        self.depth = depth;
        self.scheduling = false;
        result.map(|()| machine)
    }

    fn drive(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        while let Some(task) = machine.work.pop() {
            if let Err(error) = self.step(task, machine) {
                self.unwind(error, machine)?;
            }
        }
        Ok(())
    }

    /// Drop suspended requests up to the nearest block statement boundary.
    /// Each dropped scope restores the parser state it had replaced.
    fn unwind(&mut self, error: Diagnostic, machine: &mut Machine) -> Result<(), Diagnostic> {
        while let Some(abandoned) = machine.work.pop() {
            match abandoned {
                Work::ClosureValue { outer, .. } | Work::NamedFunctionExpression { outer } | Work::BodyStatement { outer } | Work::BlockNext { outer, .. } => self.pending = outer,
                Work::SwitchNext(mut frame) | Work::SwitchCaseValue { mut frame, .. } | Work::SwitchBodyAfter { mut frame, .. } => self.pending = std::mem::take(&mut frame.outer),
                Work::BlockAfter { body, outer, recovery }
                    if error.code == "syntax" && self.diagnostics.is_some_and(|errors| errors.borrow().len() < 63) =>
                {
                    self.diagnostics.unwrap().borrow_mut().push(error);
                    self.depth = recovery.depth;
                    self.pending.truncate(recovery.pending);
                    self.call_type_arguments.truncate(recovery.arguments);
                    self.parameter_patterns.truncate(recovery.patterns);
                    machine.values.truncate(recovery.values);
                    machine.statements.truncate(recovery.statements);
                    machine.bodies.truncate(recovery.bodies);
                    machine.parameters.truncate(recovery.parameters);
                    machine.functions.truncate(recovery.functions);
                    machine.matches.truncate(recovery.matches);
                    self.recover_statement(recovery.start);
                    machine.work.push(Work::BlockNext { body, outer });
                    return Ok(());
                }
                Work::BlockAfter { outer, .. } => self.pending = outer,
                _ => {}
            }
        }
        Err(error)
    }

    pub(super) fn recovery(&self, machine: &Machine) -> Recovery {
        Recovery {
            start: self.i,
            depth: self.depth,
            pending: self.pending.len(),
            arguments: self.call_type_arguments.len(),
            patterns: self.parameter_patterns.len(),
            values: machine.values.len(),
            statements: machine.statements.len(),
            bodies: machine.bodies.len(),
            parameters: machine.parameters.len(),
            functions: machine.functions.len(),
            matches: machine.matches.len(),
        }
    }

    fn step(&mut self, task: Work, machine: &mut Machine) -> Result<(), Diagnostic> {
        match task {
            Work::Block
            | Work::BlockNext { .. }
            | Work::BlockAfter { .. }
            | Work::Body
            | Work::BodyStatement { .. }
            | Work::Statement
            | Work::StatementBlock(_)
            | Work::StatementExpression { .. }
            | Work::Labeled { .. }
            | Work::If
            | Work::IfCondition(_)
            | Work::IfThen { .. }
            | Work::IfElse(_)
            | Work::IfElseIf(_)
            | Work::WhileCondition(_)
            | Work::WhileBody { .. }
            | Work::DoBody(_)
            | Work::DoCondition { .. }
            | Work::TryBody(_)
            | Work::TryCatch { .. }
            | Work::TryFinally { .. }
            | Work::ForOfPattern { .. }
            | Work::ForOfIterable(_)
            | Work::ForOfBody(_)
            | Work::ForInitializerList(_)
            | Work::ForInitializerDeclaration(_)
            | Work::ForCondition { .. }
            | Work::ForUpdate { .. }
            | Work::ForBody(_)
            | Work::ExpressionList(_)
            | Work::ExpressionListItem(_)
            | Work::SwitchDiscriminant(_)
            | Work::SwitchNext(_)
            | Work::SwitchCaseValue { .. }
            | Work::SwitchBodyAfter { .. }
            | Work::MatchSyntax
            | Work::MatchScrutinee(_)
            | Work::MatchNext(_)
            | Work::MatchArm { .. }
            | Work::MatchArmValue { .. }
            | Work::LowerMatchValue(_)
            | Work::LowerMatch(_) => self.statement_step(task, machine),
            Work::Declarator(_)
            | Work::DeclaratorValue { .. }
            | Work::DeclaratorPattern { .. }
            | Work::DeclaratorPatternValue { .. }
            | Work::Pattern { .. }
            | Work::PatternNext(_)
            | Work::PatternNested { .. }
            | Work::PatternNestedDefault { .. }
            | Work::PatternNameDefault { .. }
            | Work::PatternRest { .. }
            | Work::Parameters(_)
            | Work::Parameter(_)
            | Work::ParameterPattern { .. }
            | Work::ParameterDefault { .. }
            | Work::NamedFunction
            | Work::NamedFunctionParameters { .. }
            | Work::NamedFunctionBody { .. }
            | Work::LocalFunction(_)
            | Work::NamedFunctionExpression { .. }
            | Work::ClosureParameters { .. }
            | Work::FunctionParameters { .. }
            | Work::ClosureBlock(_)
            | Work::ClosureValue { .. } => self.declaration_step(task, machine),
            _ => self.expression_step(task, machine),
        }
    }

    // Entry points. None of them is reachable from a grammar step.

    pub(super) fn expression(&mut self) -> Result<Expr, Diagnostic> {
        let mut machine = self.run(Work::Parse(0))?;
        assert_eq!(machine.values.len(), 1, "one completed expression");
        Ok(machine.value())
    }

    /// One statement of a block. Declarations that follow it stay in `pending`.
    pub(super) fn block_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let mut machine = self.run(Work::Statement)?;
        assert_eq!(machine.statements.len(), 1, "one completed statement");
        Ok(machine.statement())
    }

    /// The function keyword has already been consumed.
    pub(super) fn named_function(&mut self) -> Result<Function, Diagnostic> {
        let mut machine = self.run(Work::NamedFunction)?;
        assert_eq!(machine.functions.len(), 1, "one completed function");
        Ok(machine.functions.pop().unwrap())
    }

    pub(super) fn match_statement_syntax(&mut self) -> Result<ParsedMatch, Diagnostic> {
        let mut machine = self.run(Work::MatchSyntax)?;
        assert_eq!(machine.matches.len(), 1, "one completed match statement");
        Ok(machine.matches.pop().unwrap())
    }
}
