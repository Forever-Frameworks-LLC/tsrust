#[test]
fn sql_real_conversion_rejects_rounding_and_underflow(){
 use tsrust_native_sql::exact_real;
 assert_eq!(exact_real(1.25).unwrap(),1.25_f32);
 assert!(exact_real(-0.0).unwrap().is_sign_negative());
 for value in [0.1,16777217.0,1e-300,-1e-300]{assert_eq!(exact_real(value).unwrap_err().code,"precision");}
 for value in [f64::MAX,f64::NAN,f64::INFINITY]{assert_eq!(exact_real(value).unwrap_err().code,"range");}
}
#[test]
fn sql_lifecycle_commands_and_batches_are_rejected(){
 use tsrust_native_sql::*;
 for sql in ["COMMIT","ROLLBACK","BEGIN","START TRANSACTION","SAVEPOINT x","RELEASE x","SET autocommit=1","SELECT 1; COMMIT","/*! COMMIT */", "/* outer /* nested */ */ ROLLBACK"]{assert!(validate_statement(sql,false).is_err(),"{sql}");}
 for sql in ["SELECT ';' AS value", "/* comment */ SELECT 1; -- trailing", "INSERT INTO items VALUES (?)"]{assert!(validate_statement(sql,true).is_ok(),"{sql}");}
 assert!(validate_statement("CREATE TABLE items(id BIGINT)",true).is_err());
}
#[test]
fn sql_pool_ranges_fail_before_connecting(){
 tsrust_native_futures::block_on(async {
  for (max,timeout) in [(0,1),(1025,1),(1,0),(1,31536000001)]{assert!(matches!(tsrust_native_sql::Pool::connect("sqlite::memory:",Some(max),Some(timeout)).await,Err(error)if error.code=="range"));}
 });
}

#[test]
#[ignore="opt-in SQLx database contract; use tools/check-async-sql.mjs --docker"]
fn sql_drivers_bind_scalars_and_release_transactions(){
 use tsrust_native_sql::{Pool,Parameter};
 let postgres=std::env::var("TSRUST_TEST_POSTGRES_URL").expect("PostgreSQL test URL required");
 let mysql=std::env::var("TSRUST_TEST_MYSQL_URL").expect("MySQL test URL required");
 for (driver,url) in [("sqlite","sqlite::memory:"),("postgres",postgres.as_str()),("mysql",mysql.as_str())]{
  tsrust_native_futures::block_on(async {
   let pool=Pool::connect(url,Some(1),Some(5000)).await.unwrap_or_else(|error|panic!("{driver} connect: {error:?}"));
   if driver=="postgres" {
    for (value,code) in [(0.1,"precision"),(1e-300,"precision"),(f64::MAX,"range")]{
     let failure=pool.query("SELECT CAST($1 AS REAL) AS value",&[Parameter::Real(value)]).await.err().expect("lossy REAL must reject");assert_eq!(failure.code,code);
    }

    let values=pool.query("SELECT CAST($1 AS REAL) AS r,CAST($2 AS SMALLINT) AS s,CAST($3 AS INTEGER) AS i",&[Parameter::Real(1.25),Parameter::Integer(32767),Parameter::Integer(2147483647)]).await.unwrap();
    assert_eq!(values[0].number("r").unwrap(),1.25);assert_eq!(values[0].int("s").unwrap(),32767);assert_eq!(values[0].int("i").unwrap(),2147483647);
   }
   pool.execute("DROP TABLE IF EXISTS tsrust_s18_runtime",&[]).await.unwrap();
   let binary=if driver=="postgres"{"BYTEA"}else{"BLOB"};let boolean=if driver=="postgres"{"BOOLEAN"}else{"BIGINT"};
   pool.execute(&format!("CREATE TABLE tsrust_s18_runtime(i BIGINT,r DOUBLE PRECISION,t VARCHAR(100),b {binary},n VARCHAR(100),flag {boolean})"),&[]).await.unwrap();
   let slots=if driver=="postgres"{"$1,$2,$3,$4,$5,$6"}else{"?,?,?,?,?,?"};let insert=format!("INSERT INTO tsrust_s18_runtime VALUES ({slots})");
   let values=[Parameter::Integer(i64::MAX),Parameter::Real(1.25),Parameter::Text("hello".into()),Parameter::Blob(vec![0,255]),Parameter::Null,Parameter::Boolean(true)];
   {let mut tx=pool.begin().await.unwrap();assert_eq!(tx.execute(&insert,&values).await.unwrap(),1);assert_eq!(tx.query("SELECT * FROM tsrust_s18_runtime",&[]).await.unwrap().len(),1);tx.rollback().await.unwrap();}
   {let mut tx=pool.begin().await.unwrap();tx.execute(&insert,&values).await.unwrap();}
   {let mut tx=pool.begin().await.unwrap();assert_eq!(tx.query("SELECT * FROM tsrust_s18_runtime",&[]).await.unwrap().len(),0);tx.execute(&insert,&values).await.unwrap();tx.commit().await.unwrap();}
   let rows=pool.query("SELECT * FROM tsrust_s18_runtime",&[]).await.unwrap();let row=&rows[0];
   assert_eq!(row.int("i").unwrap(),i64::MAX);assert_eq!(row.number("r").unwrap(),1.25);assert_eq!(row.text("t").unwrap(),"hello");assert_eq!(row.blob("b").unwrap(),[0,255]);assert!(row.is_null("n").unwrap());
   if driver=="postgres"{assert!(row.boolean("flag").unwrap());}else{assert_eq!(row.int("flag").unwrap(),1);}
   let failure=row.text("missing").unwrap_err();assert_eq!(failure.code,"column");assert!(!failure.message.is_empty());
   pool.execute("DROP TABLE tsrust_s18_runtime",&[]).await.unwrap();pool.close().await.unwrap();assert_eq!(row.text("t").unwrap(),"hello");
  });
 }
}

#[test]
fn migration_sources_reject_ambiguous_or_unsupported_files(){
 let directory=std::env::temp_dir().join(format!("tsrust-migration-source-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
 for (name,sql) in [("bad.sql","SELECT 1"),("0_bad.sql","SELECT 1"),("1_bad.up.sql","SELECT 1"),("1_bad.sql",""),("1_bad.sql","-- no-transaction\nSELECT 1")]{
  let path=directory.join(name);std::fs::write(&path,sql).unwrap();assert_eq!(tsrust_native_sql::load_migrations(&directory).unwrap_err().code,"migration-source");std::fs::remove_file(path).unwrap();
 }
 std::fs::write(directory.join("1_one.sql"),"SELECT 1").unwrap();std::fs::write(directory.join("01_duplicate.sql"),"SELECT 2").unwrap();assert!(tsrust_native_sql::load_migrations(&directory).is_err());std::fs::remove_file(directory.join("01_duplicate.sql")).unwrap();
 std::fs::write(directory.join("README.md"),"not SQL").unwrap();let migrator=tsrust_native_sql::load_migrations(&directory).unwrap();assert_eq!(migrator.migrations.len(),1);assert_eq!(migrator.migrations[0].version,1);std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore="opt-in SQL migration recovery; use tools/check-async-sql.mjs --docker"]
fn sql_migrations_detect_history_drift_and_recover_for_all_drivers(){
 use tsrust_native_sql::Pool;
 let postgres=std::env::var("TSRUST_TEST_POSTGRES_URL").unwrap();let mysql=std::env::var("TSRUST_TEST_MYSQL_URL").unwrap();
 let directory=std::env::temp_dir().join(format!("tsrust-migrations-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
 let sqlite=directory.join("database.sqlite");let sqlite_url=format!("sqlite:{}?mode=rwc",sqlite.display());
 for(driver,url)in [("sqlite",sqlite_url.as_str()),("postgres",postgres.as_str()),("mysql",mysql.as_str())]{
 tsrust_native_futures::block_on(async{
  let sources=directory.join(driver);std::fs::create_dir_all(&sources).unwrap();let source=sources.to_str().unwrap();
  if driver=="sqlite"{
   for ephemeral in ["sqlite::memory:","sqlite:%3Amemory%3A","sqlite:","sqlite:shared?mode=memory"]{
    let memory=Pool::connect(ephemeral,Some(1),Some(5000)).await.unwrap();assert_eq!(memory.migrate(source).await.unwrap_err().code,"migration-source");assert_eq!(memory.query("SELECT 1 AS value",&[]).await.unwrap()[0].int("value").unwrap(),1);memory.close().await.unwrap();
   }
  }
  let pool=Pool::connect(url,Some(1),Some(5000)).await.unwrap();
  pool.execute("DROP TABLE IF EXISTS _sqlx_migrations",&[]).await.unwrap();pool.execute("DROP TABLE IF EXISTS migration_notes",&[]).await.unwrap();pool.execute("DROP TABLE IF EXISTS migration_partial",&[]).await.unwrap();
  let first="CREATE TABLE migration_notes(id BIGINT PRIMARY KEY, body VARCHAR(50)); INSERT INTO migration_notes VALUES (1, 'ready');";
  std::fs::write(sources.join("1_notes.sql"),first).unwrap();pool.migrate(source).await.unwrap();pool.migrate(source).await.unwrap();
  assert_eq!(pool.query("SELECT id, body FROM migration_notes",&[]).await.unwrap()[0].text("body").unwrap(),"ready");
  std::fs::write(sources.join("1_notes.sql"),format!("{first}\n")).unwrap();assert_eq!(pool.migrate(source).await.unwrap_err().code,"migration-drift");
  std::fs::remove_file(sources.join("1_notes.sql")).unwrap();assert_eq!(pool.migrate(source).await.unwrap_err().code,"migration-drift");std::fs::write(sources.join("1_notes.sql"),first).unwrap();
  let broken="CREATE TABLE migration_partial(id BIGINT); INSERT INTO migration_missing_table VALUES (1);";std::fs::write(sources.join("2_partial.sql"),broken).unwrap();assert_eq!(pool.migrate(source).await.unwrap_err().code,"migration-failed");
  if driver=="mysql"{
   assert_eq!(pool.migrate(source).await.unwrap_err().code,"migration-dirty");
   assert!(pool.query("SELECT id FROM migration_partial",&[]).await.is_ok());
   // The disposable fixture's operator restores the exact pre-migration schema before clearing its dirty row.
   pool.execute("DROP TABLE migration_partial",&[]).await.unwrap();pool.execute("DELETE FROM _sqlx_migrations WHERE version = 2 AND success = false",&[]).await.unwrap();
  }else{assert!(pool.query("SELECT id FROM migration_partial",&[]).await.is_err());}
  std::fs::write(sources.join("2_partial.sql"),"CREATE TABLE migration_partial(id BIGINT);").unwrap();pool.migrate(source).await.unwrap();
  // History validation alone cannot detect out-of-band schema edits; a startup shape query must fail closed.
  pool.execute("ALTER TABLE migration_notes DROP COLUMN body",&[]).await.unwrap();pool.migrate(source).await.unwrap();assert!(pool.query("SELECT id, body FROM migration_notes",&[]).await.is_err());
  pool.execute("ALTER TABLE migration_notes ADD COLUMN body VARCHAR(50)",&[]).await.unwrap();pool.migrate(source).await.unwrap();assert!(pool.query("SELECT id, body FROM migration_notes",&[]).await.is_ok());
  pool.execute("INSERT INTO migration_partial VALUES (1)",&[]).await.unwrap();
  let blocker=Pool::connect(url,Some(1),Some(5000)).await.unwrap();let mut transaction=blocker.begin().await.unwrap();transaction.execute("UPDATE migration_partial SET id = 2",&[]).await.unwrap();
  std::fs::write(sources.join("3_wait.sql"),"UPDATE migration_partial SET id = id + 2;").unwrap();
  assert!(tokio::time::timeout(std::time::Duration::from_millis(100),pool.migrate(source)).await.is_err(),"{driver} migration must block on the held write lock");
  transaction.rollback().await.unwrap();blocker.close().await.unwrap();
  let observer=Pool::connect(url,Some(1),Some(5000)).await.unwrap();
  tokio::time::timeout(std::time::Duration::from_secs(5),observer.migrate(source)).await.expect("an independent pool must acquire the released migration lock").unwrap();observer.close().await.unwrap();
  pool.migrate(source).await.unwrap();assert_eq!(pool.query("SELECT id FROM migration_partial",&[]).await.unwrap()[0].int("id").unwrap(),3);
  pool.execute("DROP TABLE migration_notes",&[]).await.unwrap();pool.execute("DROP TABLE migration_partial",&[]).await.unwrap();pool.execute("DROP TABLE _sqlx_migrations",&[]).await.unwrap();pool.close().await.unwrap();
  eprintln!("{driver}: migration repeat, drift, failure, explicit recovery and schema check passed");
 });}
 std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[ignore="opt-in cursor driver acceptance: node tools/check-async-sql.mjs --docker"]
fn sql_cursors_stream_release_and_report_errors_for_all_drivers(){
 use tsrust_native_sql::{Pool,Parameter};
 let postgres=std::env::var("TSRUST_TEST_POSTGRES_URL").expect("PostgreSQL test URL required");
 let mysql=std::env::var("TSRUST_TEST_MYSQL_URL").expect("MySQL test URL required");
 for (driver,url) in [("sqlite","sqlite::memory:"),("postgres",postgres.as_str()),("mysql",mysql.as_str())]{
  tsrust_native_futures::block_on(async {
   let pool=Pool::connect(url,Some(1),Some(1000)).await.unwrap();
   let cast=if driver=="mysql"{"SIGNED"}else{"BIGINT"};
   let marker=if driver=="postgres"{"$1"}else{"?"};
   let sql=format!("SELECT CAST({marker} AS {cast}) AS value UNION ALL SELECT CAST(2 AS {cast}) AS value");
   {let mut cursor=pool.cursor(&sql,&[Parameter::Integer(1)]).await.unwrap();
    let first=cursor.next().await.unwrap().unwrap();assert_eq!(first.int("value").unwrap(),1);
    let second=cursor.next().await.unwrap().unwrap();assert_eq!(second.int("value").unwrap(),2);
    assert!(cursor.next().await.unwrap().is_none());assert!(cursor.next().await.unwrap().is_none());
    assert_eq!(pool.query("SELECT 1 AS healthy",&[]).await.unwrap().len(),1,"EOF releases the only connection");
    cursor.close();assert_eq!(first.int("value").unwrap(),1,"rows own their data");
   }
   {let mut cursor=pool.cursor(&sql,&[Parameter::Integer(1)]).await.unwrap();cursor.next().await.unwrap();cursor.close();}
   assert_eq!(pool.query("SELECT 1 AS healthy",&[]).await.unwrap().len(),1,"early close releases the connection");
   {let mut cursor=pool.cursor("SELECT 1 AS duplicate, 2 AS duplicate",&[]).await.unwrap();assert_eq!(cursor.next().await.err().unwrap().code,"column");assert!(cursor.next().await.unwrap().is_none());}
   assert!(pool.cursor(&sql,&[]).await.is_err(),"parameter count remains checked");
   let slow=match driver{"postgres"=>"SELECT 1 AS value FROM pg_sleep(0.05)","mysql"=>"SELECT SLEEP(0.05) AS value",_=>"WITH RECURSIVE counter(value) AS (SELECT 1 UNION ALL SELECT value+1 FROM counter WHERE value<2000000) SELECT COUNT(*) AS value FROM counter"};
   {let mut cursor=pool.cursor(slow,&[]).await.unwrap();assert!(tokio::time::timeout(std::time::Duration::from_millis(1),cursor.next()).await.is_err());cursor.close();}
   assert_eq!(pool.query("SELECT 1 AS healthy",&[]).await.unwrap().len(),1,"cancel then close releases the stream");

   pool.close().await.unwrap();
  });
 }
}
