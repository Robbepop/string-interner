extern crate string_interner;
use string_interner::*;
use string_interner::wrapped::*;

fn case1() {
	let s_ref;
	{
		let mut interner = StringInterner::default();
		let mut pool = StringPool::new(&mut interner); //~ ERROR does not live long enough
		s_ref = pool.get_or_intern("case1");
	}
	println!("garbage: {:?}", s_ref);
}

// OK because s_ref's validity is tied to `interner` not `pool`
fn case2() {
	let mut interner = StringInterner::default();
	let s_ref;
	{
		let mut pool = StringPool::new(&mut interner);
		s_ref = pool.get_or_intern("case2");
	}
	println!("ok: {:?}", s_ref);
}

fn case3() {
	let mut interner = StringInterner::default();
	let mut pool = StringPool::new(&mut interner);
	let s_ref = pool.get_or_intern("case2");
	drop(pool);
	drop(interner); //~ ERROR cannot move
	println!("garbage: {:?}", s_ref);
}

fn main() {}
