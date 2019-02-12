#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod bindings {
    #[allow(unused_variables)]
    include!(concat!(env!("OUT_DIR"), "/fprint.rs"));
}

pub use bindings::*;
