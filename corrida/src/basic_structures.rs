//! Just some practice implementations of some different structures to get a feeling for using the API

/// A simple binary tree implementation, with no backreferences.
pub mod binary_tree {
    use std::cell::Cell;

    use crate::*;
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
    pub struct BinaryTree<'a, T: Copy> {
        nodes: Corrida::<4096>,
        root: Cell<Option<&'a BinaryTreeNode<'a, T>>>
    }

    impl<'a, T: Copy> BinaryTree<'a, T> 
    {
        /// Creates a new binary tree.
        pub fn new() -> Self {
            Self {
                nodes: Corrida::<4096>::new(),
                root: Cell::new(None)
            }
        }

        /// Inserts a node
        pub fn insert_node(&self, node_data: T) -> &mut BinaryTreeNode<'a, T> 
        where [(); {size_of::<BinaryTreeNode<'_, T>>() <= 4096} as usize]: ,
              [(); {BLOCK_MIN_ALIGN % align_of::<BinaryTreeNode<'_, T>>() == 0} as usize]: ,
        {
            self.nodes.alloc(BinaryTreeNode::new(node_data))
        }

        pub fn iter_in_order(&self) -> IterInOrder<'a, T> {
            let mut cur_path = Vec::new();

            let mut node_opt = self.root.get();

            while let Some(cur_node) = node_opt {
                cur_path.push((cur_node, false));
                node_opt = cur_node.left.get();
            } 
            
            IterInOrder {
                cur_path
            }
        }

        // pub fn iter_pre_order(&self) -> IterPreOrder<'a, T> {
        //     todo!();
        // }
    }

    struct IterInOrder<'a, T: Copy> {
        cur_path: Vec<(&'a BinaryTreeNode<'a, T>, bool)>, // Bool if we've already done that node

    }

    impl<'a, T:Copy> Iterator for IterInOrder<'a, T> {
        type Item = T;
    
        fn next(&mut self) -> Option<Self::Item> {
            if self.cur_path.is_empty() {
                return None;
            }

            let (node, visited) = self.cur_path.last_mut().unwrap();
            let data = node.data.get();

            *visited = true;

            if let Some(right) = node.right.get() {
                self.cur_path.push((right, false));

                // For inorder we need to make sure all left nodes are done first
                let mut cur = right;
                while let Some(left) = cur.left.get() {
                    self.cur_path.push((left, false));
                    cur = left;
                }


            } else {
                // This node and its children are done, go up until we get to the next unfinished node
                while let Some((_, true)) = self.cur_path.last() {
                    self.cur_path.pop();
                }
            }
            Some(data)
        }
        
    }


    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_right_line() {
            let mut tree = BinaryTree::new();

            // Make root
            let mut cur: &BinaryTreeNode<i32> = tree.insert_node(0);
            tree.root.set(Some(cur));

            // Make Root
            for i in 1..=1_000_000 {
                let child = tree.insert_node(i);
                cur.right.set(Some(child));
                cur = child;
            }

            let mut itr = tree.iter_in_order();

            assert_eq!(itr.next(), Some(0));
            assert_eq!(itr.last(), Some(1_000_000));
        }

        #[test]
        fn test_left_line() {
            let mut tree = BinaryTree::new();

            // Make root
            let mut cur: &BinaryTreeNode<i32> = tree.insert_node(0);
            tree.root.set(Some(cur));

            // Make Root
            for i in 1..=1_000_000 {
                let child = tree.insert_node(i);
                cur.left.set(Some(child));
                cur = child;
            }

            let mut itr = tree.iter_in_order();

            assert_eq!(itr.next(), Some(1_000_000));
            assert_eq!(itr.last(), Some(0)); 
        }
    }
}

