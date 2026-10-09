//! Move proven array survivors without cloning their borrowed environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn array_result_write_code(&self, fun: &CheckedFunction, statement: &Stmt) -> Option<String> {
        let ty = fun.storage_exits.array_result_write(statement.span)?;
        let StmtKind::Assign { target, value, operator: None } = &statement.kind else { return None; };
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

    pub(super) fn array_block_storage_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize, loop_label: Option<usize>) -> bool {
        let epochs = fun.storage_exits.array_block_epochs(statement.span.start);
        if epochs.is_empty() { return false; }
        let StmtKind::Block(body) = &statement.kind else { return false; };
        let maps = fun.storage_exits.map_block_epochs(statement.span.start);
        let mut names = epochs.iter().chain(maps).map(|epoch| self.map_entry_epoch_place(fun, epoch)).collect::<Vec<_>>();
        let ordinary_maps = self.map_block_return_places(fun, declarations, body);
        for name in &ordinary_maps { if !names.contains(name) { names.push(name.clone()); } }
        let pattern = names.iter().map(|name| format!("mut {name}")).collect::<Vec<_>>().join(",");
        let indent = "    ".repeat(depth);
        let _ = writeln!(output, "{indent}let ({pattern},) = {{");
        self.control_block_epochs(output, fun, body, depth + 1, loop_label, Some(declarations));
        for epoch in maps {
            if epoch.input != epoch.output { let _ = writeln!(output, "{indent}    {}", self.map_entry_epoch_code(fun, epoch)); }
        }
        let values = names.iter().map(|name| {
            if ordinary_maps.contains(name) { self.map_branch_exit_value(fun, body, name) } else { name.clone() }
        }).collect::<Vec<_>>();
        let _ = writeln!(output, "{indent}    ({},)\n{indent}}};", values.join(","));
        self.exception_regions.renew_storage(&names, statement.span.end);
        true
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
