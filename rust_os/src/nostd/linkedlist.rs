// Eine ganz normale, einfach gelinkte Liste.
// https://rust-unofficial.github.io/too-many-lists/first-push.html

struct LinkedList<'a, T> {
    next: &'a mut LinkedList<'a,T>,
    value: Option<T>, 
}

impl <'a, T> LinkedList<'a, T> {
    fn is_head(&self)->bool{
        match self.value {
            None => true,
            Some(_)=>false
        }
    }
    /// Push tut den Wert nach der Node
    fn push(&'a mut self, value: T) -> LinkedList<T> {
        let mut node = LinkedList {
            next: self.next,
            value: Some(value),
        };
        self.next = &mut node;
        node
    }
}
