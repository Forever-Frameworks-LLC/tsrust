use tsrust_native_sqlite::{Connection,Parameter as V};
fn execute(db:&Connection,sql:&str){db.prepare(sql).unwrap().execute(&[]).unwrap();}
fn count(db:&Connection)->i64{db.prepare("SELECT count(*) AS n FROM items").unwrap().query(&[]).unwrap()[0].int("n").unwrap()}
#[test]fn commit_drop_and_failed_commit_rollback(){
 let mut db=Connection::open(":memory:",None).unwrap();execute(&db,"CREATE TABLE items(id INTEGER PRIMARY KEY)");
 {let tx=db.begin().unwrap();tx.prepare("INSERT INTO items VALUES(1)").unwrap().execute(&[]).unwrap();tx.commit().unwrap();}assert_eq!(count(&db),1);
 {let tx=db.begin().unwrap();tx.prepare("INSERT INTO items VALUES(2)").unwrap().execute(&[]).unwrap();}assert_eq!(count(&db),1);
 execute(&db,"CREATE TABLE child(id INTEGER REFERENCES items(id) DEFERRABLE INITIALLY DEFERRED)");
 {let tx=db.begin().unwrap();tx.prepare("INSERT INTO child VALUES(999)").unwrap().execute(&[]).unwrap();assert_eq!(tx.commit().unwrap_err().code,"constraint");}
 assert_eq!(db.prepare("SELECT count(*) AS n FROM child").unwrap().query(&[]).unwrap()[0].int("n").unwrap(),0);db.close().unwrap();
}
#[test]fn values_are_exact_owned_and_type_checked(){
 let db=Connection::open(":memory:",None).unwrap();let rows={let mut stmt=db.prepare("SELECT ?1 AS i,?2 AS r,?3 AS t,?4 AS b,?5 AS n").unwrap();stmt.query(&[V::Integer(i64::MAX),V::Real(1.25),V::Text("a\0é".into()),V::Blob(vec![0,255]),V::Null]).unwrap()};db.close().unwrap();
 let r=&rows[0];assert_eq!(r.int("i").unwrap(),i64::MAX);assert_eq!(r.number("r").unwrap(),1.25);assert_eq!(r.text("t").unwrap(),"a\0é");assert_eq!(r.blob("b").unwrap(),vec![0,255]);assert!(r.is_null("n").unwrap());assert_eq!(r.int("n").unwrap_err().code,"type");assert_eq!(r.number("i").unwrap_err().code,"type");assert_eq!(r.text("absent").unwrap_err().code,"column");
}
#[test]fn sql_binding_and_transaction_control_are_guarded(){
 let mut db=Connection::open(":memory:",None).unwrap();execute(&db,"CREATE TABLE items(id INTEGER PRIMARY KEY)");
 for sql in ["BEGIN","COMMIT","END","ROLLBACK","SAVEPOINT x","RELEASE x"]{assert!(db.prepare(sql).is_err(),"{sql}");}
 assert!(!db.prepare("SELECT 1;SELECT 2").err().unwrap().message.is_empty());
 db.prepare("SELECT 1").unwrap().finalize().unwrap();
 let mut s=db.prepare("SELECT ?1 AS x").unwrap();assert_eq!(s.query(&[]).err().unwrap().code,"parameters");assert_eq!(s.query(&[V::Real(f64::NAN)]).err().unwrap().code,"range");assert_eq!(s.execute(&[V::Integer(1)]).unwrap_err().code,"query");drop(s);
 assert!(db.prepare("SELECT 1 AS x,2 AS x").unwrap().query(&[]).is_err());assert!(db.prepare("SELECT CAST(x'ff' AS TEXT)").unwrap().query(&[]).is_err());
 let tx=db.begin().unwrap();let mut insert=tx.prepare("INSERT OR ROLLBACK INTO items VALUES(1)").unwrap();insert.execute(&[]).unwrap();assert!(insert.execute(&[]).is_err());assert_eq!(insert.execute(&[]).unwrap_err().code,"transaction-ended");drop(insert);assert_eq!(tx.commit().unwrap_err().code,"transaction-ended");assert_eq!(count(&db),0);
 assert!(db.prepare("INSERT INTO items VALUES(3) RETURNING id").unwrap().execute(&[]).is_err());assert!(db.prepare("INSERT INTO items VALUES(3) RETURNING id").unwrap().query(&[]).is_err());assert_eq!(count(&db),0);
}
#[test]fn explicit_rollback_and_unwind_cleanup(){
 let mut db=Connection::open(":memory:",None).unwrap();execute(&db,"CREATE TABLE items(id INTEGER PRIMARY KEY)");
 {let tx=db.begin().unwrap();tx.prepare("INSERT INTO items VALUES(1)").unwrap().execute(&[]).unwrap();tx.rollback().unwrap();}assert_eq!(count(&db),0);
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{let tx=db.begin().unwrap();tx.prepare("INSERT INTO items VALUES(2)").unwrap().execute(&[]).unwrap();panic!("test unwind");}));assert!(result.is_err());assert_eq!(count(&db),0);db.begin().unwrap().commit().unwrap();
}
#[test]fn busy_errors_release_locks_and_persist_after_close(){
 let path=std::env::temp_dir().join(format!("tsrust-sqlite-{}-busy.db",std::process::id()));
 let mut first=Connection::open(path.to_str().unwrap(),Some(0)).unwrap();execute(&first,"CREATE TABLE items(id INTEGER PRIMARY KEY)");let mut second=Connection::open(path.to_str().unwrap(),Some(0)).unwrap();
 {let tx=first.begin().unwrap();assert_eq!(second.begin().err().unwrap().code,"busy");tx.prepare("INSERT INTO items VALUES(1)").unwrap().execute(&[]).unwrap();tx.commit().unwrap();}second.begin().unwrap().rollback().unwrap();first.close().unwrap();assert_eq!(count(&second),1);second.close().unwrap();std::fs::remove_file(path).unwrap();
}
