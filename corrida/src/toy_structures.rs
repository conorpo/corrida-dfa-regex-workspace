//! Just some practice implemenations of some different structures to get a feeling for using the API

use super::Arena;

mod binary_tree {
    use crate::Arena;
    pub struct Node<'a> {
        pub data: (),
        pub left: Option<&'a Node<'a>>,
        pub right: Option<&'a Node<'a>>
    }

    impl<'a> Node<'a> {
        pub fn set_right(&'a mut self, other: &'a Node<'a>) {
            self.right = Some(other as &'a Self);
        }
    }

    impl Node<'_> {
        pub fn new(data : ()) -> Self {
            Self {
                data,
                left: None,
                right: None
            }
        }
    }



    pub struct BinaryTree<'a> {
        nodes: Arena<Node<'a>>,
    }

    impl<'a> BinaryTree<'a> {
        pub fn new() -> Self {
            Self {
                nodes: Arena::<Node>::new(),
            }
        }

        pub fn insert_node(&'a self) -> &'a mut Node<'a> {
            self.nodes.alloc(Node::new(()))   
        }
    }
}


mod test {
    use super::binary_tree::*;

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