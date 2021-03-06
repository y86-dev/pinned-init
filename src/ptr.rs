//! Module providing a special pointer trait used to transfer owned data and to
//! allow safer transmuation of data without forgetting to change other pointer
//! types.
//!
//! Pointer types need to implement the trait in this module, if they want to
//! support this library.
//!
//! The type system is used to enforce as much as possible, but implementors
//! still need to pay attention, that their type can implemen [`OwnedUniquePtr<T>`].

use crate::transmute::TransmuteInto;
use core::{ops::DerefMut, pin::Pin};

// used to dissallow other crates implementing TypesEq.
mod sealed {
    pub struct Sealed;
}

#[doc(hidden)]
pub trait TypesEq<T: ?Sized> {
    fn __no_impls_outside_this_crate(_: sealed::Sealed);
}

impl<T: ?Sized> TypesEq<T> for T {
    fn __no_impls_outside_this_crate(_: sealed::Sealed) {}
}

/// A (smart) unique pointer which owns its data (e.g. [`alloc::boxed::Box`]).
/// This pointer provides access to T via [`DerefMut`].
///
/// Transmuting the pointee is also supported, if it implements
/// [`TransmuteInto<U>`] for some `U`.
///
/// # Safety
///
/// All types implementing this trait need to
/// - own the data they point to.
/// - be the only way to access the data behind this pointer.
/// - provide the same pointer type as `Self` with only a different pointee via the
/// [`Self::Ptr`] associated type.
pub unsafe trait OwnedUniquePtr<T: ?Sized>: DerefMut<Target = T> + Sized
where
    Self: TypesEq<Self::Ptr<T>>,
{
    /// Access the same underlying pointer type with a different pointee type.
    /// `Self == Self::Ptr<T>`
    type Ptr<U: ?Sized>: DerefMut<Target = U>;

    /// Transmute the type behind this pointer while being pinned.
    ///
    /// # Safety
    ///
    /// This function does not
    /// - move the pointee.
    /// - mutate the pointee.
    /// The caller needs to guarantee, that it is safe to transmute `T` to `U` (or
    /// equivalently, that it is safe to call [`TransmuteInto::transmute_ptr`]).
    unsafe fn transmute_pointee_pinned<U>(this: Pin<Self>) -> Pin<Self::Ptr<U>>
    where
        T: TransmuteInto<U>;
}

#[cfg(feature = "alloc")]
unsafe impl<T: ?Sized> OwnedUniquePtr<T> for alloc::boxed::Box<T> {
    type Ptr<U: ?Sized> = alloc::boxed::Box<U>;

    #[inline]
    unsafe fn transmute_pointee_pinned<U>(this: Pin<Self>) -> Pin<Self::Ptr<U>>
    where
        T: TransmuteInto<U>,
    {
        #[cfg(not(feature = "std"))]
        use alloc::boxed::Box;
        unsafe {
            // SAFETY: we later repin the pointer and never move the data behind it.
            let this = Pin::into_inner_unchecked(this);
            // this is safe, due to the requriements of this function
            let this: Box<U> = Box::from_raw(Box::into_raw(this) as *mut U);
            Pin::new_unchecked(this)
        }
    }
}
