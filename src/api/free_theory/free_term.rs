use crate::{
  api::{
    dag_node::{DagNode, DagNodeFlag, DagNodePtr},
    term::{Term, TheoryTerm}
  }
};

pub struct FreeTerm {}

impl FreeTerm {
  pub fn new() -> Self {
    Self {}
  }
}

impl TheoryTerm for FreeTerm {
  fn dagify(&self, parent: &Term) -> DagNodePtr {
    DagNode::new(parent.symbol)
  }
}
