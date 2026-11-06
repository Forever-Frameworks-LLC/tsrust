//! Owner footprints and holder paths are distinct sides of a retained loan.
use super::native_view_loans::{Loan, Segment};
use super::*;

pub(super) fn access_fields(context: &Context, root: usize) -> &[String] {
    if let Some(projection) = &context.projection
        && projection.root == context.bindings[root].name
    {
        return &projection.path;
    }
    context
        .mutation_place
        .as_ref()
        .filter(|place| place.root == root)
        .map_or(&[], |place| place.fields.as_slice())
}

pub(super) fn overlaps(left: &[String], right: &[String]) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

pub(super) fn under(loan: &Loan, fields: &[String]) -> bool {
    (loan.whole || loan.path.len() >= fields.len())
        && loan
            .path
            .iter()
            .zip(fields)
            .all(|(segment, field)| matches!(segment,Segment::Field(name) if name == field))
}

pub(super) fn consume(loans: &mut [Loan], fields: &[String]) {
    for loan in loans {
        if loan.path.len() >= fields.len() && under(loan, fields) {
            loan.retained = false;
        }
    }
}

pub(super) fn sources(
    context: &Context,
    source: &Expr,
) -> Result<Vec<crate::flow::Place>, Diagnostic> {
    let mut pending = vec![source];
    let mut places = Vec::new();
    while let Some(value) = pending.pop() {
        match &value.kind {
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                pending.push(consequent);
                pending.push(alternate);
            }
            ExprKind::Sequence(values) => {
                if let Some(last) = values.last() {
                    pending.push(last);
                }
            }
            // Readonly injection builds a descriptor over the same payload owner.
            ExprKind::Inject { value: payload, .. } if matches!(value.ty, Some(Type::BorrowedUnion(_))) => pending.push(payload),
            ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } => {
                pending.push(value)
            }
            _ if Analyzer::named_storage(value) => {
                let footprint = crate::flow::mutation_footprint(value, &context.names);
                let roots = Analyzer::view_sources(context, value);
                if roots.is_empty() {
                    return Err(source_error(value.span));
                }
                for root in roots {
                    let fields = footprint
                        .as_ref()
                        .filter(|place| place.root == root)
                        .map_or_else(Vec::new, |place| place.fields.clone());
                    places.push(crate::flow::Place { root, fields });
                }
            }
            _ => return Err(source_error(value.span)),
        }
    }
    if places.is_empty() {
        return Err(source_error(source.span));
    }
    places.sort_by(|left, right| (left.root, &left.fields).cmp(&(right.root, &right.fields)));
    places.dedup();
    Ok(places)
}
fn source_error(span: Span) -> Diagnostic {
    Diagnostic::new(
        "native-view-source",
        "Every source alternative of a stored native view requires named storage that outlives the view.",
        span,
    )
}


/// Physical callable-reference slots need holder paths but do not add a byte-view
/// ownership rule. Actual native payload loans keep the existing strict places.
fn collection_holder(context:&Context,mut value:&Expr,incoming:&[Loan])->Option<(usize,Vec<Segment>)>{
    if let Some(place)=crate::flow::place(value,&context.names){
        return Some((place.root,place.fields.into_iter().map(Segment::Field).collect()));
    }
    if incoming.is_empty()||!incoming.iter().all(Loan::environment_borrow){return None;}
    let mut path=Vec::new();
    loop{match &value.kind{
        ExprKind::Index(base,_)=>{path.push(Segment::Element);value=base;}
        ExprKind::Field(base,name)=>{path.push(Segment::Field(name.clone()));value=base;}
        ExprKind::Project{value:base,..}=>value=base,
        ExprKind::TypeAssertion{value:base,..}if value.ty==base.ty=>value=base,
        ExprKind::Sequence(values)=>value=values.last()?,
        ExprKind::Name(name)=>{let root=value.binding.or_else(||context.names.get(name).copied())?;path.reverse();return Some((root,path));}
        _=>return None,
    }}
}

impl Analyzer {
    /// A replacement in the block that declares the local redeclares its native
    /// storage, which starts a new lifetime. Any other replacement assigns to the same
    /// native variable. Rust releases the earlier owners for conflict checks, but a
    /// lifetime in a function signature is unified over every value the variable
    /// ever holds. The earlier origins therefore stay as dormant provenance: they do
    /// not block their owners and they still reach a returned value.
    pub(super) fn retain_unified_native_view_loans(context: &Context, holder: usize, incoming: &mut Vec<Loan>) {
        let binding = &context.bindings[holder];
        let redeclared = !binding.parameter
            && !binding.capture_reference
            && binding.borrowed_place.is_none()
            && !binding.mutable_place
            && context.native_view_declaration_depths.get(&holder) == Some(&context.lexical_scopes.len());
        if redeclared {
            return;
        }
        Self::extend_dormant(incoming, binding.native_view_loans.as_deref().into_iter().flatten());
    }
    fn extend_dormant<'a>(incoming: &mut Vec<Loan>, previous: impl Iterator<Item = &'a Loan>) {
        for loan in previous {
            let mut dormant = loan.clone();
            dormant.retained = false;
            let known = incoming.iter().any(|current| {
                let mut current = current.clone();
                current.retained = false;
                current == dormant
            });
            if !known {
                incoming.push(dormant);
            }
        }
    }
    /// An appended element joins the array's storage: the array then borrows from
    /// the new origins as well as the ones it already holds.
    pub(super) fn append_native_view_element(context: &mut Context, array: &Expr, value: &Expr) -> Result<(), Diagnostic> {
        if !context.native_view_calls {
            return Ok(());
        }
        let mut incoming = Self::native_view_value_loans(context, value)?;
        if incoming.is_empty() {
            return Ok(());
        }
        let Some((root,path)) = collection_holder(context,array,&incoming) else {
            return Err(Diagnostic::new(
                "native-view-source",
                "A byte view can only be stored in a collection held by a named local or its fields.",
                array.span,
            ));
        };
        for loan in &mut incoming {
            native_view_projections::wrap(loan, &Segment::Element);
            for segment in path.iter().rev() {
                native_view_projections::wrap(loan, segment);
            }
        }
        let mut retained = context.bindings[root].native_view_loans.as_deref().cloned().unwrap_or_default();
        retained.extend(incoming);
        native_view_projections::normalize(&mut retained);
        Self::set_native_view_loans(context, root, retained);
        Ok(())
    }
    pub(super) fn replace_native_view_field(
        context: &mut Context,
        target: &Expr,
        value: &Expr,
    ) -> Result<(), Diagnostic> {
        if !context.native_view_calls {
            return Ok(());
        }
        let Some(place) = crate::flow::place(target, &context.names) else {
            return Ok(());
        };
        if place.fields.is_empty() {
            return Ok(());
        }
        let mut incoming = Self::native_view_value_loans(context, value)?;
        for loan in &mut incoming {
            for field in place.fields.iter().rev() {
                native_view_projections::wrap(loan, &Segment::Field(field.clone()));
            }
        }
        let mut retained = context.bindings[place.root]
            .native_view_loans
            .as_deref()
            .cloned()
            .unwrap_or_default();
        let replaced = retained.iter().filter(|loan| loan.path.len() >= place.fields.len() && under(loan, &place.fields)).cloned().collect::<Vec<_>>();
        retained.retain(|loan| loan.path.len() < place.fields.len() || !under(loan, &place.fields));
        // The field keeps one native variable: see retain_unified_native_view_loans.
        Self::extend_dormant(&mut incoming, replaced.iter());
        retained.extend(incoming);
        native_view_projections::normalize(&mut retained);
        Self::set_native_view_loans(context, place.root, retained);
        Ok(())
    }
}


#[cfg(test)]mod collection_holder_tests{
    use super::*;
    use super::super::native_view_loans::LifetimeSource;
    use crate::native_return_borrows::BorrowAccess;
    fn loan(environment:bool)->Loan{Loan{whole:false,root:1,owner_fields:vec![],source:LifetimeSource::Storage,access:BorrowAccess::ReadOnly,creation:Span::default(),retained:!environment,path:if environment{vec![Segment::CaptureBorrow(2)]}else{vec![]},guards:vec![]}}
    #[test]fn indexed_environment_collection_holder_keeps_field_and_element_paths(){
        let context=Context::new("test".into());
        let mut named=Expr::new(ExprKind::Name("boxes".into()),Span::default());named.binding=Some(0);
        let index=|base:Expr|Expr::new(ExprKind::Index(Box::new(base),Box::new(Expr::new(ExprKind::Integer("0".into()),Span::default()))),Span::default());
        let field=Expr::new(ExprKind::Field(Box::new(index(index(named.clone()))),"items".into()),Span::default());
        let (root,path)=collection_holder(&context,&field,&[loan(true)]).unwrap();assert_eq!(root,0);assert_eq!(path,vec![Segment::Element,Segment::Element,Segment::Field("items".into())]);
        let mut incoming=loan(true);native_view_projections::wrap(&mut incoming,&Segment::Element);for segment in path.iter().rev(){native_view_projections::wrap(&mut incoming,segment);}
        assert_eq!(incoming.path,vec![Segment::Element,Segment::Element,Segment::Field("items".into()),Segment::Element,Segment::CaptureBorrow(2)]);assert!(!incoming.retained);
        assert!(collection_holder(&context,&field,&[loan(false)]).is_none());assert!(collection_holder(&context,&field,&[loan(true),loan(false)]).is_none());
        assert!(collection_holder(&context,&named,&[loan(false)]).is_some());
    }
}
