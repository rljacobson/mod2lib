/*!

The `DagNode` is the heart of the engine. Speed hinges on efficient management of `DagNode` objects. Their creation,
reuse, and destruction are managed by an arena based garbage collecting allocator which relies on the fact that
every `DagNode` is of the same size. Since `DagNode`s can be of different types and have arguments, we make careful use
of transmute and bitflags.

The following compares Maude's `DagNode` to our implementation here.

|                | Maude                                        | mod2lib                  |
|:---------------|:---------------------------------------------|:-------------------------|
| size           | Fixed 3 word size                            | Fixed size struct        |
| tag            | implicit via vtable pointer                  | enum variant             |
| flags          | `MemoryInfo` in first word                   | `BitFlags` field         |
| shared impl    | base class impl                              | enum impl                |
| specialization | virtual function calls                       | match on variant in impl |
| args           | `reinterpret_cast` of 2nd word based on flag | Nested enum              |

*/

use std::{
  fmt::{Display, Formatter},
  cmp::max,
  marker::PhantomPinned
};
use std::ptr::NonNull;
use enumflags2::{bitflags, make_bitflags, BitFlags};

use crate::{
  api::symbol::{Symbol, SymbolPtr},
  core::{
    allocator::{
      allocate_dag_node,
      increment_active_node_count,
      node_vector::{NodeVector, NodeVectorMutRef}
    }
  }
};
use crate::api::Arity;
use crate::core::sort::SortPtr;

pub type DagNodePtr = *mut DagNode;
// pub type DagNodeMutPtr = *mut DagNode;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
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

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DagNodeFlag {
  /// Marked as in use
  Marked,
  /// Has args that need destruction
  NeedsDestruction,
  /// Reduced up to strategy by equations
  Reduced,
  /// Copied in current copy operation; copyPointer valid
  Copied,
  /// Reduced and not rewritable by rules
  Unrewritable,
  /// Unrewritable and all subterms unstackable or frozen
  Unstackable,
  /// No variables occur below this node
  GroundFlag,
  /// Node has a valid hash value (storage is theory dependent)
  HashValid,
}
impl DagNodeFlag {
  #![allow(non_upper_case_globals)]

  /// An alias - We can share the same bit for this flag since the rule rewriting
  /// strategy that needs `Unrewritable` will never be combined with variant narrowing.
  pub const IrreducibleByVariantEquations: DagNodeFlag = DagNodeFlag::Unrewritable;

  // Conjunctions

  /// Flags for rewriting
  pub const RewritingFlags: DagNodeFlags = make_bitflags!(
    DagNodeFlag::{
      Reduced | Unrewritable | Unstackable | GroundFlag
    }
  );
}

pub type DagNodeFlags = BitFlags<DagNodeFlag, u8>;

#[derive(Default)]
pub enum DagNodeArgument{
  #[default]
  None,
  Single(DagNodePtr),
  Many(NodeVectorMutRef)
}


pub struct DagNode {
  pub(crate) symbol   : SymbolPtr,
  pub(crate) args     : DagNodeArgument,
  pub(crate) sort     : Option<NonNull<SortPtr>>,
  pub(crate) node_kind: DagNodeKind,
  pub(crate) flags    : DagNodeFlags,

  // Opt out of `Unpin`
  _pin: PhantomPinned,
}


impl DagNode {
  // region Constructors

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    DagNode::with_kind(symbol, DagNodeKind::default())
  }

  pub fn with_kind(symbol: SymbolPtr, kind: DagNodeKind) -> DagNodePtr {
    assert!(!symbol.is_null());
    let node: DagNodePtr = allocate_dag_node();
    let node_mut         = unsafe { &mut *node };

    let arity = match unsafe{ &*symbol }.arity {
      // ToDo: How do we allocate a NodeVec for variadic nodes?
      | Arity::Unspecified
      | Arity::Any
      | Arity::None
      | Arity::Variadic => 0,

      Arity::Value(v) => v as usize,

    };

    node_mut.node_kind = kind;
    node_mut.flags     = DagNodeFlags::empty();
    node_mut.symbol    = symbol;
    node_mut.args      = if arity > 1 {
      DagNodeArgument::Many(NodeVector::with_capacity(arity))
    } else {
      DagNodeArgument::None
    };
    node
  }

  pub fn with_args(symbol: SymbolPtr, args: &mut Vec<DagNodePtr>, kind: DagNodeKind) -> DagNodePtr {
    assert!(!symbol.is_null());
    let node: DagNodePtr = { allocate_dag_node() };
    let node_mut         = unsafe { &mut *node };

    node_mut.node_kind = kind;
    node_mut.flags     = DagNodeFlags::empty();
    node_mut.symbol    = symbol;

    // ToDo: How do we allocate a NodeVec for variadic nodes?
    let arity = if let Arity::Value(v) = unsafe{ &*symbol }.arity { v as usize } else { 0 };

    if arity > 1 || args.len() > 1 {
      let capacity = max(arity, args.len());
      let node_vector = NodeVector::with_capacity(capacity);

      for node in args.iter().cloned() {
        _  = node_vector.push(node);
      }

      node_mut.args = DagNodeArgument::Many(node_vector);
    }
    else if args.len() == 1 {
      node_mut.args = DagNodeArgument::Single(args[0]);
    } else {
      node_mut.args = DagNodeArgument::None;
    };

    node
  }

  // endregion Constructors

  // region Accessors

  pub fn iter_children(&self) -> std::slice::Iter<'static, DagNodePtr> {
    // For assertions
    // ToDo: These assertions will need to change for variadic nodes.
    let arity = if let Arity::Value(v) = self.arity() { v } else { 0 };

    match &self.args {
      DagNodeArgument::None => {
        assert_eq!(arity, 0);
        [].iter()
      }
      DagNodeArgument::Single(node) => {
        assert_eq!(arity, 1);
        // Make a fat pointer to the single node and return an iterator to it. This allows `self` to
        // escape the method. Of course, `self` actually points to a `DagNode` that is valid for the
        // lifetime of the program, so even in the event of the GC equivalent of a dangling pointer
        // or use after free, this will be safe. (Strictly speaking, it's probably UB.)
        let v = unsafe { std::slice::from_raw_parts(node, 1) };
        v.iter()
      }
      DagNodeArgument::Many(node_vector) => {
        assert!(arity>1);
        // We need to allow `self` to escape the method, same as `Single(..)` branch.
        let node_vector_ptr: *const NodeVector = *node_vector;
        unsafe{ &*node_vector_ptr }.iter()
      }
    }
  }

  #[inline(always)]
  pub fn symbol(&self) -> &Symbol {
    unsafe {
      &*self.symbol
    }
  }

  #[inline(always)]
  pub fn arity(&self) -> Arity {
    self.symbol().arity
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    match &self.args {
      DagNodeArgument::None      => 0,
      DagNodeArgument::Single(_) => 1,
      DagNodeArgument::Many(v)   => v.len()
    }
  }

  pub fn insert_child(&mut self, new_child: DagNodePtr) -> Result<(), String>{
    assert!(!new_child.is_null());

    match self.args {

      DagNodeArgument::None => {
        self.args = DagNodeArgument::Single(new_child);
        Ok(())
      }

      DagNodeArgument::Single(first_child) => {
        let vec   = NodeVector::from_slice(&[first_child, new_child]);
        self.args = DagNodeArgument::Many(vec);
        Ok(())
      }

      DagNodeArgument::Many(ref mut vec) => {
        vec.push(new_child)
      }

    }
  }

  // endregion

  // region GC related methods
  #[inline(always)]
  pub fn is_marked(&self) -> bool {
    self.flags.contains(DagNodeFlag::Marked)
  }

  #[inline(always)]
  pub fn needs_destruction(&self) -> bool {
    if let DagNodeArgument::Many(_) = self.args {
      true
    } else {
      false
    }
  }

  #[inline(always)]
  pub fn simple_reuse(&self) -> bool {
    !self.flags.contains(DagNodeFlag::Marked) && !self.needs_destruction()
  }

  #[inline(always)]
  pub fn mark(&'static mut self) {
    if self.flags.contains(DagNodeFlag::Marked) {
      return;
    }

    increment_active_node_count();
    self.flags.insert(DagNodeFlag::Marked);

    match &mut self.args {

      DagNodeArgument::None => { /* pass */ }

      DagNodeArgument::Single(node) => {
        if let Some(node) = unsafe { node.as_mut() } {
          node.mark();
        }
      }

      DagNodeArgument::Many(ref mut node_vec) => {


        for node in node_vec.iter() {
          if let Some(node) = unsafe { node.as_mut() } {
            node.mark();
          } else {
            eprintln!("Bad node found.")
          }
        }

        // Sanity check
        // ToDo: This will need to change for variadic nodes.
        let arity = if let Arity::Value(v) = unsafe{ (&*self.symbol).arity } { v } else { 0 };
        if node_vec.capacity() != arity as usize || node_vec.len() > node_vec.capacity() {
          panic!("Node vector capacity mismatch.")
        }

        // Reallocate
        *node_vec = node_vec.shallow_copy();
      }

    }

  }
  //endregion

}

impl Display for DagNode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "node<{}>", self.symbol())
  }
}
