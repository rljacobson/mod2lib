/*!

Definitions related to symbols. Symbols can be thought of as names to which additional information is attached, such as
arity and theory axioms.

In an expression like, `f[x, Thing1, 45]`, the symbols are `f`, `x`, and `Thing1`. There is also an implicit symbol
shared by all data constants, the number `45` in this case, which is defined by the client code that defined the
`DataAtom` type. Integers might be represented by the `IntegerAtom` type (implementing the `DataAtom` trait) and have
the symbol `Integer` for example.

*/

use std::fmt::Display;
use enumflags2::{bitflags, make_bitflags, BitFlags};
use crate::abstractions::IString;
use crate::api::Arity;

pub type SymbolPtr = *const Symbol;

pub struct Symbol {
  pub name       : IString,

  pub arity      : Arity,
  pub attributes : SymbolAttributes,
  pub symbol_type: SymbolType
}

impl Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // match self.arity {
    //   Arity::Variadic => write!(f, "{}ᵥ", self.name),
    //   Arity::Value(arity) if arity > 0 => write!(f, "{}/{}", self.name, arity)
    //   _ => write!(f, "{}", self.name),
    // }
    write!(f, "{}", self.name)
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum SymbolType {
  #[default]
  Standard,
  Variable,
  Operator,
  Data
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SymbolAttribute {
  // Syntactic attributes
  Precedence,
  Gather,
  Format,
  Latex,

  // Semantic attributes
  Strategy,
  Memoized,
  Frozen,
  Constructor,

  // Theory attributes
  Associative,
  Commutative,
  LeftIdentity,
  RightIdentity,
  Idempotent,
  Iterated,
}

pub type SymbolAttributes = BitFlags<SymbolAttribute, u32>;

impl SymbolAttribute {
  //	Conjunctions
  #![allow(non_upper_case_globals)]

  /// Theory Axioms
  pub const Axioms: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      Associative
      | Commutative
      | LeftIdentity
      | RightIdentity
      | Idempotent
    }
  );

  pub const Collapse: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      LeftIdentity
      | RightIdentity
      | Idempotent
    }
  );

  ///	Simple attributes are just a flag without additional data. They produce a warning if given twice.
  pub const SimpleAttributes: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      Associative
      | Commutative
      | Idempotent
      | Memoized
      | Constructor
      | Iterated
    }
  );

  /// All flagged attributes. They need to agree between declarations of an
  /// operator.
  pub const Attributes: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      Precedence
      | Gather
      | Format
      | Latex
      | Strategy
      | Memoized
      | Frozen
      | Associative
      | Commutative
      | LeftIdentity
      | RightIdentity
      | Idempotent
      | Iterated
    }
  );
}


