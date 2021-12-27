mod private {
	#![allow(clippy::expl_impl_clone_on_copy)]

	use std::marker::PhantomData; // Leaks from `#[phantom]`.

	pub struct Ghost<T: ?Sized>(pub PhantomData<T>);

	pub trait Sealed {}
	impl<T: ?Sized> Sealed for Ghost<T> {}
}

use std::marker::PhantomData;

use private::Sealed;

/// A thread safety marker.
pub trait ThreadSafety: Sealed {}

/// A phantom that's neither [`Send`] nor [`Sync`].
pub type Bound = private::Ghost<*mut ()>;
#[doc(hidden)]
#[allow(non_upper_case_globals)]
pub const Bound: Bound = private::Ghost(PhantomData);

/// A phantom that's both [`Send`] and [`Sync`].
pub type Safe = private::Ghost<dyn Send + Sync>;
#[doc(hidden)]
#[allow(non_upper_case_globals)]
pub const Safe: Safe = private::Ghost(PhantomData);

/// A phantom that's neither [`Send`] nor [`Sync`].
pub type Sendable = private::Ghost<dyn Send>;
#[doc(hidden)]
#[allow(non_upper_case_globals)]
pub const Sendable: Sendable = private::Ghost(PhantomData);

/// A phantom that's both [`Send`] and [`Sync`].
pub type Sharable = private::Ghost<dyn Sync>;
#[doc(hidden)]
#[allow(non_upper_case_globals)]
pub const Sharable: Sharable = private::Ghost(PhantomData);

impl ThreadSafety for Bound {}
impl ThreadSafety for Safe {}
impl ThreadSafety for Sendable {}
impl ThreadSafety for Sharable {}
