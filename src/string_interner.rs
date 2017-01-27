use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use std::mem::transmute;

use ::{InternRef, InternRefSize};

#[derive(Debug, Default, Clone)]
pub struct StringInterner<R = InternRefSize>
	where R: InternRef
{
	map   : HashMap<&'static str, R>,
	values: Vec<Box<str>>
}

impl<R> StringInterner<R>
	where R: InternRef
{
	fn remove_lifetime(val: &str) -> &'static str {
		unsafe {
			transmute::<&str, &'static str>(val)
		}
	}

	pub fn intern_str(&mut self, val: &str) -> R {
		match self.map.get(val) {
			Some(&intern_ref) => intern_ref,
			_                 => {
				let new_val = val.to_owned().into_boxed_str();
				let new_id  = self.make_ref();
				self.values.push(new_val);
				let new_ref = self.values.last().unwrap();
				self.map.insert(Self::remove_lifetime(new_ref), new_id);
				new_id
			}
		}
	}

	pub fn intern_string(&mut self, val: String) -> R {
		match self.map.get(Self::remove_lifetime(&val)) {
			Some(&intern_ref) => intern_ref,
			_                 => {
				let new_val = val.into_boxed_str();
				let new_id  = self.make_ref();
				self.values.push(new_val);
				let new_ref = self.values.last().unwrap();
				self.map.insert(Self::remove_lifetime(new_ref), new_id);
				new_id
			}
		}
	}

	fn make_ref(&self) -> R {
		R::from_index(self.len())
	}

	pub fn get(&self, index: R) -> Option<&str> {
		match self.values.get(index.to_index()) {
			Some(box_str) => Some(&box_str),
			None          => None
		}
	}

	pub fn find_str(&self, val: &str) -> Option<R> {
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

impl<R> Index<R> for StringInterner<R>
	where R: InternRef
{
    type Output = Box<str>;

    fn index(&self, index: R) -> &Self::Output {
    	&self.values[index.to_index()]
    }
}

impl<R> IndexMut<R> for StringInterner<R>
	where R: InternRef
{
    fn index_mut(&mut self, index: R) -> &mut Self::Output {
    	&mut self.values[index.to_index()]
    }
}

#[cfg(test)]
mod tests {
	use ::StringInterner;
	use ::{InternRefSize};

	fn make_dummy_interner() -> (StringInterner, [InternRefSize; 8]) {
		let mut interner = StringInterner::default();
		let name0 = interner.intern_str("foo");
		let name1 = interner.intern_str("bar");
		let name2 = interner.intern_str("baz");
		let name3 = interner.intern_str("foo");
		let name4 = interner.intern_str("rofl");
		let name5 = interner.intern_str("bar");
		let name6 = interner.intern_str("mao");
		let name7 = interner.intern_str("foo");
		(interner, [name0, name1, name2, name3, name4, name5, name6, name7])
	}

	#[test]
	fn intern_str() {
		let (_, names) = make_dummy_interner();
		assert_eq!(names[0], InternRefSize(0));
		assert_eq!(names[1], InternRefSize(1));
		assert_eq!(names[2], InternRefSize(2));
		assert_eq!(names[3], InternRefSize(0));
		assert_eq!(names[4], InternRefSize(3));
		assert_eq!(names[5], InternRefSize(1));
		assert_eq!(names[6], InternRefSize(4));
		assert_eq!(names[7], InternRefSize(0));
	}

	#[test]
	fn index() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner[InternRefSize(0)], "foo".to_owned().into_boxed_str());
		assert_eq!(interner[InternRefSize(1)], "bar".to_owned().into_boxed_str());
		assert_eq!(interner[InternRefSize(2)], "baz".to_owned().into_boxed_str());
		assert_eq!(interner[InternRefSize(3)], "rofl".to_owned().into_boxed_str());
		assert_eq!(interner[InternRefSize(4)], "mao".to_owned().into_boxed_str());
	}

	#[test]
	#[should_panic]
	fn index_out_of_bounds() {
		let (interner, _) = make_dummy_interner();
		assert_ne!(interner[InternRefSize(5)], "".to_owned().into_boxed_str());
	}

	#[test]
	fn len() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.len(), 5);	
	}

	#[test]
	fn get() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.get(InternRefSize(0)), Some("foo"));
		assert_eq!(interner.get(InternRefSize(1)), Some("bar"));
		assert_eq!(interner.get(InternRefSize(2)), Some("baz"));
		assert_eq!(interner.get(InternRefSize(3)), Some("rofl"));
		assert_eq!(interner.get(InternRefSize(4)), Some("mao"));
		assert_eq!(interner.get(InternRefSize(5)), None);
	}

	#[test]
	fn find_str() {
		let (interner, _) = make_dummy_interner();
		assert_eq!(interner.find_str("foo"),  Some(InternRefSize(0)));
		assert_eq!(interner.find_str("bar"),  Some(InternRefSize(1)));
		assert_eq!(interner.find_str("baz"),  Some(InternRefSize(2)));
		assert_eq!(interner.find_str("rofl"), Some(InternRefSize(3)));
		assert_eq!(interner.find_str("mao"),  Some(InternRefSize(4)));
		assert_eq!(interner.find_str("xD"),   None);
	}
}
