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

fn case2() {
	let mut interner = StringInterner::default();
	let s_ref;
	{
		let mut pool = StringPool::new(&mut interner); //~ ERROR does not live long enough
		s_ref = pool.get_or_intern("case2");
	}
	println!("garbage: {:?}", s_ref);
}

fn case3() {
	let s;
	{
		let mut interner = StringInterner::default();
		let mut pool = StringPool::new(&mut interner); //~ ERROR does not live long enough
		s = &*pool.get_or_intern("case3"); //~ ERROR does not live long enough
	}
	println!("garbage: {:?}", s);
}

fn case4() {
	let mut interner = StringInterner::default();
	let s;
	{
		let mut pool = StringPool::new(&mut interner);
		s = &*pool.get_or_intern("case4"); //~ ERROR does not live long enough
	}
	println!("garbage: {:?}", s);
}

fn case5() {
	let mut interner = StringInterner::default();
	let mut pool = StringPool::new(&mut interner);
	let s;
	{
		s = &*pool.get_or_intern("case5"); //~ ERROR does not live long enough
	}
	println!("ok: {:?}", s);
}

fn main() {}
