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
            let _=writeln!(output,"{indent}let ({pattern},) = {{");
            self.map_iteration_work(output,fun,statement,declarations,depth+1,false);
            let _=writeln!(output,"{indent}    ({names},)\n{indent}}};");
        }
        let _=writeln!(output,"{indent}'loop_{label}: while {} {{",self.expr(fun,condition,Mode::Copy));
        let _=writeln!(output,"{indent}    ({names},) = {{");
        self.map_iteration_work(output,fun,statement,declarations,depth+2,true);
        let _=writeln!(output,"{indent}        ({names},)\n{indent}    }};\n{indent}}}");
        self.map_post_entry_epochs(output,fun,statement.span,depth);
        true
    }
    fn map_iteration_work(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,declarations:&[Stmt],depth:usize,widen:bool) {
        let epochs=fun.storage_exits.map_iteration_epochs(statement.span);
        let indent="    ".repeat(depth);
        if widen {
            for epoch in epochs {
                let working=crate::analysis::MapEntryEpoch{place:epoch.entry.place.clone(),input:epoch.entry.output,output:epoch.entry.input,empty:epoch.back.empty,before:true};
                if working.input!=working.output{let _=writeln!(output,"{indent}{}",self.map_entry_epoch_code(fun,&working));}
            }
        }
        let body=match &statement.kind{StmtKind::While{body,..}|StmtKind::Loop{body,..}|StmtKind::ForOf{body,..}=>body,_=>unreachable!("checked iteration body")};
        let places=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,&epoch.entry)).collect::<Vec<_>>();
        let pattern=places.iter().map(|place|format!("mut {place}")).collect::<Vec<_>>().join(",");
        let names=places.join(",");
        let _=writeln!(output,"{indent}let ({pattern},) = {{");
        self.control_block_epochs(output,fun,body,depth+1,Some(statement.span.start),Some(declarations));
        let _=writeln!(output,"{indent}    ({names},)\n{indent}}};");
        if let StmtKind::Loop{update,..}=&statement.kind {
            self.control_block_epochs(output,fun,update,depth,Some(statement.span.start),Some(declarations));
        }
        for epoch in epochs {
            if epoch.back.input!=epoch.back.output{let _=writeln!(output,"{indent}{}",self.map_entry_epoch_code(fun,&epoch.back));}
        }
    }
    pub(super) fn map_for_of_iteration_body(&self,output:&mut String,fun:&CheckedFunction,statement:&Stmt,depth:usize,declarations:Option<&[Stmt]>)->bool {
        let epochs=fun.storage_exits.map_iteration_epochs(statement.span);
        if epochs.is_empty(){return false;}
        let StmtKind::ForOf{body,..}=&statement.kind else{return false;};
        let names=epochs.iter().map(|epoch|self.map_entry_epoch_place(fun,&epoch.entry)).collect::<Vec<_>>().join(",");
        let indent="    ".repeat(depth);
        let _=writeln!(output,"{indent}({names},) = {{");
        self.map_iteration_work(output,fun,statement,declarations.unwrap_or(body),depth+1,true);
        let _=writeln!(output,"{indent}    ({names},)\n{indent}}};");
        true
    }

}
