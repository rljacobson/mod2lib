/*!

Types/type aliases that abstract over the implementing backing type.

A motivating example is the `RcCell` type, a reference-counting smart pointer that provides run-time checked mutable
access to its contents and supports weak references. A number of external crates could provide this functionality. This
module redirects to whatever chosen implementation we want.

*/

mod nat_set;
mod rccell;
mod string_join;
mod heap;

// Logging
pub mod log;

// A set of (small) natural numbers
pub use nat_set::NatSet;

// Reference counted pointers with mutable stable, and complementary weak pointers.
pub use rccell::{rc_cell, RcCell, WeakCell};

// Interned string. Use `DefaultAtom` for a global cache that can be used across threads. Use `Atom` for a thread-local
// string cache.
pub use string_cache::DefaultAtom as IString;

// Join sequences with a separator
pub use string_join::{join_string, join_iter};

// Heap construction/destruction
pub use heap::{heap_construct, heap_destroy};
