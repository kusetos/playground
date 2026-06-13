// pub struct LinkedList<T> {
//     head: Node<T>,
//     len: usize,
// }
// pub struct Node<T> {
//     next: Option<Box<T>>,
// }

// impl<T> Node<T> {
//     pub fn new() -> Self {
//         Self { next: None }
//     }
//     pub fn push(head: &mut Node<T>) {
//         let mut tail = head;
//         while tail.next.is_some() {
//             tail = tail.next;
//         }
//     }
// }

// impl<T> LinkedList<T>
// // where
// //     T: Copy,
// {
//     pub fn new() -> Self {
//         Self {
//             head: Node::new(),
//             len: 0,
//         }
//     }

//     pub fn push(&mut self, val: T) {}
// }

#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}
fn main() {}

#[cfg(test)]
pub mod tests {
    use crate::linked_list::List;

    #[test]
    pub fn create() {
        let list = List::Cons(10, Box::new(List::Cons(20, Box::new(List::Nil))));
        println!("{:?}", list);
    }
    #[test]
    pub fn InsertTest() -> () {
        if 1 != 0 {
            return;
        }
    }
    pub fn DeleteTest() {}
    pub fn GetTest() {}
}
