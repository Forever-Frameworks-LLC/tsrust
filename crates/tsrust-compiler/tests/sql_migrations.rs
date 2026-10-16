mod support;
#[test]
fn migration_example_checks_and_pool_borrow_is_preserved(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/sql-migrations/main.tsr"));support::success(&support::run(support::cli().arg("check").arg(directory.source(&source)),30));
 for source in ["import {connect} from 'std:sql';async function main():Task<void>{const pool=await connect('sqlite:test.db');const pending=pool.migrate('migrations');await pool.close();await pending;}","import {connect} from 'std:sql';async function main():Task<void>{const pool=await connect('sqlite:test.db');await pool.migrate(12);}"]{let output=support::run(support::cli().arg("check").arg(directory.source(&support::format_round_trip(source))),30);assert!(!output.status.success());}
}
#[test]
#[ignore="opt-in native SQL migrations; use tools/check-async-sql.mjs --docker"]
fn native_migration_example_runs(){
 let directory=support::TempDirectory::new();let source=support::format_round_trip(include_str!("../../../examples/sql-migrations/main.tsr"));let input=directory.source(&source);let package=directory.0.join("package");
 support::success(&support::run(support::cli().arg("emit").arg(&input).arg("--package").arg("-o").arg(&package),30));
 std::fs::remove_file(input).unwrap();let relocated=directory.0.join("relocated");std::fs::rename(package,&relocated).unwrap();
 let target=std::path::PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("shared Cargo target required"));let home=directory.0.join("empty-home");std::fs::create_dir(&home).unwrap();
 let mut build=std::process::Command::new("rustup");build.args(["run",tsrust_compiler::RUST_BACKEND_VERSION,"cargo","build","--offline","--locked","--jobs","1"]).current_dir(&relocated).env("CARGO_HOME",home).env("CARGO_INCREMENTAL","0").env("CARGO_TARGET_DIR",&target);
 support::success(&support::run(&mut build,300));
 let mut execute=std::process::Command::new(target.join("debug/program"));execute.current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).env("DATABASE_URL",format!("sqlite:{}?mode=rwc",directory.0.join("database.sqlite").display()));
 let output=support::run(&mut execute,30);support::success(&output);assert_eq!(output.stdout,b"SQL migrations accepted\n");
}
