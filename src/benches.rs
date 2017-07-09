use super::*;

use test::{Bencher, black_box};

use fnv::FnvHasher;
use std::hash::BuildHasherDefault;

fn read_file_to_string(path: &str) -> String {
	use std::io::prelude::*;
	use std::fs::File;

	let mut f = File::open(path).expect("bench file not found");
	let mut s = String::new();

	f.read_to_string(&mut s).expect("encountered problems writing bench file to string");
	s
}

fn read_default_test() -> String {
	read_file_to_string("bench/input.txt")
}

fn empty_setup<'a>(input: &'a str) -> (Vec<&'a str>, DefaultStringInterner) {
	let lines = input.split_whitespace().collect::<Vec<&'a str>>();
	let interner = DefaultStringInterner::with_capacity(lines.len());
	(lines, interner)
}

fn empty_setup_with_hasher<'a, H>(input: &'a str, hash_builder: H) -> (Vec<&'a str>, StringInterner<usize, H>)
	where H: BuildHasher
{
	let lines = input.split_whitespace().collect::<Vec<&'a str>>();
	let interner = StringInterner::with_capacity_and_hasher(lines.len(), hash_builder);
	(lines, interner)
}

fn filled_setup<'a>(input: &'a str) -> (Vec<usize>, DefaultStringInterner) {
	let (lines, mut interner) = empty_setup(&input);
	let symbols = lines.iter().map(|&line| interner.get_or_intern(line)).collect::<Vec<_>>();
	(symbols, interner)
}

#[bench]
fn get_or_intern_empty(bencher: &mut Bencher) {
	let input = read_default_test();
	let (lines, interner) = empty_setup(&input);
	bencher.iter(|| {
		let mut interner = interner.clone();
		for &line in lines.iter() {
			black_box(interner.get_or_intern(line));
		}
	});
}

#[bench]
fn get_or_intern_filled(bencher: &mut Bencher) {
	let input = read_default_test();
	let (lines, mut interner) = empty_setup(&input);
	for &line in lines.iter() {
		interner.get_or_intern(line);
	}
	bencher.iter(|| {
		for &line in lines.iter() {
			black_box(interner.get_or_intern(line));
		}
	});
}

#[bench]
fn resolve(bencher: &mut Bencher) {
	let input = read_default_test();
	let (symbols, interner) = filled_setup(&input);
	bencher.iter(|| {
		for &sym in symbols.iter() {
			black_box(interner.resolve(sym));
		}
	});
}

#[bench]
fn resolve_unchecked(bencher: &mut Bencher) {
	let input = read_default_test();
	let (symbols, interner) = filled_setup(&input);
	bencher.iter(|| {
		for &sym in symbols.iter() {
			unsafe{ black_box(interner.resolve_unchecked(sym)) };
		}
	});
}

#[bench]
fn iter(bencher: &mut Bencher) {
	let input = read_default_test();
	let (_, interner) = filled_setup(&input);
	bencher.iter(|| {
		for (sym, strref) in interner.iter() {
			black_box((sym, strref));
		}
	})
}

#[bench]
fn values_iter(bencher: &mut Bencher) {
	let input = read_default_test();
	let (_, interner) = filled_setup(&input);
	bencher.iter(|| {
		for strref in interner.iter_values() {
			black_box(strref);
		}
	})
}

/// Mainly needed to approximate the `into_iterator` test below.
#[bench]
fn clone(bencher: &mut Bencher) {
	let input = read_default_test();
	let (_, interner) = filled_setup(&input);
	bencher.iter(|| {
		black_box(interner.clone());
	})
}

#[bench]
fn into_iterator(bencher: &mut Bencher) {
	let input = read_default_test();
	let (_, interner) = filled_setup(&input);
	bencher.iter(|| {
		for (sym, string) in interner.clone() {
			black_box((sym, string));
		}
	})
}

#[bench]
fn fnv_get_or_intern_empty(bencher: &mut Bencher) {
	let input = read_default_test();
	let (lines, interner) = empty_setup_with_hasher(
		&input, BuildHasherDefault::<FnvHasher>::default());
	bencher.iter(|| {
		let mut interner = interner.clone();
		for &line in lines.iter() {
			black_box(interner.get_or_intern(line));
		}
	})
}

#[bench]
fn fnv_get_or_intern_filled(bencher: &mut Bencher) {
	let input = read_default_test();
	let (lines, mut interner) = empty_setup_with_hasher(
		&input, BuildHasherDefault::<FnvHasher>::default());
	for &line in lines.iter() {
		interner.get_or_intern(line);
	}
	bencher.iter(|| {
		for &line in lines.iter() {
			black_box(interner.get_or_intern(line));
		}
	});
}
