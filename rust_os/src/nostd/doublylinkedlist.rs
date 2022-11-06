/*

Doubly linked lists are not possible in Rust, because that means that several 
nodes have mutable references to one node, which is not allowed in Rust. 
However, this could be a fix (using Rc *and* RefCell)
https://rtoch.com/posts/rust-doubly-linked-list/
*/

// https://doc.rust-lang.org/rust-by-example/custom_types/enum/testcase_linked_List.html
// https://dev.to/arunanshub/self-referential-structs-in-rust-33cm#solution
// https://medium.com/swlh/implementing-a-linked-List-in-rust-c25e460c3676

/*
This is a doubly-linked Node. 
It has a head that is simultaneously the tail.
The Empty List is just a single node pointing to itself. 
*/
#![allow(dead_code)]

// ouroboros
#[derive(PartialEq)]
struct Node<'a, T> where T: Copy {
    value: Option<T>,
    _next: Option<&'a mut Node<'a, T>>,
    _prev: Option<&'a Node<'a, T>>,
}

impl <'a, T> Node<'a,T> where T: PartialEq + Copy {
    fn new() -> Node<'a, T> {
        // I would like head to point to itself, but that doesn't work.
        let mut head: Node<T> = Node {
            value: None,
            _next: None,
            _prev: None
        };
        head._next = Some(&mut head);
        head._prev = Some(& head);
        head
    }
    #[allow(unused_variables)]
    fn from_array(array: &'a[T]) -> Node<T> {
        let head = Self::new();
        // TODO
        head
    }
    fn node(value: T) -> Node<'a,T> {
        Node {
            value: Some(value),
            _next: None,
            _prev: None,
        }
    }
    fn next(&'a mut self) -> &mut Node<T> {
        match self._next {
            Some(node) => node,
            None => self
        }
    }
    fn prev(&'a self) -> &Node<T> {
        match self._prev {
            Some(node) => node,
            None => self
        }
    }
    fn is_empty(&self) -> bool {
        (self.value, self._next, self._prev) == (None, None, None)
    }
    fn is_head(&self) -> bool {
        self.value == None
    }
    fn is_tail(&self) -> bool {
        self.next().is_head()
    }
    fn find(&self) -> &Node<T> {
        self
    }
    fn find_head(&self) -> &Node<T> {
        if self.is_head() {
            return self;
        }
        self.next().find_head()
    }
    fn find_tail(&self) -> &Node<T> {
        if self.is_head() {
            self.prev()
        } else if self.is_tail() {
            self
        } else {
            self.next().find_tail()
        }
    }
    fn insert(&'a mut self, node: &'a mut Node<'a,T>) {
        node._next = Some(self.next());
        node._prev = Some(self);
        self.next()._prev = Some(node);
        self._next = Some(node);
    }
    #[allow(unused_variables)]
    // adds a Node to the end of the list
    fn push(value: T) {

    }
}