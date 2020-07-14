//! Compatibility layer for `no_std` compilations.

use cfg_if::cfg_if;

pub use ::hashbrown::hash_map as hash_map;
pub use ::hashbrown::hash_map::{
    DefaultHashBuilder,
    HashMap,
};

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use ::std::{
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
