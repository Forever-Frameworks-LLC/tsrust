mod support;

#[test]
fn sql_ownership_and_async_errors_are_checked_without_a_database() {
    for body in [
        "const tx=await db.begin();await db.close();await tx.rollback();",
        "const tx=await db.begin();const pending=tx.query('SELECT 1');await tx.commit();await pending;",
        "const pending=db.query('SELECT 1');await db.close();await pending;",
        "await db.close();await db.query('SELECT 1');",
        "const tx=await db.begin();await tx.commit();await tx.rollback();",
        "const other=clone(db);",
        "const tx=await db.begin();const finish=tx.commit();await db.close();await finish;",
        "const tx=await db.begin();const finish=tx.rollback();await db.close();await finish;",
        "const connectLater=connect;",
        "await db.query('SELECT 1',[{id:1}]);",
    ] {
        let source=format!("import {{connect}} from 'std:sql';async function main():Task<void>{{const db=await connect('sqlite::memory:',1);{body}}}");
        let dir=support::TempDirectory::new();
        let output=support::run(support::cli().arg("check").arg(dir.source(&source)),30);
        assert!(!output.status.success(),"accepted {source}");
        let message=String::from_utf8_lossy(&output.stderr);
        assert!(!message.contains("error[syntax]")&&!message.contains("error[import]"),"{message}");
    }
}

#[test]
fn sql_examples_check_and_format_without_a_database(){
    for source in [include_str!("../../../examples/async-sql/real-boundary.tsr"),include_str!("../../../examples/async-sql/people.tsr"),include_str!("../../../examples/async-sql/rollback.tsr")]{
        let source=support::format_round_trip(source);let dir=support::TempDirectory::new();
        support::success(&support::run(support::cli().arg("check").arg(dir.source(&source)),30));
    }
}

#[test]
#[ignore = "opt-in database acceptance: node tools/check-async-sql.mjs --docker"]
fn all_sqlx_drivers_execute_with_rollback_and_cleanup(){
    let postgres=std::env::var("TSRUST_TEST_POSTGRES_URL").expect("set TSRUST_TEST_POSTGRES_URL or use tools/check-async-sql.mjs --docker");
    let mysql=std::env::var("TSRUST_TEST_MYSQL_URL").expect("set TSRUST_TEST_MYSQL_URL or use tools/check-async-sql.mjs --docker");
    let sqlite_dir=support::TempDirectory::new();
    let sqlite_url=format!("sqlite://{}?mode=rwc",sqlite_dir.0.join("acceptance.sqlite").display());
    let resume=std::env::var("TSRUST_TEST_SQL_RESUME").unwrap_or_default();
    assert!(matches!(resume.as_str(),""|"postgres"|"mysql"),"unknown SQL resume driver");
    for (driver,url) in [("sqlite",sqlite_url.as_str()),("postgres",postgres.as_str()),("mysql",mysql.as_str())]{
        if (resume=="postgres" && driver=="sqlite") || (resume=="mysql" && driver!="mysql"){eprintln!("Reusing previously passed {driver} evidence");continue;}

        eprintln!("Checking {driver}: bindings, rollback, cancellation, timeout and cleanup");
        let bind=if driver=="postgres"{"$1, $2"}else{"?, ?"};
        let slow_sql=match driver {"postgres"=>"SELECT 1 AS marker FROM pg_sleep(0.25)","mysql"=>"SELECT SLEEP(0.25) AS marker",_=>"WITH RECURSIVE counter(value) AS (SELECT 1 UNION ALL SELECT value+1 FROM counter WHERE value<2000000) SELECT COUNT(*) AS marker FROM counter"};
        let blob_type=if driver=="postgres"{"BYTEA"}else{"BLOB"};
        let flag_type=if driver=="postgres"{"BOOLEAN"}else{"BIGINT"};
        let values=if driver=="postgres"{"$1,$2,$3,$4,$5,$6"}else{"?,?,?,?,?,?"};
        let flag_check=if driver=="postgres"{"strictEqual(unwrap(typed[0].boolean('flag')),true);"}else{"strictEqual(unwrap(typed[0].int('flag')),1);"};
        let real_check=if driver=="postgres" {r#"
 let precision=false;try{await db.query('SELECT CAST($1 AS REAL) AS value',[0.1]);}catch(error){strictEqual(error.code,'precision');precision=true;}strictEqual(precision,true);
 let underflow=false;try{await db.query('SELECT CAST($1 AS REAL) AS value',[1e-300]);}catch(error){strictEqual(error.code,'precision');underflow=true;}strictEqual(underflow,true);
 const exact=await db.query('SELECT CAST($1 AS REAL) AS value',[1.25]);strictEqual(unwrap(exact[0].number('value')),1.25);
 {const tx=await db.begin();let rejected=false;try{await tx.query('SELECT CAST($1 AS REAL) AS value',[0.1]);}catch(error){strictEqual(error.code,'precision');rejected=true;}strictEqual(rejected,true);let inactive=false;try{await tx.commit();}catch(error){strictEqual(error.code,'transaction');inactive=true;}strictEqual(inactive,true);}
"#}else{""};
        let source=format!(r#"
import {{ connect }} from 'std:sql';
import {{ strictEqual }} from 'std:testing';
import {{ fromArray, length, at }} from 'std:bytes';
import {{ Channel, Sender, delay }} from 'std:futures';
async function cancelledWrite(url:string,ready:Sender<int>):Task<void>{{
 const worker=await connect(url,1,2000);
 const tx=await worker.begin();
 await tx.execute('INSERT INTO tsrust_s18_people VALUES ({bind})',[5,'Cancelled']);
 await ready.send(1);
 await tx.query('{slow_sql}');
 await delay(60000);
 await tx.commit();
}}
async function main():Task<void>{{
 const db=await connect({url},1,2000);
 {real_check}
 await db.execute('DROP TABLE IF EXISTS tsrust_s18_values');
 await db.execute('CREATE TABLE tsrust_s18_values(i BIGINT,r DOUBLE PRECISION,t VARCHAR(100),b {blob_type},n VARCHAR(100),flag {flag_type})');
 const blob=unwrap(fromArray([0,255]));
 await db.execute('INSERT INTO tsrust_s18_values VALUES ({values})',[9223372036854775807,1.25,'bound',blob,null,true]);
 const typed=await db.query('SELECT * FROM tsrust_s18_values');
 strictEqual(unwrap(typed[0].int('i')),9223372036854775807);strictEqual(unwrap(typed[0].number('r')),1.25);
 strictEqual(unwrap(typed[0].text('t')),'bound');strictEqual(unwrap(typed[0].isNull('n')),true);{flag_check}
 const copied=unwrap(typed[0].blob('b'));strictEqual(length(copied),2);strictEqual(unwrap(at(copied,1)),255);
 let wrong=false;try{{unwrap(typed[0].int('t'));}}catch(error){{wrong=true;}}strictEqual(wrong,true);
 let missing=false;try{{unwrap(typed[0].text('missing'));}}catch(error){{missing=true;}}strictEqual(missing,true);
 let arity=false;try{{await db.execute('INSERT INTO tsrust_s18_values VALUES ({values})',[]);}}catch(error){{arity=true;}}strictEqual(arity,true);
 let duplicate=false;try{{await db.query('SELECT i AS same,t AS same FROM tsrust_s18_values');}}catch(error){{duplicate=true;}}strictEqual(duplicate,true);
 await db.execute('DROP TABLE tsrust_s18_values');
 await db.execute('DROP TABLE IF EXISTS tsrust_s18_people');
 await db.execute('CREATE TABLE tsrust_s18_people(id BIGINT PRIMARY KEY,name VARCHAR(100))');
 {{const tx=await db.begin();strictEqual(await tx.execute('INSERT INTO tsrust_s18_people VALUES ({bind})',[1,'Ada']),1);await tx.commit();}}
 {{const tx=await db.begin();await tx.execute('INSERT INTO tsrust_s18_people VALUES ({bind})',[2,'Rolled back']);await tx.rollback();}}
 {{const tx=await db.begin();await tx.execute('INSERT INTO tsrust_s18_people VALUES ({bind})',[3,'Dropped']);}}
 try{{const tx=await db.begin();await tx.execute('INSERT INTO tsrust_s18_people VALUES ({bind})',[4,'Thrown']);throw 'stop';}}catch(error){{}}
 const ready=new Channel<int>(1);
 let cancelled=false;
 const job=cancelledWrite({url},ready.sender()).start();
 try{{await ready.receive();await delay(20);}}finally{{job.cancel();try{{await job;}}catch(error){{cancelled=true;}}}}
 strictEqual(cancelled,true);
 {{const tx=await db.begin();await tx.execute('INSERT INTO tsrust_s18_people VALUES ({bind})',[6,'Timed out']);
 let expired=false;try{{await tx.query('{slow_sql}').timeout(10);}}catch(error){{expired=true;}}strictEqual(expired,true);
 let refused=false;try{{await tx.commit();}}catch(error){{refused=true;}}strictEqual(refused,true);}}
 {{const tx=await db.begin();let timedOut=false;try{{await db.query('SELECT 1');}}catch(error){{timedOut=true;}}strictEqual(timedOut,true);await tx.rollback();}}
 const rows=await db.query('SELECT id,name FROM tsrust_s18_people ORDER BY id');
 strictEqual(rows.length,1);strictEqual(unwrap(rows[0].int('id')),1);strictEqual(unwrap(rows[0].text('name')),'Ada');
 let blocked=false;try{{await db.execute('COMMIT');}}catch(error){{blocked=true;}}strictEqual(blocked,true);
 await db.execute('DROP TABLE tsrust_s18_people');
 await db.close();
 strictEqual(unwrap(rows[0].text('name')),'Ada');
}}
"#,url=serde_json::to_string(url).unwrap());
        let dir=support::TempDirectory::new();let source=support::format_round_trip(&source);
        let output=support::run(support::cli().arg("run").arg(dir.source(&source)),600);
        assert!(output.status.success(),"{driver}: {}",String::from_utf8_lossy(&output.stderr));
        eprintln!("Passed {driver}");
    }
    for (source,expected) in [(include_str!("../../../examples/async-sql/real-boundary.tsr"),"1.25\nPrecision\n"),(include_str!("../../../examples/async-sql/people.tsr"),"1: Ada\n2: Grace\n"),(include_str!("../../../examples/async-sql/rollback.tsr"),"Transfer rolled back\n0\n")]{let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("run").arg(dir.source(&support::format_round_trip(source))),600);support::success(&output);assert_eq!(output.stdout,expected.as_bytes());}
}

#[test]
fn mixed_sql_parameters_preserve_full_width_integers_without_a_database(){
 let source="import {connect} from 'std:sql';async function main():Task<void>{const db=await connect('sqlite::memory:',1);await db.execute('INSERT INTO values_table VALUES (?, ?)',[9223372036854775807,1.25]);await db.query('SELECT 1',[]);await db.close();}";
 let dir=support::TempDirectory::new();let output=support::run(support::cli().arg("emit").arg(dir.source(&support::format_round_trip(source))),30);support::success(&output);
 let rust=String::from_utf8(output.stdout).unwrap();assert!(rust.contains("9223372036854775807i64"));assert!(rust.contains("1.25f64"));
}

fn domain_example(driver:&str,url:&str)->String{
 let mut source=include_str!("../../../examples/sql-domain-values/sqlite.tsr").replace("'sqlite::memory:'",&serde_json::to_string(url).unwrap());
 if driver=="postgres"{
  source=source.replace("amount TEXT,day TEXT,clock TEXT,stamp TEXT,zone TEXT,duration BIGINT,id BLOB,payload TEXT,optional TEXT","amount NUMERIC(38,18),day DATE,clock TIME(6),stamp TIMESTAMP(6),zone TEXT,duration BIGINT,id UUID,payload JSONB,optional TEXT")
   .replace("VALUES (?,?,?,?,?,?,?,?,?)","VALUES (CAST($1 AS TEXT)::NUMERIC(38,18),CAST($2 AS TEXT)::DATE,CAST($3 AS TEXT)::TIME(6),CAST($4 AS TEXT)::TIMESTAMP(6),$5,$6,CAST($7 AS TEXT)::UUID,CAST($8 AS TEXT)::JSONB,$9)")
   .replace("const id = toBytes(","const id = formatUuid(")
   .replace("SELECT amount,day,clock,stamp,zone,duration,id,payload,optional FROM domains","SELECT amount::TEXT AS amount,day::TEXT AS day,clock::TEXT AS clock,stamp::TEXT AS stamp,zone,duration,id::TEXT AS id,payload::TEXT AS payload,optional FROM domains")
   .replace("fromBytes(unwrap(rows[0].blob('id')))","parseUuid(unwrap(rows[0].text('id')))");
 }else if driver=="mysql"{
  source=source.replace("amount TEXT,day TEXT,clock TEXT,stamp TEXT,zone TEXT,duration BIGINT,id BLOB,payload TEXT,optional TEXT","amount DECIMAL(38,18),day DATE,clock TIME(6),stamp DATETIME(6),zone VARCHAR(128),duration BIGINT,id BINARY(16),payload JSON,optional TEXT")
   .replace("SELECT amount,day,clock,stamp,zone,duration,id,payload,optional FROM domains","SELECT CAST(amount AS CHAR) AS amount,CAST(day AS CHAR) AS day,CAST(clock AS CHAR) AS clock,CAST(stamp AS CHAR) AS stamp,zone,duration,id,CAST(payload AS CHAR) AS payload,optional FROM domains");
 }
 source
}
#[test]
fn sql_domain_examples_check_without_a_database(){
 for driver in ["sqlite","postgres","mysql"]{
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("encoding.tsr"),include_str!("../../../examples/sql-domain-values/encoding.tsr")).unwrap();
  let source=support::format_round_trip(&domain_example(driver,"sqlite::memory:"));
  support::success(&support::run(support::cli().arg("check").arg(dir.source(&source)),30));
 }
}
#[test]
#[ignore = "opt-in domain SQL acceptance: node tools/check-async-sql.mjs --docker"]
fn all_sqlx_drivers_preserve_explicit_domain_encodings(){
 let postgres=std::env::var("TSRUST_TEST_POSTGRES_URL").expect("PostgreSQL URL");let mysql=std::env::var("TSRUST_TEST_MYSQL_URL").expect("MySQL URL");
 let resume=std::env::var("TSRUST_TEST_SQL_RESUME").unwrap_or_default();
 for (driver,url) in [("sqlite","sqlite::memory:"),("postgres",postgres.as_str()),("mysql",mysql.as_str())]{
  if (resume=="postgres"&&driver=="sqlite")||(resume=="mysql"&&driver!="mysql"){continue;}
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("encoding.tsr"),include_str!("../../../examples/sql-domain-values/encoding.tsr")).unwrap();
  let source=support::format_round_trip(&domain_example(driver,url));
  let result=support::run(support::cli().arg("run").arg(dir.source(&source)),600);support::success(&result);eprintln!("Passed {driver} exact domain encodings");
 }
}
