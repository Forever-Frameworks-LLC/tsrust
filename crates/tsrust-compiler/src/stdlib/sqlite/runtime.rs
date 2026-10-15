mod tsrust_native_sqlite {
 use rusqlite::{Connection as Database,params_from_iter,types::{Value,ValueRef},hooks::{AuthAction,AuthContext,Authorization}};
 use std::{cell::Cell,time::Duration};
 pub use rusqlite::types::Value as Parameter;
 #[derive(Debug)] pub struct Error {pub code:&'static str,pub message:String}
 fn failure(code:&'static str,message:impl Into<String>)->Error{Error{code,message:message.into()}}
 impl From<rusqlite::Error> for Error {fn from(error:rusqlite::Error)->Self{
  use rusqlite::{Error as E,ErrorCode as C};
  let code=match &error{E::SqliteFailure(e,_)=>match e.code{C::DatabaseBusy=>"busy",C::DatabaseLocked=>"locked",C::ConstraintViolation=>"constraint",C::AuthorizationForStatementDenied=>"transaction-control",_=>"sqlite"},E::InvalidParameterCount(..)|E::InvalidParameterName(..)=>"parameters",E::MultipleStatement=>"sql",_=>"sqlite"};failure(code,error.to_string())
 }}
 fn authorize(context:AuthContext<'_>)->Authorization{match context.action{AuthAction::Transaction{..}|AuthAction::Savepoint{..}=>Authorization::Deny,_=>Authorization::Allow}}
 pub struct Connection {db:Database,poisoned:Cell<bool>}
 impl Connection {
  pub fn open(path:&str,busy_timeout:Option<i64>)->Result<Self,Error>{
   let timeout=busy_timeout.unwrap_or(5000);if !(0..=2_147_483_647).contains(&timeout){return Err(failure("range","busyTimeout must be between 0 and 2147483647 milliseconds"));}
   if path.is_empty() || path.contains('\0'){return Err(failure("path","Use a nonempty filesystem path or :memory:"));}
   let flags=rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE|rusqlite::OpenFlags::SQLITE_OPEN_CREATE|rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
   let db=Database::open_with_flags(path,flags)?;db.busy_timeout(Duration::from_millis(timeout as u64))?;db.execute_batch("PRAGMA foreign_keys=ON")?;db.authorizer(Some(authorize))?;
   Ok(Self{db,poisoned:Cell::new(false)})
  }
  fn check(&self)->Result<(),Error>{if self.poisoned.get(){Err(failure("closed","Connection cannot be reused after rollback failure"))}else{Ok(())}}
  fn control(&self,sql:&str)->Result<(),Error>{
   self.db.authorizer(None::<fn(AuthContext<'_>)->Authorization>)?;
   let result=self.db.execute_batch(sql).map_err(Error::from);
   if let Err(error)=self.db.authorizer(Some(authorize)){self.poisoned.set(true);return Err(error.into());}result
  }
  pub fn prepare(&self,sql:&str)->Result<Statement<'_>,Error>{self.check()?;Ok(Statement{statement:self.db.prepare(sql)?,connection:self,in_transaction:!self.db.is_autocommit()})}
  pub fn begin(&mut self)->Result<Transaction<'_>,Error>{self.check()?;self.control("BEGIN IMMEDIATE")?;Ok(Transaction{connection:self,finished:false})}
  pub fn close(self)->Result<(),Error>{self.db.close().map_err(|(_,error)|error.into())}
 }
 pub struct Transaction<'a>{connection:&'a mut Connection,finished:bool}
 impl Transaction<'_>{
  pub fn prepare(&self,sql:&str)->Result<Statement<'_>,Error>{if self.connection.db.is_autocommit(){return Err(failure("transaction-ended","Transaction was rolled back by SQLite"));}self.connection.prepare(sql)}
  pub fn commit(mut self)->Result<(),Error>{
   if self.connection.db.is_autocommit(){return Err(failure("transaction-ended","Transaction was rolled back by SQLite"));}
   self.connection.control("COMMIT")?;self.finished=true;Ok(())
  }
  pub fn rollback(mut self)->Result<(),Error>{self.rollback_inner()?;self.finished=true;Ok(())}
  fn rollback_inner(&self)->Result<(),Error>{
   if self.connection.db.is_autocommit(){return Ok(());}
   let result=self.connection.control("ROLLBACK");if result.is_err(){self.connection.poisoned.set(true);}result
  }
 }
 impl Drop for Transaction<'_>{fn drop(&mut self){if !self.finished{let _=self.rollback_inner();}}}
 pub struct Statement<'a>{statement:rusqlite::Statement<'a>,connection:&'a Connection,in_transaction:bool}
 impl Statement<'_>{
  fn check(&self,params:&[Parameter])->Result<(),Error>{
   self.connection.check()?;
   if self.in_transaction && self.connection.db.is_autocommit(){return Err(failure("transaction-ended","Transaction was rolled back by SQLite"));}
   if params.len()!=self.statement.parameter_count(){return Err(failure("parameters","Supply exactly one value for each positional parameter"));}
   if params.iter().any(|v|matches!(v,Value::Real(x) if !x.is_finite())){return Err(failure("range","SQL numbers must be finite"));}Ok(())
  }
  pub fn execute(&mut self,params:&[Parameter])->Result<i64,Error>{
   self.check(params)?;if self.statement.column_count()!=0{return Err(failure("query","execute requires a statement without result columns"));}
   Ok(self.statement.execute(params_from_iter(params))? as i64)
  }
  pub fn query(&mut self,params:&[Parameter])->Result<Vec<Row>,Error>{
   self.check(params)?;if !self.statement.readonly() || self.statement.column_count()==0{return Err(failure("query","query requires a read-only statement with result columns"));}
   let names=self.statement.column_names().into_iter().map(str::to_owned).collect::<Vec<_>>();
   let mut unique=std::collections::HashSet::new();if names.iter().any(|name|!unique.insert(name)){return Err(failure("column","Duplicate column names; use SQL aliases"));}
   let mut cursor=self.statement.query(params_from_iter(params))?;let mut result=Vec::new();
   while let Some(row)=cursor.next()?{let mut values=Vec::with_capacity(names.len());for index in 0..names.len(){values.push(match row.get_ref(index)?{
    ValueRef::Null=>Value::Null,ValueRef::Integer(v)=>Value::Integer(v),ValueRef::Real(v)=>Value::Real(v),ValueRef::Blob(v)=>Value::Blob(v.to_vec()),ValueRef::Text(v)=>Value::Text(std::str::from_utf8(v).map_err(|_|failure("utf8","SQL text is not valid UTF-8"))?.into())
   });}result.push(Row{names:names.clone(),values});}Ok(result)
  }
  pub fn finalize(self)->Result<(),Error>{self.statement.finalize().map_err(Into::into)}
 }
 pub struct Row{names:Vec<String>,values:Vec<Value>}
 impl Row{
  fn get(&self,column:&str)->Result<&Value,Error>{self.names.iter().position(|name|name==column).map(|index|&self.values[index]).ok_or_else(||failure("column",format!("No column named {column}")))}
  pub fn int(&self,column:&str)->Result<i64,Error>{match self.get(column)?{Value::Integer(v)=>Ok(*v),_=>Err(failure("type","Expected INTEGER; use isNull for SQL NULL"))}}
  pub fn number(&self,column:&str)->Result<f64,Error>{match self.get(column)?{Value::Real(v) if v.is_finite()=>Ok(*v),_=>Err(failure("type","Expected finite REAL; integer conversion is explicit"))}}
  pub fn text(&self,column:&str)->Result<String,Error>{match self.get(column)?{Value::Text(v)=>Ok(v.clone()),_=>Err(failure("type","Expected TEXT; use isNull for SQL NULL"))}}
  pub fn blob(&self,column:&str)->Result<Vec<u8>,Error>{match self.get(column)?{Value::Blob(v)=>Ok(v.clone()),_=>Err(failure("type","Expected BLOB; use isNull for SQL NULL"))}}
  pub fn is_null(&self,column:&str)->Result<bool,Error>{Ok(matches!(self.get(column)?,Value::Null))}
 }
}
