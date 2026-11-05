//! Contracts from checked Router registrations; exporting a client never exposes a route.
use super::{Builder, Schema};
use crate::{analysis::CheckedProgram, project::ProjectAnalysis, syntax::{CallTarget, Expr, ExprKind, Stmt, Type}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct RouterContract {
    pub schema_version: u32,
    pub routes: Vec<Route>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Route {
    pub method: String,
    pub path: String,
    /// None means the handler returns a response whose body cannot be inferred.
    pub json: Option<Schema>,
}

fn expressions(body: &[Stmt], mut visit: impl FnMut(&Expr)) {
    // Export runs once, outside compilation; reuse the existing complete AST walker.
    crate::source_walk::body(&mut body.to_vec(), &mut |_| Ok(()), &mut |value| { visit(value); Ok(()) }).expect("infallible contract visitor");
}
fn native_name(expression: &Expr) -> Option<&'static str> {
    let Some(CallTarget::NativeBinding(id))=expression.target else{return None};
    Some(crate::native_bindings::BINDINGS[id].name)
}

impl RouterContract {
    pub fn from_project(project: &ProjectAnalysis) -> Result<Self,String> {
        let program=&project.analysis.program;
        let mut registrations=Vec::new();
        let mut error=None;
        let bodies=program.functions.iter().filter(|function|!function.template).map(|function| &function.body)
            .chain(program.closures.iter().map(|closure| &closure.function.body));
        for body in bodies {
            expressions(body, |expression| {
                if native_name(expression)!=Some("__routerRoute"){return;}
                let ExprKind::Call(_,args)=&expression.kind else{return;};
                let (Some(Expr{kind:ExprKind::String(method),..}),Some(Expr{kind:ExprKind::String(path),..}))=(args.get(1),args.get(2)) else {
                    error=Some("Client generation needs literal Router methods and paths; dynamic registrations cannot form a static contract".to_owned());return;
                };
                let handlers=args.iter().skip(3).filter_map(|arg|arg.closure_origin().or_else(||if let Some(Type::Closure(id))=arg.ty{Some(id)}else{None})).collect::<Vec<_>>();
                registrations.push((method.clone(),path.clone(),handlers));
            });
        }
        if let Some(error)=error{return Err(error);}
        if registrations.is_empty(){return Err("No checked Router routes found in the entry project".into());}
        let mut routes=BTreeMap::<(String,String),Option<Schema>>::new();
        for (method,path,handlers) in registrations {
            let json=handlers.last().and_then(|id| response_schema(program,*id));
            let key=(method,path);
            if let Some(previous)=routes.get(&key) {
                if previous!=&json{return Err(format!("Conflicting Router contracts for {} {}; use one service entry",key.0,key.1));}
            }else{routes.insert(key,json);}
        }
        Ok(Self{schema_version:1,routes:routes.into_iter().map(|((method,path),json)|Route{method,path,json}).collect()})
    }
}

fn response_type(program:&CheckedProgram,ty:Option<Type>)->bool{
    let mut pending=ty.into_iter().collect::<Vec<_>>();let mut seen=HashSet::new();
    while let Some(ty)=pending.pop(){if !seen.insert(ty){continue;}match ty{
        Type::Record(id) if program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::HttpResponse)=>return true,
        Type::Closure(id) if program.task_origins.contains(&id)=>pending.push(program.closures[id].function.result),
        Type::Union(id)=>pending.extend(program.unions[id.index()].members.iter().copied()),
        _=>{}
    }}false
}
fn response_schema(program:&CheckedProgram, origin:usize)->Option<Schema>{
    let mut pending=vec![(true,origin)];let mut seen=HashSet::new();let mut shapes=Vec::new();let mut unknown=false;
    while let Some((closure,id))=pending.pop(){
        if !seen.insert((closure,id)){continue;}
        let function=if closure{&program.closures.get(id)?.function}else{program.functions.get(id)?};
        let returns=crate::source_walk::returned_values(&function.body);if returns.is_empty(){unknown=true;}
        for value in returns{
            let mut found=false;
            crate::source_walk::expression(&mut value.clone(),&mut |expression|{
                if response_type(program,expression.ty){
                    match expression.target{
                        Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=>{pending.push((false,id));found=true;},
                        Some(CallTarget::Closure(id))=>{pending.push((true,id));found=true;},
                        _=>{}
                    }
                    if matches!(expression.kind,ExprKind::Name(_)|ExprKind::Field(_,_)|ExprKind::Index(_,_)){unknown=true;}
                    if let ExprKind::Closure{..}=&expression.kind && let Some(Type::Closure(id))=expression.ty{pending.push((true,id));found=true;}
                }
                if let ExprKind::Call(_,args)=&expression.kind && native_name(expression)==Some("__contextJson"){
                    found=true;let mut ty=None;
                    if let Some(value)=args.get(1){
                        crate::source_walk::expression_pruned(&mut value.clone(),&mut |expression|{
                            if expression.target==Some(CallTarget::JsonEncode) && let ExprKind::Call(_,args)=&expression.kind{ty=args.first().and_then(|arg|arg.ty);return Ok(false);}
                            Ok(true)
                        }).expect("infallible response visitor");
                    }
                    let shape=ty.and_then(|ty|Builder{program,active:HashSet::new()}.schema(ty,false,0).ok());
                    if let Some(shape)=shape{if !shapes.contains(&shape){shapes.push(shape);}}else{unknown=true;}
                }else if native_name(expression).is_some() && response_type(program,expression.ty){unknown=true;}
                Ok(())
            }).expect("infallible response visitor");
            if !found{unknown=true;}
        }
    }
    if unknown||shapes.is_empty(){None}else if shapes.len()==1{shapes.pop()}else{Some(Schema::Union{members:shapes})}
}

impl RouterContract {
    pub fn typescript_package(&self)->Result<BTreeMap<String,String>,String>{
        self.validate()?;
        let mut methods=String::new();
        for route in &self.routes {
            let ty=route.json.as_ref().map(super::typescript::ty).unwrap_or_else(||"Response".into());
            methods.push_str(&format!("  {}(path: {}, options?: RequestOptions): Promise<{}>;\n",route.method.to_lowercase(),serde_json::to_string(&route.path).unwrap(),ty));
        }
        let declarations=format!("export interface RequestOptions extends Omit<RequestInit, 'method'> {{ param?: Record<string,string>; query?: Record<string,string|number|boolean|readonly string[]|undefined>; json?: unknown; }}\nexport interface Client {{\n{methods}}}\nexport declare class HttpError extends Error {{ readonly response: Response; }}\nexport declare const contract: Readonly<{{ schemaVersion: 1; routes: readonly Readonly<{{method:string;path:string;json:unknown}}>[] }}>;\nexport declare function createClient(baseURL: string | URL, options?: {{ fetch?: typeof globalThis.fetch }}): Client;\n");
        let mut files=self.description_files()?;
        files.insert("index.js".into(),include_str!("router-client.js").replace("__TSRUST_ROUTER_CONTRACT__",&serde_json::to_string(self).map_err(|error|error.to_string())?));
        files.insert("index.d.ts".into(),declarations);
        files.insert("wire.js".into(),include_str!("wire.js").into());
        files.insert("package.json".into(),"{\"name\":\"tsrust-service-client\",\"version\":\"0.0.0\",\"private\":true,\"type\":\"module\",\"exports\":{\".\":{\"types\":\"./index.d.ts\",\"default\":\"./index.js\"}}}\n".into());
        files.insert("README.md".into(),"Generated from checked Router source. Import createClient, pass the service base URL, then call client.get('/path') or another registered method. Options accept ordinary fetch settings plus param, query and json. JSON response shapes are checked at runtime. Non-2xx responses throw HttpError with the original response. Other body types return a Response. Regenerate after route or response changes. Generation does not start a service, add routes or change middleware.\n".into());
        Ok(files)
    }
    pub fn parse(text:&str)->Result<Self,String>{
        let contract:Self=serde_json::from_str(text).map_err(|error|format!("Invalid Router contract: {error}"))?;
        contract.validate()?;Ok(contract)
    }
    fn validate(&self)->Result<(),String>{
        if self.schema_version!=1 || self.routes.is_empty(){return Err("Unsupported or empty Router contract".into());}
        let mut seen=HashSet::new();
        for route in &self.routes {
            if !matches!(route.method.as_str(),"GET"|"POST"|"PUT"|"PATCH"|"DELETE"|"HEAD"|"OPTIONS"|"CONNECT"|"TRACE") || (!route.path.starts_with('/') && route.path!="*") || route.path.contains(['?','#','\\']) || route.path.chars().any(char::is_control) || !seen.insert((&route.method,&route.path)) {
                return Err("Router contracts require unique HTTP methods and absolute route paths without query strings, fragments or backslashes".into());
            }
            if let Some(schema)=&route.json {
                let manifest=super::Manifest{schema_version:1,operations:vec![super::Operation{name:"response".into(),kind:super::Kind::Query,input:Schema::Void,output:schema.clone()}]};
                manifest.validate()?;
            }
        }
        Ok(())
    }
    fn description_files(&self)->Result<BTreeMap<String,String>,String>{
        Ok(BTreeMap::from([("http-api.json".into(),serde_json::to_string_pretty(self).map_err(|error|error.to_string())?), ("openapi.json".into(),serde_json::to_string_pretty(&self.openapi()?).map_err(|error|error.to_string())?)]))
    }
}

impl RouterContract {
    pub fn rust_package(&self)->Result<BTreeMap<String,String>,String>{
        self.validate()?;let mut files=super::rust_client::router_package(self)?;files.extend(self.description_files()?);Ok(files)
    }
    pub fn openapi(&self)->Result<serde_json::Value,String>{
        use serde_json::json;
        self.validate()?;let mut paths=BTreeMap::<String,BTreeMap<String,serde_json::Value>>::new();
        for route in &self.routes{
            let path=route.path.split('/').map(|segment|segment.strip_prefix(':').map(|name|format!("{{{name}}}")).unwrap_or_else(||if segment=="*"{"{wildcard}".into()}else{segment.into()})).collect::<Vec<_>>().join("/");
            let parameters=route.path.split('/').filter_map(|segment|segment.strip_prefix(':').or_else(||(segment=="*").then_some("wildcard"))).map(|name|json!({"name":name,"in":"path","required":true,"schema":{"type":"string"}})).collect::<Vec<_>>();
            let mut success=json!({"description":"Successful handler response"});
            if let Some(schema)=&route.json{success["content"]=json!({"application/json":{"schema":native_schema(schema)}});}
            paths.entry(if path.starts_with('/'){path}else{format!("/{path}")}).or_default().insert(route.method.to_lowercase(),json!({"parameters":parameters,"responses":{"2XX":success,"default":{"description":"Other response; access and middleware are application-owned"}}}));
        }
        Ok(json!({"openapi":"3.1.1","info":{"title":"Router service","version":"0.0.0"},"paths":paths}))
    }
}

fn native_schema(schema:&Schema)->serde_json::Value{
    let mut value=super::description::schema(schema);let mut pending=vec![&mut value];
    while let Some(value)=pending.pop(){match value{
        serde_json::Value::Object(fields)=>{
            if fields.get("type").and_then(serde_json::Value::as_str)==Some("integer"){
                fields.remove("minimum");fields.remove("maximum");fields.insert("format".into(),serde_json::json!("int64"));
            }
            pending.extend(fields.values_mut());
        }
        serde_json::Value::Array(items)=>pending.extend(items),_=>{}
    }}value
}
