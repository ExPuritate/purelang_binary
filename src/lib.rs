#![feature(slice_as_array)]
#![feature(iterator_try_collect)]
#![feature(try_find)]
#![feature(const_trait_impl)]
#![feature(decl_macro)]
#![feature(marker_trait_attr)]
#![feature(generic_const_exprs)]
#![feature(macro_metavar_expr)]
#![feature(maybe_uninit_array_assume_init)]
#![allow(incomplete_features)]

pub mod assembly;
pub mod core;
mod implement;
#[cfg(test)]
mod tests;
pub mod traits;
mod ty;

pub(crate) type Error = global::errors::BinaryError;

pub use assembly::Assembly;
pub use ty::*;
