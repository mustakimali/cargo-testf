# cargo-testf
A wrapper for `cargo test` that remembers and runs failed tests

## Install
```
cargo install cargo-testf
```

## Use `cargo testf` instead of `cargo test`
It will run all tests, unless one or many tests failed previously. In which case only those tests will run.