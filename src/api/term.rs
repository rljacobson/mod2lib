/*!

A `Term` is a node in the expression tree. That is, an expression tree is a term, and
each subexpression is a term. The algorithms do not operate on expression trees (terms).
Instead, the algorithms operate on a directed acyclic graph (DAG) is constructed from the
tree. Thus, for each `Term` type, there is a corresponding `DagNode` type. However, because
of structural sharing, the node instances themselves are not in 1-to-1 correspondence.

*/

use std::cmp::Ordering;
use enumflags2::{bitflags, BitFlags};
use crate::abstractions::{NatSet, RcCell, Set};
use crate::api::symbol::{Symbol, SymbolPtr};
use crate::api::{Substitution, SymbolSet, UNDEFINED};
use crate::api::dag_node::DagNode;

pub type RcTerm    = RcCell<Term>;
pub type MaybeTerm = Option<RcTerm>;
pub type TermSet   = Set<RcTerm>;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TermAttribute {
  ///	A subterm is stable if its top symbol cannot change under instantiation.
  Stable,

  ///	A subterm is in an eager context if the path to its root contains only
  ///	eagerly evaluated positions.
  EagerContext,

  ///	A subterm "honors ground out match" if its matching algorithm guarantees
  ///	never to return a matching subproblem when all the terms variables
  ///	are already bound.
  HonorsGroundOutMatch,
}

pub type TermAttributes = BitFlags<TermAttribute, u8>;

pub struct Term {
  /// The top symbol of the term
  pub(crate) symbol:   SymbolPtr,
  /// The handles (indices) for the variable terms that occur in this term or its descendants
  pub(crate) occurs_set:   NatSet,
  pub(crate) context_set:  NatSet,
  pub(crate) collapse_symbols: SymbolSet,
  pub(crate) attributes:   TermAttributes,
  pub(crate) save_index:   i32, // NoneIndex = -1
  // pub(crate) hash_value: u32,
  /// The number of nodes in the term tree
  pub(crate) cached_size:  i32,
}

impl Term {
  pub fn new(symbol: SymbolPtr) -> Term {
    Term {
      symbol:   symbol,
      occurs_set:   Default::default(),
      context_set:  Default::default(),
      collapse_symbols: Default::default(),
      attributes:   TermAttributes::default(),
      save_index:   0,
      // hash_value: 0,
      cached_size:  UNDEFINED,
    }
  }

  /// Normalizes the term, returning the computed hash and `true` if the normalization changed
  /// the term or `false` otherwise.
  pub fn normalize(&mut self, _full: bool) -> (u32, bool){
    unimplemented!()
  }

  // region Accessors

  /// Is the term stable?
  #[inline(always)]
  pub fn is_stable(&self) -> bool {
    self.attributes.contains(TermAttribute::Stable)
  }

  /// A subterm "honors ground out match" if its matching algorithm guarantees never to return a matching subproblem
  /// when all the terms variables are already bound.
  #[inline(always)]
  pub fn honors_ground_out_match(&self) -> bool {
    self.attributes.contains(TermAttribute::HonorsGroundOutMatch)
  }

  #[inline(always)]
  pub fn set_honors_ground_out_match(&mut self, value: bool) {
    if value {
      self.attributes.insert(TermAttribute::HonorsGroundOutMatch);
    } else {
      self.attributes.remove(TermAttribute::HonorsGroundOutMatch);
    }
  }

  #[inline(always)]
  pub fn is_eager_context(&self) -> bool {
    self.attributes.contains(TermAttribute::EagerContext)
  }

  #[inline(always)]
  pub fn is_variable(&self) -> bool {
    unsafe {
      let symbol: &Symbol = &*self.symbol;
      symbol.is_variable()
    }
  }

  #[inline(always)]
  pub fn ground(&self) -> bool {
    self.occurs_set.is_empty()
  }

  /// Returns an iterator over the arguments of the term
  fn iter_args(&self) -> Box<dyn Iterator<Item = RcTerm> + '_>{
    // Box::new(std::iter::empty::<RcTerm>())
    unimplemented!("Implement empty iterator as Box::new(std::iter::empty::<RcTerm>())")
  }

  /// Compute the number of nodes in the term tree
  fn compute_size(&mut self) -> i32 {
    if self.cached_size != UNDEFINED {
      self.cached_size
    } else {
      let mut size = 1; // Count self.
      for arg in self.iter_args() {
        size += arg.borrow_mut().compute_size();
      }
      self.cached_size = size;
      size
    }
  }

  #[inline(always)]
  pub fn symbol(&self) -> &'static Symbol {
    unsafe {
      &*self.symbol
    }
  }

  // endregion Accessors

  // region Comparison Functions

  /// Delegates to TheoryTerm::compare_term_arguments
  fn compare_term_arguments(&self, _other: &Term) -> Ordering{
    unimplemented!()
  }

  #[inline(always)]
  fn compare_dag_node(&self, other: &DagNode) -> Ordering {
    if self.symbol().hash_value == other.symbol().hash_value {
      self.compare_dag_arguments(other)
    } else {
      self.symbol().compare(other.symbol())
    }
  }

  /// Delegates to TheoryTerm
  fn compare_dag_arguments(&self, _other: &DagNode) -> Ordering{
    unimplemented!()
  }


  fn partial_compare(&self, partial_substitution: &mut Substitution, other: &DagNode) -> Option<Ordering> {
    if !self.is_stable() {
      // Only used for `VariableTerm`
      return self.partial_compare_unstable(partial_substitution, other);
    }

    if self.symbol == other.symbol {
      // Only used for `FreeTerm`
      return self.partial_compare_arguments(partial_substitution, other);
    }

    if self.symbol().compare(other.symbol()) == Ordering::Less {
      Some(Ordering::Less)
    } else {
      Some(Ordering::Greater)
    }
  }

  #[inline(always)]
  fn compare(&self, other: &Term) -> Ordering {
    let r = self.symbol().compare(other.symbol());
    if r == Ordering::Equal {
      return self.compare_term_arguments(other);
    }
    r
  }

  /// Overridden in `VariableTerm`
  fn partial_compare_unstable(&self, _partial_substitution: &mut Substitution, _other: &DagNode) -> Option<Ordering> {
    None
  }

  /// Overridden in `FreeTerm`
  fn partial_compare_arguments(&self, _partial_substitution: &mut Substitution, _other: &DagNode) -> Option<Ordering> {
    None
  }

  // endregion

}
