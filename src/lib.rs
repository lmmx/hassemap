//! # HasseMap
//!
//! A hash map that maintains partial orders from batch updates.
//!
//! This data structure tracks partial order constraints established by batch updates
//! and maintains a linear extension that respects all constraints. When batch updates
//! are performed, the internal order is computed via topological sorting.
mod hasse_map;

pub use hasse_map::Poset;
