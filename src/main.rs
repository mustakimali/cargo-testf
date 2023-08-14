use std::{
    io::{BufRead, BufReader, Error, Read},
    process::{Command, Stdio},
};

fn main() {
    let current_dir = std::env::current_dir()
        .expect("get current dir")
        .to_string_lossy()
        .to_string();
    let out_dir = std::env::var("CARGO_TARGET_DIR")
        .or(find_out_dir())
        .unwrap_or_default();
    let toml_file = find_cargo_toml()
        .expect("unable to find a Cargo.toml, command must run inside a rust project");
    println!(
        "Current Dir: {}\nOut Dir: {}\nCargo file: {}",
        current_dir, out_dir, toml_file
    );
    //cargo test -- test::works,test::pass --exact
    let mut failed_tests = State::default();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
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
        return;
    }

    dbg!(&failed_tests);
}

fn find_out_dir() -> std::io::Result<String> {
    let mut root = std::env::current_dir()?;

    loop {
        let p = root.join("target");
        if p.exists() {
            return Ok(p.to_string_lossy().to_string());
        }
        if !root.pop() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Can not find target folder",
            ));
        }
    }
}

fn find_cargo_toml() -> std::io::Result<String> {
    let mut root = std::env::current_dir()?;

    loop {
        let p = root.join("Cargo.toml");
        if p.exists() {
            return Ok(p.to_string_lossy().to_string());
        }
        if !root.pop() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Can not find target folder",
            ));
        }
    }
}

fn cmd_for_failed_tests(names: Vec<&str>) -> String {
    format!("cargo test -- {} --exact", names.join(" "))
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

fn failed_tests(output: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut f_count = 0;
    for l in output.lines() {
        if l == "failures:" {
            f_count += 1;
        }

        if f_count == 2 && l.starts_with("    ") {
            let name = l.trim().clone();
            if name.len() > 0 {
                names.push(name);
            }
        }
    }

    names
}

#[cfg(test)]
mod test {
    use crate::failed_tests;

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
        let failed = failed_tests(output);

        assert_eq!(failed, &["test::fail", "test::inner_test::fail"]);
    }

    mod inner_test {
        #[test]
        fn fail() {
            panic!("fail");
        }
    }

    #[test]
    fn pass() {}
    #[test]
    fn fail() {
        panic!("fail");
    }
}
