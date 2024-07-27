//! Just some practice implementations of some different structures to get a feeling for using the API

/// A simple binary tree implementation, with no backreferences.
pub mod binary_tree {
    use std::cell::Cell;

    use crate::Corrida;
    /// A node in the tree, does not have a reference to its parent
    pub struct BinaryTreeNode<'a, T: Copy> {
        /// Data associated with node
        pub data: Cell<T>,
        /// Reference to left child
        pub left: Cell<Option<&'a BinaryTreeNode<'a, T>>>,
        /// Reference to right child
        pub right: Cell<Option<&'a BinaryTreeNode<'a, T>>>
    }

    impl<T: Copy> BinaryTreeNode<'_,T> {
        /// Creates a new node (not in the tree struct yet)
        pub fn new(data : T) -> Self {
            Self {
                data: Cell::new(data),
                left: Cell::new(None),
                right: Cell::new(None)
            }
        }
    }



    /// An API for construction and traversal of Binary Trees.
    /// No backreferences
    pub struct BinaryTree<'a> {
        nodes: Corrida,
    }

    impl<'a> BinaryTree<'a> {
        /// Creates a new binary tree.
        pub fn new() -> Self {
            Self {
                nodes: Arena::<BinaryTreeNode>::new(),
            }
        }

        /// Inserts a node
        pub fn insert_node(&'a self) -> &'a mut BinaryTreeNode<'a> {
            self.nodes.alloc(BinaryTreeNode::new(()))
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_line() {
            let mut tree = BinaryTree::new();

            // Make Root

            for i in 0..1_000_000 {
                let mut cur = tree.insert_node();
                let child = tree.insert_node();
                //cur.set_right(child);
                cur = child;
            }
        }
    }
}

