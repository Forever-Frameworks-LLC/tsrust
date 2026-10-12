//! Preserve typed factory environments when presence builders widen their output.
use crate::syntax::{Type,UnionId};
pub(crate) fn inherited_factories(output:Type,factories:&std::collections::HashMap<Type,usize>,members:impl Fn(UnionId)->Vec<Type>)->Vec<(Type,usize)> {
    let Type::Union(target)=output else{return Vec::new()};
    let target=members(target);
    let mut result=factories.iter().filter_map(|(&source,&family)|{
        let Type::Union(union)=source else{return None};
        let source_members=members(union);
        (source!=output&&source_members.len()<target.len()&&source_members.iter().all(|ty|target.contains(ty))).then_some((source,family))
    }).collect::<Vec<_>>();
    result.sort_by_key(|(ty,_)|{let Type::Union(id)=ty else{unreachable!()};id.index()});
    result
}
