mod tsrust_native_sql {
    use sqlx::{Any, AnyPool, Arguments, Column, Row as _, ValueRef, Executor, Statement, SqlSafeStr};
    use sqlx::any::{AnyArguments, AnyPoolOptions, AnyRow, AnyTypeInfoKind};
    use std::time::Duration;

    #[derive(Debug)]
    pub struct Error { pub code: &'static str, pub message: String }
    fn error(code: &'static str, message: impl Into<String>) -> Error { Error { code, message: message.into() } }
    fn database(error: sqlx::Error) -> Error {
        let code=match &error {sqlx::Error::PoolTimedOut=>"acquire-timeout",sqlx::Error::PoolClosed=>"closed",sqlx::Error::ColumnNotFound(_)=>"column",sqlx::Error::ColumnDecode{..}|sqlx::Error::Decode(_)=>"type",_=>"database"};
        // Preserve server SQLSTATE alongside the database message.
        let message=if let sqlx::Error::Database(db)=&error {format!("{}: {}",db.code().unwrap_or_default(),db.message())}else{error.to_string()};
        self::error(code,message)
    }
    pub(super) fn exact_real(value:f64)->Result<f32,Error>{
        if !value.is_finite(){return Err(error("range","SQL numbers must be finite"));}
        let narrowed=value as f32;
        if !narrowed.is_finite(){return Err(error("range","Value exceeds REAL"));}
        if f64::from(narrowed)!=value{return Err(error("precision","Value is not exactly representable as REAL"));}
        Ok(narrowed)
    }
    pub enum Parameter { Integer(i64), Real(f64), Boolean(bool), Text(String), Blob(Vec<u8>), Null }
    async fn arguments(connection:&mut sqlx::AnyConnection,sql:&str,parameters:&[Parameter])->Result<AnyArguments,Error>{
        let statement=connection.prepare(sqlx::AssertSqlSafe(sql).into_sql_str()).await.map_err(database)?;
        let (types,count)=match statement.parameters(){Some(sqlx::Either::Left(types))=>(Some(types),types.len()),Some(sqlx::Either::Right(count))=>(None,count),None=>(None,0)};
        if count!=parameters.len(){return Err(error("parameter",format!("Expected {count} bound values, received {}",parameters.len())));}
        let mut args=AnyArguments::default();
        for (index,value) in parameters.iter().enumerate(){
            let kind=types.map(|types|types[index].kind());
            let result=match value {
                Parameter::Integer(value)=>match kind {
                    Some(AnyTypeInfoKind::SmallInt)=>args.add(i16::try_from(*value).map_err(|_|error("range","Value exceeds SMALLINT"))?),
                    Some(AnyTypeInfoKind::Integer)=>args.add(i32::try_from(*value).map_err(|_|error("range","Value exceeds INTEGER"))?),
                    _=>args.add(*value),
                },
                Parameter::Real(value)=>{if !value.is_finite(){return Err(error("range","SQL numbers must be finite"));}if kind==Some(AnyTypeInfoKind::Real){args.add(exact_real(*value)?)}else{args.add(*value)}},
                Parameter::Boolean(value)=>args.add(*value),Parameter::Text(value)=>args.add(value.clone()),Parameter::Blob(value)=>args.add(value.clone()),
                Parameter::Null=>match kind{
                    Some(AnyTypeInfoKind::Text)=>args.add(Option::<String>::None),Some(AnyTypeInfoKind::Blob)=>args.add(Option::<Vec<u8>>::None),
                    Some(AnyTypeInfoKind::Bool)=>args.add(Option::<bool>::None),Some(AnyTypeInfoKind::BigInt)=>args.add(Option::<i64>::None),
                    Some(AnyTypeInfoKind::SmallInt)=>args.add(Option::<i16>::None),Some(AnyTypeInfoKind::Real)=>args.add(Option::<f32>::None),
                    Some(AnyTypeInfoKind::Double)=>args.add(Option::<f64>::None),_=>args.add(Option::<i32>::None),
                },
            };result.map_err(|e|error("parameter",e.to_string()))?;
        }
        Ok(args)
    }
    // A single prepared statement cannot escape the transaction lifecycle through raw control SQL.
    pub(super) fn validate_statement(sql:&str,transaction:bool)->Result<(),Error>{
        let bytes=sql.as_bytes();let mut i=0;let mut first=None;let mut ended=false;
        while i<bytes.len(){
            if bytes[i].is_ascii_whitespace(){i+=1;continue;}
            if bytes[i..].starts_with(b"--"){while i<bytes.len()&&bytes[i]!=b'\n'{i+=1;}continue;}
            if bytes[i..].starts_with(b"/*!") || bytes[i..].starts_with(b"/*M!"){return Err(error("sql","Executable SQL comments are unsupported"));}
            if bytes[i..].starts_with(b"/*") {i+=2;let mut depth=1;while i<bytes.len()&&depth>0{if bytes[i..].starts_with(b"/*"){depth+=1;i+=2;}else if bytes[i..].starts_with(b"*/"){depth-=1;i+=2;}else{i+=1;}}if depth>0{return Err(error("sql","Unclosed SQL comment"));}continue;}
            if ended{return Err(error("sql","Use one SQL statement per call"));}
            if bytes[i]==b';'{ended=true;i+=1;continue;}
            if matches!(bytes[i],b'\''|b'"'|b'`'){let quote=bytes[i];i+=1;while i<bytes.len(){if bytes[i]==b'\\'{i=(i+2).min(bytes.len());}else if bytes[i]==quote {i+=1;if i<bytes.len()&&bytes[i]==quote{i+=1;}else{break;}}else{i+=1;}}continue;}
            if first.is_none(){let start=i;while i<bytes.len()&&bytes[i].is_ascii_alphabetic(){i+=1;}if i==start{return Err(error("sql","SQL must start with a supported statement keyword"));}first=Some(sql[start..i].to_ascii_uppercase());}else{i+=1;}
        }
        let keyword=first.ok_or_else(||error("sql","SQL must not be empty"))?;
        let allowed=if transaction{matches!(keyword.as_str(),"SELECT"|"INSERT"|"UPDATE"|"DELETE"|"WITH")}else{matches!(keyword.as_str(),"SELECT"|"INSERT"|"UPDATE"|"DELETE"|"WITH"|"CREATE"|"ALTER"|"DROP"|"TRUNCATE")};
        if !allowed{return Err(error("sql","Use begin, commit and rollback for transactions; session control SQL is unsupported"));}
        Ok(())
    }
    pub struct Pool { inner:AnyPool, sqlite:bool }
    pub struct Transaction<'a> { inner:Option<sqlx::Transaction<'static,Any>>, owner:std::marker::PhantomData<&'a Pool> }
    pub struct Row { inner:AnyRow }
    fn rows(values:Vec<AnyRow>)->Result<Vec<Row>,Error>{
        for row in &values{let mut names=std::collections::HashSet::new();for column in row.columns(){if !names.insert(column.name()){return Err(error("column","Duplicate column names; use SQL aliases"));}}}
        Ok(values.into_iter().map(|inner|Row{inner}).collect())
    }
    impl Pool {
        pub async fn connect(url:&str,max:Option<i64>,timeout:Option<i64>)->Result<Self,Error>{
            let max=max.unwrap_or(5);let timeout=timeout.unwrap_or(30000);
            if !(1..=1024).contains(&max)||!(1..=31536000000).contains(&timeout){return Err(error("range","maxConnections must be 1..1024 and acquireTimeout must be 1..31536000000 milliseconds"));}
            if !["postgres://","postgresql://","mysql://","sqlite:"].iter().any(|prefix|url.starts_with(prefix)){return Err(error("url","Expected a postgres, mysql or sqlite URL"));}
            sqlx::any::install_default_drivers();
            let inner=AnyPoolOptions::new().max_connections(max as u32).acquire_timeout(Duration::from_millis(timeout as u64)).connect(url).await.map_err(database)?;
            Ok(Self{inner,sqlite:url.starts_with("sqlite:")})
        }
        pub async fn migrate(&self,directory:&str)->Result<(),Error>{
            let directory=std::path::PathBuf::from(directory);
            let migrator=tokio::task::spawn_blocking(move||load_migrations(&directory)).await.map_err(|e|error("migration-source",e.to_string()))??;
            let mut connection=self.inner.acquire().await.map_err(database)?;
            if self.sqlite {
                // Inspect the actual database: SQLite URI filenames can conceal memory/temporary modes.
                let file=sqlx::query_scalar::<Any,String>("SELECT file FROM pragma_database_list WHERE name = 'main'").fetch_one(&mut *connection).await.map_err(database)?;
                if file.is_empty(){return Err(error("migration-source","Migrations require file-backed SQLite; memory and temporary databases are unsupported"));}
            }
            let mut guard=MigrationConnection(Some(connection));
            let result=migrator.run(&mut **guard.0.as_mut().unwrap()).await.map_err(migration_error);
            if result.is_ok(){drop(guard.0.take());}else{guard.close().await;}
            // Errors and cancellation must never return a possibly locked connection to the pool.
            drop(guard);
            result
        }
        pub async fn begin(&self)->Result<Transaction<'_>,Error>{Ok(Transaction{inner:Some(self.inner.begin().await.map_err(database)?),owner:std::marker::PhantomData})}
        pub async fn close(self)->Result<(),Error>{self.inner.close().await;Ok(())}
        pub async fn execute(&self,sql:&str,parameters:&[Parameter])->Result<i64,Error>{
            validate_statement(sql,false)?;let mut connection=self.inner.acquire().await.map_err(database)?;let args=arguments(&mut connection,sql,parameters).await?;
            let result=sqlx::query_with(sqlx::AssertSqlSafe(sql),args).execute(&mut *connection).await.map_err(database)?;
            i64::try_from(result.rows_affected()).map_err(|_|error("range","Affected row count exceeds int"))
        }
        pub async fn query(&self,sql:&str,parameters:&[Parameter])->Result<Vec<Row>,Error>{
            validate_statement(sql,false)?;let mut connection=self.inner.acquire().await.map_err(database)?;let args=arguments(&mut connection,sql,parameters).await?;
            rows(sqlx::query_with(sqlx::AssertSqlSafe(sql),args).fetch_all(&mut *connection).await.map_err(database)?)
        }
    }
    struct MigrationConnection(Option<sqlx::pool::PoolConnection<Any>>);
    impl MigrationConnection {
        async fn close(&mut self){
            if let Some(connection)=self.0.take(){
                let (done,wait)=tokio::sync::oneshot::channel();
                super::tsrust_native_futures::cleanup(async move{let _=connection.close().await;let _=done.send(());});
                let _=wait.await;
            }
        }
    }
    impl Drop for MigrationConnection {
        fn drop(&mut self){if let Some(connection)=self.0.take(){super::tsrust_native_futures::cleanup(async move{let _=connection.close().await;});}}
    }
    fn migration_error(failure:sqlx::migrate::MigrateError)->Error{
        use sqlx::migrate::MigrateError::*;
        let code=match &failure{Dirty(_)=>"migration-dirty",VersionMissing(_)|VersionMismatch(_)=>"migration-drift",Source(_)=>"migration-source",_=>"migration-failed"};
        error(code,failure.to_string())
    }
    pub(super) fn load_migrations(directory:&std::path::Path)->Result<sqlx::migrate::Migrator,Error>{
        use sqlx::migrate::{Migration,MigrationType,Migrator};
        let source=|e:std::io::Error|error("migration-source",e.to_string());
        let mut migrations=Vec::new();let mut versions=std::collections::HashSet::new();
        for entry in std::fs::read_dir(directory).map_err(source)?{
            let entry=entry.map_err(source)?;let path=entry.path();
            if path.extension().is_none_or(|extension|extension!="sql"){continue;}
            let name=entry.file_name();let name=name.to_str().ok_or_else(||error("migration-source","Migration filenames must be UTF-8"))?;
            let invalid=||error("migration-source",format!("Expected unique positive VERSION_description.sql: {name}"));
            let (version,description)=name.strip_suffix(".sql").and_then(|name|name.split_once('_')).ok_or_else(invalid)?;
            if version.is_empty()||!version.bytes().all(|b|b.is_ascii_digit())||description.is_empty()||description.ends_with(".up")||description.ends_with(".down"){return Err(invalid());}
            let version=version.parse::<i64>().map_err(|_|invalid())?;
            if version<=0||!versions.insert(version){return Err(invalid());}
            let sql=std::fs::read_to_string(path).map_err(source)?;
            if sql.trim().is_empty()||sql.starts_with("-- no-transaction"){return Err(error("migration-source","Empty and no-transaction migrations are unsupported"));}
            migrations.push(Migration::new(version,description.replace('_'," ").into(),MigrationType::Simple,sqlx::AssertSqlSafe(sql).into_sql_str(),false));
        }
        migrations.sort();
        Ok(Migrator{migrations:migrations.into(),..Migrator::DEFAULT})
    }
    struct RollbackOnCancel<'a> {inner:&'a mut Option<sqlx::Transaction<'static,Any>>,armed:bool}
    impl Drop for RollbackOnCancel<'_> {fn drop(&mut self){if self.armed{rollback_on_drop(self.inner);}}}
    fn rollback_on_drop(inner:&mut Option<sqlx::Transaction<'static,Any>>){if let Some(transaction)=inner.take(){super::tsrust_native_futures::cleanup(async move{let _=transaction.rollback().await;});}}
    fn inactive()->Error{error("transaction","Transaction is no longer active")}
    impl Transaction<'_> {
        pub async fn execute(&mut self,sql:&str,parameters:&[Parameter])->Result<i64,Error>{
            validate_statement(sql,true)?;
            let mut guard=RollbackOnCancel{inner:&mut self.inner,armed:true};
            let transaction=guard.inner.as_mut().ok_or_else(inactive)?;
            let args=arguments(&mut *transaction,sql,parameters).await?;
            let result=sqlx::query_with(sqlx::AssertSqlSafe(sql),args).execute(&mut **transaction).await.map_err(database)?;
            let count=i64::try_from(result.rows_affected()).map_err(|_|error("range","Affected row count exceeds int"))?;
            guard.armed=false;Ok(count)
        }
        pub async fn query(&mut self,sql:&str,parameters:&[Parameter])->Result<Vec<Row>,Error>{
            validate_statement(sql,true)?;
            let mut guard=RollbackOnCancel{inner:&mut self.inner,armed:true};
            let transaction=guard.inner.as_mut().ok_or_else(inactive)?;
            let args=arguments(&mut *transaction,sql,parameters).await?;
            let result=rows(sqlx::query_with(sqlx::AssertSqlSafe(sql),args).fetch_all(&mut **transaction).await.map_err(database)?)?;
            guard.armed=false;Ok(result)
        }
        pub async fn commit(mut self)->Result<(),Error>{self.inner.take().ok_or_else(inactive)?.commit().await.map_err(database)}
        pub async fn rollback(mut self)->Result<(),Error>{self.inner.take().ok_or_else(inactive)?.rollback().await.map_err(database)}
    }
    impl Drop for Transaction<'_> {fn drop(&mut self){rollback_on_drop(&mut self.inner);}}
    impl Drop for Pool {fn drop(&mut self){let pool=self.inner.clone();super::tsrust_native_futures::cleanup(async move{pool.close().await;});}}
    impl Row {
        pub fn int(&self,column:&str)->Result<i64,Error>{self.inner.try_get(column).map_err(database)}
        pub fn number(&self,column:&str)->Result<f64,Error>{let value:f64=self.inner.try_get_unchecked(column).map_err(database)?;if !value.is_finite(){return Err(error("range","SQL number is not finite"));}Ok(value)}
        pub fn boolean(&self,column:&str)->Result<bool,Error>{self.inner.try_get(column).map_err(database)}
        pub fn text(&self,column:&str)->Result<String,Error>{self.inner.try_get(column).map_err(database)}
        pub fn blob(&self,column:&str)->Result<Vec<u8>,Error>{self.inner.try_get(column).map_err(database)}
        pub fn is_null(&self,column:&str)->Result<bool,Error>{Ok(self.inner.try_get_raw(column).map_err(database)?.is_null())}
    }
}
