use crate::core::Node;

/// LocalNode is a trait that represents a single node in a local skip graph.
trait LocalNode: Node<Address = &Self> {}
