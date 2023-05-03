use alloc::{vec::Vec, string::ToString};
use alloc::vec;
use alloc::string::String;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use spin::Mutex;
use lazy_static::*;


lazy_static! {
    static ref RNG: Mutex<XorShiftRng> = Mutex::new(XorShiftRng::seed_from_u64(0x1020304050607080u64));
}

pub fn matrix_mul_test(n: usize) {
    let mut a = vec![vec![0_u64; n]; n];
    for i in 0..n
    {
        for j in 0..n
        {
            a[i][j] = RNG.lock().next_u64() % 1000;
        }
    }

    let _result = matrix_multiply(n, &a, &a);
}

pub fn matrix_multiply(n:usize, a1: &[Vec<u64>], a2: &[Vec<u64>]) -> Vec<Vec<u64>>
{
    let mut result = vec![vec![0_u64; n]; n];
    for i in 0..n
    {
        for j in 0..n
        {
            for k in 0..n
            {
                result[i][j] += a1[i][k] * a2[k][j];
            }
        }
    }
    return result;
}

pub fn get_matrix(n: usize) -> Vec<Vec<u64>> {
    let mut a = vec![vec![0_u64; n]; n];
    for i in 0..n
    {
        for j in 0..n
        {
            a[i][j] = RNG.lock().next_u64() % 1000;
        }
    }
    a
}

pub fn print_matrix(matrix: &Vec<Vec<u64>>) {
    for vec in matrix.iter() {
        for elem in vec.iter() {
            print!("{} ", elem);
        }
        println!("");
    }
}

pub fn matrix_to_string(matrix: &Vec<Vec<u64>>) -> String {
    let mut ans = String::new();
    for vec in matrix.iter() {
        for elem in vec.iter() {
            ans += &elem.to_string();
            ans += " ";
        }
    }
    ans.pop();
    ans
}

pub fn string_to_matrix(n: usize, matrix: &String) -> Vec<Vec<u64>> {
    let mut ans = vec![vec![0_u64; n]; n];
    let vec_string: Vec<&str> = matrix.split(" ").collect();
    // assert!(n * n ==  vec_string.len());
    println!("{} vs {}", n * n, vec_string.len());
    for i in 0..n {
        for j in 0..n {
            ans[i][j] = vec_string[i * n + j].parse::<u64>().unwrap();
        }
    }
    ans
}
