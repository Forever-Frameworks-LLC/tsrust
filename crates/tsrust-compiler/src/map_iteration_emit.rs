//! Move carried survivors into and out of each native iteration scope.
use super::*;
impl Emitter<'_> {
    pub(super) fn map_iteration_epoch(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,declarations:&[Stmt],depth:usize)->bool {
        let epochs=fun.storage_exits.map_iteration_epochs(statement.span);
        if epochs.is_empty(){return false;}
        if matches!(statement.kind,StmtKind::ForOf{..}) {
            self.for_of_loop(output,fun,statement,depth,Some(declarations),&[]);
            self.map_post_entry_epochs(output,fun,statement.span,depth);
            return true;
        }
        let (condition,post_test)=match &statement.kind {
            StmtKind::While{condition,..}=>(condition,false),
            StmtKind::Loop{condition:Some(condition),post_test,..}=>(condition,*post_test),
            _=>return false,
        };
        let places=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,&epoch.entry)).collect::<Vec<_>>();
        let names=places.join(",");let indent="    ".repeat(depth);
        let label=statement.span.start;
        let _scope=self.control_targets.enter(if matches!(statement.kind,StmtKind::Loop{..}){ControlTarget::Tail(label)}else{ControlTarget::Loop(label)});
        if post_test {
            let pattern=places.iter().map(|place|format!("mut {place}")).collect::<Vec<_>>().join(",");
            let _=writeln!(output,"{indent}let (mut map_stop_{label},{pattern},) = {{");
            self.map_iteration_work(output,fun,statement,declarations,depth+1,false);
            let _=writeln!(output,"{indent}    (map_stop_{label},{names},)\n{indent}}};");
            // A body that cannot reach its tail has no checked condition or
            // update to emit. Return/error/control already ran its cleanup.
            if matches!(statement.kind,StmtKind::Loop{tail_reachable:false,..}) {
                self.map_post_entry_epochs(output,fun,statement.span,depth);
                return true;
            }
        }
        if !post_test {let _=writeln!(output,"{indent}let mut map_stop_{label}=false;");}
        let _=writeln!(output,"{indent}'loop_{label}: while !map_stop_{label} && ({}) {{",self.expr(fun,condition,Mode::Copy));
        let _=writeln!(output,"{indent}    (map_stop_{label},{names},) = {{");
        self.map_iteration_work(output,fun,statement,declarations,depth+2,true);
        let _=writeln!(output,"{indent}        (map_stop_{label},{names},)\n{indent}    }};\n{indent}}}");
        self.map_post_entry_epochs(output,fun,statement.span,depth);
        true
    }
    fn map_iteration_work(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,declarations:&[Stmt],depth:usize,widen:bool) {
        let epochs=fun.storage_exits.map_iteration_epochs(statement.span);
        let label=statement.span.start;
        let indent="    ".repeat(depth);
        if widen {
            for epoch in epochs {
                let working=crate::analysis::MapEntryEpoch{place:epoch.entry.place.clone(),input:epoch.entry.output,output:epoch.entry.input,empty:epoch.carrier_empty,before:true};
                if working.input!=working.output{let _=writeln!(output,"{indent}{}",self.map_entry_epoch_code(fun,&working));}
            }
        }
        let body=match &statement.kind{StmtKind::While{body,..}|StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>body,_=>unreachable!("checked iteration body")};
        let places=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,&epoch.entry)).collect::<Vec<_>>();
        let pattern=places.iter().map(|place|format!("mut {place}")).collect::<Vec<_>>().join(",");
        let names=places.join(",");
        let _=writeln!(output,"{indent}let (map_stop_{label},{pattern},) = 'map_body_{label}: {{");
        self.control_block_epochs(output,fun,body,depth+1,Some(statement.span.start),Some(declarations));
        let abrupt=fun.storage_exits.has_map_iteration_controls(label);
        if abrupt {for epoch in epochs {
            if epoch.body.input!=epoch.body.output{let _=writeln!(output,"{indent}    {}",self.map_entry_epoch_code(fun,&epoch.body));}
        }}
        let _=writeln!(output,"{indent}    (false,{names},)\n{indent}}};");
        if !abrupt {
            if let StmtKind::Loop{update,tail_reachable:true,..}=&statement.kind {
                self.control_block_epochs(output,fun,update,depth,Some(label),Some(declarations));
            }
            for epoch in epochs {
                if epoch.back.input!=epoch.back.output{let _=writeln!(output,"{indent}{}",self.map_entry_epoch_code(fun,&epoch.back));}
            }
            return;
        }
        // A break can retain storage even when no backedge exists. Select the
        // reaching edge proof before converting its actual working owner.
        let _=writeln!(output,"{indent}let ({pattern},) = if !map_stop_{label} {{");
        if let StmtKind::Loop{update,tail_reachable:true,..}=&statement.kind {
            self.control_block_epochs(output,fun,update,depth+1,Some(statement.span.start),Some(declarations));
        }
        for epoch in epochs {
            if epoch.back.input!=epoch.back.output{let _=writeln!(output,"{indent}    {}",self.map_entry_epoch_code(fun,&epoch.back));}
        }
        let _=writeln!(output,"{indent}    ({names},)\n{indent}}} else {{");
        for epoch in epochs {
            let stopped=crate::analysis::MapEntryEpoch{place:epoch.entry.place.clone(),input:epoch.body.output,output:epoch.back.output,empty:epoch.break_empty,before:false};
            if stopped.input!=stopped.output{let _=writeln!(output,"{indent}    {}",self.map_entry_epoch_code(fun,&stopped));}
        }
        let _=writeln!(output,"{indent}    ({names},)\n{indent}}};");
    }
    pub(super) fn map_for_of_iteration_body(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,depth:usize,declarations:Option<&[Stmt]>)->bool {
        let epochs=fun.storage_exits.map_iteration_epochs(statement.span);
        if epochs.is_empty(){return false;}
        let StmtKind::ForOf{body,..}=&statement.kind else{return false;};
        let label=statement.span.start;
        let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,&epoch.entry)).collect::<Vec<_>>().join(",");
        let indent="    ".repeat(depth);
        let _=writeln!(output,"{indent}let map_stop_{label};\n{indent}(map_stop_{label},{names},) = {{");
        self.map_iteration_work(output,fun,statement,declarations.unwrap_or(body),depth+1,true);
        let _=writeln!(output,"{indent}    (map_stop_{label},{names},)\n{indent}}};\n{indent}if map_stop_{label} {{break 'loop_{label};}}");
        true
    }

    pub(super) fn map_iteration_control_code(&self,fun:&CheckedFunction,statement:&Stmt)->Option<String> {
        let (label,continuing,epochs)=fun.storage_exits.map_iteration_control(statement.span)?;
        let mut code=String::new();
        for epoch in epochs {if epoch.input!=epoch.output{code.push_str(&self.map_entry_epoch_code(fun,epoch));}}
        let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,epoch)).collect::<Vec<_>>().join(",");
        code.push_str(&format!("break 'map_body_{label} ({},{names},);",!continuing));
        Some(code)
    }

}
