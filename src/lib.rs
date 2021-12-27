//! Zero-runtime-cost heterogeneous lists.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Fgeode)](https://iteration-square.schichler.dev/#narrow/stream/project.2Fgeode)

#![doc(html_root_url = "https://docs.rs/geode/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub mod iterators;
pub mod thread_safety;

//TODO: Macro to generate a custom list type with optional Cons and RCons implementations.
//TODO: Macro to privately implement iteration.

/// A dynamic dispatch iteration target.
///
/// # Safety
///
/// For each marker trait that `Self` implements, the target of the [`dyn IterateeMut<T>`](`IterateeMut`) [reference](https://doc.rust-lang.org/stable/core/primitive.reference.html) returned from [`IterateeMut::head_rest_mut`] must implement it too.
pub unsafe trait Iteratee<T: ?Sized> {
	/// Returns the first item reference, if available, and "rest of the sequence"-iteratee.
	fn head_rest(&self) -> (Option<&T>, &dyn Iteratee<T>);

	/// Used to implement [`Iterator::size_hint`] on [`iterators::Iter`] and [`iterators::IterMut`].
	fn size_hint(&self) -> (usize, Option<usize>);
}

/// A target for mutating dynamic dispatch iteration.
///
/// # Safety
///
/// For each marker trait that `Self` implements,
///
/// * the target of the [`&dyn IterateeMut<T>`](`IterateeMut`) returned from [`IterateeMut::head_rest_mut`] must implement it too and
/// * the target of the [`dyn Iteratee<T>`](`Iteratee`) [reference](https://doc.rust-lang.org/stable/core/primitive.reference.html) returned from [`IterateeMut::as_iteratee`] must implement it too.
pub unsafe trait IterateeMut<T: ?Sized>: Iteratee<T> {
	/// Returns the first item reference, if available, and "rest of the sequence"-iteratee.
	fn head_rest_mut(&mut self) -> (Option<&mut T>, &mut dyn IterateeMut<T>);

	/// Borrows this instance as shared [`Iteratee<T>`];
	fn as_iteratee(&self) -> &dyn Iteratee<T>;
}

mod private {
	use std::ops::{Deref, DerefMut};

	use crate::{DynIteratee, Iteratee, IterateeMut};

	pub trait Sealed {}
	impl<T: ?Sized> Sealed for &dyn Iteratee<T> {}
	impl<T: ?Sized> Sealed for &mut dyn IterateeMut<T> {}

	#[doc(hidden)]
	pub trait DynIterateeImpl {
		type Item: Deref;

		fn as_ref(&self) -> &dyn Iteratee<<Self::Item as Deref>::Target>;
	}
	impl<'a, T: ?Sized> DynIterateeImpl for &'a dyn Iteratee<T> {
		type Item = &'a T;

		fn as_ref(&self) -> &dyn Iteratee<<Self::Item as Deref>::Target> {
			*self
		}
	}

	#[doc(hidden)]
	pub trait DynIterateeMutImpl {
		type Item: DerefMut;
		type DynIteratee: DynIteratee;

		fn into_dyn_iteratee_impl(self) -> Self::DynIteratee;

		fn as_mut(&mut self) -> &mut dyn IterateeMut<<Self::Item as Deref>::Target>;
	}
	impl<'a, T: ?Sized> DynIterateeMutImpl for &'a mut dyn IterateeMut<T> {
		type Item = &'a mut T;
		type DynIteratee = &'a dyn Iteratee<T>;

		fn into_dyn_iteratee_impl(self) -> Self::DynIteratee {
			self.as_iteratee()
		}

		fn as_mut(&mut self) -> &mut dyn IterateeMut<<Self::Item as Deref>::Target> {
			*self
		}
	}
}
use private::{DynIterateeImpl, DynIterateeMutImpl, Sealed};

pub trait DynIteratee: Sealed + DynIterateeImpl {}
impl<T: ?Sized> DynIteratee for &dyn Iteratee<T> {}
pub trait DynIterateeMut: Sealed + DynIterateeMutImpl {}
impl<T: ?Sized> DynIterateeMut for &mut dyn IterateeMut<T> {}
