//! Just some practice implementations of some different structures to get a feeling for using the API

/// A simple binary tree implementation, with no backreferences.
pub mod binary_tree {
    use crate::Arena;
    /// A node in the tree, does not have a reference to its parent
    pub struct BinaryTreeNode<'a> {
        /// Data associated with node
        pub data: (),
        /// Reference to left child
        pub left: Option<&'a BinaryTreeNode<'a>>,
        /// Reference to right child
        pub right: Option<&'a BinaryTreeNode<'a>>
    }
    impl BinaryTreeNode<'_> {
        /// Creates a new node (not in the tree struct yet)
        /// TODO: replace () with generic type
        pub fn new(data : ()) -> Self {
            Self {
                data,
                left: None,
                right: None
            }
        }
    }



    /// An API for construction and traversal of Binary Trees.
    /// No backreferences
    pub struct BinaryTree<'a> {
        nodes: Arena<BinaryTreeNode<'a>>,
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
                cur.set_right(child);
                cur = child;
            }
        }
    }
}

