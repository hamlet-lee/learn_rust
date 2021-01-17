// reference 1 : https://stackoverflow.com/questions/54613966/error-reached-the-recursion-limit-while-instantiating-funcclosure
// reference 2 : https://stevedonovan.github.io/rustifications/2018/08/18/rust-closures-are-hard.html

struct TreeNode {
    val: u64,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>
}

struct Tree {
    root: Option<Box<TreeNode>>,
}

impl Tree {
    fn new() -> Tree {
        Tree {root: None}
    }

    fn insert(&mut self, val: u64) {
        match &mut self.root {
            None => {
                self.root = Some(Box::new(TreeNode{val: val, left:None, right:None}));
            },
            Some(n) => {
                Tree::insert_inner(n, val);
            }
        }
    }

    fn insert_inner (tn: &mut TreeNode, val: u64) {
        let to_insert: &mut Option<Box<TreeNode>>;

        if val < tn.val {
            to_insert = &mut tn.left;
        } else {
            to_insert = &mut tn.right;
        }

        match to_insert {
            Some(n) => {
                Tree::insert_inner(n, val);
            },
            None => {
                *to_insert = Some(Box::new(TreeNode{val: val, left:None, right:None}));
            }
        }
    }

    // 需要加上 Copy Trait 的限定！否则会报错
    fn traverse<F>(&self, cb: F ) where F: Fn(u64)  + Copy {
        if let Some(tn) = &self.root {
            Tree::traverse_inner(tn, cb);
        }
    }

    // 需要加上 Copy Trait 的限定！否则会报错
    fn traverse_inner<F> (tn: &TreeNode, cb: F ) where F: Fn(u64) + Copy {
        match &tn.left {
            Some(tleft) => {
                Self::traverse_inner(&tleft, cb);
            },
            None => {}
        }

        cb (tn.val);

        match &tn.right {
            Some(tright) => {
                Self::traverse_inner(&tright, cb);
            },
            None => {}
        }
    }

    // 需要加上 Copy Trait 的限定！否则会报错
    fn traverse2<F: FnMut(u64)>(&self, mut cb: F ) {
        if let Some(tn) = &self.root {
            Tree::traverse_inner2(tn, cb);
        }
        // cb (1);
        // cb (2);
        // cb (3);
    }

    // 需要加上 Copy Trait 的限定！否则会报错
    fn traverse_inner2 (tn: &TreeNode, mut cb: impl FnMut(u64) ) {
        // if let Some(tleft) = &tn.left {
        //     Tree::traverse_inner2(tleft, cb);
        // };
        // println!("calling ");
        cb (tn.val);

        // if let Some(tright) = &tn.right {
        //     Tree::traverse_inner2(tright, cb);
        // };
    }
}

fn main() {
    let mut t = Tree::new();
    t.insert(1);
    t.insert(3);
    t.insert(2);
    t.insert(6);

    // t.traverse(|x| { println!("{}", x)});

    let mut sum = 0;
    // 需要加上 move，否则会报错
    t.traverse2(Box::new(|x| { sum +=x; println!("add {} produces {}", x, sum);}));
    println!("final sum = {}", sum);
}
