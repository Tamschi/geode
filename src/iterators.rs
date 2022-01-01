//! Concrete iterator types.

use crate::{DynIteratee, DynIterateeMut, Iteratee, IterateeMut};
use core::{
	fmt::{self, Debug, Formatter},
	marker::PhantomData,
	mem,
};
use pretty_type_name::pretty_type_name;

/// A dynamic dispatch iterator.
#[derive(Clone)]
pub struct Iter<'a, I: DynIteratee>
where
	I::Item: 'a,
{
	iteratee: I,
	_phantom: PhantomData<&'a ()>,
}

impl<I: DynIteratee> Iter<'_, I> {
	/// Creates a new instance of [`Iter`] targeting the given [`Iteratee`].
	#[must_use]
	pub fn new(iteratee: I) -> Self {
		Self {
			iteratee,
			_phantom: PhantomData,
		}
	}
}

/// A mutating dynamic dispatch iterator.
///
/// Note that this type is downgradeable into a [`Clone`] [`Iter`] via [`From`]/[`Into`] conversion.
pub struct IterMut<'a, I: DynIterateeMut>
where
	I::Item: 'a,
{
	iteratee: I,
	_phantom: PhantomData<&'a ()>,
}

impl<I: DynIterateeMut> IterMut<'_, I> {
	/// Creates a new instance of [`IterMut`] targeting the given [`IterateeMut`].
	#[must_use]
	pub fn new(iteratee: I) -> Self {
		Self {
			iteratee,
			_phantom: PhantomData,
		}
	}

	/// Creates a new [`IterMut`] advancing independently of this one.
	#[must_use]
	pub fn fork(&mut self) -> IterMut<'_, I> {
		IterMut::new(unsafe { (&mut self.iteratee as *mut I).read() })
	}

	/// Creates an [`Iter`] advancing independently of this one.
	#[must_use]
	pub fn fork_shared(&self) -> Iter<I::DynIteratee> {
		Iter::new(unsafe {
			(&self.iteratee.as_ref().as_iteratee() as *const &dyn Iteratee<I::Item>)
				.cast::<I::DynIteratee>()
				.read()
		})
	}
}

impl<'a, I: DynIteratee> Iterator for Iter<'a, I> {
	type Item = &'a I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		let (head, rest) = self.iteratee.as_ref().head_rest();
		unsafe {
			let head = mem::transmute(head);
			self.iteratee = (&rest as *const &dyn Iteratee<I::Item>).cast::<I>().read();
			head
		}
	}
}

impl<'a, I: DynIterateeMut> Iterator for IterMut<'a, I> {
	type Item = &'a mut I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		let iteratee = &mut self.iteratee as *mut I;
		unsafe {
			//SAFETY:
			// `IterateeMut::head_rest_mut` may not leak references if it panics, and must still be in a memory-safe state too.
			// As such, in that case, the previous mutable reference isn't borrowed anymore and can be left in place.
			//
			// Iff `head_rest_mut` returns, the target of `self.iteratee` may be borrowed through head for longer than `'_`,
			// but we can replace it with `rest` whose target is guaranteed to be disjoint (as far as pointer restrictions go).
			//
			// TODO: Safety notes on the transmutes below.
			let mut iteratee_ = iteratee.read();
			let (head, mut rest) = iteratee_.as_mut().head_rest_mut();
			let head = mem::transmute(head);
			iteratee.write(
				(&mut rest as *mut &mut dyn IterateeMut<I::Item>)
					.cast::<I>()
					.read(),
			);
			head
		}
	}
}

/// Downgrades an [`IterMut`] into a ([`Clone`]) [`Iter`].
impl<'a, I: DynIterateeMut> From<IterMut<'a, I>> for Iter<'a, I::DynIteratee> {
	fn from(iter_mut: IterMut<I>) -> Self {
		Self::new(iter_mut.iteratee.into_dyn_iteratee())
	}
}

impl<I: DynIteratee> Debug for Iter<'_, I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(&pretty_type_name::<Self>())
			.finish_non_exhaustive()
	}
}

impl<I: DynIterateeMut> Debug for IterMut<'_, I> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct(&pretty_type_name::<Self>())
			.finish_non_exhaustive()
	}
}
