use linked_list::LinkedList;
pub mod linked_list;
use crate::linked_list::ComputeNorm;

fn main() {
    // let mut list: LinkedList<u32> = LinkedList::new();
    // assert!(list.is_empty());
    // assert_eq!(list.get_size(), 0);
    // for i in 1..12 {
    //     list.push_front(i);
    // }
    // println!("{}", list);
    // println!("list size: {}", list.get_size());
    // println!("top element: {}", list.pop_front().unwrap());
    // println!("{}", list);
    // println!("size: {}", list.get_size());
    // println!("{}", list.to_string()); // ToString impl for anything impl Display
    // // If you implement iterator trait:
    // //for val in &list {
    // //    println!("{}", val);
    // //}

    {   
        println!("***************** Create U32 LinkedList *****************");
        let mut u32_list = LinkedList::new();
        for i in 1..=10 {
            u32_list.push(i);
        }
        println!("LinkedList (size = {}): {}", u32_list.get_size(), u32_list);
        while !u32_list.empty() {
            u32_list.pop();
        }
        println!("LinkedList (size = {}): {}", u32_list.get_size(), u32_list);
    }

    // String LinkedList
    {
        println!("\n***************** Create String LinkedList *****************");
        let mut string_list = LinkedList::new();
        for _ in 1..=10 {
            string_list.push(String::from("abcd"));
        }
        println!("LinkedList (size = {}): {}", string_list.get_size(), string_list);
        while !string_list.empty() {
            string_list.pop();
        }
        println!("LinkedList (size = {}): {}", string_list.get_size(), string_list);
    }

    // for clone
    {
        println!("\n***************** Test Clone *****************");
        let mut list1 =  LinkedList::new();
        for i in 1..=10 {
            list1.push(i);
        }
        let mut list2 = list1.clone();
        println!("LinkedList (size = {}): {}", list1.get_size(), list1);
        println!("LinkedList (size = {}): {}", list2.get_size(), list2);
        println!("list2 pop {}", list2.pop().unwrap());
        println!("list2 pop {}", list2.pop().unwrap());
        println!("LinkedList (size = {}): {}", list1.get_size(), list1);
        println!("LinkedList (size = {}): {}", list2.get_size(), list2);

    }

    // for PartialEq
    {
        println!("\n***************** Test PartialEq *****************");
        let mut list1 =  LinkedList::new();
        for i in 1..=10 {
            list1.push(i);
        }
        let mut list2 = list1.clone();
        println!("LinkedList (size = {}): {}", list1.get_size(), list1);
        println!("LinkedList (size = {}): {}", list2.get_size(), list2);
        println!("list1 == list2: {}", list1==list2);
        list2.pop();
        println!("LinkedList (size = {}): {}", list1.get_size(), list1);
        println!("LinkedList (size = {}): {}", list2.get_size(), list2);
        println!("list1 == list2: {}", list1==list2);
        list2.push(100);
        println!("LinkedList (size = {}): {}", list1.get_size(), list1);
        println!("LinkedList (size = {}): {}", list2.get_size(), list2);
        println!("list1 == list2: {}", list1==list2);
        list2.pop();
        list2.push(10);
        println!("LinkedList (size = {}): {}", list1.get_size(), list1);
        println!("LinkedList (size = {}): {}", list2.get_size(), list2);
        println!("list1 == list2: {}", list1==list2);
    }

    // for ComputeNorm
    {
        println!("\n***************** Test ComputeNorm *****************");
        let mut list: LinkedList<f64> =  LinkedList::new();
        for i in 1..=10 {
            list.push(i as f64);
        }
        println!("list compute_norm = {}", list.compute_norm());
    }

    // for Iterator
    {
        println!("\n***************** Test Iterator and Iterator to reference*****************");
        let mut list: LinkedList<String> =  LinkedList::new();
        for i in 1..=10 {
            list.push(String::from(format!("{}",i)));
        }
        let list1=list;
        for e in list1.into_iter() {
            println!("{:?}", e);
        }
        let list2=&list1;
        for e in list2.into_iter() {
            println!("{:?}", e);
        }
    }
}
