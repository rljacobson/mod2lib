/*!

The `DagNode` is the heart of the engine. Speed hinges on efficient management of `DagNode` objects. Their creation,
reuse, and destruction are managed by an arena based garbage collecting allocator which relies on the fact that
every `DagNode` is of the same size. Since `DagNode`s can be of different types and have arguments, we make careful use
of transmute and bitflags.

The following compares Maude's `DagNode` to our implementation here.

|                | Maude                                        | mod2lib                     |
|:---------------|:---------------------------------------------|:----------------------------|
| size           | Fixed 3 word size (or 6 words?)              | Fixed size struct (4 words) |
| tag            | implicit via vtable pointer                  | enum variant                |
| flags          | `MemoryInfo` in first word                   | `BitFlags` field            |
| shared impl    | base class impl                              | enum impl                   |
| specialization | virtual function calls                       | match on variant in impl    |
| args           | `reinterpret_cast` of 2nd word based on flag | Nested enum                 |

*/



mod root_container;
pub(crate) mod allocator;
pub mod sort;
pub mod module;
pub mod pre_equation;

#[allow(unused_imports)]
pub use root_container::RootContainer;

/// A `*mut Void` is a pointer to a `u8`
pub type Void = u8;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum DagNodeKind {
  #[default]
  Free = 0,
  ACU,
  AU,
  CUI,
  Variable,
  NA,
  Data,
  // Integer,
  // Float
}

#[cfg(test)]
mod tests {
  use crate::{
    api::dag_node::{
      DagNodeKind,
      DagNodeFlags,
      DagNode,
      DagNodeArgument
    },
    api::symbol::SymbolPtr
  };

  #[test]
  fn size_of_dag_node() {
    println!("size of SymbolPtr: {}", size_of::<SymbolPtr>());
    println!("size of DagNodeArgument: {}", size_of::<DagNodeArgument>());
    println!("size of DagNodeKind: {}", size_of::<DagNodeKind>());
    println!("size of DagNodeFlags: {}", size_of::<DagNodeFlags>());
    println!("size of DagNode: {}", size_of::<DagNode>());
    assert_eq!(size_of::<DagNode>(), 4 * size_of::<usize>());
  }
}
