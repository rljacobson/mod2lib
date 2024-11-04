pub mod atom;
pub mod symbol;
mod variable;

// Small utility types used throughout
pub enum Arity {
  Unspecified,
  Variadic,
  Value(i16)
}

impl From<Arity> for i16 {
  fn from(arity: Arity) -> Self {
    match arity {
      Arity::Unspecified => -2,
      Arity::Variadic => -1,
      Arity::Value(val) => val
    }
  }
}

impl From<i16> for Arity {
  fn from(i: i16) -> Self {
    if i < -2 {
      panic!("Negative arity encountered: {}", i);
    } else if i == -2 {
      return Arity::Unspecified;
    } else if i == -1 {
      return Arity::Variadic;
    } else {
      return Arity::Value(i)
    }
  }
}
