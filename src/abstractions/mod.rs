/*!

Types/type aliases that abstract over the implementing backing type.

A motivating example is the `RcCell` type, a reference-counting smart pointer that provides run-time checked mutable
access to its contents and supports weak references. A number of external crates could provide this functionality. This
module redirects to whatever chosen implementation we want.

*/

mod nat_set;
mod rccell;
mod string_join;

// Logging
pub mod log;

// A set of natural numbers

// Reference counted pointers with mutable stable, and complementary weak pointers.

// Interned string.

// Join sequences with a separator
