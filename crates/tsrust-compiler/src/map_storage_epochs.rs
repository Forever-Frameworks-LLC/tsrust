//! Renew concrete scoped Map storage lifetimes across checked local paths.
use super::*;

impl Emitter<'_> {
    /// Leading clears are infallible and execute before every completion path.
    /// Keep their fresh storage in the enclosing scope, then enter the checked
    /// exception region for all remaining effects and handlers.
    #[inline(never)]
    pub(super) fn map_try_prefix_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize) -> bool {
        if !statement.labels.is_empty() { return false; }
        let StmtKind::Try(region) = &statement.kind else { return false; };
        let mut resets = String::new();
        let mut count = 0;
        for item in &region.body {
            if !item.labels.is_empty() { break; }
            let Some(reset) = self.local_map_clear_epoch(fun, declarations, item) else { break; };
            resets.push_str(&reset);
            count += 1;
        }
        if count == 0 { return false; }
        let mut remaining = region.as_ref().clone();
        remaining.body.drain(..count);
        let indent = "    ".repeat(depth);
        let _ = writeln!(output, "{indent}{resets}");
        let code = self.nested_completion(fun, statement, &remaining);
        let _ = writeln!(output, "{indent}{code}");
        true
    }

    /// End a straight-line local storage lifetime before a later refill.
    /// Declarations outside this block must retain their surrounding storage.
    pub(super) fn local_map_clear_epoch(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<String> {
        let (place, native) = self.local_map_clear_storage(fun, body, statement)?;
        let (binding,names)=self.map_storage_binding(fun,&place).expect("checked Map storage");
        let transferred=fun.storage_exits.transfer_before(statement.span.start,binding,&names);
        let drop = if matches!(statement.kind, StmtKind::Assign { .. }) && transferred.is_some()
            && !self.exception_regions.native_storage_available(&place,transferred) { String::new() }
            else { format!("drop({place});") };
        Some(format!("{drop} let mut {place}: {native} = <{native}>::new();"))
    }

    pub(super) fn local_map_clear_storage(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<(String, String)> {
        let base = match &statement.kind {
            StmtKind::Expr(call) if call.target == Some(CallTarget::MapMethod(MapMethod::Clear)) => {
                let ExprKind::Call(callee, _) = &call.kind else { return None; };
                let ExprKind::Field(base, _) = &callee.kind else { return None; };
                base.as_ref()
            },
            StmtKind::Assign { target, value, operator: None }
                if matches!(&value.kind, ExprKind::MapConstructor(arguments, _) if arguments.is_empty()) => target,
            _ => return None,
        };
        let mut root = base;
        let mut names = Vec::new();
        while let ExprKind::Field(parent, name) = &root.kind { names.push(name); root = parent; }
        if !matches!(root.kind, ExprKind::Name(_)) { return None; }
        let binding = root.binding?;
        if !body.iter().any(|item| matches!(item.kind, StmtKind::Let { binding: Some(id), .. } if id == binding)) { return None; }
        let mut ty = fun.bindings[binding].ty;
        let mut path = Vec::new();
        for name in names.into_iter().rev() {
            let Type::Record(record) = ty else { return None; };
            let index = self.fields[record].get(name.as_str()).copied()?;
            path.push(index);
            ty = self.program.records[record].fields[index].ty?;
        }
        let place = if path.is_empty() { format!("value{binding}") }
            else { self.owned_record_leaf_place(fun, binding, &path)? };
        let Type::Map(id) = base.ty.unwrap_or(ty) else { return None; };
        let value = self.program.records[id].map_definition.as_ref()?.value;
        if !self.scoped_record_native_parameter(value) { return None; }
        let native = self.native_type(Type::Map(id));
        Some((place, native))
    }
}

impl Emitter<'_> {
    /// A block containing only storage resets introduces no bindings or control
    /// flow. Emit its resets in the declaring scope to retain fresh lifetimes.
    pub(super) fn map_clear_block_epoch(&self, fun: &CheckedFunction, body: &[Stmt], statement: &Stmt) -> Option<String> {
        if !statement.labels.is_empty() { return None; }
        let StmtKind::Block(inner) = &statement.kind else { return None; };
        if inner.is_empty() { return None; }
        let mut code = String::new();
        for item in inner {
            let reset = self.local_map_clear_epoch(fun, body, item)
                .or_else(|| self.map_clear_block_epoch(fun, body, item))?;
            code.push_str(&reset);
        }
        Some(code)
    }
}

impl Emitter<'_> {
    pub(super) fn map_block_return_places(&self, fun: &CheckedFunction, declaring_body: &[Stmt], inner: &[Stmt]) -> Vec<String> {
        let mut places = Vec::new();
        let mut pending = inner.iter().collect::<Vec<_>>();
        while let Some(statement) = pending.pop() {
            // A labelled loop keeps the same storage result as its unlabelled
            // form. The surrounding lexical for initializer must return it.
            if !statement.labels.is_empty() && !(matches!(statement.kind,StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..}) && !fun.storage_exits.map_iteration_epochs(statement.span).is_empty()) { return Vec::new(); }
            for epoch in fun.storage_exits.map_entry_epochs(statement.span) {
                if declaring_body.iter().any(|item|matches!(item.kind,StmtKind::Let{binding:Some(binding),..} if binding==epoch.place.binding)) {
                    let place=self.map_entry_epoch_place(fun,epoch);
                    if !places.contains(&place){places.push(place);}
                }
            }
            match &statement.kind {
                StmtKind::Block(body) => pending.extend(body.iter()),
                StmtKind::If { then_body, else_body, .. } => {
                    let left = self.map_block_return_places(fun, declaring_body, then_body);
                    let right = self.map_block_return_places(fun, declaring_body, else_body);
                    for place in left.into_iter().chain(right) { if !places.contains(&place) { places.push(place); } }
                },
                StmtKind::Let { .. } | StmtKind::Expr(_) | StmtKind::Assign { .. } | StmtKind::Throw(_) => {
                    if let Some((place, _)) = self.local_map_clear_storage(fun, declaring_body, statement)
                        && !places.contains(&place) { places.push(place); }
                },
                StmtKind::While{..}|StmtKind::Loop{..}|StmtKind::ForOf{..} => {},
                _ => return Vec::new(),
            }
        }
        places.sort();
        places
    }
}

impl Emitter<'_> {
    #[inline(never)]
    pub(super) fn map_clear_if_epoch(&self, output: &mut String, fun: &CheckedFunction, statement: &Stmt, declarations: &[Stmt], depth: usize, loop_label: Option<usize>) -> bool {
        let StmtKind::If { condition, then_body, else_body } = &statement.kind else { return false; };
        let mut places = self.map_block_return_places(fun, declarations, then_body);
        for place in self.map_block_return_places(fun, declarations, else_body) {
            if !places.contains(&place) { places.push(place); }
        }
        for right in [false,true] {
            for epoch in fun.storage_exits.map_branch_restore(statement.span,right) {
                let place=self.map_entry_epoch_place(fun,epoch);
                if !places.contains(&place){places.push(place);}
            }
        }
        if places.is_empty() { return false; }
        places.sort();
        let comma = if places.is_empty() { "" } else { "," };
        let indent = "    ".repeat(depth);
        let pattern = places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(",");
        let condition = self.expr(fun, condition, Mode::Copy);
        let _ = writeln!(output, "{indent}let ({pattern}{comma}) = if {condition} {{");
        {
            let _branch=self.exception_regions.storage_branch();
            self.control_block_epochs(output, fun, then_body, depth + 1, loop_label, Some(declarations));
            for epoch in fun.storage_exits.map_branch_restore(statement.span,false) {
                let _=writeln!(output,"{indent}    {}",self.map_entry_epoch_code(fun,epoch));
            }
            let then_values=places.iter().map(|place|self.map_branch_exit_value(fun,then_body,place)).collect::<Vec<_>>().join(",");
            let _ = writeln!(output, "{indent}    ({then_values}{comma})\n{indent}}} else {{");
        }
        {
            let _branch=self.exception_regions.storage_branch();
            self.control_block_epochs(output, fun, else_body, depth + 1, loop_label, Some(declarations));
            for epoch in fun.storage_exits.map_branch_restore(statement.span,true) {
                let _=writeln!(output,"{indent}    {}",self.map_entry_epoch_code(fun,epoch));
            }
            let else_values=places.iter().map(|place|self.map_branch_exit_value(fun,else_body,place)).collect::<Vec<_>>().join(",");
            let _ = writeln!(output, "{indent}    ({else_values}{comma})\n{indent}}};");
        }
        let boundary=places.iter().flat_map(|place|[self.map_storage_transfer_at_exit(fun,then_body,place),self.map_storage_transfer_at_exit(fun,else_body,place)]).flatten().map(|span|span.end).max().unwrap_or(statement.span.end).max(statement.span.end);
        self.exception_regions.renew_storage(&places,boundary);
        true
    }
}

impl Emitter<'_> {
    fn map_storage_binding(&self, fun:&CheckedFunction, place:&str)->Option<(usize,Vec<String>)> {
        for (binding,value) in fun.bindings.iter().enumerate() {
            if place==format!("value{binding}") {return Some((binding,Vec::new()));}
            let Some(layout)=self.owned_record_layout(fun,binding) else {continue;};
            for leaf in &layout.leaves {
                if leaf.native_name(binding)!=place {continue;}
                let mut ty=value.ty;
                let mut names=Vec::new();
                for &index in &leaf.path {
                    let Type::Record(record)=ty else {return None;};
                    let field=&self.program.records[record].fields[index];
                    names.push(field.name.text.clone());ty=field.ty.unwrap();
                }
                return Some((binding,names));
            }
        }
        None
    }
    pub(super) fn map_storage_value_before(&self,fun:&CheckedFunction,start:usize,place:&str)->super::native_storage_availability::Value {
        let (binding,names)=self.map_storage_binding(fun,place).expect("checked Map storage");
        self.map_storage_value(fun,place,fun.storage_exits.transfer_before(start,binding,&names),start)
    }
    pub(super) fn map_storage_value_at_expression(&self,fun:&CheckedFunction,point:(usize,usize),place:&str)->super::native_storage_availability::Value {
        let (binding,names)=self.map_storage_binding(fun,place).expect("checked Map storage");
        self.map_storage_value(fun,place,fun.storage_exits.transfer_at_expression(point,binding,&names),point.0)
    }
    fn map_storage_value(&self,fun:&CheckedFunction,place:&str,transferred:Option<crate::Span>,start:usize)->super::native_storage_availability::Value {
        let (binding,_)=self.map_storage_binding(fun,place).expect("checked Map storage");
        self.map_epoch_storage_value(fun,place,transferred,start,fun.bindings[binding].ty)
    }
    pub(super) fn map_epoch_storage_value(&self,fun:&CheckedFunction,place:&str,transferred:Option<crate::Span>,start:usize,output:Type)->super::native_storage_availability::Value {
        let (binding,_)=self.map_storage_binding(fun,place).expect("checked Map storage");
        self.map_typed_storage_value(fun,place,transferred,fun.storage_exits.map_native_input(start,binding),output)
    }
    fn map_typed_storage_value(&self,fun:&CheckedFunction,place:&str,transferred:Option<crate::Span>,input:Option<Type>,output:Type)->super::native_storage_availability::Value {
        let (_,fields)=self.map_storage_binding(fun,place).expect("checked Map storage");
        let target=self.map_storage_leaf_type(output,&fields).expect("destination Map path");
        let empty=format!("<{}>::new()",self.native_type(target));
        let live=input.and_then(|input| {
            let source=self.map_storage_leaf_type(input,&fields)?;
            let target=self.map_storage_leaf_type(output,&fields)?;
            if source==target{return None;}
            let (Type::Map(source),Type::Map(target))=(source,target) else{return None;};
            let source=self.program.records[source].map_definition.as_ref()?.value;
            let target_value=self.program.records[target].map_definition.as_ref()?.value;
            let native=self.native_type(Type::Map(target));
            let converted=self.map_entry_payload_projection(source,target_value,"entry_value");
            Some(format!("<{native}>::from_values(({place}).tsr_owned_items().map(|entry|{{let (entry_key,entry_value)=TsrMapEntry::into_pair(entry);(entry_key,{converted})}}))"))
        });
        super::native_storage_availability::Value{place:place.into(),empty,live,transferred}
    }
    pub(super) fn map_storage_leaf_type(&self,mut ty:Type,fields:&[String])->Option<Type> {
        for name in fields {
            let Type::Record(record)=ty else{return None;};
            ty=self.program.records[record].fields.iter().find(|field|field.name.text==*name)?.ty?;
        }
        Some(ty)
    }

}

impl Emitter<'_> {
    pub(super) fn map_storage_transfer_at_exit(&self,fun:&CheckedFunction,body:&[Stmt],place:&str)->Option<crate::Span> {
        let (binding,names)=self.map_storage_binding(fun,place).expect("checked Map storage");
        fun.storage_exits.transfer_at_exit(body,binding,&names)
    }
    pub(super) fn map_branch_exit_value(&self, fun: &CheckedFunction, body: &[Stmt], place: &str) -> String {
        let transferred=self.map_storage_transfer_at_exit(fun,body,place);
        if transferred.is_none() || self.exception_regions.native_storage_available(place,transferred) {return place.to_owned();}
        self.map_empty_storage_value(fun,place)
    }
    pub(super) fn map_completion_exit_value(&self, fun: &CheckedFunction, body: &[Stmt], place: &str) -> String {
        let transferred=self.map_storage_transfer_at_exit(fun,body,place);
        let start=body.iter().map(|statement|statement.span.end).max().unwrap_or(0);
        let (binding,_)=self.map_storage_binding(fun,place).expect("checked Map storage");
        let input=body.first().and_then(|statement|fun.storage_exits.map_native_exit(statement.span.start,binding)).or_else(||fun.storage_exits.map_native_input(start,binding));
        let value=self.map_typed_storage_value(fun,place,transferred,input,fun.bindings[binding].ty);
        if transferred.is_none() || self.exception_regions.native_storage_available(place,transferred) {return value.live.unwrap_or_else(||place.to_owned());}
        value.empty
    }
    pub(super) fn map_empty_storage_value(&self, fun: &CheckedFunction, place: &str) -> String {
        // Source availability remains consumed. This empty native owner only
        // balances branch storage; it has no captured payload to copy or drop.
        for (binding, value) in fun.bindings.iter().enumerate() {
            let ty = if place == format!("value{binding}") { Some(value.ty) }
                else { self.owned_record_layout(fun, binding).and_then(|layout| layout.leaves.iter().find(|leaf| leaf.native_name(binding) == place).map(|leaf| leaf.ty)) };
            if let Some(ty @ Type::Map(_)) = ty { return format!("<{}>::new()", self.native_type(ty)); }
        }
        unreachable!("checked Map storage leaf")
    }
}
