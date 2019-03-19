use crate::condition::Condition;
use tokyo::{
    analyzer::Analyzer,
    behavior::{Behavior, Noop},
};

#[derive(Debug)]
pub struct Strategy {
    tree: StrategyNode,
}

impl Strategy {
    pub fn new(branches: Vec<(Box<Condition>, Box<StrategyNode>)>) -> Self {
        Self { tree: StrategyNode::Branch(branches) }
    }

    pub fn next_behavior(&mut self, analyzer: &Analyzer) -> Option<PrioritizedBehavior> {
        self.tree.next_behavior(analyzer)
    }
}

#[derive(Debug)]
pub enum StrategyNode {
    Branch(Vec<(Box<Condition>, Box<StrategyNode>)>),
    Leaf(PrioritizedBehavior),
}

impl StrategyNode {
    pub fn next_behavior(&mut self, analyzer: &Analyzer) -> Option<PrioritizedBehavior> {
        match self {
            StrategyNode::Branch(nodes) => {
                for (condition, node) in nodes.iter_mut() {
                    if condition.evaluate(analyzer) {
                        return node.next_behavior(analyzer);
                    }
                }
                None
            },
            StrategyNode::Leaf(leaf) => Some(leaf.clone()),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
}

// TODO: Replace with a pair.
#[derive(Clone, Debug)]
pub struct PrioritizedBehavior {
    pub priority: Priority,
    pub behavior: Box<Behavior>,
}

impl PrioritizedBehavior {
    pub fn new() -> Self {
        Self { priority: Priority::Low, behavior: Box::new(Noop {}) }
    }

    pub fn with_low<T: Behavior>(behavior: T) -> Self {
        Self { priority: Priority::Low, behavior: behavior.box_clone() }
    }

    pub fn with_medium<T: Behavior>(behavior: T) -> Self {
        Self { priority: Priority::Medium, behavior: behavior.box_clone() }
    }

    pub fn with_high<T: Behavior>(behavior: T) -> Self {
        Self { priority: Priority::High, behavior: behavior.box_clone() }
    }
}
