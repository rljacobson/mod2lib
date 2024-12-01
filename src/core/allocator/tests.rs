use rand::Rng;

use crate::{
  abstractions::IString,
  api::{
    Arity,
    dag_node::{DagNode, DagNodePtr},
    symbol::Symbol
  },
  core::allocator::*,
  core::RootContainer
};
use crate::api::free_theory::FreeDagNode;
use crate::api::symbol::SymbolPtr;
use crate::core::dag_node_core::{DagNodeCore, DagNodeTheory};
/*
Recursively builds a random tree of `DagNode`s with a given height and arity rules.

Because this function holds on to iterators of `NodeVec`s, the GC cannot run during
the building of the tree. Run the GC before or after.

 - `symbols`: List of `Symbol` objects of each arity from 0 to `max_width`.
 - `parent`: Pointer to the current parent node.
 - `max_height`: Maximum allowed height for the tree.
*/
pub fn build_random_tree(
  symbols   : &mut [Symbol],
  parent    : DagNodePtr,
  max_height: usize,
  max_width : usize,
  min_width : usize,
) {
  if max_height == 0 {
    return; // Reached the maximum depth
  }

  // idiot-proof
  let min_width = std::cmp::min(max_width, min_width);
  let max_width = std::cmp::max(max_width, min_width);

  let mut rng   = rand::thread_rng();

  // Get the parent node's arity from its symbol
  let parent_arity = if let Arity::Value(v) = unsafe { (*parent).arity() } { v as usize } else { 0 };

  // For each child based on the parent's arity, create a new node
  for i in 0..parent_arity as usize {
    // Determine the arity of the child node
    let child_arity = if max_height == 1 {
      0 // Leaf nodes must have arity 0
    } else {
      rng.gen_range(min_width..=max_width) // Random arity between min_width and max_width
    };

    // Create the child node with the symbol corresponding to its arity
    let child_symbol: SymbolPtr = &mut symbols[child_arity];
    let child_node   = FreeDagNode::new(child_symbol);

    // Insert the child into the parent node
    let parent_mut = unsafe{ parent.as_mut_unchecked() };
    if let Arity::Value(v) = parent_mut.arity(){
      if i > v as usize {
        panic!("Incorrect arity");
      }



    }
    parent_mut.insert_child(child_node);

    // Recursively build the subtree for the child
    build_random_tree(symbols, child_node, max_height - 1, max_width, min_width);
  }
}

/// Recursively prints a tree structure using ASCII box-drawing symbols.
///
/// - `node`: The current node to print.
/// - `prefix`: The string prefix to apply to the current node's line.
/// - `is_tail`: Whether the current node is the last child of its parent.
pub fn print_tree(node: DagNodePtr, prefix: String, is_tail: bool) {
  assert!(!node.is_null());

  let is_head = prefix.is_empty();
  let node: &dyn DagNode = unsafe{ &*node };

  let arity = if let Arity::Value(v) = node.arity() {
    v
  } else {
    0
  };

  if arity as usize != node.len() {
    panic!("Incorrect arity/len. arity: {}  len: {}", arity, node.len());
  }

  // Print the current node
  let new_prefix = if is_head {
    ""
  } else {
    if is_tail { "╰──" } else { "├──" }
  };
  println!(
    "{}{}{}",
    prefix,
    new_prefix,
    node
  );

  // Determine the new prefix for children
  let new_prefix = if is_tail {
    format!("{}    ", prefix)
  } else if is_head {
    format!(" ")
  }
  else {
    format!("{}│   ", prefix)
  };

  // Print each child
  for (i, child_ptr) in node.iter_args().enumerate() {
    print_tree(
      child_ptr,
      new_prefix.clone(),
      i == node.len() - 1, // Is this the last child?
    );
  }
}



#[test]
fn test_allocate_dag_node() {
  let node_ptr = allocate_dag_node();
  let node_mut = match unsafe { node_ptr.as_mut() } {
    None => {
      panic!("allocate_dag_node returned None");
    }
    Some(node) => { node }
  };

  node_mut.theory_tag = DagNodeTheory::Free;
}


#[test]
fn test_dag_creation() {
  let mut symbols = (0..=10)
      .map(|x| {
        // let name = IString::from(format!("sym({})", x).as_str());
        let name = IString::from("sym");
        Symbol::new(name, Arity::Value(x))
      })
      .collect::<Vec<_>>();

  let root = FreeDagNode::new(&mut symbols[3]);
  let _root_container = RootContainer::new(root);

  // Maximum tree height
  let max_height: usize = 6;
  let max_width : usize = 3;

  // Recursively build the random tree
  build_random_tree(&mut symbols, root, max_height, max_width, 0);
  print_tree(root, String::new(), false);
  // println!("Symbols: {:?}", symbols);
  #[cfg(feature = "gc_debug")]
  acquire_node_allocator("dump_memory_variables").dump_memory_variables()
}


#[test]
fn test_garbage_collection() {
  let mut symbols = (0..=10)
      .map(|x| {
        let name = IString::from(format!("sym({})", x).as_str());
        Symbol::new(name, Arity::Value(x))
      })
      .collect::<Vec<_>>();

  for _ in 0..100 {
    let mut root_vec = Vec::with_capacity(10);

    for _ in 0..10 {
      let root: DagNodePtr = DagNodeCore::new(&mut symbols[4]);
      let root_container = RootContainer::new(root);
      root_vec.push(root_container);

      // Maximum tree height
      let max_height: usize = 6; // exponent
      let max_width : usize = 4; // base

      // Recursively build the random tree
      build_random_tree(&mut symbols, root, max_height, max_width, 0);
    }
    acquire_node_allocator("ok_to_collect_garbage").ok_to_collect_garbage();

    // root_vec dropped
  }
  #[cfg(feature = "gc_debug")]
  acquire_node_allocator("dump_memory_variables").dump_memory_variables()
}


#[test]
fn test_arena_exhaustion() {
  let mut symbol = Symbol::new(IString::from("mysymbol"), Arity::Value(1));
  let symbol_ptr = &mut symbol;
  let root: DagNodePtr = DagNodeCore::new(symbol_ptr);
  println!("root: {:p}", root);

  let _root_container = RootContainer::new(root);

  let mut last_node = root;

  for _ in 1..=10000 {
    let node_ptr = allocate_dag_node();
    let node_mut = match unsafe { node_ptr.as_mut() } {
      None => {
        panic!("allocate_dag_node returned None");
      }
      Some(node) => {
        node
      }
    };
    node_mut.theory_tag = DagNodeTheory::Free;
    let node_ptr = DagNodeCore::upgrade(node_ptr);
    unsafe {
      (&mut*last_node).insert_child(node_ptr);
    }
    last_node     = node_ptr;
  }

}

