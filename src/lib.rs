use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Represents indices into the StringInterner.
/// 
/// Values of this type shall be lightweight as the whole purpose
/// of interning values is to be able to store them efficiently in memory.
/// 
/// This trait allows definitions of custom InternIndices besides
/// the already supported unsigned integer primitives.
pub trait InternIndex: Copy {
	fn from_index(idx: usize) -> Self;
	fn to_index(&self) -> usize;
}

macro_rules! impl_intern_ref {
	( $primitive:ty ) => {
		impl InternIndex for $primitive {
			fn from_index(idx: usize) -> Self {
				idx as $primitive
			}

			fn to_index(&self) -> usize {
				*self as usize
			}
		}
	}
}

impl_intern_ref!(u8);
impl_intern_ref!(u16);
impl_intern_ref!(u32);
impl_intern_ref!(u64);
impl_intern_ref!(usize);

/// Internal reference to str used only within the StringInterner itself
/// to encapsulate the unsafe behaviour of interor references.
#[derive(Debug, Copy, Clone, Eq)]
struct InternalStrRef(*const str);

impl InternalStrRef {
	/// Creates an InternalStrRef from a str.
	/// 
	/// This just wraps the str internally.
	fn from_str(val: &str) -> Self {
		InternalStrRef(
			unsafe{ &*(val as *const str) }
		)
	}

	/// Reinterprets this InternalStrRef as a str.
	/// 
	/// Does not allocate memory!
	fn as_str(&self) -> &str {
		unsafe{ &*self.0 as &str }
	}
}

impl<'a> From<&'a str> for InternalStrRef {
	fn from(val: &str) -> InternalStrRef {
		InternalStrRef::from_str(val)
	}
}

impl Hash for InternalStrRef {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl PartialEq for InternalStrRef {
	fn eq(&self, other: &InternalStrRef) -> bool {
		self.as_str() == other.as_str()
	}
}

/// Provides a bidirectional mapping between String stored within
/// the interner and indices.
/// The main purpose is to store every unique String only once and
/// make it possible to reference it via lightweight indices.
/// 
/// Compilers often use this for implementing a symbol table.
/// 
/// The main goal of this StringInterner is to store String
/// with as low memory overhead as possible.
#[derive(Debug, Default, Clone)]
pub struct StringInterner<Idx = usize>
	where Idx: InternIndex
{
	map   : HashMap<InternalStrRef, Idx>,
	values: Vec<Box<str>>
}

impl<Idx> StringInterner<Idx>
	where Idx: InternIndex
{
	/// Interns the given str if it was not interned already
	/// and returns an index to access the newly interned String or
	/// to the already interned String.
	/// 
	/// This copies the contents of the given str.
	pub fn get_or_intern_str(&mut self, val: &str) -> Idx {
		match self.map.get(&val.into()) {
			Some(&idx) => idx,
			None       => self.gensym(val.to_owned())
		}
	}

	/// Interns the given String if it was not interned already
	/// and returns an index to access the newly interned String or
	/// to the already interned String.
	/// 
	/// This consumes the given String.
	pub fn get_or_intern_string(&mut self, val: String) -> Idx {
		match self.map.get(&val.as_str().into()) {
			Some(&idx) => idx,
			None       => self.gensym(val)
		}
	}

	/// Interns the given String and returns an index to access it.
	/// 
	/// This does not check for collissions!
	fn gensym(&mut self, new_val: String) -> Idx {
		let new_id  = self.make_idx();
		let new_ref = InternalStrRef::from_str(new_val.as_str());
		self.values.push(new_val.into_boxed_str());
		self.map.insert(new_ref, new_id);
		new_id
	}

	/// Creates a new index for the current state of the interner.
	fn make_idx(&self) -> Idx {
		Idx::from_index(self.len())
	}

	/// Returns a string slice to the string identified by the given index if available.
	/// Else, None is returned.
	pub fn get(&self, index: Idx) -> Option<&str> {
		self.values
			.get(index.to_index())
			.map(|string| &**string)
	}

	/// Returns the index that is mapped for the given string if available.
	/// Else, None is returned.
	pub fn lookup_index(&self, val: &str) -> Option<Idx> {
		self.map
			.get(&val.into())
			.map(|&idx| idx)
	}

	/// Returns the number of uniquely stored Strings interned within this interner.
	pub fn len(&self) -> usize {
		self.values.len()
	}

	/// Removes all interned Strings from this interner.
	pub fn clear(&mut self) {
		self.map.clear();
		self.values.clear()
	}
}

#[cfg(test)]
mod tests {
	use ::{StringInterner, InternalStrRef};

	fn make_dummy_interner() -> (StringInterner, [usize; 8]) {
		let mut interner = StringInterner::default();
		let name0 = interner.get_or_intern_str("foo");
		let name1 = interner.get_or_intern_str("bar");
		let name2 = interner.get_or_intern_str("baz");
		let name3 = interner.get_or_intern_str("foo");
		let name4 = interner.get_or_intern_str("rofl");
		let name5 = interner.get_or_intern_str("bar");
		let name6 = interner.get_or_intern_str("mao");
		let name7 = interner.get_or_intern_str("foo");
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
		let mut interner = StringInterner::<usize>::default();
		let name_0 = interner.get_or_intern_string("Hello".to_owned());
		let name_1 = interner.get_or_intern_string("World".to_owned());
		let name_2 = interner.get_or_intern_string("I am a String".to_owned());
		let name_3 = interner.get_or_intern_string("Foo".to_owned());
		let name_4 = interner.get_or_intern_string("Bar".to_owned());
		let name_5 = interner.get_or_intern_string("I am a String".to_owned());
		let name_6 = interner.get_or_intern_string("Next is empty".to_owned());
		let name_7 = interner.get_or_intern_string("".to_owned());
		let name_8 = interner.get_or_intern_string("I am a String".to_owned());
		let name_9 = interner.get_or_intern_string("I am a String".to_owned());
		let name10 = interner.get_or_intern_string("Foo".to_owned());

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
		assert_eq!(interner.get(0), Some("foo"));
		assert_eq!(interner.get(1), Some("bar"));
		assert_eq!(interner.get(2), Some("baz"));
		assert_eq!(interner.get(3), Some("rofl"));
		assert_eq!(interner.get(4), Some("mao"));
		assert_eq!(interner.get(5), None);
	}

	#[test]
	fn lookup_index() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.lookup_index("foo"),  Some(0));
		assert_eq!(interner.lookup_index("bar"),  Some(1));
		assert_eq!(interner.lookup_index("baz"),  Some(2));
		assert_eq!(interner.lookup_index("rofl"), Some(3));
		assert_eq!(interner.lookup_index("mao"),  Some(4));
		assert_eq!(interner.lookup_index("xD"),   None);
	}

	#[test]
	fn clear() {
		let (mut interner, _) = make_dummy_interner();
		assert_eq!(interner.len(), 5);
		interner.clear();
		assert_eq!(interner.len(), 0);
	}
}
