use std::collections::HashMap;

/// Represents references into Interner datastructures.
/// 
/// Values of this type shall be lightweight as the whole purpose
/// of interning values is to be able to store them very memory efficiently.
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

#[derive(Debug, Default, Clone)]
pub struct StringInterner<Idx = usize>
	where Idx: InternIndex
{
	map   : HashMap<&'static str, Idx>,
	values: Vec<Box<str>>
}

impl<Idx> StringInterner<Idx>
	where Idx: InternIndex
{
	unsafe fn to_static_str(val: &str) -> &'static str {
		std::mem::transmute::<&str, &'static str>(val)
	}

	pub fn get_or_intern_str(&mut self, val: &str) -> Idx {
		match self.map.get(val) {
			Some(&intern_ref) => intern_ref,
			_                 => {
				let new_val = val.to_owned().into_boxed_str();
				let new_id  = self.make_idx();
				self.values.push(new_val);
				let new_ref = &*self.values.last().unwrap();
				self.map.insert(unsafe { Self::to_static_str(new_ref) }, new_id);
				new_id
			}
		}
	}

	pub fn get_or_intern_string(&mut self, val: String) -> Idx {
		match self.map.get(unsafe { Self::to_static_str(&val) }) {
			Some(&intern_ref) => intern_ref,
			_                 => {
				let new_val = val.into_boxed_str();
				let new_id  = self.make_idx();
				self.values.push(new_val);
				let new_ref = &*self.values.last().unwrap();
				self.map.insert(unsafe { Self::to_static_str(new_ref) }, new_id);
				new_id
			}
		}
	}

	fn make_idx(&self) -> Idx {
		Idx::from_index(self.len())
	}

	pub fn get(&self, index: Idx) -> Option<&str> {
		match self.values.get(index.to_index()) {
			Some(box_str) => Some(&box_str),
			None          => None
		}
	}

	pub fn lookup_index(&self, val: &str) -> Option<Idx> {
		self.map.get(val).map(|&idx| idx)
	}

	pub fn len(&self) -> usize {
		self.values.len()
	}

	pub fn clear(&mut self) {
		self.map.clear();
		self.values.clear()
	}
}

#[cfg(test)]
mod tests {
	use ::StringInterner;

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
