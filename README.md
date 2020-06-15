# maybe-sync

This crates helps creating flexible libraries that may work in either
multithreaded and singlethreaded environments.

[![crates](https://img.shields.io/crates/v/maybe-sync.svg?label=maybe-sync)](https://crates.io/crates/maybe-sync)
[![docs](https://docs.rs/maybe-sync/badge.svg)](https://docs.rs/maybe-sync)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE-APACHE)

## Why it exists?

Creating library that works in either desktop/server or web-browser environment
could be tough.

In desktop/server environments crates tend to be usable with multiple threads
and add [`Send`] and [`Sync`] trait bounds where needed.
In the same time [`web-sys`] - a de facto standard crate to use browser API -
provides programmer with `!Send + !Sync` types that cannot be used when [`Send`]
or [`Sync`] bounds are placed on the API calls.

For example asset loading crate [`goods`] uses generic executors to drive
loading process.
Some executors work with sendable futures only: [`tokio`],
some requre sendable future to run them on multiple threads:
[`async-std`], [`actix-rt`],
Others work with any futures in single thread: [`wasm_bindgen_futures::spawn_local`]

In [`goods`] one of the basic data sources for web - [`FetchSource`] - uses
browser's Fetch API to retreive data from web and produces non-sendable future.

That's why [`goods::Spawn::spawn`] takes [`maybe_sync::BoxFuture`] that is
a sendable boxed future when "sync" feature is enabled,
allowing using multithreaded future executors.
And without "sync" feature [`maybe_sync::BoxFuture`] is
a non-sendable boxed future and only then [`FetchSource`] exists.

Similar story with ECS crates.
Most of them require that components are [`Send`] + [`Sync`]
so systems can access them from any thread.
This makes it impossible to have types from [`web-sys`] in components.

Some small application may desire to turn off "sync" feature in their
dependencies as they are singlethreaded and would rather not pay for what
they don't use.

## [`MaybeSend`] and [`MaybeSync`]

Marker traits [`MaybeSend`] and [`MaybeSync`] can be used in place of
[`Send`] and [`Sync`] traits in where clauses and bounds.

When "sync" feature is enabled then [`MaybeSend`] is actually reexported [`Send`]
and [`MaybeSync`] is reexported [`Sync`] trait.
Thus allowing types with those bounds to be sent or shared across threads.

When "sync" feature is not enabled then [`MaybeSend`] and [`MaybeSync`] are empty
traits implemented for all types and when used as bounds permit types that
does not satisfy [`Send`] or [`Sync`] bounds.

This makes it impossible to forget enable "sync" feature and accidentally send
`!Send` value to another thread.
Function that uncodintionally send value to another thread
should not use [`MaybeSend`] bound, but an actual [`Send`].

## BoxFuture

Type alias for boxed future. Sendable if "sync" feature is enabled.
It is designed to be used as return type from trait functions
where trait implementations that produce non-sendable futures
exist only when "sync" feature is not enabled.
It can be used as function argument type when [`MaybeSend`] bound is placed.

## Rc

Type alias to [`alloc::rc::Rc`] when "sync" feature is not enabled, or
[`alloc::sync::Arc`] when "sync" feature is enabled. Serves for optimization
purposes for crates that already use [`maybe-sync`] crate.

## Mutex

Type alias to [`parking_lot::Mutex`] when "sync" feature is enabled, or
thin wrapper arond [`core::cell::RefCell`] otherwise. Serves for optimization
purposes for crates that already use [`maybe-sync`] crate.

[`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
[`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
[`web-sys`]: https://docs.rs/web-sys
[`goods`]: https://docs.rs/goods
[`tokio`]: https://docs.rs/tokio
[`async-std`]: https://docs.rs/async-std
[`actix-rt`]: https://docs.rs/actix-rt
[`FetchSource`]: https://docs.rs/goods/0.5/wasm32-unknown-unknown/goods/struct.FetchSource.html
[`wasm_bindgen_futures::spawn_local`]: https://docs.rs/wasm-bindgen-futures/0.4/wasm_bindgen_futures/fn.spawn_local.html
[`goods::Spawn::spawn`]: https://docs.rs/goods/0.5/goods/trait.Spawn.html#tymethod.spawn
[`maybe-sync::BoxFuture`]: ./type.BoxFuture.html
[`MaybeSend`]: ./trait.MaybeSend.html
[`MaybeSync`]: ./trait.MaybeSync.html
[`alloc::rc::Rc`]: https://doc.rust-lang.org/alloc/rc/struct.Rc.html
[`alloc::sync::Arc`]: https://doc.rust-lang.org/alloc/sync/struct.Arc.html
[`maybe-sync`]: ./index.html
[`parking_lot::Mutex`]: https://docs.rs/parking_lot/0.10/parking_lot/type.Mutex.html
[`core::cell::RefCell`]: https://doc.rust-lang.org/core/cell/struct.RefCell.html

## License

This repository is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
