#![warn(clippy::all)]

mod bindings {
    #![allow(
        non_upper_case_globals,
        non_camel_case_types,
        non_snake_case,
        unused_variables,
        clippy::unreadable_literal,
        clippy::redundant_static_lifetimes
    )]
    include!(concat!(env!("OUT_DIR"), "/fprint.rs"));
}

pub use bindings::*;
