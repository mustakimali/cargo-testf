use std::{
    io::{BufRead, BufReader, Error},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() {
    let current_dir = std::env::current_dir().expect("get current dir");
    let out_dir = std::env::var("CARGO_TARGET_DIR")
        .map(|p| Path::new(&p).to_path_buf())
        .or(find_out_dir())
        .unwrap_or_default();
    let toml_file = find_cargo_toml()
        .expect("unable to find a Cargo.toml, command must run inside a rust project");
    let result_path = {
        let mut root = out_dir.clone();
        root.push(format!("testf-{}.txt", hash(&toml_file.to_string_lossy())));
        root
    };
    println!(
        "Current Dir: {}\nOut Dir: {}\nCargo file: {}\nResult file: {}",
        current_dir.display(),
        out_dir.display(),
        toml_file.display(),
        result_path.display()
    );

    let mut failed_tests = State::default();
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    if result_path.exists() {
        let content =
            std::fs::read_to_string(result_path.clone()).expect("read name of failed tests");
        let failed = content.split(" ").collect::<Vec<_>>();
        if !args.contains(&"--".to_string()) {
            args.push("--".into());
        }
        for f in failed {
            args.push(f.to_string());
        }
        args.push("--exact".to_string());
    }

    let mut cmd = Command::new("cargo")
        .arg("test")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("start cargo run");
    {
        let stdout = cmd.stdout.as_mut().expect("read stdout");
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            if let Ok(line) = line {
                failed_tests_s(&line, &mut failed_tests);
                println!("{}", line);
            }
        }
    }

    let result = cmd.wait().expect("run cargo test");

    if result.success() {
        if result_path.exists() {
            std::fs::remove_file(result_path).expect("remove result file");
        }
        return;
    }

    let failed = failed_tests.names.join(" ");
    std::fs::write(result_path, failed).expect("write failed tests");
}

fn hash(input: &str) -> u64 {
    const PRIME: u64 = 31;
    let mut hash: u64 = 0;

    for (_, c) in input.chars().enumerate() {
        hash = hash.wrapping_mul(PRIME).wrapping_add(c as u64);
    }

    hash
}

fn find_out_dir() -> std::io::Result<PathBuf> {
    let mut root = std::env::current_dir()?;

    loop {
        let p = root.join("target");
        if p.exists() {
            return Ok(p);
        }
        if !root.pop() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Can not find target folder",
            ));
        }
    }
}

fn find_cargo_toml() -> std::io::Result<PathBuf> {
    let mut root = std::env::current_dir()?;

    loop {
        let p = root.join("Cargo.toml");
        if p.exists() {
            return Ok(p);
        }
        if !root.pop() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Can not find target folder",
            ));
        }
    }
}

#[derive(Default, Debug)]
struct State {
    names: Vec<String>,
    f_count: u32,
}

fn failed_tests_s(l: &str, state: &mut State) {
    if l == "failures:" {
        state.f_count += 1;
    }

    if state.f_count == 2 && l.starts_with("    ") {
        let name = l.trim().clone();
        if name.len() > 0 {
            state.names.push(name.to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn works() {
        let output = r#"
running 3 tests
test test::fail ... FAILED
test test::inner_test::fail ... FAILED
test test::pass ... ok

failures:

---- test::fail stdout ----
thread 'test::fail' panicked at 'fail', src/main.rs:18:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test::inner_test::fail stdout ----
thread 'test::inner_test::fail' panicked at 'fail', src/main.rs:10:13


failures:
    test::fail
    test::inner_test::fail

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
        "#;
        let mut f = State::default();
        for l in output.lines() {
            failed_tests_s(l, &mut f);
        }

        assert_eq!(f.names, &["test::fail", "test::inner_test::fail"]);
    }
}
