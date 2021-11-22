use std::fmt;
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

pub struct LinkedListIter<'a,T> {
    current: &'a Option<Box<Node<T>>>,
}


impl<T:Clone> Iterator for LinkedListIter<'_,T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.current {
            Some(node) =>  {
                self.current=&node.next;
                Some(node.value.clone())
            },
            None => None,
        }
    }
}

impl<'a,T:Clone> IntoIterator for &'a LinkedList<T> {
    type Item = T;
    type IntoIter = LinkedListIter<'a,T>;
    
    fn into_iter(self) -> LinkedListIter<'a,T> {
        LinkedListIter {current: &self.head}
    }
}
 
impl<T:Copy> Clone for Node<T>{
    fn clone(&self) -> Node<T>{
        Node::new(self.value,self.next.clone())
    }
}

impl<T:Copy> Clone for LinkedList<T> {
    fn clone(&self) -> LinkedList<T>{
        let mut list=LinkedList::new();
        list.head=self.head.clone();
        list.size=self.size;
        list
    }
}

impl<T:PartialEq> PartialEq for LinkedList<T>{
    fn eq(&self,other:&Self) -> bool{
        if self.size!=other.size{
            return false
        }
        let mut p=&self.head;
        let mut q=&other.head;
            
        while let (Some(_p),Some(_q)) = (p,q){
            if _p.value==_q.value {
                p=&_p.next;
                q=&_q.next;
            }else{
                return false
            }
        }
        true
    }
}

// impl<T> Iterator for LinkedList<T>{
//     type Item= T;
//     fn next(&mut self) -> Option<T>{
//         self.pop()
//     }
// }

pub trait ComputeNorm{
    fn compute_norm(&self) -> f64{
        0.0
    }
}

impl ComputeNorm for LinkedList<f64> {
    fn compute_norm(&self) -> f64{
        let mut cur = &self.head;
        let mut norm=0.0;
        while let Some(_cur) = cur {
            norm+=_cur.value * _cur.value;
            cur=&_cur.next;
        }
        norm.sqrt()
    }
}


impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn empty(&self) -> bool {
        self.get_size() == 0
    }
    
    pub fn push(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }
    
    pub fn pop(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}


impl<T: fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}



