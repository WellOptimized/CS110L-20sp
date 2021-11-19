/* The following exercises were borrowed from Will Crichton's CS 242 Rust lab. */

use std::collections::HashSet;

fn main() {
    println!("Hi! Try running \"cargo test\" to run tests.");
}

fn add_n(v: Vec<i32>, n: i32) -> Vec<i32> {
    let mut _v=v;
    let first_element=_v.get_mut(0);
    match first_element{
        Some(i) => *i+=n, 
        None=> (),
    }; 
    _v
}

fn add_n_inplace(v: &mut Vec<i32>, n: i32) {
    let first_element=v.get_mut(0);
    match first_element{
        Some(i) => *i+=n,
        None=> (),
    }; 
}

// 有提示要用 HashSet。。一开始没看见
fn dedup(v: &mut Vec<i32>) {
    let mut set:HashSet<i32>=HashSet::new();
    // 下面这个例子其实也是说明了remove会导致迭代器失效，因此remove的参数是index，而不是迭代器，干脆让你断了用迭代器进行remove的念头
    // for i in v{
    //     if !set.contains(i){
    //         set.insert(*i);
    //     }else{
    //         v.remove(i);
    //     }
    // }
    let mut i=0;
    while i< v.len(){
        if !set.contains(&v[i]){
            set.insert(v[i]);
            i+=1;
        }else{
            v.remove(i);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_n() {
        assert_eq!(add_n(vec![1], 2), vec![3]);
    }

    #[test]
    fn test_add_n_inplace() {
        let mut v = vec![1];
        add_n_inplace(&mut v, 2);
        assert_eq!(v, vec![3]);
    }

    #[test]
    fn test_dedup() {
        let mut v = vec![3, 1, 0, 1, 4, 4];
        dedup(&mut v);
        assert_eq!(v, vec![3, 1, 0, 4]);
    }
}
