#![allow(unused)]
/*!

Types/type aliases that abstract over the implementing backing type.

# Background and Motivation

A motivating example is the `IString` type, an interned string type. A number of external crates could provide this
functionality. This module redirects to whatever chosen implementation we want. To use the
[`string_cache` crate](https://crates.io/crates/string_cache), we just define `IString` as an alias for
`string_cache::DefaultAtom`:

```ignore
pub use string_cache::DefaultAtom as IString;
```

If we want to later change to the [`ustr` crate](https://crates.io/crates/ustr), we just define `IString` to be an
alias for `ustr::Ustr` instead:

```ignore
pub use ustr::Ustr as IString;
```

The `ustr` and `string_cache` crates conveniently have very similar public APIs. For types or infrastructure with very
different backing implementations, we define an abstraction layer over the implementation. For example, the `log`
module could use any of a number of logging frameworks or even a bespoke solution for its implementation. However, its
(crate) public interface consists only of `set_global_logging_threshold()`/`get_global_logging_threshold()` and the
macros `critical!`, `error!`, `warning!`, `info!`, `debug!`, and `trace!`. The (private) backing implementation is
encapsulated in the `log` module.

*/

mod nat_set;
mod rccell;
mod string_join;
mod heap;
pub(crate) mod erased;

use std::collections::HashSet as StdHashSet;


// Logging
pub mod log;

// Interned string. Use `DefaultAtom` for a global cache that can be used across threads. Use `Atom` for a thread-local
// string cache.
pub use string_cache::DefaultAtom as IString;

// Heap construction/destruction
// pub use heap::{heap_construct, heap_destroy};

// region Items meant to be used only internally

// A set of (small) natural numbers
pub(crate) use nat_set::NatSet;

// Reference counted pointers with mutable stable, and complementary weak pointers.
pub(crate) use rccell::{rc_cell, RcCell, WeakCell};

// Join sequences with a separator
pub(crate) use string_join::{join_string, join_iter};


/// A `ThingSet` is a hash set of `*const dyn Things`. They are useful if you need to test membership but never need
/// to access the original `Thing`.
pub type Set<T> = StdHashSet<T>; // This replaces Maude's `PointerSet` in most situations.


// endregion
