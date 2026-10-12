//! Shared scalar fields and literal discriminator facts for record alternatives.
use super::*;
impl Analyzer {
    pub(super) fn record_union_field(&self,union:UnionId,name:&str)->Option<Type>{
        let fields=self.unions.members(union).into_iter().map(|member|{
            let Type::Record(record)=member else{return None;};
            self.fields[record].get(name).copied()
        }).collect::<Option<Vec<_>>>()?;
        let first=*fields.first()?;
        if fields.iter().all(|field|*field==first){return Some(first);}
        let base=first.scalar_base();
        if fields.iter().all(|field|field.scalar_base()==base){Some(base)}else{self.unions.intern(fields,Span::default()).ok()}
    }

    pub(super) fn record_discriminator_guard(&self,context:&Context,left:&Expr,right:&Expr)->Option<(crate::flow::Place,Vec<Type>,Vec<Type>)>{
        let pair=|field:&Expr,literal:&Expr|{
            let ExprKind::Field(base,name)=&field.kind else{return None;};
            let place=crate::flow::place(base,&context.names)?;
            let union=self.place_type(context,&place)?.union()?;
            let kind=if literal.ty.map(Type::scalar_base)==Some(Type::Number){crate::literal_types::LiteralKind::Number}else{crate::literal_types::LiteralKind::Int};
            use crate::stdlib::schema::plans::Tag;
            let value=match &literal.kind {
                ExprKind::Null=>Tag::Null,
                ExprKind::EnumMember{enumeration,member}=>match &self.program.records[*enumeration].enum_definition.as_ref()?.members[*member].value{
                    crate::syntax::EnumValue::String(value)=>Tag::String(value.clone()),
                    crate::syntax::EnumValue::Number(value)=>Tag::number(*value),
                },
                _=>Tag::literal(&crate::literal_types::LiteralValue::expression(literal,kind)?.ok()?),
            };
            let members=context.flow.get(&place).map(|fact|fact.members.clone()).unwrap_or_else(||self.unions.members(union));
            let mut yes=Vec::new();let mut no=Vec::new();
            for member in members{
                let Type::Record(record)=member else{return None;};
                let field=*self.fields[record].get(name)?;
                let alternatives=if let Type::Union(union)=field{self.unions.members(union)}else{vec![field]};
                let mut matches=false;let mut differs=false;
                for alternative in alternatives{
                    let compared=literal.ty.unwrap_or(alternative).scalar_base();
                    if (matches!(alternative,Type::Enum(_,_))||matches!(compared,Type::Enum(_,_)))&&alternative!=compared{
                        differs=true;continue;
                    }
                    let numeric=|ty:Type|matches!(ty.scalar_base(),Type::Int|Type::Number);
                    if numeric(alternative)&&numeric(compared)&&alternative.scalar_base()!=compared{
                        // Mixed runtime numeric equality can round. It cannot
                        // prove an exact integer alternative absent on either side.
                        matches=true;differs=true;continue;
                    }
                    if let Ok(tags)=self.schema_discriminator_tags(alternative,literal.span){
                        matches|=tags.iter().any(|tag|tag==&value);
                        differs|=tags.iter().any(|tag|tag!=&value);
                    }else{
                        // A broad field cannot prove either outcome impossible.
                        matches=true;differs=true;
                    }
                }
                if matches{yes.push(member);}if differs{no.push(member);}
            }
            Some((place,yes,no))
        };
        pair(left,right).or_else(||pair(right,left))
    }
}
