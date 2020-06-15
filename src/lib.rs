//! This crates helps creating flexible libraries that may work in either
//! multithreaded and singlethreaded environments.
//!
//! [![crates](https://img.shields.io/crates/v/maybe-sync.svg?label=maybe-sync)](https://crates.io/crates/maybe-sync)
//! [![docs](https://docs.rs/maybe-sync/badge.svg)](https://docs.rs/maybe-sync)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
//! [![License](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE-APACHE)
//!
//! # Why it exists?
//!
//! Creating library that works in either desktop/server or web-browser environment
//! could be tough.
//!
//! In desktop/server environments crates tend to be usable with multiple threads
//! and add [`Send`] and [`Sync`] trait bounds where needed.
//! In the same time [`web-sys`] - a de facto standard crate to use browser API -
//! provides programmer with `!Send + !Sync` types that cannot be used when [`Send`]
//! or [`Sync`] bounds are placed on the API calls.
//!
//! For example asset loading crate [`goods`] uses generic executors to drive
//! loading process.
//! Some executors work with sendable futures only: [`tokio`],
//! some requre sendable future to run them on multiple threads:
//! [`async-std`], [`actix-rt`],
//! Others work with any futures in single thread: [`wasm_bindgen_futures::spawn_local`]
//!
//! In [`goods`] one of the basic data sources for web - [`FetchSource`] - uses
//! browser's Fetch API to retreive data from web and produces non-sendable future.
//!
//! That's why [`goods::Spawn::spawn`] takes [`maybe_sync::BoxFuture`] that is
//! a sendable boxed future when "sync" feature is enabled,
//! allowing using multithreaded future executors.
//! And without "sync" feature [`maybe_sync::BoxFuture`] is
//! a non-sendable boxed future and only then [`FetchSource`] exists.
//!
//! Similar story with ECS crates.
//! Most of them require that components are [`Send`] + [`Sync`]
//! so systems can access them from any thread.
//! This makes it impossible to have types from [`web-sys`] in components.
//!
//! Some small application may desire to turn off "sync" feature in their
//! dependencies as they are singlethreaded and would rather not pay for what
//! they don't use.
//!
//! # [`MaybeSend`] and [`MaybeSync`]
//!
//! Marker traits [`MaybeSend`] and [`MaybeSync`] can be used in place of
//! [`Send`] and [`Sync`] traits in where clauses and bounds.
//!
//! When "sync" feature is enabled then [`MaybeSend`] is actually reexported [`Send`]
//! and [`MaybeSync`] is reexported [`Sync`] trait.
//! Thus allowing types with those bounds to be sent or shared across threads.
//!
//! When "sync" feature is not enabled then [`MaybeSend`] and [`MaybeSync`] are empty
//! traits implemented for all types and when used as bounds permit types that
//! does not satisfy [`Send`] or [`Sync`] bounds.
//!
//! This makes it impossible to forget enable "sync" feature and accidentally send
//! `!Send` value to another thread.
//! Function that uncodintionally send value to another thread
//! should not use [`MaybeSend`] bound, but an actual [`Send`].
//!
//! # BoxFuture
//!
//! Type alias for boxed future. Sendable if "sync" feature is enabled.
//! It is designed to be used as return type from trait functions
//! where trait implementations that produce non-sendable futures
//! exist only when "sync" feature is not enabled.
//! It can be used as function argument type when [`MaybeSend`] bound is placed.
//!
//! # Rc
//!
//! Type alias to [`alloc::rc::Rc`] when "sync" feature is not enabled, or
//! [`alloc::sync::Arc`] when "sync" feature is enabled. Serves for optimization
//! purposes for crates that already use [`maybe-sync`] crate.
//!
//! # Mutex
//!
//! Type alias to [`parking_lot::Mutex`] when "sync" feature is enabled, or
//! thin wrapper arond [`core::cell::RefCell`] otherwise. Serves for optimization
//! purposes for crates that already use [`maybe-sync`] crate.
//!
//! [`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
//! [`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
//! [`web-sys`]: https://docs.rs/web-sys
//! [`goods`]: https://docs.rs/goods
//! [`tokio`]: https://docs.rs/tokio
//! [`async-std`]: https://docs.rs/async-std
//! [`actix-rt`]: https://docs.rs/actix-rt
//! [`FetchSource`]: https://docs.rs/goods/0.5/wasm32-unknown-unknown/goods/struct.FetchSource.html
//! [`wasm_bindgen_futures::spawn_local`]: https://docs.rs/wasm-bindgen-futures/0.4/wasm_bindgen_futures/fn.spawn_local.html
//! [`goods::Spawn::spawn`]: https://docs.rs/goods/0.5/goods/trait.Spawn.html#tymethod.spawn
//! [`maybe_sync::BoxFuture`]: ./type.BoxFuture.html
//! [`MaybeSend`]: ./trait.MaybeSend.html
//! [`MaybeSync`]: ./trait.MaybeSync.html
//! [`alloc::rc::Rc`]: https://doc.rust-lang.org/alloc/rc/struct.Rc.html
//! [`alloc::sync::Arc`]: https://doc.rust-lang.org/alloc/sync/struct.Arc.html
//! [`maybe-sync`]: ./index.html
//! [`parking_lot::Mutex`]: https://docs.rs/parking_lot/0.10/parking_lot/type.Mutex.html
//! [`core::cell::RefCell`]: https://doc.rust-lang.org/core/cell/struct.RefCell.html

#![no_std]
#![cfg_attr(all(doc, feature = "unstable-doc"), feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "sync")]
mod sync {
    #[cfg(feature = "alloc")]
    use core::{future::Future, pin::Pin};

    /// Reexports of the actual marker traits from core.
    pub use core::marker::{Send as MaybeSend, Sync as MaybeSync};

    /// An owned dynamically typed [`Future`] for use at return position in cases
    /// when type is opaque and existential type cannot be used,
    /// or when multiple types can be returned.
    ///
    /// A type alias equal to `futures::future::BoxFuture`
    /// when "sync" feature is enabled.\
    /// A type alias equal to `futures::future::LocalBoxFuture`
    /// when "sync" feature is not enabled.
    #[cfg(feature = "alloc")]
    #[cfg_attr(all(doc, feature = "unstable-doc"), doc(cfg(feature = "alloc")))]
    pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>;

    /// A pointer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A pointer type which can be shared, but only within single thread
    /// where it was created when "sync" feature is not enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use {maybe_sync::{MaybeSend, Rc}, std::fmt::Debug};
    ///
    /// fn maybe_sends<T: MaybeSend + Debug + 'static>(val: T) {
    ///   #[cfg(feature = "sync")]
    ///   {
    ///     // If this code is compiled then `MaybeSend` is alias to `std::marker::Send`.
    ///     std::thread::spawn(move || { println!("{:?}", val) });
    ///   }
    /// }
    ///
    /// // Unlike `std::rc::Rc` this `maybe_sync::Rc` always satisfies `MaybeSend` bound.
    /// maybe_sends(Rc::new(42));
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(all(doc, feature = "unstable-doc"), doc(cfg(feature = "alloc")))]
    pub type Rc<T> = alloc::sync::Arc<T>;

    /// Mutex implementation to use in conjunction with `MaybeSync` bound.
    ///
    /// A type alias to `parking_lot::Mutex` when "sync" feature is enabled.\
    /// A wrapper type around `std::cell::RefCell` when "sync" feature is not enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use {maybe_sync::{MaybeSend, Mutex}, std::{fmt::Debug, sync::Arc}};
    ///
    /// fn maybe_sends<T: MaybeSend + Debug + 'static>(val: Arc<Mutex<T>>) {
    ///   #[cfg(feature = "sync")]
    ///   {
    ///     // If this code is compiled then `MaybeSend` is alias to `std::marker::Send`,
    ///     // and `Mutex` is `parking_lot::Mutex`.
    ///     std::thread::spawn(move || { println!("{:?}", *val.lock()) });
    ///   }
    /// }
    ///
    /// // `maybe_sync::Mutex<T>` would always satisfy `MaybeSync` and `MaybeSend`
    /// // bounds when `T: MaybeSend`,
    /// // even if feature "sync" is enabeld.
    /// maybe_sends(Arc::new(Mutex::new(42)));
    /// ```
    pub type Mutex<T> = parking_lot::Mutex<T>;

    /// A boolean type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A boolean type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a bool.
    pub type AtomicBool = core::sync::atomic::AtomicBool;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i8.
    pub type AtomicI8 = core::sync::atomic::AtomicI8;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i16.
    pub type AtomicI16 = core::sync::atomic::AtomicI16;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i32.
    pub type AtomicI32 = core::sync::atomic::AtomicI32;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a isize.
    pub type AtomicIsize = core::sync::atomic::AtomicIsize;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i8.
    pub type AtomicU8 = core::sync::atomic::AtomicU8;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i16.
    pub type AtomicU16 = core::sync::atomic::AtomicU16;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i32.
    pub type AtomicU32 = core::sync::atomic::AtomicU32;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a isize.
    pub type AtomicUsize = core::sync::atomic::AtomicUsize;

    /// A raw pointer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A raw pointer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a isize.
    pub type AtomicPtr<T> = core::sync::atomic::AtomicPtr<T>;
}

#[cfg(not(feature = "sync"))]
mod unsync {
    use core::cell::{RefCell, RefMut};

    #[cfg(feature = "alloc")]
    use core::{future::Future, pin::Pin};

    /// Marker trait that can represent nothing if feature "sync" is not enabled.
    /// Or be reexport of `std::marker::Send` if "sync" feature is enabled.
    ///
    /// It is intended to be used as trait bound where `std::marker::Send` bound
    /// is required only when application is compiled for multithreaded environment.\
    /// If "sync" feature is not enabled then this trait bound will *NOT* allow
    /// value to cross thread boundary or be used where sendable value is expected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use {maybe_sync::MaybeSend, std::{fmt::Debug, rc::Rc}};
    ///
    /// fn maybe_sends<T: MaybeSend + Debug + 'static>(val: T) {
    ///   #[cfg(feature = "sync")]
    ///   {
    ///     // If this code is compiled then `MaybeSend` is alias to `std::marker::Send`.
    ///     std::thread::spawn(move || { println!("{:?}", val) });
    ///   }
    /// }
    ///
    /// #[cfg(not(feature = "sync"))]
    /// {
    ///   // If this code is compiled then `MaybeSend` dummy markerd implemented for all types.
    ///   maybe_sends(Rc::new(42));
    /// }
    /// ```
    pub trait MaybeSend {}

    /// All values are maybe sendable.
    impl<T> MaybeSend for T where T: ?Sized {}

    /// Marker trait that can represent nothing if feature "sync" is not enabled.
    /// Or be reexport of `std::marker::Sync` if "sync" feature is enabled.
    ///
    /// It is intended to be used as trait bound where `std::marker::Sync` bound
    /// is required only when application is compiled for multithreaded environment.\
    /// If "sync" feature is not enabled then this trait bound will *NOT* allow
    /// reference to the value to cross thread boundary or be used where sync
    /// value is expected.
    ///
    /// # Examples
    ///
    /// ```
    /// # use {maybe_sync::MaybeSync, std::{sync::Arc, fmt::Debug, cell::Cell}};
    ///
    /// fn maybe_shares<T: MaybeSync + Debug + 'static>(val: Arc<T>) {
    ///   #[cfg(feature = "sync")]
    ///   {
    ///     // If this code is compiled then `MaybeSync` is alias to `std::marker::Sync`.
    ///     std::thread::spawn(move || { println!("{:?}", val) });
    ///   }
    /// }
    ///
    /// #[cfg(not(feature = "sync"))]
    /// {
    ///   // If this code is compiled then `MaybeSync` dummy markerd implemented for all types.
    ///   maybe_shares(Arc::new(Cell::new(42)));
    /// }
    /// ```
    pub trait MaybeSync {}

    /// All values are maybe sync.
    impl<T> MaybeSync for T where T: ?Sized {}

    /// An owned dynamically typed [`Future`] for use at return position in cases
    /// when type is opaque and existential type cannot be used,
    /// or when multiple types can be returned.
    ///
    /// A type alias equal to `futures::future::BoxFuture`
    /// when "sync" feature is enabled.\
    /// A type alias equal to `futures::future::LocalBoxFuture`
    /// when "sync" feature is not enabled.
    #[cfg(feature = "alloc")]
    #[cfg_attr(all(doc, feature = "unstable-doc"), doc(cfg(feature = "alloc")))]
    pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + 'a>>;

    /// A pointer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A pointer type which can be shared, but only within single thread
    /// where it was created when "sync" feature is not enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use {maybe_sync::{MaybeSend, Rc}, std::fmt::Debug};
    ///
    /// fn maybe_sends<T: MaybeSend + Debug + 'static>(val: T) {
    ///   #[cfg(feature = "sync")]
    ///   {
    ///     // If this code is compiled then `MaybeSend` is alias to `std::marker::Send`.
    ///     std::thread::spawn(move || { println!("{:?}", val) });
    ///   }
    /// }
    ///
    /// // Unlike `std::rc::Rc` this `maybe_sync::Rc<T>` would always
    /// // satisfy `MaybeSend` bound when `T: MaybeSend + MaybeSync`,
    /// // even if feature "sync" is enabeld.
    /// maybe_sends(Rc::new(42));
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(all(doc, feature = "unstable-doc"), doc(cfg(feature = "alloc")))]
    pub type Rc<T> = alloc::rc::Rc<T>;

    /// Mutex implementation to use in conjunction with `MaybeSync` bound.
    ///
    /// A type alias to `parking_lot::Mutex` when "sync" feature is enabled.\
    /// A wrapper type around `std::cell::RefCell` when "sync" feature is not enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use {maybe_sync::{MaybeSend, Mutex}, std::{fmt::Debug, sync::Arc}};
    ///
    /// fn maybe_sends<T: MaybeSend + Debug + 'static>(val: Arc<Mutex<T>>) {
    ///   #[cfg(feature = "sync")]
    ///   {
    ///     // If this code is compiled then `MaybeSend` is alias to `std::marker::Send`,
    ///     // and `Mutex` is `parking_lot::Mutex`.
    ///     std::thread::spawn(move || { println!("{:?}", *val.lock()) });
    ///   }
    /// }
    ///
    /// // `maybe_sync::Mutex<T>` would always satisfy `MaybeSync` and `MaybeSend`
    /// // bounds when `T: MaybeSend`,
    /// // even if feature "sync" is enabeld.
    /// maybe_sends(Arc::new(Mutex::new(42)));
    /// ```
    #[repr(transparent)]
    #[derive(Debug, Default)]
    pub struct Mutex<T: ?Sized> {
        cell: RefCell<T>,
    }

    impl<T> Mutex<T> {
        /// Creates a new mutex in an unlocked state ready for use.
        pub fn new(value: T) -> Self {
            Mutex {
                cell: RefCell::new(value),
            }
        }
    }

    impl<T> Mutex<T>
    where
        T: ?Sized,
    {
        /// Acquires a mutex, blocking the current thread until it is able to do so.\
        /// This function will block the local thread until it is available to acquire the mutex.\
        /// Upon returning, the thread is the only thread with the mutex held.\
        /// An RAII guard is returned to allow scoped unlock of the lock.\
        /// When the guard goes out of scope, the mutex will be unlocked.\
        /// Attempts to lock a mutex in the thread which already holds the lock will result in a deadlock.
        pub fn lock(&self) -> RefMut<T> {
            self.cell.borrow_mut()
        }

        /// Attempts to acquire this lock.\
        /// If the lock could not be acquired at this time, then `None` is returned.\
        /// Otherwise, an RAII guard is returned.\
        /// The lock will be unlocked when the guard is dropped.\
        /// This function does not block.
        pub fn try_lock(&self) -> Option<RefMut<T>> {
            self.cell.try_borrow_mut().ok()
        }

        /// Returns a mutable reference to the underlying data.\
        /// Since this call borrows the `Mutex` mutably,\
        /// no actual locking needs to take place -
        /// the mutable borrow statically guarantees no locks exist.
        pub fn get_mut(&mut self) -> &mut T {
            self.cell.get_mut()
        }
    }

    /// A boolean type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A boolean type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a bool.
    pub type AtomicBool = core::cell::Cell<bool>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i8.
    pub type AtomicI8 = core::cell::Cell<i8>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i16.
    pub type AtomicI16 = core::cell::Cell<i16>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i32.
    pub type AtomicI32 = core::cell::Cell<i32>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a isize.
    pub type AtomicIsize = core::cell::Cell<isize>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i8.
    pub type AtomicU8 = core::cell::Cell<u8>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i16.
    pub type AtomicU16 = core::cell::Cell<u16>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a i32.
    pub type AtomicU32 = core::cell::Cell<u32>;

    /// A integer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A integer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a isize.
    pub type AtomicUsize = core::cell::Cell<usize>;

    /// A raw pointer type which can be safely shared between threads
    /// when "sync" feature is enabled.\
    /// A raw pointer type with non-threadsafe interior mutability
    /// when "sync" feature is not enabled.
    ///
    /// This type has the same in-memory representation as a isize.
    pub type AtomicPtr<T> = core::cell::Cell<*mut T>;
}

#[cfg(feature = "sync")]
pub use sync::*;

#[cfg(not(feature = "sync"))]
pub use unsync::*;

/// Expands to `dyn $traits` with `Send` marker trait
/// added when "sync" feature is enabled.
///
/// Expands to `dyn $traits` without `Send` marker trait
/// added "sync" feature is not enabled.
///
/// # Example
/// ```
/// # use maybe_sync::{MaybeSend, dyn_maybe_send};
/// fn foo<T: MaybeSend>(_: T) {}
/// // `x` will implement `MaybeSend` whether "sync" feature is enabled or not.
/// let x: Box<dyn_maybe_send!(std::future::Future<Output = u32>)> = Box::new(async move { 42 });
/// foo(x);
/// ```
#[cfg(feature = "sync")]
#[macro_export]
macro_rules! dyn_maybe_send {
    ($($traits:tt)+) => {
        dyn $($traits)+ + Send
    };
}

/// Expands to `dyn $traits` with `Send` marker trait
/// added when "sync" feature is enabled.
///
/// Expands to `dyn $traits` without `Send` marker trait
/// added "sync" feature is not enabled.
///
/// # Example
/// ```
/// # use maybe_sync::{MaybeSend, dyn_maybe_send};
/// fn foo<T: MaybeSend>(_: T) {}
/// // `x` will implement `MaybeSend` whether "sync" feature is enabled or not.
/// let x: Box<dyn_maybe_send!(std::future::Future<Output = u32>)> = Box::new(async move { 42 });
/// foo(x);
/// ```
#[cfg(not(feature = "sync"))]
#[macro_export]
macro_rules! dyn_maybe_send {
    ($($traits:tt)+) => {
        dyn $($traits)+
    };
}

/// Expands to `dyn $traits` with `Sync` marker trait
/// added when "sync" feature is enabled.
///
/// Expands to `dyn $traits` without `Sync` marker trait
/// added "sync" feature is not enabled.
///
/// # Example
/// ```
/// # use maybe_sync::{MaybeSync, dyn_maybe_sync};
/// fn foo<T: MaybeSync + ?Sized>(_: &T) {}
///
/// let x: &dyn_maybe_sync!(AsRef<str>) = &"qwerty";
/// // `x` will implement `MaybeSync` whether "sync" feature is enabled or not.
/// foo(x);
/// ```
#[cfg(feature = "sync")]
#[macro_export]
macro_rules! dyn_maybe_sync {
    ($($traits:tt)+) => {
        dyn $($traits)+ + Sync
    };
}

/// Expands to `dyn $traits` with `Sync` marker trait
/// added when "sync" feature is enabled.
///
/// Expands to `dyn $traits` without `Sync` marker trait
/// added "sync" feature is not enabled.
///
/// # Example
/// ```
/// # use maybe_sync::{MaybeSync, dyn_maybe_sync};
/// fn foo<T: MaybeSync + ?Sized>(_: &T) {}
/// // `x` will implement `MaybeSync` whether "sync" feature is enabled or not.
/// let x: &dyn_maybe_sync!(AsRef<str>) = &"qwerty";
/// foo(x);
/// ```
#[cfg(not(feature = "sync"))]
#[macro_export]
macro_rules! dyn_maybe_sync {
    ($($traits:tt)+) => {
        dyn $($traits)+
    };
}

/// Expands to `dyn $traits` with `Send` and `Sync` marker trait
/// added when "sync" feature is enabled.
///
/// Expands to `dyn $traits` without `Send` and `Sync` marker trait
/// added "sync" feature is not enabled.
///
/// # Example
/// ```
/// # use maybe_sync::{MaybeSend, MaybeSync, dyn_maybe_send_sync};
/// fn foo<T: MaybeSend + MaybeSync + ?Sized>(_: &T) {}
/// // `x` will implement `MaybeSend` and `MaybeSync` whether "sync" feature is enabled or not.
/// let x: &dyn_maybe_send_sync!(AsRef<str>) = &"qwerty";
/// foo(x);
/// ```
#[cfg(feature = "sync")]
#[macro_export]
macro_rules! dyn_maybe_send_sync {
    ($($traits:tt)+) => {
        dyn $($traits)+ + Send + Sync
    };
}

/// Expands to `dyn $traits` with `Sync` marker trait
/// added when "sync" feature is enabled.
///
/// Expands to `dyn $traits` without `Sync` marker trait
/// added "sync" feature is not enabled.
///
/// # Example
/// ```
/// # use maybe_sync::{MaybeSend, MaybeSync, dyn_maybe_send_sync};
/// fn foo<T: MaybeSend + MaybeSync + ?Sized>(_: &T) {}
/// // `x` will implement `MaybeSend` and `MaybeSync` whether "sync" feature is enabled or not.
/// let x: &dyn_maybe_send_sync!(AsRef<str>) = &"qwerty";
/// foo(x);
/// ```
#[cfg(not(feature = "sync"))]
#[macro_export]
macro_rules! dyn_maybe_send_sync {
    ($($traits:tt)+) => {
        dyn $($traits)+
    };
}
