use std::sync::{Arc, Weak, Mutex};

// #[macro_use]
extern crate crossbeam_channel;

pub mod supervisor;
pub mod populations;
pub mod devices;
pub mod components;
pub mod operation;
pub mod connectivity;
mod utils;
// pub mod ffi;

type AcMx<T> = Arc<Mutex<T>>;
type WcMx<T> = Weak<Mutex<T>>;
