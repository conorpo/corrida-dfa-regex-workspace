//! Just some practice implementations of some different structures to get a feeling for using the API

/// A simple binary tree implementation, with no backreferences.
pub mod binary_tree {
    /// A node in the tree, does not have a reference to its parent
    pub struct BTree<'a, T: Copy> {
        /// Data associated with node
        pub data: T,
        /// Reference to left child
        pub left: Option<&'a BTree<'a, T>>,
        /// Reference to right child
        pub right: Option<&'a BTree<'a, T>>
    }

    impl<T: Copy> BTree<'_,T> {
        /// Creates a new node (not in the tree struct yet)
        pub fn new(data : T) -> Self {
            Self {
                data,
                left: None,
                right: None
            }
        }

        /// Returns an iterator that traverses the binary tree 'inorder'.
        pub fn iter_in_order(&self) -> IterInOrder<T> {
            IterInOrder {
                stack: vec![(self, false)]
            }
        }
    }

    /// An iterator that traverses the binary tree 'inorder'.
    pub struct IterInOrder<'a, T: Copy> {
        stack: Vec<(&'a BTree<'a, T>, bool)>, // Bool if we've already gotten the left child

    }

    impl<T:Copy> Iterator for IterInOrder<'_, T> {
        type Item = T;
    
        fn next(&mut self) -> Option<Self::Item> {
            if self.stack.is_empty() {
                return None;
            }

            let mut last = self.stack.last_mut().unwrap();
            while !last.1 {
                last.1 = true;
                if let Some(left) = last.0.left {
                    self.stack.push((left, false));
                    last = self.stack.last_mut().unwrap();
                }
            }

            let last = self.stack.pop().unwrap();

            if let Some(right) = last.0.right {
                self.stack.push((right, false));
            }

            Some(last.0.data)
        }
    }



    #[cfg(test)]
    mod test {
        use super::*;
        use crate::*;

        macro_rules! node_creator {
            ($d: tt, $macro_name: ident, $arena: expr, $type: ty) => {
                macro_rules! $macro_name {
                    ($data: expr) => {
                        $arena.alloc(<$type>::new($data))
                    };
                }
            };
        }
    

        #[test]
        fn test_line() {
            let arena = Corrida::new(None);
            node_creator!($, create_node, arena, BTree<i32>);

            let mut cur = create_node!(1_000_000);

            for i in (0..1_000_000).rev() {
                let parent = create_node!(i);
                parent.right = Some(cur);
                cur = parent;
            }

            let mut itr = cur.iter_in_order();

            assert_eq!(itr.next(), Some(0));
            assert_eq!(itr.last(), Some(1_000_000));
        }
    }
}

