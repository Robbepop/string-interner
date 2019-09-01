use super::*;

use test::{black_box, Bencher};
use lazy_static::lazy_static;
use ::fnv::FnvHasher;
use std::hash::BuildHasherDefault;

fn read_file_to_string(path: &str) -> String {
	use std::{fs::File, io::prelude::*};
	let mut f = File::open(path).expect("bench file not found");
	let mut s = String::new();
	f.read_to_string(&mut s)
		.expect("encountered problems writing bench file to string");
	s
}

fn read_bench_file_to_string() -> String {
	read_file_to_string("bench/input.txt")
}

lazy_static! {
	static ref BENCH_INPUT: String = read_bench_file_to_string();
	static ref BENCH_LINES: Vec<&'static str> =
		{ BENCH_INPUT.split_whitespace().collect::<Vec<&str>>() };
}

fn bench_lines() -> &'static [&'static str] {
	&BENCH_LINES
}

struct EmptySetup<H> {
	lines: &'static [&'static str],
	build_hasher: H,
}

impl EmptySetup<RandomState> {
	pub fn new() -> Self {
		let lines = bench_lines();
		EmptySetup {
			lines,
			build_hasher: RandomState::new(),
		}
	}
}

impl<S> EmptySetup<BuildHasherDefault<S>>
where
	S: Hasher,
{
	pub fn new_with_hasher() -> Self {
		let lines = bench_lines();
		let build_hasher = BuildHasherDefault::<S>::default();
		EmptySetup {
			lines,
			build_hasher,
		}
	}
}

impl<H> EmptySetup<H>
where
	H: BuildHasher,
{
	pub fn lines(&self) -> &'static [&'static str] {
		self.lines
	}
}

impl<H> EmptySetup<H>
where
	H: BuildHasher + Clone,
{
	pub fn empty_interner(&self) -> StringInterner<Sym, H> {
		StringInterner::with_capacity_and_hasher(self.lines.len(), self.build_hasher.clone())
	}
}

fn empty_setup() -> EmptySetup<RandomState> {
	EmptySetup::new()
}

struct FilledSetup<H>
where
	H: BuildHasher,
{
	lines: &'static [&'static str],
	interner: StringInterner<Sym, H>,
	symbols: Vec<Sym>,
}

impl FilledSetup<RandomState> {
	pub fn new() -> Self {
		let lines = bench_lines();
		let mut interner = StringInterner::with_capacity(lines.len());
		let symbols = lines
			.into_iter()
			.map(|&line| interner.get_or_intern(line))
			.collect::<Vec<_>>();
		FilledSetup {
			lines,
			interner,
			symbols,
		}
	}
}

impl<S> FilledSetup<BuildHasherDefault<S>>
where
	S: Hasher + Default,
{
	pub fn new_with_hasher() -> Self {
		let lines = bench_lines();
		let build_hasher = BuildHasherDefault::<S>::default();
		let mut interner = StringInterner::with_capacity_and_hasher(lines.len(), build_hasher);
		let symbols = lines
			.into_iter()
			.map(|&line| interner.get_or_intern(line))
			.collect::<Vec<_>>();
		FilledSetup {
			lines,
			interner,
			symbols,
		}
	}
}

impl<H> FilledSetup<H>
where
	H: BuildHasher,
{
	pub fn lines(&self) -> &'static [&'static str] {
		self.lines
	}

	pub fn filled_interner(&self) -> &StringInterner<Sym, H> {
		&self.interner
	}

	pub fn filled_interner_mut(&mut self) -> &mut StringInterner<Sym, H> {
		&mut self.interner
	}

	pub fn interned_symbols(&self) -> &[Sym] {
		&self.symbols
	}
}

fn filled_setup() -> FilledSetup<RandomState> {
	FilledSetup::new()
}

#[bench]
fn from_iterator(bencher: &mut Bencher) {
	let setup = empty_setup();
	bencher.iter(|| {
		black_box(DefaultStringInterner::from_iter(
			setup.lines().into_iter().map(|&line| line),
		))
	})
}

#[bench]
fn get_or_intern_empty(bencher: &mut Bencher) {
	let setup = empty_setup();
	bencher.iter(|| {
		let mut interner = setup.empty_interner();
		for &line in setup.lines() {
			black_box(interner.get_or_intern(line));
		}
	});
}

#[bench]
fn get_or_intern_filled(bencher: &mut Bencher) {
	let mut setup = filled_setup();
	bencher.iter(|| {
		for &line in setup.lines() {
			black_box(setup.filled_interner_mut().get_or_intern(line));
		}
	});
}

#[bench]
fn get_empty(bencher: &mut Bencher) {
	let setup = empty_setup();
	bencher.iter(|| {
		let interner = setup.empty_interner();
		for &line in setup.lines() {
			black_box(interner.get(line));
		}
	});
}

#[bench]
fn get_filled(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		for &line in setup.lines() {
			black_box(setup.filled_interner().get(line));
		}
	});
}

#[bench]
fn resolve(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		for &sym in setup.interned_symbols() {
			black_box(setup.filled_interner().resolve(sym));
		}
	});
}

#[bench]
fn resolve_unchecked(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		for &sym in setup.interned_symbols() {
			black_box(unsafe { setup.filled_interner().resolve_unchecked(sym) });
		}
	});
}

#[bench]
fn iter(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		for (sym, str_ref) in setup.filled_interner().iter() {
			black_box((sym, str_ref));
		}
	})
}

#[bench]
fn values_iter(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		for str_ref in setup.filled_interner().iter_values() {
			black_box(str_ref);
		}
	})
}

/// Mainly needed to approximate the `into_iterator` test below.
#[bench]
fn clone(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		black_box(setup.filled_interner().clone());
	})
}

/// This benchmark performs an internal `StringInterner::clone` so that
/// has to be subtracted for the real timing of this operation.
#[bench]
fn into_iterator(bencher: &mut Bencher) {
	let setup = filled_setup();
	bencher.iter(|| {
		for (sym, string) in setup.filled_interner().clone().into_iter() {
			black_box((sym, string));
		}
	})
}

mod fnv {
	use super::*;

	type FnvBuildHasher = BuildHasherDefault<FnvHasher>;

	fn empty_fnv_setup() -> EmptySetup<FnvBuildHasher> {
		EmptySetup::<FnvBuildHasher>::new_with_hasher()
	}

	fn filled_fnv_setup() -> FilledSetup<BuildHasherDefault<FnvHasher>> {
		FilledSetup::new_with_hasher()
	}

	#[bench]
	fn new_empty(bencher: &mut Bencher) {
		let setup = empty_fnv_setup();
		bencher.iter(|| {
			for &_line in setup.lines() {
				black_box(setup.empty_interner());
			}
		})
	}

	#[bench]
	fn get_or_intern_empty(bencher: &mut Bencher) {
		let setup = empty_fnv_setup();
		bencher.iter(|| {
			let mut interner = setup.empty_interner();
			for &line in setup.lines() {
				black_box(interner.get_or_intern(line));
			}
		})
	}

	#[bench]
	fn get_or_intern_filled(bencher: &mut Bencher) {
		let mut setup = filled_fnv_setup();
		bencher.iter(|| {
			for &line in setup.lines() {
				black_box(setup.filled_interner_mut().get_or_intern(line));
			}
		});
	}

	#[bench]
	fn get_empty(bencher: &mut Bencher) {
		let setup = empty_fnv_setup();
		bencher.iter(|| {
			let interner = setup.empty_interner();
			for &line in setup.lines() {
				black_box(interner.get(line));
			}
		})
	}

	#[bench]
	fn get_filled(bencher: &mut Bencher) {
		let setup = filled_fnv_setup();
		bencher.iter(|| {
			for &line in setup.lines() {
				black_box(setup.filled_interner().get(line));
			}
		});
	}
}
