use crate::arena::*;
use std::pin::Pin;

type Link<'arena,T> = Option<Pin<&'arena mut TreeNode<'arena, T>>>;

struct TreeNode<'arena, T: Unpin> {
    pub val: T,
    left: Link<'arena, T>,
    right: Link<'arena, T>,
    parent: Link<'arena, T>,
}

impl<T:Unpin> TreeNode<'_, T> {
    fn new(val: T) -> Self {
        Self {
            val,
            left: None,
            right: None,
            parent: None
        }
    }
    
}

// impl<T: Unpin> Unpin for TreeNode<'_, T> {}

/// Provides an API for construction of binary tree.
pub struct BinaryTree<'arena, T: Unpin> {
    arena: Arena<TreeNode<'arena, T>>,
    root: Link<'arena, T>
}

impl<'arena, T: Unpin> BinaryTree<'arena, T> {
    /// Creates a new binary tree with no nodes, but an arena ready for allocation
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            root: None,
        }
    }

    pub fn alloc_node(&self, val: T) -> Pin<&mut TreeNode<'arena, T>> {
        Pin::new(self.arena.alloc(
            TreeNode::<'arena,T>::new(val)
        ))
    }

    pub fn alloc_connected_node(&self, val: T, parent: Link<'arena,T>, left: Link<'arena,T>, right: Link<'arena,T>) -> Pin<&'arena mut TreeNode<'_, T>> {
        Pin::new(self.arena.alloc(TreeNode::<'arena,T> {
            val,
            left,
            right,
            parent
        }))

    }
}


mod test {
    use super::BinaryTree;
    use std::pin::Pin;

    #[test]
    fn test_binary_tree() {
        let tree = BinaryTree::<i32>::new();

        {
            let node1 = tree.alloc_node(1);
            let node2 = tree.alloc_node(2);

            let node3 = tree.alloc_connected_node(0, None, Some(node1), Some(node2));
        }
    }
}