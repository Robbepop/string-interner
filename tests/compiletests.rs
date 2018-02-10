extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
	let mut config = compiletest::Config::default();

	config.mode = mode.parse().expect("Invalid mode");
	config.src_base = PathBuf::from(format!("tests/{}", mode));
	config.target_rustcflags = Some("-L target/debug/deps".to_string());
	config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

	compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
	run_mode("compile-fail");
}
