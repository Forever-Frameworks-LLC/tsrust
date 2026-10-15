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
