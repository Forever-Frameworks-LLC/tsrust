//! Preserve actual supplied object fields while checking pipeline input types.
use super::*;
use crate::stdlib::schema::pipelines::{Operation,WrapperKind};

impl Analyzer {
    fn schema_pipeline_accepts_missing(&self,id:usize)->bool {
        if let Some(plan)=self.schema_pipelines.wrapper(id) {
            return matches!(plan.kind,WrapperKind::Default(_)|WrapperKind::Presence(Type::Undefined))
                || self.schema_pipeline_accepts_missing(plan.source);
        }
        if let Some(stage)=self.schema_pipelines.get(id) {
            return if stage.operation==Operation::Coerce {
                stage.callback_input==Type::Undefined || matches!(stage.callback_input,Type::Union(union) if self.unions.members(union).contains(&Type::Undefined))
            }else{self.schema_pipeline_accepts_missing(stage.source)};
        }
        if let Some(plan)=self.schema_pipelines.union(id) {
            return plan.branches.iter().any(|child|self.schema_pipeline_accepts_missing(*child));
        }
        if self.schema_pipelines.types(id).is_some(){return false;}
        let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
        self.schema_may_default(output)||matches!(output,Type::Union(union) if self.unions.members(union).contains(&Type::Undefined))
    }

    pub(super) fn schema_pipeline_input_type(&mut self,context:&Context,id:usize,value:&Expr)->Result<Type,Diagnostic> {
        if let Some(plan)=self.schema_pipelines.wrapper(id).cloned() {
            if let WrapperKind::Presence(added)=plan.kind
                && self.expression_type_hint(context,value)==Some(added){return Ok(added);}
            return self.schema_pipeline_input_type(context,plan.source,value);
        }
        if let Some(stage)=self.schema_pipelines.get(id).cloned() {
            return if stage.operation==Operation::Coerce{Ok(stage.callback_input)}
                else{self.schema_pipeline_input_type(context,stage.source,value)};
        }
        if let Some(plan)=self.schema_pipelines.object(id).cloned() {
            let Type::Record(shape)=plan.input else{unreachable!()};
            let fields=self.program.records[shape].fields.clone();
            let hinted=self.expression_type_hint(context,value);
            let names=match &value.kind {
                ExprKind::Record(values)=>values.iter().map(|(name,_)|name.text.clone()).collect::<HashSet<_>>(),
                _=>if let Some(Type::Record(actual))=hinted {
                    if self.program.records[actual].native_type.is_some(){return Ok(plan.input);}
                    self.program.records[actual].fields.iter().map(|field|field.name.text.clone()).collect()
                }else{return Ok(plan.input)},
            };
            if names.iter().any(|name|!fields.iter().any(|field|field.name.text==*name)) {return Ok(plan.input);}
            if fields.iter().enumerate().any(|(index,field)|!names.contains(&field.name.text)&&!self.schema_pipeline_accepts_missing(plan.fields[index])){return Ok(plan.input);}
            if let Some(Type::Record(actual))=hinted
                && self.program.records[actual].fields.iter().all(|provided|fields.iter().any(|field|field.name.text==provided.name.text&&self.schema_owned_input_compatible(field.ty.unwrap(),provided.ty.unwrap()))) {
                return Ok(Type::Record(actual));
            }
            let mut supplied=Vec::new();
            for (index,mut field) in fields.into_iter().enumerate() {
                if !names.contains(&field.name.text){continue;}
                if let ExprKind::Record(values)=&value.kind
                    && let Some((_,value))=values.iter().find(|(name,_)|name.text==field.name.text) {
                    let ty=self.schema_pipeline_input_type(context,plan.fields[index],value)?;
                    field.ty=Some(ty);field.annotation=self.checked_value_annotation(ty,field.name.span)?;
                }
                field.optional=false;
                supplied.push(field);
            }
            return Ok(Type::Record(self.schema_pipeline_shape(supplied,"input",value.span)));
        }
        let input=self.schema_pipelines.types(id).map(|(input,_,_)|input)
            .unwrap_or_else(||self.program.records[id].application.as_ref().unwrap().resolved_arguments[0]);
        self.schema_default_input(context,input,value)
    }
}
