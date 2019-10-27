use crate::wu::parser::ast::Expression;
use std::collections::HashMap;

/// This guy stores which variables are available in a given scope.
/// Scopes have parents; if they don't know about a variable, they'll
/// tell you to ask their parent about it. That's why parent is stored here.
pub struct Context {
    pub map: HashMap<String, Expression>,
    /// An index into the array of Contexts Evaluator stores.
    /// There's always going to be one Context which doesn't have a parent,
    /// so it's stored as an Option for this one OG Context that raised itself.
    pub parent: Option<usize>,
}

impl Context {
    /// Create a new scope which is capable of storing new values,
    /// but also has the index of its parent so that all of its values
    /// (and the values of the parent's parent's, recursively) are accessible.
    pub fn empty_child(parent: usize) -> Self {
        Self {
            map: HashMap::new(),
            parent: Some(parent),
        }
    }
}
