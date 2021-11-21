use grid::Grid; // For lcs()
use std::env;

use std::io::prelude::*;
use std::io::{self, BufReader}; // For read_file_lines()
use std::fs::File; // For read_file_lines()
use std::collections::HashSet;
use std::process;

pub mod grid;

/// Reads the file at the supplied path, and returns a vector of strings.s
fn read_file_lines(filename: &String) -> Result<Vec<String>, io::Error> {
    let f = File::open(filename)?;
    let f = BufReader::new(f);
    let mut _v:Vec<String>=Vec::new();
    for line in f.lines() {
        match line{
            Ok(_i)=>_v.push(_i),
            Err(e)=>return Err(e),
        };
    }

    Ok(_v)
}

// 求的是最长公共子序列，而不是最长公共子串
fn lcs(seq1: &Vec<String>, seq2: &Vec<String>) -> Grid {
    // Note: Feel free to use unwrap() in this code, as long as you're basically certain it'll
    // never happen. Conceptually, unwrap() is justified here, because there's not really any error
    // condition you're watching out for (i.e. as long as your code is written correctly, nothing
    // external can go wrong that we would want to handle in higher-level functions). The unwrap()
    // calls act like having asserts in C code, i.e. as guards against programming error.
    let m=seq1.len();
    let n=seq2.len();
    let mut grid = Grid::new(m,n);   

    for i in 0..m{
        let c1= & seq1[i];
        let c2= & seq2[0];
        if c1 == c2 {
            match grid.set(i,0,1){
                Ok(_)=>(),
                Err(_)=>(),
            };
        }else{
            if i>0{
                let _r:usize= match grid.get(i-1, 0){
                    Some(_i)=>_i,
                    None=>0,
                };
                match grid.set(i,0, _r){
                    Ok(_)=>(),
                    Err(_)=>(),
                };
            }
        }
    }
    for j in 0..n{
        let c1= & seq1[0];
        let c2= & seq2[j];
        if c1 == c2 {
            match grid.set(0,j,1){
                Ok(_)=>(),
                Err(_)=>(),
            };
        }else{
            if j>0{
                let _r:usize= match grid.get(0, j-1){
                    Some(_i)=>_i,
                    None=>0,
                };
                match grid.set(0,j, _r){
                    Ok(_)=>(),
                    Err(_)=>(),
                };
            }
        }
    }

    for i in 1..m{
        let c1=& seq1[i];
        for j in 1..n {
            let c2= & seq2[j];
            if c1 == c2 {
                let res:usize = match grid.get(i-1, j-1){
                    Some(_i)=>_i+1,
                    None=>1,
                };
                match grid.set(i,j,res){
                    Ok(_)=>(),
                    Err(_)=>(),
                };
            } else {
                let res1:usize = match grid.get(i-1, j){
                    Some(_i)=>_i,
                    None=>0,
                };
                let res2:usize = match grid.get(i, j-1){
                    Some(_i)=>_i,
                    None=>0,
                };
                if res1>res2{
                    match grid.set(i,j,res1){
                        Ok(_)=>(),
                        Err(_)=>(),
                    };
                }else{
                    match grid.set(i,j,res2){
                        Ok(_)=>(),
                        Err(_)=>(),
                    };
                }
            }
        }
    }
    grid
}

fn helper(lcs_table: &Grid, lines1: &Vec<String>, lines2: &Vec<String>, i: usize, j: usize,lcs_in_1: &mut HashSet<usize>,lcs_in_2: &mut HashSet<usize>){
    if i==0 && j==0{
        if lines1[i]==lines2[j]{
            lcs_in_1.insert(i);
            lcs_in_2.insert(j);
        }
        return
    }else if i==0{
        if lines1[i]==lines2[j]{
            lcs_in_1.insert(i);
            lcs_in_2.insert(j);
        }
        helper(lcs_table, lines1, lines2, i, j-1, lcs_in_1, lcs_in_2);
        return 
    }else if j==0{
        if lines1[i]==lines2[j]{
            lcs_in_1.insert(i);
            lcs_in_2.insert(j);
        }
        helper(lcs_table, lines1, lines2, i-1, j, lcs_in_1, lcs_in_2);
        return
    }
    if lines1[i]==lines2[j]{
        lcs_in_1.insert(i);
        lcs_in_2.insert(j);
        helper(lcs_table, lines1, lines2, i-1, j-1, lcs_in_1, lcs_in_2);
    }else if lcs_table.get(i-1,j) > lcs_table.get(i,j-1){
        helper(lcs_table, lines1, lines2, i-1, j, lcs_in_1, lcs_in_2);
    }else{
        helper(lcs_table, lines1, lines2, i, j-1, lcs_in_1, lcs_in_2);
    }
}

fn delete_index_in_old(v1_helper:&Vec<usize>,lcs_in_2: &HashSet<usize>,v2_helper:&Vec<usize>,new_index:usize) ->usize{
    let mut k= new_index as i32 -1 ;
    while k>=0{
        if lcs_in_2.contains(&(k as usize)){ 
            for (index, value) in v2_helper.iter().enumerate() {
                if *value == k as usize{
                    return v1_helper[index]+1;
                }
            }
        }
        k-=1;
    }
    return 0;
}

fn print_diff(lcs_table: &Grid, lines1: &Vec<String>, lines2: &Vec<String>, i: usize, j: usize) {
    let mut lcs_in_1:HashSet<usize>=HashSet::new();
    let mut lcs_in_2:HashSet<usize>=HashSet::new();
    helper(lcs_table, lines1, lines2, i-1, j-1, &mut lcs_in_1 ,&mut lcs_in_2);
    println!("{:?}",lcs_in_1);
    println!("{:?}",lcs_in_2);
    
    let mut delete_vec:Vec<usize>=Vec::new();
    let mut add_vec:Vec<usize>=Vec::new();

    for m in 0..i{
        if !lcs_in_1.contains(&m){
            delete_vec.push(m);
        }
    }
    for n in 0..j{
        if !lcs_in_2.contains(&n){
            add_vec.push(n);
        }
    }

    let mut v1_helper=Vec::new();
    let mut v2_helper=Vec::new();
    for m in &lcs_in_1{
        v1_helper.push(*m);
    }
    for n in &lcs_in_2{
        v2_helper.push(*n);
    }
    v1_helper.sort();
    v2_helper.sort();

    for m in &delete_vec{    
        println!("- {} {}",m ,lines1[*m]);
    }
    for n in &add_vec{     // 1,4,6
        let tmp=delete_index_in_old(&v1_helper,&lcs_in_2,&v2_helper,*n);
        println!("+ {} {}",tmp,lines2[*n]);
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename1 = &args[1];
    let filename2 = &args[2];
    println!("{}",filename1);
    println!("{}",filename2);

    let _v1=match read_file_lines(filename1){
        Ok(_i) => _i,
        Err(_e) => return (),
    };
    let _v2=match read_file_lines(filename2){
        Ok(_i) => _i,
        Err(_e) => return (),
    };
    if _v1.len()==0 && _v2.len()==0{
        return ()
    }else if _v1.len()==0 && _v2.len()>0{
        for i in _v2{
            println!("+ {} {}",0,i);
        }
        return ()
    }else if _v1.len()>0 && _v2.len()==0{
        for i in _v1{
            println!("- {} {}",0,i);
        }
        return ()
    }

    let grid=lcs(&_v1,&_v2);
    println!("{:?}",grid.display());

    print_diff(&grid, &_v1, &_v2,  _v1.len(), _v2.len());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_file_lines() {
        let lines_result = read_file_lines(&String::from("handout-a.txt"));
        assert!(lines_result.is_ok());
        let lines = lines_result.unwrap();
        assert_eq!(lines.len(), 8);
        assert_eq!(
            lines[0],
            "This week's exercises will continue easing you into Rust and will feature some"
        );
    }

    #[test]
    fn test_lcs() {
        let mut expected = Grid::new(4, 3);
        expected.set(0, 0, 1).unwrap();
        expected.set(0, 1, 1).unwrap();
        expected.set(0, 2, 1).unwrap();
        expected.set(1, 0, 1).unwrap();
        expected.set(1, 1, 1).unwrap();
        expected.set(1, 2, 2).unwrap();
        expected.set(2, 0, 1).unwrap();
        expected.set(2, 1, 1).unwrap();
        expected.set(2, 2, 2).unwrap();
        expected.set(3, 0, 1).unwrap();
        expected.set(3, 1, 2).unwrap();
        expected.set(3, 2, 2).unwrap();

        println!("Expected:");
        expected.display();
        let result = lcs(
            &"abcd".chars().map(|c| c.to_string()).collect(),
            &"adb".chars().map(|c| c.to_string()).collect(),
        );
        println!("Got:");
        result.display();
        assert_eq!(result.size(), expected.size());
        for row in 0..expected.size().0 {   // 0..4 
            for col in 0..expected.size().1 { // 0..3
                assert_eq!(result.get(row, col), expected.get(row, col));
            }
        }
    }
}
