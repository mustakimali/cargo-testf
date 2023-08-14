use std::{io::Error, process::Command};

fn main() {
    // Command::new("cargo")
    //     .arg("test")
    let out_dir = std::env::var("CARGO_TARGET_DIR")
        .or(find_out_dir())
        .unwrap_or_default();
    println!("{}", out_dir);
    //cargo test -- test::works,test::pass --exact
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

fn cmd_for_failed_tests(names: Vec<&str>) -> String {
    format!("cargo test -- {} --exact", names.join(" "))
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
