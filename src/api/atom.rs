/*!

An `Atom` is an indivisible (atomic) element. An `Atom` can be
  - **Variable**:
    - Singleton wildcard, also called a blank.
    - One-or-more wildcard, also called a blank sequence.
    - Zero-or-more wildcard, also called a null sequence.
  - **Constant**: Integer, real, byte, string.
  - **Symbol**: Name which may or may not be bound to a value.

The type system is independent of these categories; that is, sorts/kinds are not represented by any of these entities.

# Defining Constants



*/

use std::{
    fmt::Debug,
    any::Any,
    hash::Hash
};

pub trait ConstantType: Debug + Any + PartialEq + Hash {
  fn as_any(&self) -> &dyn Any;
}

/// A convenience macro for defining newtype `ConstantType` of a type that satisfies the type constraints
/// `Debug + Any + PartialEq + Hash`.
#[macro_export]
macro_rules! declare_constant_newtype {
    ($name:ident, $inner:ty) => {
        pub struct $name(pub $inner);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl ConstantType for $name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

pub use declare_constant_newtype;

/// A variable is a symbolic name that can be unbound or bound to a term or sequence.

pub enum Atom<ConstantType> {
  // Variable(Variable),
  Constant(Box<ConstantType>),
  // Symbol(Symbol)
}
