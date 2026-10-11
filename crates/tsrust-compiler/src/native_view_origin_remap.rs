//! Callable alternatives in lifetime paths follow checked closure compaction.
use crate::analysis::native_view_loans::{LifetimeSource,Loan,Segment,VariantGuard};
use crate::{Diagnostic,Span};

pub(crate) fn path(path:&mut [Segment],ids:&[Option<usize>])->Result<(),Diagnostic> {
    for segment in path {
        if let Segment::CallableAlternative(origin)=segment {
            *origin=ids.get(*origin).copied().flatten().ok_or_else(||Diagnostic::new("native-view-callable-storage","A carried view path lost its callable environment alternative.",Span::default()))?;
        }
    }
    Ok(())
}

pub(crate) fn guards(guards:&mut [VariantGuard],ids:&[Option<usize>])->Result<(),Diagnostic> {
    for guard in guards {path(&mut guard.path,ids)?;}
    Ok(())
}

pub(crate) fn loan(value:&mut Loan,ids:&[Option<usize>])->Result<(),Diagnostic> {
    let mut next=value.clone();
    path(&mut next.path,ids)?;
    guards(&mut next.guards,ids)?;
    match &mut next.source {
        LifetimeSource::Storage=>{},
        LifetimeSource::Carried(source)=>path(source,ids)?,
        LifetimeSource::VariantCarried{path:source,guards:source_guards}=>{
            path(source,ids)?;guards(source_guards,ids)?;
        }
    }
    *value=next;
    Ok(())
}

#[cfg(test)]
#[test]
fn native_view_alternative_remap_preserves_roots_and_record_guards_atomically() {
    let mut value=Loan { whole: false,
        root:9,owner_fields:vec!["owner".into()],source:LifetimeSource::VariantCarried {
            path:vec![Segment::CallableAlternative(0),Segment::Field("capture3".into())],
            guards:vec![VariantGuard{path:vec![Segment::CallableAlternative(1)],success:42}],
        },
        access:crate::native_return_borrows::BorrowAccess::Exclusive,creation:Span::default(),retained:true,
        path:vec![Segment::CallableAlternative(1)],guards:vec![VariantGuard{path:vec![Segment::CallableAlternative(0)],success:43}],
    };
    let original=value.clone();
    assert!(loan(&mut value,&[Some(0),None]).is_err());assert_eq!(value,original);
    loan(&mut value,&[Some(1),Some(0)]).unwrap();
    assert_eq!(value.root,9);assert_eq!(value.owner_fields,vec!["owner"]);
    assert_eq!(value.path,vec![Segment::CallableAlternative(0)]);
    assert_eq!(value.guards,vec![VariantGuard{path:vec![Segment::CallableAlternative(1)],success:43}]);
    assert_eq!(value.source,LifetimeSource::VariantCarried{path:vec![Segment::CallableAlternative(1),Segment::Field("capture3".into())],guards:vec![VariantGuard{path:vec![Segment::CallableAlternative(0)],success:42}]});
}
