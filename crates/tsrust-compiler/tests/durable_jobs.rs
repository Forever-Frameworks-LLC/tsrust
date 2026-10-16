mod support;

#[test]
fn durable_job_application_checks_and_formats(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 for name in ["jobs.tsr","main.tsr"]{let source=std::fs::read_to_string(root.join("examples/durable-jobs").join(name)).unwrap();support::format_round_trip(&source);}
 support::success(&support::run(support::cli().arg("check").arg(root.join("examples/durable-jobs/main.tsr")),30));
}

#[test]
#[ignore="opt-in file-backed SQLite job acceptance"]
fn durable_jobs_recover_and_fence_stale_workers(){
 let directory=support::TempDirectory::new();
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
 let binary=directory.0.join("jobs");
 support::success(&support::run(support::cli().arg("build").arg(root.join("examples/durable-jobs/main.tsr")).arg("-o").arg(&binary),300));
 let url=format!("sqlite:{}?mode=rwc",directory.0.join("jobs.sqlite").display());
 let output=support::run(std::process::Command::new(binary).env("DATABASE_URL",&url),30);
 support::success(&output);assert_eq!(output.stdout,b"Durable jobs accepted\n");
 tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
  sqlx::any::install_default_drivers();
  let db=sqlx::AnyPool::connect(&url).await.unwrap();
  let jobs=sqlx::query_as::<_,(String,String,i64,i64)>("SELECT id,state,attempts,generation FROM jobs ORDER BY id").fetch_all(&db).await.unwrap();
  assert_eq!(jobs,vec![("abandoned".into(),"failed".into(),3,3),("poison".into(),"failed".into(),3,3),("receipt-1".into(),"done".into(),3,3)]);
  let effects=sqlx::query_as::<_,(String,String)>("SELECT job_id,body FROM effects").fetch_all(&db).await.unwrap();
  assert_eq!(effects,vec![("receipt-1".into(),"receipt body".into())]);
  db.close().await;
 });
}
