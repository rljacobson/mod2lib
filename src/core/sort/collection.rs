use std::collections::{HashMap, HashSet};
use std::collections::hash_map::{Entry, Iter};
use std::iter::Map;
use std::ops::Index;
use crate::abstractions::{IString, heap_construct};
use crate::core::sort::{Sort, SortPtr};

/// A set of unique sorts with helper methods for creating new sorts. Helper collection only used during module construction.
#[derive(Default)]
pub struct SortCollection {
  sorts: HashMap<IString, SortPtr>
}

impl SortCollection {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn get_or_create_sort(&mut self, name: IString) -> SortPtr {
    match self.sorts.entry(name.clone()) {
      Entry::Occupied(s) => s.get().clone(),
      Entry::Vacant(v) => {
        let s = heap_construct!(Sort::new(name));
        v.insert(s);
        s
      }
    }
  }

  /// Given a list of sort names, inserts or creates a sort for each name.
  pub fn create_implicit_sorts(&mut self, sort_names: &mut HashSet<IString>) {
    for sort_name in sort_names.drain() {
      self.get_or_create_sort(sort_name);
    }
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    self.sorts.len()
  }
  /// Creates and returns an iterator over the `SortCollection`.
  // Can we just stop to appreciate how stupid the return type of this method is? And how obnoxious it is to have to
  // specify it?
  pub(crate) fn iter(&self) -> Map<Iter<'_, IString, SortPtr>, fn((&IString, &SortPtr)) -> (IString, SortPtr)> {
    self.sorts.iter().map(|(istr, rcs)| (istr.clone(), *rcs))
  }
}
