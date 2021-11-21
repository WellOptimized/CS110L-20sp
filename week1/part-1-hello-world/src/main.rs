fn main() {
    println!("Hello, world!");
    let mut s = String::from("hello");
    let s1=&mut s;

    println!("{} ",s1);
    s1.push_str("123");
    println!("{} ",s1);
}
