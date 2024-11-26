/*!

A `Term` is a node in the expression tree. That is, an expression tree is a term, and
each subexpression is a term. The algorithms do not operate on expression trees (terms).
Instead, the algorithms operate on a directed acyclic graph (DAG) is constructed from the
tree. Thus, for each `Term` type, there is a corresponding `DagNode` type. However, because
of structural sharing, the node instances themselves are not in 1-to-1 correspondence.

*/

use std::{
  collections::{
    HashMap,
    hash_map::Entry
  },
  cmp::Ordering,
  ptr::NonNull,
  sync::{
    atomic::{
      Ordering::Relaxed,
      AtomicBool
    },
    Mutex
  },
};
use std::cell::Cell;
use enumflags2::{bitflags, BitFlags};
use once_cell::sync::Lazy;

use crate::{
  abstractions::{NatSet, RcCell, Set},
  api::{
    Substitution,
    SymbolSet,
    UNDEFINED,
    dag_node::{DagNode, DagNodeFlag, DagNodePtr},
    free_theory::FreeTerm,
    symbol::{Symbol, SymbolPtr}
  },
  core::sort::{Sort, SortPtr}
};

pub type BxTerm    = Box<Term>;
pub type RcTerm    = RcCell<Term>;
pub type MaybeTerm = Option<BxTerm>;
pub type TermSet   = HashMap<u32, usize>;

/*
  static Vector<DagNode*> subDags;
  static TermSet converted;
  static bool setSortInfoFlag;
*/
static mut CONVERTED_TERMS: Lazy<TermSet> =  Lazy::new(|| {
  TermSet::new()
});

static mut SUBDAG_CACHE : Lazy<Vec<DagNodePtr>> = Lazy::new(|| {
  Vec::new()
});

static SET_SORT_INFO_FLAG: AtomicBool = AtomicBool::new(false);

/// This trait holds the theory-specific parts of a term.
pub trait TheoryTerm {
  fn dagify(&self, parent: &Term) -> DagNodePtr;
}



#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TermKind {
  Free,
  Bound,
  Ground,
  NonGround,
}

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
  pub(crate) symbol: SymbolPtr,
  pub(crate) sort  : Option<NonNull<SortPtr>>,
  theory_term      : Box<dyn TheoryTerm>,
  /// The handles (indices) for the variable terms that occur in this term or its descendants
  pub(crate) occurs_set      : NatSet,
  pub(crate) context_set     : NatSet,
  pub(crate) collapse_symbols: SymbolSet,
  pub(crate) attributes      : TermAttributes,
  pub(crate) term_kind       : TermKind,
  pub(crate) save_index      : i32,            // NoneIndex = -1
  hash_value                 : u32,

  /// The number of nodes in the term tree
  pub(crate) cached_size:  Cell<i32>,
}

impl Term {
  pub fn new(symbol: SymbolPtr) -> Term {
    Term {
      symbol,
      sort            : None,
      theory_term     : Box::new(FreeTerm::new()),
      occurs_set      : Default::default(),
      context_set     : Default::default(),
      collapse_symbols: Default::default(),
      attributes      : TermAttributes::default(),
      term_kind       : TermKind::Free,
      save_index      : 0,
      hash_value      : 0,
      cached_size     : Cell::new(UNDEFINED),
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

  /// The handles (indices) for the variable terms that occur in this term or its descendants
  #[inline(always)]
  fn occurs_below(&self) -> &NatSet {
    &self.occurs_set
  }

  #[inline(always)]
  fn occurs_below_mut(&mut self) -> &mut NatSet {
    &mut self.occurs_set
  }

  #[inline(always)]
  fn occurs_in_context(&self) -> &NatSet {
    &self.context_set
  }

  #[inline(always)]
  fn occurs_in_context_mut(&mut self) -> &mut NatSet {
    &mut self.context_set
  }

  #[inline(always)]
  fn collapse_symbols(&self) -> &SymbolSet {
    &self.collapse_symbols
  }

  /// Returns an iterator over the arguments of the term
  fn iter_args(&self) -> Box<dyn Iterator<Item = &Term> + '_>{
    // Box::new(std::iter::empty::<RcTerm>())
    unimplemented!("Implement empty iterator as Box::new(std::iter::empty::<RcTerm>())")
  }

  /// Compute the number of nodes in the term tree
  fn compute_size(&self) -> i32 {
    if self.cached_size.get() != UNDEFINED {
      self.cached_size.get()
    } else {
      let mut size = 1; // Count self.
      for arg in self.iter_args() {
        size += arg.compute_size();
      }
      self.cached_size.set(size);
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


  // region DAG Creation

  #[inline(always)]
  fn term_to_dag(&self, set_sort_info: bool) -> DagNodePtr {
    SET_SORT_INFO_FLAG.store(set_sort_info, Relaxed);
    unsafe { SUBDAG_CACHE.clear(); }
    unsafe { CONVERTED_TERMS.clear(); }
    self.dagify()
  }

  /// Create a directed acyclic graph from this term. This trait-level implemented function takes care of structural
  /// sharing. Each implementing type will supply its own implementation of `dagify_aux(…)`, which recursively
  /// calls `dagify(…)` on its children and then converts itself to a type implementing DagNode, returning `DagNodePtr`.
  fn dagify(&self) -> DagNodePtr {
    if let Entry::Occupied(occupied_entry) = unsafe{ CONVERTED_TERMS.entry(self.semantic_hash()) } {
      let idx = *occupied_entry.get();
      return unsafe{ SUBDAG_CACHE[idx] };
    }

    let mut d = self.dagify_aux();
    if SET_SORT_INFO_FLAG.load(Relaxed) {
      assert_ne!(
        self.sort,
        None,
        "Missing sort info"
      );
      let mut d_mut: &mut DagNode = unsafe{ &mut *d };
      d_mut.sort = self.sort;
      d_mut.flags.insert(DagNodeFlag::Reduced.into());
    }

    let idx = unsafe{ SUBDAG_CACHE.len() };
    unsafe{ SUBDAG_CACHE.push(d) };
    // sub_dags.insert(self_hash, d.clone());
    unsafe{ CONVERTED_TERMS.insert(self.semantic_hash(), idx) };
    d
  }

  /// Create a directed acyclic graph from this term. This method has the implementation-specific stuff.
  fn dagify_aux(&self) -> DagNodePtr{
    self.theory_term.dagify(self)
  }

  // endregion

}


