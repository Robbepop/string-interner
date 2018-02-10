/*extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
	let mut config = compiletest::Config::default();

	config.mode = mode.parse().expect("Invalid mode");
	config.src_base = PathBuf::from(format!("tests/{}", mode));
	if cfg!(target_os = "windows") {
		// circumvent laumann/compiletest-rs#81 where it matters most
		config.target_rustcflags = Some("-L target/debug/deps".to_string());
	} else {
		config.link_deps();
	}
	config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464

	compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
	// Currently, this test fails on Travis because string-interner is duplicated in the link args
	// https://travis-ci.org/Robbepop/string-interner/jobs/339735884
	// FIXME(CAD97): compile-fail tests are therefore disabled until this can be figured out
	run_mode("compile-fail");
}*/
