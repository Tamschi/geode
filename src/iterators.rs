//! Concrete iterator types.

use crate::{DynIteratee, DynIterateeMut, Iteratee, IterateeMut};
use pretty_type_name::pretty_type_name;
use std::fmt::{self, Debug, Formatter};

/// A dynamic dispatch iterator.
#[derive(Clone)]
pub struct Iter<I: DynIteratee> {
	iteratee: I,
}

impl<I: DynIteratee> Iter<I> {
	/// Creates a new instance of [`Iter`] targeting the given [`Iteratee`].
	#[must_use]
	pub fn new(iteratee: I) -> Self {
		Self { iteratee }
	}
}

/// A mutating dynamic dispatch iterator.
///
/// Note that this type is downgradeable into a [`Clone`] [`Iter`] via [`From`]/[`Into`] conversion.
pub struct IterMut<I: DynIterateeMut> {
	iteratee: I,
}

impl<I: DynIterateeMut> IterMut<I> {
	/// Creates a new instance of [`IterMut`] targeting the given [`IterateeMut`].
	#[must_use]
	pub fn new(iteratee: I) -> Self {
		Self { iteratee }
	}

	/// Creates a new [`IterMut`] advancing independently of this one.
	#[must_use]
	pub fn fork(&mut self) -> IterMut<I> {
		IterMut::new(self.iteratee)
	}

	/// Creates an [`Iter`] advancing independently of this one.
	#[must_use]
	pub fn fork_shared(&self) -> Iter<I::DynIteratee> {
		Iter::new(self.iteratee.into_dyn_iteratee_impl())
	}
}

impl<I: DynIteratee> Iterator for Iter<I> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		let (head, rest) = self.iteratee.as_ref().head_rest();
		self.iteratee = rest;
		head
	}
}

impl<I: DynIterateeMut> Iterator for IterMut<I> {
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		let iteratee = &mut self.iteratee as *mut &mut I;
		unsafe {
			//SAFETY:
			// `IterateeMut::head_rest_mut` may not leak references if it panics, and must still be in a memory-safe state too.
			// As such, in that case, the previous mutable reference isn't borrowed anymore and can be left in place.
			//
			// Iff `head_rest_mut` returns, the target of `self.iteratee` may be borrowed through head for longer than `'_`,
			// but we can replace it with `rest` whose target is guaranteed to be disjoint (as far as pointer restrictions go).
			let (head, rest) = iteratee.read().head_rest_mut();
			iteratee.write(rest);
			head
		}
	}
}

/// Downgrades an [`IterMut`] into a ([`Clone`]) [`Iter`].
impl<I: DynIterateeMut> From<IterMut<I>> for Iter<I::DynIteratee> {
	fn from(iter_mut: IterMut<I>) -> Self {
		Self::new(iter_mut.iteratee.into_dyn_iteratee())
	}
}

impl<I: DynIteratee> Debug for Iter<I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(&pretty_type_name::<Self>())
			.finish_non_exhaustive()
	}
}

impl<I: DynIterateeMut> Debug for IterMut<I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(&pretty_type_name::<Self>())
			.finish_non_exhaustive()
	}
}
