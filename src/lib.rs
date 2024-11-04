pub mod api;
mod abstractions;

// We re-export abstractions that are meant to be used publicly.
pub use abstractions::{
  log,
  IString
};

pub fn add(left: u64, right: u64) -> u64 {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
