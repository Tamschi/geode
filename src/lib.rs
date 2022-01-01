//! Zero-runtime-cost heterogeneous lists.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Fgeode)](https://iteration-square.schichler.dev/#narrow/stream/project.2Fgeode)

#![doc(html_root_url = "https://docs.rs/geode/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![no_std]

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub mod iterators;

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
	use core::ops::{Deref, DerefMut};

	use crate::{DynIteratee, Iteratee, IterateeMut};

	pub trait Sealed {}

	macro_rules! impl_sealed {
		($(
			$ty:ty
		),*$(,)?) => {$(
			impl<T: ?Sized> Sealed for $ty {}
		)*};
	}

	impl_sealed!(
		&dyn Iteratee<T>,
		&(dyn Iteratee<T> + Send),
		&(dyn Iteratee<T> + Sync),
		&(dyn Iteratee<T> + Send + Sync),
		&mut dyn IterateeMut<T>,
		&mut (dyn IterateeMut<T> + Send),
		&mut (dyn IterateeMut<T> + Sync),
		&mut (dyn IterateeMut<T> + Send + Sync),
	);

	#[doc(hidden)]
	pub trait DynIterateeImpl: Deref {
		type Item: ?Sized;

		fn as_ref(&self) -> &dyn Iteratee<Self::Item>;
	}

	macro_rules! impl_dyn_iteratee_impl {
		($(
			$ty:ty
		),*$(,)?) => {$(
			impl<'a, T: ?Sized> DynIterateeImpl for &'a $ty {
				type Item = T;

				fn as_ref(&self) -> &dyn Iteratee<Self::Item> {
					*self
				}
			}
		)*};
	}

	impl_dyn_iteratee_impl!(
		dyn Iteratee<T>,
		dyn Iteratee<T> + Send,
		dyn Iteratee<T> + Sync,
		dyn Iteratee<T> + Send + Sync,
	);

	#[doc(hidden)]
	pub trait DynIterateeMutImpl: DerefMut {
		type Item: ?Sized;
		type DynIteratee: DynIteratee;

		fn into_dyn_iteratee(self) -> Self::DynIteratee;

		fn as_ref(&self) -> &dyn IterateeMut<Self::Item>;
		fn as_mut(&mut self) -> &mut dyn IterateeMut<Self::Item>;
	}

	macro_rules! impl_dyn_iteratee_mut_impl {
		($(
			$ty:ty
		),*$(,)?) => {$(
			impl<'a, T: ?Sized> DynIterateeMutImpl for &'a mut $ty {
				type Item = T;
				type DynIteratee = &'a dyn Iteratee<T>;

				fn into_dyn_iteratee(self) -> Self::DynIteratee {
					self.as_iteratee()
				}

				fn as_ref(&self) -> &dyn IterateeMut<Self::Item> {
					*self
				}

				fn as_mut(&mut self) -> &mut dyn IterateeMut<Self::Item> {
					*self
				}
			}
		)*};
	}

	impl_dyn_iteratee_mut_impl!(
		dyn IterateeMut<T>,
		dyn IterateeMut<T> + Send,
		dyn IterateeMut<T> + Sync,
		dyn IterateeMut<T> + Send + Sync,
	);
}
use core::{convert::Infallible, mem::ManuallyDrop};

use iterators::{Iter, IterMut};
use private::{DynIterateeImpl, DynIterateeMutImpl, Sealed};

pub trait DynIteratee: Sealed + DynIterateeImpl {}
impl<T: ?Sized> DynIteratee for &dyn Iteratee<T> {}
impl<T: ?Sized> DynIteratee for &(dyn Iteratee<T> + Send) {}
impl<T: ?Sized> DynIteratee for &(dyn Iteratee<T> + Sync) {}
impl<T: ?Sized> DynIteratee for &(dyn Iteratee<T> + Send + Sync) {}
pub trait DynIterateeMut: Sealed + DynIterateeMutImpl {}
impl<T: ?Sized> DynIterateeMut for &mut dyn IterateeMut<T> {}
impl<T: ?Sized> DynIterateeMut for &mut (dyn IterateeMut<T> + Send) {}
impl<T: ?Sized> DynIterateeMut for &mut (dyn IterateeMut<T> + Sync) {}
impl<T: ?Sized> DynIterateeMut for &mut (dyn IterateeMut<T> + Send + Sync) {}

pub trait DynIter {
	type DynIteratee: DynIteratee;
	fn dyn_iter(&self) -> Iter<'_, Self::DynIteratee>;
}
pub trait DynIterMut {
	type DynIterateeMut: DynIterateeMut;
	fn dyn_iter_mut(&self) -> IterMut<'_, Self::DynIterateeMut>;
}

pub trait StaticIter<T: ?Sized> {
	fn try_for_each<E>(self, on_each: impl FnMut(T) -> Result<(), E>) -> Result<(), E>
	where
		T: Sized;
	fn try_for_each_ref<E>(&self, on_each: impl FnMut(&T) -> Result<(), E>) -> Result<(), E>;
	fn try_for_each_mut<E>(
		&mut self,
		on_each: impl FnMut(&mut T) -> Result<(), E>,
	) -> Result<(), E>;

	fn for_each(self, mut on_each: impl FnMut(T))
	where
		Self: Sized,
		T: Sized,
	{
		self.try_for_each(move |item| Ok::<_, Infallible>(on_each(item)))
			.unwrap()
	}
	fn for_each_ref(&self, mut on_each: impl FnMut(&T)) {
		self.try_for_each_ref(move |item| Ok::<_, Infallible>(on_each(item)))
			.unwrap()
	}
	fn for_each_mut(&mut self, mut on_each: impl FnMut(&mut T)) {
		self.try_for_each_mut(move |item| Ok::<_, Infallible>(on_each(item)))
			.unwrap()
	}

	fn try_fold<A, E, F: FnMut(A, T) -> Result<A, E>>(
		self,
		initial: A,
		mut on_fold: F,
	) -> Result<A, E>
	where
		Self: Sized,
		T: Sized,
	{
		let mut aggregate = ManuallyDrop::new(initial);
		let aggregate_ = &mut *aggregate as *mut A;
		self.try_for_each(move |item| unsafe {
			Ok(aggregate_.write(on_fold(aggregate_.read(), item)?))
		})?;
		Ok(ManuallyDrop::into_inner(aggregate))
	}

	fn try_fold_ref<A, E, F: FnMut(A, &T) -> Result<A, E>>(
		&self,
		initial: A,
		mut on_fold: F,
	) -> Result<A, E>
	where
		Self: Sized,
		T: Sized,
	{
		let mut aggregate = ManuallyDrop::new(initial);
		let aggregate_ = &mut *aggregate as *mut A;
		self.try_for_each_ref(move |item| unsafe {
			Ok(aggregate_.write(on_fold(aggregate_.read(), item)?))
		})?;
		Ok(ManuallyDrop::into_inner(aggregate))
	}

	fn try_fold_mut<A, E, F: FnMut(A, &mut T) -> Result<A, E>>(
		&mut self,
		initial: A,
		mut on_fold: F,
	) -> Result<A, E>
	where
		Self: Sized,
		T: Sized,
	{
		let mut aggregate = ManuallyDrop::new(initial);
		let aggregate_ = &mut *aggregate as *mut A;
		self.try_for_each_mut(move |item| unsafe {
			Ok(aggregate_.write(on_fold(aggregate_.read(), item)?))
		})?;
		Ok(ManuallyDrop::into_inner(aggregate))
	}

	fn fold<A, E, F: FnMut(A, T) -> A>(self, initial: A, mut on_fold: F) -> A
	where
		Self: Sized,
		T: Sized,
	{
		self.try_fold(initial, move |aggregate, item| {
			Ok::<_, Infallible>(on_fold(aggregate, item))
		})
		.unwrap()
	}

	fn fold_ref<A, E, F: FnMut(A, &T) -> A>(&self, initial: A, mut on_fold: F) -> A
	where
		Self: Sized,
		T: Sized,
	{
		self.try_fold_ref(initial, move |aggregate, item| {
			Ok::<_, Infallible>(on_fold(aggregate, item))
		})
		.unwrap()
	}

	fn fold_mut<A, E, F: FnMut(A, &mut T) -> A>(&mut self, initial: A, mut on_fold: F) -> A
	where
		Self: Sized,
		T: Sized,
	{
		self.try_fold_mut(initial, move |aggregate, item| {
			Ok::<_, Infallible>(on_fold(aggregate, item))
		})
		.unwrap()
	}
}

#[doc(hidden)]
pub mod __ {
	#[doc(hidden)]
	#[macro_export]
	macro_rules! custom_list_types {
		(
			$(#[$listMeta:meta])*
			$listVis:vis struct $List:ident
				$([$($generics:tt)*][$($generics0:tt)*])?
				$(where [$($constraints:tt)*][$($constraints2:tt)*])?
			{
				$listFieldVis:vis head: $itemTy:ty,
				..
			},

			$(#[$endMeta:meta])*
			$endVis:vis struct $End:ident,

			$(#[$consMeta:meta])*
			$consVis:vis trait $Cons:ident$(,
				$wrapperVis:vis struct $Wrapper:ident
			)?$(,)?
		) => {
			$(#[$listMeta])*
			$listVis struct $List<$($($generics)*,)? R>
				$(where $($constraints)*,)?
			{
				/// The first item of the list.
				$listFieldVis head: $itemTy,

				/// The rest of the list.
				$listFieldVis rest: R,
			}

			$(#[$endMeta])*
			$endVis struct End;

			$(#[$consMeta])*
			$consVis trait Cons$(<$($generics)*>)?
				$(where
					$($constraints)*,
					$($constraints2)*,
				)?
			{
				/// The result of a `cons` operation, with `T` prepended as head.
				type Cons;
				/// The results of an `r_cons` operation, with `T` appended as tail.
				type RCons;

				/// Transforms `self` by prepending `head`.
				fn cons(self, head: $itemTy) -> Self::Cons;

				/// Transforms `self` by appending `tail`.
				fn r_cons(self, tail: $itemTy) -> Self::RCons;
			}
		};
	}
	pub use custom_list_types;
}

/// Creates and implements a custom cons list.
#[macro_export]
macro_rules! custom_list {
	(
		$(#[$listMeta:meta])*
		$listVis:vis struct $List:ident
			$([$($generics:tt)*][$($generics0:tt)*])?
			$(where [$($constraints:tt)*][$($constraints2:tt)*])?
		{
			$listFieldVis:vis head: $itemTy:ty,
			..
		},

		$(#[$endMeta:meta])*
		$endVis:vis struct $End:ident,

		$(#[$consMeta:meta])*
		$consVis:vis trait $Cons:ident
		$(,)?
	) => {
		$crate::__::custom_list_types!(
			$(#[$listMeta])* $listVis struct $List$([$($generics)*][$($generics0)*])? {
				$listFieldVis head: $itemTy,
				..
			},
			$(#[$endMeta])* $endVis struct $End,
			$(#[$consMeta])* $consVis trait $Cons,
		);

		impl$(<$($generics)*>)? $Cons$(<$($generics)*>)? for $End {
			type Cons = $List<$($($generics)*,)? Self>;
			type RCons = $List<$($($generics)*,)? Self>;
			fn cons(self, head: $itemTy) -> <Self as $Cons$(<$($generics)*>)?>::Cons {
				$List {
					head,
					rest: self,
				}
			}
			fn r_cons(self, tail: $itemTy) -> <Self as $Cons$(<$($generics)*>)?>::RCons {
				$List {
					head: tail,
					rest: self,
				}
			}
  		}

		impl<
			$(
				$($generics)*,
				$($generics0)*,
			)?
			R: $Cons$(<$($generics)*>)?,
		> $Cons$(<$($generics)*>)? for $List<$($($generics0)*,)? R> {
			type Cons = $List<$($($generics)*,)? Self>;
			type RCons = $List<$($($generics0)*,)? <R as $Cons$(<$($generics)*>)?>::RCons>;
			fn cons(self, head: $itemTy) -> <Self as $Cons$(<$($generics)*>)?>::Cons {
				$List {
					head,
					rest: self,
				}
			}
			fn r_cons(self, tail: $itemTy) -> <Self as $Cons$(<$($generics)*>)?>::RCons {
				$List {
					head: self.head,
					rest: <R as $Cons$(<$($generics)*>)?>::r_cons(self.rest, tail),
				}
			}
		}

		impl<X: ?Sized> $crate::StaticIter<X> for $End {
			fn try_for_each<E>(
				self,
				_: impl ::core::ops::FnMut(X) -> ::core::result::Result<(), E>,
			) -> ::core::result::Result<(), E> where X: Sized {
				Ok(())
			}
			fn try_for_each_ref<E>(
				&self,
				_: impl ::core::ops::FnMut(&X) -> ::core::result::Result<(), E>,
			) -> ::core::result::Result<(), E> {
				Ok(())
			}
			fn try_for_each_mut<E>(
				&mut self,
				_: impl ::core::ops::FnMut(&mut X) -> ::core::result::Result<(), E>,
			) -> ::core::result::Result<(), E> {
				Ok(())
			}
  		}
	};

	// (
	// 	$(#[$listMeta:meta])*
	// 	$listVis:vis $List:ident $({$listFieldVis:vis})?,

	// 	$(#[$endMeta:meta])*
	// 	$endVis:vis $End:ident,

	// 	$(#[$consMeta:meta])*
	// 	$consVis:vis $Cons:ident,

	// 	$wrapperVis:vis $Wrapper:ident
	// 	$(,)?
	// ) => {
	// 	compile_error!("TODO: Cons list with wrapper.")
	// };
}

custom_list!(
	/// A basic cons list with public implementation details.
	///
	/// Use [`Cons`] methods on [`End`] to start constructing a [`List`].
	pub struct List[T][T0] {
		pub head: T,
		..
	},

	/// An empty rest for [`List`].
	pub struct End,

	/// Builder functionality for [`List`] and [`End`].
	pub trait Cons,
);
