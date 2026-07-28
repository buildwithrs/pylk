// binary tree
struct BinaryTree {
    root: i64,
    left: Node,
    right: Node,
}

struct Node {
    val: i64,
    left: Box<Node>,
    right: Box<Node>,
}

fn main() {}
