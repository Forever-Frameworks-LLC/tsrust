//! Ordered collection emission, outside the shared expression frame.
use super::*;
#[path = "runtime_sets.rs"]
mod runtime;

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn set_method_expression(&self, function: &CheckedFunction, expression: &Expr, callee: &Expr, arguments: &[Expr], method: SetMethod) -> String {
        let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
        if matches!(method,SetMethod::Union|SetMethod::Intersection|SetMethod::Difference|SetMethod::SymmetricDifference){
            return self.set_algebra_expression(function,expression,base,&arguments[0],method);
        }
        if matches!(method,SetMethod::IsSubsetOf|SetMethod::IsSupersetOf|SetMethod::IsDisjointFrom){
            return self.set_relation_expression(function,base,&arguments[0],method);
        }
        let mutable=matches!(method,SetMethod::Add|SetMethod::Delete|SetMethod::Clear);
        let receiver=self.expr(function,base,if mutable {Mode::Mutate}else{Mode::Read});
        let action=match method {
            SetMethod::Add|SetMethod::Has|SetMethod::Delete => {
                let key=self.expr(function,&arguments[0],if method==SetMethod::Add {Mode::Consume}else{Mode::Read});
                let name=match method {SetMethod::Add=>"add",SetMethod::Has=>"has",_=>"delete"};
                format!("let set_key={key};set_receiver.{name}(set_key)")
            }
            SetMethod::IsSubsetOf|SetMethod::IsSupersetOf|SetMethod::IsDisjointFrom=>unreachable!("relation handled separately"),
            SetMethod::Union|SetMethod::Intersection|SetMethod::Difference|SetMethod::SymmetricDifference=>unreachable!("algebra handled separately"),
            SetMethod::Clear=>"set_receiver.clear()".into(),
            SetMethod::Values=>self.flat_array_result(expression.ty.expect("checked Set snapshot"),"set_receiver.iter().cloned().collect::<Vec<_>>()"),
            SetMethod::Entries(pair)=>self.flat_array_result(expression.ty.unwrap(),&format!("set_receiver.iter().map(|key|Record{pair}{{field0:key.clone(),field1:key.clone()}}).collect::<Vec<_>>()")),
        };
        format!("{{let set_receiver={receiver};{action}}}")
    }

    #[inline(never)]
    fn set_algebra_expression(&self,function:&CheckedFunction,expression:&Expr,base:&Expr,other:&Expr,method:SetMethod)->String{
        let receiver=self.expr(function,base,Mode::Read);let argument=self.expr(function,other,Mode::Read);
        let Type::Set(left)=base.ty.unwrap()else{unreachable!()};let Type::Set(right)=other.ty.unwrap()else{unreachable!()};
        let Type::Set(result)=expression.ty.unwrap()else{unreachable!()};
        let a=self.program.records[left].set_definition.as_ref().unwrap().key;
        let b=self.program.records[right].set_definition.as_ref().unwrap().key;
        let key=self.program.records[result].set_definition.as_ref().unwrap().key;
        let left_copy=self.element_alternative(a,key,"set_key.clone()");let right_copy=if matches!(method,SetMethod::Union|SetMethod::SymmetricDifference){self.element_alternative(b,key,"set_key.clone()")}else{String::new()};
        let forward=self.set_cross_key_same(a,b);let reverse=self.set_cross_key_same(b,a);
        let left_loop=format!("for set_key in set_receiver.iter(){{set_result.add({left_copy});}}");
        let right_loop=format!("for set_key in set_other.iter(){{set_result.add({right_copy});}}");
        let action=match method{
            SetMethod::Union=>format!("{left_loop}{right_loop}"),
            SetMethod::Difference=>format!("for set_key in set_receiver.iter(){{if !set_other.has_with(set_key,set_key.tsr_hash(),&set_forward){{set_result.add({left_copy});}}}}"),
            SetMethod::SymmetricDifference=>format!("for set_key in set_receiver.iter(){{if !set_other.has_with(set_key,set_key.tsr_hash(),&set_forward){{set_result.add({left_copy});}}}}for set_key in set_other.iter(){{if !set_receiver.has_with(set_key,set_key.tsr_hash(),&set_reverse){{set_result.add({right_copy});}}}}"),
            SetMethod::Intersection=>format!("if set_receiver.len()<=set_other.len(){{for set_key in set_receiver.iter(){{if set_other.has_with(set_key,set_key.tsr_hash(),&set_forward){{set_result.add({left_copy});}}}}}}else{{for other_key in set_other.iter(){{for set_key in set_receiver.matches_with(other_key,other_key.tsr_hash(),&set_reverse){{set_result.add({left_copy});}}}}}}"),
            _=>unreachable!(),
        };
        format!("{{let set_receiver={receiver};let set_other={argument};let set_forward=|set_left:&{},set_right:&{}|{{{forward}}};let set_reverse=|set_left:&{},set_right:&{}|{{{reverse}}};let mut set_result={}::new();{action}set_result}}",self.native_type(a),self.native_type(b),self.native_type(b),self.native_type(a),self.native_type(expression.ty.unwrap()))
    }

    #[inline(never)]
    fn set_relation_expression(&self,function:&CheckedFunction,base:&Expr,other:&Expr,method:SetMethod)->String{
        let receiver=self.expr(function,base,Mode::Read);
        let argument=self.expr(function,other,Mode::Read);
        let Type::Set(left)=base.ty.unwrap() else {unreachable!()};
        let Type::Set(right)=other.ty.unwrap() else {unreachable!()};
        let left=self.program.records[left].set_definition.as_ref().unwrap().key;
        let right=self.program.records[right].set_definition.as_ref().unwrap().key;
        let (iterated,queried,a,b)=if method==SetMethod::IsSupersetOf{("set_other","set_receiver",right,left)}else{("set_receiver","set_other",left,right)};
        let comparison=self.set_cross_key_same(a,b);
        let operation=if method==SetMethod::IsDisjointFrom{"all(|key| !set_other.has_with(key,key.tsr_hash(),&set_compare))"}else{"all(|key| set_query.has_with(key,key.tsr_hash(),&set_compare))"};
        format!("{{let set_receiver={receiver};let set_other={argument};let set_compare=|set_left:&{},set_right:&{}|{{{comparison}}};let set_query={queried};{iterated}.iter().{operation}}}",self.native_type(a),self.native_type(b))
    }

    #[inline(never)]
    fn set_cross_key_same(&self,left:Type,right:Type)->String{
        if left==right{return "set_left.tsr_same(set_right)".into();}
        let lefts=left.union().map(|id|self.program.unions[id.index()].members.clone()).unwrap_or_else(||vec![left]);
        let rights=right.union().map(|id|self.program.unions[id.index()].members.clone()).unwrap_or_else(||vec![right]);
        let mut arms=Vec::new();
        for (i,a) in lefts.iter().enumerate(){for(j,b)in rights.iter().enumerate(){
            let lp=left.union().map_or_else(||"a".into(),|id|format!("Union{}::V{i}(a)",id.index()));
            let rp=right.union().map_or_else(||"b".into(),|id|format!("Union{}::V{j}(b)",id.index()));
            let comparison=if a==b{"a.tsr_same(b)"}else if matches!(a,Type::Int|Type::Number)&&matches!(b,Type::Int|Type::Number){"tsr_set_number_same(*a as f64,*b as f64)"}else{"false"};
            arms.push(format!("({lp},{rp})=>{comparison}"));
        }}
        format!("match(set_left,set_right){{{}}}",arms.join(","))
    }

    pub(super) fn set_types(&self, output: &mut String) {
        if !self.program.records.iter().any(|record| record.set_definition.is_some() || record.map_definition.is_some()) { return; }
        output.push_str(runtime::SET_RUNTIME);
        let mut unions = std::collections::BTreeSet::new();
        for (id, record) in self.program.records.iter().enumerate() {
            if let Some(definition) = &record.set_definition {
                let _ = writeln!(output, "type Set{id} = TsrSet<{}>;", self.native_type(definition.key));
                if let Some(union) = definition.key.union() { unions.insert(union.index()); }
            }
            if let Some(map) = &record.map_definition { if let Some(union) = map.key.union() { unions.insert(union.index()); } }
            if let Some(definition) = &record.enum_definition {
                let hash = match definition.kind {
                    crate::syntax::EnumKind::Number => format!("tsr_set_hash({}u64,tsr_set_number_hash(self.tsr_value()))", id+5),
                    crate::syntax::EnumKind::String => format!("tsr_set_hash({}u64,self.tsr_value())", id+5),
                };
                let same = match definition.kind {
                    crate::syntax::EnumKind::Number => "tsr_set_number_same(self.tsr_value(),other.tsr_value())",
                    crate::syntax::EnumKind::String => "self.tsr_value()==other.tsr_value()",
                };
                let _ = writeln!(output,"impl TsrSetKey for Enum{id} {{ fn tsr_hash(&self)->u64{{{hash}}} fn tsr_same(&self,other:&Self)->bool{{{same}}} }}");
            }
        }
        for id in unions {
            let members = &self.program.unions[id].members;
            let hash = members.iter().enumerate().map(|(variant, member)| format!("Self::V{variant}(value)=>{{let value:&{}=value;value.tsr_hash()}}",self.native_type(*member))).collect::<Vec<_>>().join(",");
            let normalize = members.iter().enumerate().map(|(variant, _)| format!("Self::V{variant}(value)=>Self::V{variant}(value.tsr_normalize())")).collect::<Vec<_>>().join(",");
            let same = self.set_union_same(members, "Self", "Self");
            let _ = writeln!(output,"impl TsrSetKey for Union{id} {{ fn tsr_hash(&self)->u64{{match self{{{hash}}}}} fn tsr_same(&self,other:&Self)->bool{{match(self,other){{{same}}}}} fn tsr_normalize(self)->Self{{match self{{{normalize}}}}} }}");
            if members.iter().any(|member| !member.is_copy()) {
                let same = self.set_union_same(members,&format!("ViewUnion{id}"),&format!("Union{id}"));
                let _ = writeln!(output,"impl<'a> TsrSetQuery<Union{id}> for ViewUnion{id}<'a> {{ fn tsr_query_hash(&self)->u64{{match self{{{hash}}}}} fn tsr_query_same(&self,other:&Union{id})->bool{{match(self,other){{{same}}}}} }}");
            }
        }
    }

    fn set_union_same(&self, members: &[Type], left: &str, right: &str) -> String {
        let mut arms = Vec::new();
        for (i, a) in members.iter().enumerate() {
            for (j, b) in members.iter().enumerate() {
                let comparison = if a == b { "left.tsr_same(right)".into() }
                    else if matches!(a,Type::Int|Type::Number) && matches!(b,Type::Int|Type::Number) {
                        "tsr_set_number_same(*left as f64,*right as f64)".into()
                    } else { String::from("false") };
                arms.push(format!("({left}::V{i}(left),{right}::V{j}(right))=>{{let left:&{}=left;let right:&{}=right;{comparison}}}",self.native_type(*a),self.native_type(*b)));
            }
        }
        arms.join(",")
    }

    #[inline(never)]
    pub(super) fn set_constructor_expression(&self, function: &CheckedFunction, expression: &Expr, mode: Mode) -> String {
        let ExprKind::SetConstructor(arguments, _) = &expression.kind else { unreachable!() };
        let native = self.native_type(expression.ty.expect("checked Set"));
        let code = if let Some(source) = arguments.first() {
            let value = self.expr(function,source,Mode::Read);
            let values: Option<String> = match source.ty.expect("checked Set source") {
                Type::Null | Type::Undefined => None,
                Type::String => Some("set_source.as_ref().chars().map(|value|String::Owned(value.to_string()))".into()),
                Type::Array(_) if self.expression_shape(source) == Some(crate::fixed_storage::Shape::LiteralStrings) => Some("set_source.literal_iter().map(String::Borrowed)".into()),
                _ => Some("set_source.iter().cloned()".into()),
            };
            match values {
                Some(values) => {
                    let Type::Set(id) = expression.ty.unwrap() else { unreachable!() };
                    let target = self.program.records[id].set_definition.as_ref().unwrap().key;
                    let source_key = match source.ty.unwrap() {
                        Type::Array(element) => element.ty(),
                        Type::Set(id) => self.program.records[id].set_definition.as_ref().unwrap().key,
                        Type::String => Type::String,
                        _ => unreachable!(),
                    };
                    let adapted = self.element_alternative(source_key, target, "set_item");
                    format!("{{let set_source={value};{native}::from_values(({values}).map(|set_item|{adapted}))}}")
                },
                None => format!("{{let _={value};{native}::new()}}"),
            }
        } else { format!("{native}::new()") };
        match mode { Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code }
    }
}
