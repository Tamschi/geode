//! Concrete iterator types.

use std::{
	fmt::{self, Debug, Formatter},
	marker::PhantomData,
	panic::{RefUnwindSafe, UnwindSafe},
};

use pretty_type_name::pretty_type_name;

use crate::{Iteratee, IterateeMut};

/// Similar to [`PhantomData`] but implements all markers unconditionally.
///
/// We don't have to care about them on `T` in [`Iter`] and [`IterMut`] because relevant constraints are already inherited through `I`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PhantomUse<T: ?Sized>(PhantomData<T>);
unsafe impl<T: ?Sized> Send for PhantomUse<T> {}
unsafe impl<T: ?Sized> Sync for PhantomUse<T> {}
impl<T: ?Sized> UnwindSafe for PhantomUse<T> {}
impl<T: ?Sized> RefUnwindSafe for PhantomUse<T> {}
impl<T: ?Sized> Unpin for PhantomUse<T> {}
impl<T: ?Sized> PhantomUse<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

/// A dynamic dispatch iterator.
#[derive(Clone)]
pub struct Iter<'a, T: ?Sized, I: ?Sized + Iteratee<T>> {
	iteratee: &'a I,
	_phantom: PhantomUse<&'a T>,
}

impl<'a, T: ?Sized, I: ?Sized + Iteratee<T>> Iter<'a, T, I> {
	/// Creates a new instance of [`Iter`] targeting the given [`Iteratee`].
	#[must_use]
	pub fn new(iteratee: &'a I) -> Self {
		Self {
			iteratee,
			_phantom: PhantomUse::new(),
		}
	}
}

/// A mutating dynamic dispatch iterator.
///
/// Note that this type is downgradeable into a [`Clone`] [`Iter`] via [`From`]/[`Into`] conversion.
pub struct IterMut<'a, T: ?Sized, I: ?Sized + IterateeMut<T>> {
	iteratee: &'a mut I,
	_phantom: PhantomUse<&'a T>,
}

impl<'a, T: ?Sized, I: ?Sized + IterateeMut<T>> IterMut<'a, T, I> {
	/// Creates a new instance of [`IterMut`] targeting the given [`IterateeMut`].
	#[must_use]
	pub fn new(iteratee: &'a mut I) -> Self {
		Self {
			iteratee,
			_phantom: PhantomUse::new(),
		}
	}

	/// Creates a new [`IterMut`] advancing independently of this one.
	#[must_use]
	pub fn fork(&mut self) -> IterMut<'_, T, I> {
		IterMut::new(self.iteratee)
	}

	/// Creates an [`Iter`] advancing independently of this one.
	#[must_use]
	pub fn fork_shared(&self) -> Iter<'_, T, I> {
		Iter::new(self.iteratee.as_iteratee())
	}
}

impl<'a, T: ?Sized, I: ?Sized + Iteratee<T>> Iterator for Iter<'a, T, I> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		let (head, rest) = self.iteratee.head_rest();
		self.iteratee = rest;
		head
	}
}

impl<'a, T: ?Sized, I: ?Sized + IterateeMut<T>> Iterator for IterMut<'a, T, I> {
	type Item = &'a mut T;

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
impl<'a, T: ?Sized, I: ?Sized + IterateeMut<T>> From<IterMut<'a, T, I>> for Iter<'a, T, I> {
	fn from(iter_mut: IterMut<'a, T, I>) -> Self {
		Self::new(iter_mut.iteratee.as_iteratee())
	}
}

impl<'a, T: ?Sized, I: ?Sized + Iteratee<T>> Debug for Iter<'a, T, I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(&pretty_type_name::<Self>())
			.finish_non_exhaustive()
	}
}

impl<'a, T: ?Sized, I: ?Sized + IterateeMut<T>> Debug for IterMut<'a, T, I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(&pretty_type_name::<Self>())
			.finish_non_exhaustive()
	}
}
