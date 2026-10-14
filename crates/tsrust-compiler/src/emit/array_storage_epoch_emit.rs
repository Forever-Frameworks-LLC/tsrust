//! Move proven array survivors without cloning their borrowed environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn array_result_write_code(&self, fun: &CheckedFunction, statement: &Stmt) -> Option<String> {
        let ty = fun.storage_exits.array_result_write(statement.span)?;
        let StmtKind::Assign { target, value, operator: None } = &statement.kind else { return None; };
        if matches!(target.kind,ExprKind::Name(_))
            && let Some(binding)=target.binding
            && let Some(layout)=self.owned_record_layout(fun,binding) {
            let initialized=self.expr(fun,value,Mode::Consume);
            let mut drops=String::new();
            for leaf in &layout.leaves {
                if leaf.ty.is_copy() {continue;}
                let mut record=layout.record;
                let mut path=Vec::new();
                for &position in &leaf.path {
                    let field=&self.program.records[record].fields[position];
                    path.push(field.name.text.clone());
                    if let Some(Type::Record(child))=field.ty {record=child;}
                }
                if fun.storage_exits.transfer_before(statement.span.start,binding,&path).is_none() {
                    write!(drops,"drop({});",leaf.native_name(binding)).unwrap();
                }
            }
            let declaration=self.owned_record_declaration(fun,binding,"native_storage_replacement").expect("checked owning record");
            return Some(format!("let native_storage_replacement={initialized};{drops}{declaration}"));
        }
        let mut root=target;
        let mut names=Vec::new();
        while let ExprKind::Field(base,name)=&root.kind {names.push(name);root=base;}
        let binding=root.binding?;
        let place=if names.is_empty(){format!("value{binding}")}else{
            let mut current=fun.bindings[binding].ty;
            let mut path=Vec::new();
            for name in names.into_iter().rev(){
                let Type::Record(record)=current else{return None;};
                let index=self.program.records[record].fields.iter().position(|field|field.name.text==*name)?;
                path.push(index);current=self.program.records[record].fields[index].ty?;
            }
            self.owned_record_leaf_place(fun,binding,&path)?
        };
        Some(format!("let array_result_replacement = {}; drop({place}); let mut {place}: {} = array_result_replacement;", self.expr(fun, value, Mode::Consume), self.native_type(ty)))
    }

    pub(super) fn array_storage_epoch_code(&self, fun: &CheckedFunction, epoch: &crate::analysis::MapEntryEpoch) -> String {
        let mut source = epoch.input;
        let mut target = epoch.output;
        let mut indices = Vec::new();
        for name in &epoch.place.fields {
            let (Type::Record(input), Type::Record(output)) = (source, target) else { unreachable!("checked array field path") };
            let index = self.program.records[input].fields.iter().position(|field| field.name.text == *name).expect("checked array field");
            indices.push(index);
            source = self.program.records[input].fields[index].ty.unwrap();
            target = self.program.records[output].fields[index].ty.unwrap();
        }
        let (Type::Array(source), Type::Array(target)) = (source, target) else { unreachable!("checked array epoch") };
        let native = self.native_type(Type::Array(target));
        let binding = epoch.place.binding;
        let place = if indices.is_empty() { format!("value{binding}") } else { self.owned_record_leaf_place(fun, binding, &indices).expect("owning array leaf") };
        if epoch.empty { return format!("drop({place}); let mut {place}: {native} = Vec::new();"); }
        let converted = self.map_entry_payload_projection(source.ty(), target.ty(), "array_epoch_item");
        format!("let mut {place}: {native} = ({place}).tsr_owned_items().map(|array_epoch_item|{{{converted}}}).collect::<{native}>();")
    }
}
