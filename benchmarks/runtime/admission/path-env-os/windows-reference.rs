//! Injected lexical controls; this does not exercise Windows OS or signals.
pub mod existing_resolution {
    include!("../../../../crates/tsrust-compiler/src/stdlib/path/relative.rs");
    include!("windows-resolve.rs");
}

fn main() {
    let mut count = 0;
    for line in include_str!("windows-resolve.tsv").lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
        let row = line.split('\t').collect::<Vec<_>>();
        let mut cwd_reads = 0;
        let mut drive_reads = 0;
        let result = existing_resolution::windows_many(
            &row[5..],
            || {
                cwd_reads += 1;
                if let Some(error) = row[0].strip_prefix("@error:") { Err(error.into()) } else { Ok(row[0].into()) }
            },
            |drive| {
                drive_reads += 1;
                if let Some(error) = row[1].strip_prefix("@error:") {
                    Err(error.into())
                } else {
                    Ok(row[1].get(..2).filter(|prefix| prefix.eq_ignore_ascii_case(drive)).map(|_| row[1].to_owned()))
                }
            },
        );
        let actual = result.unwrap_or_else(|error| format!("@error:{error}"));
        assert_eq!(actual, row[2], "fixed lexical row {count}");
        assert_eq!(cwd_reads, row[3].parse::<usize>().unwrap(), "cwd observation row {count}");
        assert_eq!(drive_reads, row[4].parse::<usize>().unwrap(), "drive observation row {count}");
        count += 1;
    }
    println!("B03 Windows lexical reference checked {count} cases");
}
