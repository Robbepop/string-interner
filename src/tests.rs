use ::{DefaultStringInterner, InternalStrRef};

fn make_dummy_interner() -> (DefaultStringInterner, [usize; 8]) {
	let mut interner = DefaultStringInterner::new();
	let name0 = interner.get_or_intern("foo");
	let name1 = interner.get_or_intern("bar");
	let name2 = interner.get_or_intern("baz");
	let name3 = interner.get_or_intern("foo");
	let name4 = interner.get_or_intern("rofl");
	let name5 = interner.get_or_intern("bar");
	let name6 = interner.get_or_intern("mao");
	let name7 = interner.get_or_intern("foo");
	(interner, [name0, name1, name2, name3, name4, name5, name6, name7])
}

#[test]
fn internal_str_ref() {
	use std::mem;
	assert_eq!(mem::size_of::<InternalStrRef>(), mem::size_of::<&str>());

	let s0 = "Hello";
	let s1 = ", World!";
	let s2 = "Hello";
	let s3 = ", World!";
	let r0 = InternalStrRef::from_str(s0);
	let r1 = InternalStrRef::from_str(s1);
	let r2 = InternalStrRef::from_str(s2);
	let r3 = InternalStrRef::from_str(s3);
	assert_eq!(r0, r2);
	assert_eq!(r1, r3);
	assert_ne!(r0, r1);
	assert_ne!(r2, r3);

	use std::collections::hash_map::DefaultHasher;
	use std::hash::Hash;
	let mut sip = DefaultHasher::new();
	assert_eq!(r0.hash(&mut sip), s0.hash(&mut sip));
	assert_eq!(r1.hash(&mut sip), s1.hash(&mut sip));
	assert_eq!(r2.hash(&mut sip), s2.hash(&mut sip));
	assert_eq!(r3.hash(&mut sip), s3.hash(&mut sip));
}

#[test]
fn intern_str() {
	let (_, names) = make_dummy_interner();
	assert_eq!(names[0], 0);
	assert_eq!(names[1], 1);
	assert_eq!(names[2], 2);
	assert_eq!(names[3], 0);
	assert_eq!(names[4], 3);
	assert_eq!(names[5], 1);
	assert_eq!(names[6], 4);
	assert_eq!(names[7], 0);
}

#[test]
fn intern_string() {
	let mut interner = DefaultStringInterner::new();
	let name_0 = interner.get_or_intern("Hello".to_owned());
	let name_1 = interner.get_or_intern("World".to_owned());
	let name_2 = interner.get_or_intern("I am a String".to_owned());
	let name_3 = interner.get_or_intern("Foo".to_owned());
	let name_4 = interner.get_or_intern("Bar".to_owned());
	let name_5 = interner.get_or_intern("I am a String".to_owned());
	let name_6 = interner.get_or_intern("Next is empty".to_owned());
	let name_7 = interner.get_or_intern("".to_owned());
	let name_8 = interner.get_or_intern("I am a String".to_owned());
	let name_9 = interner.get_or_intern("I am a String".to_owned());
	let name10 = interner.get_or_intern("Foo".to_owned());

	assert_eq!(interner.len(), 7);

	assert_eq!(name_0, 0);
	assert_eq!(name_1, 1);
	assert_eq!(name_2, 2);
	assert_eq!(name_3, 3);
	assert_eq!(name_4, 4);
	assert_eq!(name_5, 2);
	assert_eq!(name_6, 5);
	assert_eq!(name_7, 6);
	assert_eq!(name_8, 2);
	assert_eq!(name_9, 2);
	assert_eq!(name10, 3);
}

#[test]
fn len() {
	let (interner, _) = make_dummy_interner();
	assert_eq!(interner.len(), 5);	
}

#[test]
fn get() {
	let (interner, _) = make_dummy_interner();
	assert_eq!(interner.resolve(0), Some("foo"));
	assert_eq!(interner.resolve(1), Some("bar"));
	assert_eq!(interner.resolve(2), Some("baz"));
	assert_eq!(interner.resolve(3), Some("rofl"));
	assert_eq!(interner.resolve(4), Some("mao"));
	assert_eq!(interner.resolve(5), None);
}

#[test]
fn lookup_symbol() {
	let (interner, _) = make_dummy_interner();
	assert_eq!(interner.get("foo"),  Some(0));
	assert_eq!(interner.get("bar"),  Some(1));
	assert_eq!(interner.get("baz"),  Some(2));
	assert_eq!(interner.get("rofl"), Some(3));
	assert_eq!(interner.get("mao"),  Some(4));
	assert_eq!(interner.get("xD"),   None);
}

#[test]
fn clear() {
	let (mut interner, _) = make_dummy_interner();
	assert_eq!(interner.len(), 5);
	interner.clear();
	assert_eq!(interner.len(), 0);
}

#[test]
fn iter_values() {
	let (interner, _) = make_dummy_interner();
	let mut it = interner.iter_values();
	assert_eq!(it.next(), Some("foo"));
	assert_eq!(it.next(), Some("bar"));
	assert_eq!(it.next(), Some("baz"));
	assert_eq!(it.next(), Some("rofl"));
	assert_eq!(it.next(), Some("mao"));
	assert_eq!(it.next(), None);
}

#[test]
fn iter() {
	let (interner, _) = make_dummy_interner();
	let mut it = interner.iter();
	assert_eq!(it.next(), Some((0, "foo")));
	assert_eq!(it.next(), Some((1, "bar")));
	assert_eq!(it.next(), Some((2, "baz")));
	assert_eq!(it.next(), Some((3, "rofl")));
	assert_eq!(it.next(), Some((4, "mao")));
	assert_eq!(it.next(), None);
}

#[test]
fn into_iter() {
	let (interner, _) = make_dummy_interner();
	let mut it = interner.into_iter();
	assert_eq!(it.next(), Some((0, "foo".to_owned())));
	assert_eq!(it.next(), Some((1, "bar".to_owned())));
	assert_eq!(it.next(), Some((2, "baz".to_owned())));
	assert_eq!(it.next(), Some((3, "rofl".to_owned())));
	assert_eq!(it.next(), Some((4, "mao".to_owned())));
	assert_eq!(it.next(), None);
}

#[test]
#[cfg(feature = "serde_support")]
fn serde() {
	use serde_json;
	let (interner, _) = make_dummy_interner();
	let serialized    = serde_json::to_string(&interner).unwrap();
	let deserialized: DefaultStringInterner = serde_json::from_str(&serialized).unwrap();
	assert_eq!(interner, deserialized);
}
