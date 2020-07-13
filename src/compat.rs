//! Compatibility layer for `no_std` compilations.

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(not(feature = "std"), not(feature = "hashbrown")))] {
        // If the compile for `no_std` we need to enable `hashbrown`.
        compile_error!(
            "encountered invalid set of crate features. use `std` or `no_std` + `hashbrown`"
        )
    }
}

cfg_if! {
    if #[cfg(feature = "hashbrown")] {
        pub use ::hashbrown::HashMap;
        pub use ::hashbrown::hash_map::DefaultHashBuilder;
    } else {
        pub use ::std::collections::HashMap;
    }
}

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use ::std::{
            collections::{
                hash_map::RandomState,
                hash_map::RandomState as DefaultHashBuilder,
            },
            vec,
            vec::Vec,
            string::{String, ToString},
            boxed::Box,
        };
    } else {
        extern crate alloc;
        pub use self::alloc::{
            vec,
            vec::Vec,
            string::{String, ToString},
            boxed::Box,
        };
    }
}
