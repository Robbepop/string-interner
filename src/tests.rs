use crate::{
    symbol::expect_valid_symbol,
    DefaultStringInterner,
    DefaultSymbol,
    StringInterner,
};

mod sym {
    use super::*;

    #[test]
    fn same_size_as_optional() {
        use std::mem;
        assert_eq!(
            mem::size_of::<DefaultSymbol>(),
            mem::size_of::<Option<DefaultSymbol>>()
        );
    }
}

mod len {
    use super::*;

    #[test]
    fn new_len() {
        assert_eq!(DefaultStringInterner::new().len(), 0)
    }

    #[test]
    fn len_after_intern() {
        let mut interner = DefaultStringInterner::new();
        interner.get_or_intern("foo");
        assert_eq!(interner.len(), 1)
    }

    #[test]
    fn len_after_same() {
        let mut interner = DefaultStringInterner::new();
        interner.get_or_intern("foo");
        interner.get_or_intern("foo");
        assert_eq!(interner.len(), 1)
    }

    #[test]
    fn len_after_diff() {
        let mut interner = DefaultStringInterner::new();
        interner.get_or_intern("foo");
        interner.get_or_intern("bar");
        assert_eq!(interner.len(), 2)
    }
}

mod is_empty {
    use super::*;

    #[test]
    fn new() {
        assert_eq!(DefaultStringInterner::new().is_empty(), true)
    }

    #[test]
    fn not_empty() {
        let mut interner = DefaultStringInterner::with_capacity(1);
        interner.get_or_intern("foo");
        assert_eq!(interner.is_empty(), false)
    }
}

mod get_or_intern {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(
            DefaultStringInterner::new().get_or_intern("foo"),
            expect_valid_symbol(0),
        )
    }

    #[test]
    fn empty_string() {
        assert_eq!(
            DefaultStringInterner::new().get_or_intern(""),
            expect_valid_symbol(0),
        )
    }

    #[test]
    fn same_twice() {
        let mut interner = DefaultStringInterner::new();
        let fst = interner.get_or_intern("foo");
        let snd = interner.get_or_intern("foo");
        assert_eq!(fst, snd);
    }

    #[test]
    fn two_different() {
        let mut interner = DefaultStringInterner::new();
        let fst = interner.get_or_intern("foo");
        let snd = interner.get_or_intern("bar");
        assert_ne!(fst, snd);
    }

    #[test]
    fn act_same() {
        let mut interner1 = DefaultStringInterner::new();
        let mut interner2 = DefaultStringInterner::new();
        let sym1 = interner1.get_or_intern("foo");
        let sym2 = interner2.get_or_intern("foo");
        assert_eq!(sym1, sym2);
    }

    #[test]
    fn intern_string() {
        assert_eq!(
            DefaultStringInterner::new().get_or_intern(String::from("foo")),
            expect_valid_symbol(0),
        )
    }
}

mod default {
    use super::*;

    #[test]
    fn same_as_empty() {
        assert_eq!(StringInterner::default(), DefaultStringInterner::new())
    }
}

mod capacity {
    use super::*;

    #[test]
    fn new() {
        assert_eq!(DefaultStringInterner::new().capacity(), 0)
    }

    #[test]
    fn with_capacity() {
        assert_eq!(DefaultStringInterner::with_capacity(42).capacity(), 42)
    }

    #[test]
    fn with_capacity_len_0() {
        assert_eq!(DefaultStringInterner::with_capacity(5).len(), 0)
    }

    #[test]
    fn reserve() {
        let mut interner = DefaultStringInterner::new();
        assert_eq!(interner.capacity(), 0);
        interner.reserve(1337);
        assert_eq!(interner.capacity(), 1337);
    }

    #[test]
    fn with_capacity_eq_reserve() {
        let interner1 = DefaultStringInterner::with_capacity(42);
        let mut interner2 = DefaultStringInterner::new();
        assert_ne!(interner1.capacity(), interner2.capacity());
        interner2.reserve(42);
        assert_eq!(interner1.capacity(), interner2.capacity());
    }

    #[test]
    fn empty_shrink_to_fit() {
        let mut interner = DefaultStringInterner::with_capacity(100);
        assert_eq!(interner.capacity(), 100);
        interner.shrink_to_fit();
        assert_eq!(interner.capacity(), 0);
    }

    #[test]
    fn full_shrink_to_fit() {
        let mut interner = DefaultStringInterner::with_capacity(1);
        interner.get_or_intern("foo");
        assert_eq!(interner.capacity(), 1);
        interner.shrink_to_fit();
        assert_eq!(interner.capacity(), 1);
    }

    #[test]
    fn partial_shrink_to_fit() {
        let mut interner = DefaultStringInterner::with_capacity(3);
        interner.get_or_intern("foo");
        interner.get_or_intern("bar");
        assert_eq!(interner.capacity(), 3);
        interner.shrink_to_fit();
        assert_eq!(interner.capacity(), 2);
    }
}

mod resolve {
    use super::*;

    #[test]
    fn simple() {
        let mut interner = DefaultStringInterner::new();
        let sym = interner.get_or_intern("foo");
        assert_eq!(interner.resolve(sym), Some("foo"));
    }

    #[test]
    fn not_found() {
        let interner = DefaultStringInterner::new();
        assert_eq!(interner.resolve(expect_valid_symbol(0)), None);
    }

    #[test]
    fn unchecked() {
        let mut interner = DefaultStringInterner::new();
        let sym = interner.get_or_intern("foo");
        assert_eq!(unsafe { interner.resolve_unchecked(sym) }, "foo");
    }
}

mod get {
    use super::*;

    #[test]
    fn simple() {
        let mut interner = DefaultStringInterner::new();
        let sym = interner.get_or_intern("foo");
        assert_eq!(interner.get("foo"), Some(sym));
    }

    #[test]
    fn not_founds() {
        let interner = DefaultStringInterner::new();
        assert_eq!(interner.get("foo"), None);
    }

    #[test]
    fn simple_strings() {
        let mut interner = DefaultStringInterner::new();
        let sym = interner.get_or_intern("foo");
        assert_eq!(interner.get(String::from("foo")), Some(sym));
    }
}

mod iter {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(DefaultStringInterner::new().iter().next(), None)
    }

    #[test]
    fn simple() {
        let interner: DefaultStringInterner =
            vec!["foo", "bar", "baz", "foo"].into_iter().collect();
        let mut iter = interner.iter();
        assert_eq!(iter.next(), Some((expect_valid_symbol(0), "foo")));
        assert_eq!(iter.next(), Some((expect_valid_symbol(1), "bar")));
        assert_eq!(iter.next(), Some((expect_valid_symbol(2), "baz")));
        assert_eq!(iter.next(), None);
    }
}

mod iter_values {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(DefaultStringInterner::new().iter_values().next(), None)
    }

    #[test]
    fn simple() {
        let interner: DefaultStringInterner =
            vec!["foo", "bar", "baz", "foo"].into_iter().collect();
        let mut iter = interner.iter_values();
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), Some("bar"));
        assert_eq!(iter.next(), Some("baz"));
        assert_eq!(iter.next(), None);
    }
}

mod into_iter {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(DefaultStringInterner::new().into_iter().next(), None)
    }

    #[test]
    fn simple() {
        let interner: DefaultStringInterner =
            vec!["foo", "bar", "baz", "foo"].into_iter().collect();
        let mut iter = interner.into_iter();
        assert_eq!(
            iter.next(),
            Some((expect_valid_symbol(0), String::from("foo")))
        );
        assert_eq!(
            iter.next(),
            Some((expect_valid_symbol(1), String::from("bar")))
        );
        assert_eq!(
            iter.next(),
            Some((expect_valid_symbol(2), String::from("baz")))
        );
        assert_eq!(iter.next(), None);
    }
}

mod from_iterator {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(
            DefaultStringInterner::new(),
            Vec::<&str>::new()
                .into_iter()
                .collect::<DefaultStringInterner>()
        )
    }

    #[test]
    fn simple() {
        assert_eq!(
            vec!["foo", "bar"]
                .into_iter()
                .collect::<DefaultStringInterner>(),
            {
                let mut interner = DefaultStringInterner::new();
                interner.get_or_intern("foo");
                interner.get_or_intern("bar");
                interner
            }
        );
    }

    #[test]
    fn multiple_same() {
        assert_eq!(
            vec!["foo", "foo"]
                .into_iter()
                .collect::<DefaultStringInterner>(),
            {
                let mut interner = DefaultStringInterner::new();
                interner.get_or_intern("foo");
                interner
            }
        );
    }
}

mod extend {
    use super::*;

    #[test]
    fn empty() {
        let mut interner = DefaultStringInterner::new();
        interner.extend(Vec::<&str>::new());
        assert_eq!(interner, DefaultStringInterner::new(),);
    }

    #[test]
    fn simple() {
        assert_eq!(
            {
                let mut interner = DefaultStringInterner::new();
                interner.extend(vec!["foo", "bar"]);
                interner
            },
            {
                let mut interner = DefaultStringInterner::new();
                interner.get_or_intern("foo");
                interner.get_or_intern("bar");
                interner
            }
        );
    }

    #[test]
    fn multiple_same() {
        assert_eq!(
            {
                let mut interner = DefaultStringInterner::new();
                interner.extend(vec!["foo", "foo"]);
                interner
            },
            {
                let mut interner = DefaultStringInterner::new();
                interner.get_or_intern("foo");
                interner.get_or_intern("foo");
                interner
            }
        );
    }
}

// See <https://github.com/Robbepop/string-interner/issues/9>.
mod clone_and_drop {
    use super::*;

    fn clone_and_drop() -> (DefaultStringInterner, DefaultSymbol) {
        let mut old = DefaultStringInterner::new();
        let foo = old.get_or_intern("foo");

        // Return newly created (cloned) interner, and drop the original `old` itself.
        (old.clone(), foo)
    }

    #[test]
    fn no_use_after_free() {
        let (mut new, foo) = clone_and_drop();

        // This assert may fail if there are use after free bug.
        // See <https://github.com/Robbepop/string-interner/issues/9> for detail.
        assert_eq!(
            new.get_or_intern("foo"),
            foo,
            "`foo` should represent the string \"foo\" so they should be equal"
        );
    }

    #[test]
    // Test for new (non-`derive`) `Clone` impl.
    fn clone() {
        let mut old = DefaultStringInterner::new();
        let strings = &["foo", "bar", "baz", "qux", "quux", "corge"];
        let syms = strings
            .iter()
            .map(|&s| old.get_or_intern(s))
            .collect::<Vec<_>>();

        let mut new = old.clone();
        for (&s, &sym) in strings.iter().zip(&syms) {
            assert_eq!(new.resolve(sym), Some(s));
            assert_eq!(new.get_or_intern(s), sym);
        }
    }
}
