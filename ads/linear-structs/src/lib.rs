pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
mod linked_list;

#[cfg(test)]
mod tests {
    //use crate::linked_list::LinkedList;

    use super::*;

    #[test]
    fn it_works() {
        let i = 10;
        let result = add(i, 2);
        assert_eq!(result, 12);
        //let ll: LinkedList<i32> = LinkedList::new();
    }
}
