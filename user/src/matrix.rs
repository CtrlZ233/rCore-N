use alloc::{vec::Vec, string::ToString};
use alloc::vec;
use alloc::sync::Arc;
use alloc::string::String;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use spin::Mutex;
use lazy_static::*;


lazy_static! {
    static ref RNG: Mutex<XorShiftRng> = Mutex::new(XorShiftRng::seed_from_u64(0x1020304050607080u64));
}

pub type Matrix<const N: usize> = [[u64; N]; N];

pub fn matrix_mul_test(n: usize) {
    let mut a = vec![vec![0_u64; n]; n];
    for i in 0..n
    {
        for j in 0..n
        {
            a[i][j] = RNG.lock().next_u64() % 1000;
        }
    }

    let _result = matrix_multiply2(n, &a, &a);
}

pub fn matrix_multiply2(n: usize, a1: &[Vec<u64>], a2: &[Vec<u64>]) -> Vec<Vec<u64>>
{
    let mut result = vec![vec![0; n]; n];
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

pub fn matrix_multiply<const N: usize>(a1: Arc<Matrix<N>>, a2: Arc<Matrix<N>>) -> Arc<Matrix<N>>
{
    let mut result = [[0_u64; N]; N];
    for i in 0..N
    {
        for j in 0..N
        {
            for k in 0..N
            {
                result[i][j] += a1[i][k] * a2[k][j];
            }
        }
    }
    return Arc::new(result);
}


pub fn print_matrix<const N: usize>(matrix: Arc<Matrix<N>>) {
    for vec in matrix.iter() {
        for elem in vec.iter() {
            print!("{} ", elem);
        }
        println!("");
    }
}

pub fn matrix_to_string<const N: usize>(matrix: Arc<Matrix<N>>) -> String {
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

pub fn string_to_matrix<const N: usize>(matrix: &String) -> Arc<Matrix<N>> {
    let mut ans = [[0_u64; N]; N];
    let vec_string: Vec<&str> = matrix.split(" ").collect();
    assert_eq!(N * N, vec_string.len());
    for i in 0..N {
        for j in 0..N {
            ans[i][j] = vec_string[i * N + j].parse::<u64>().unwrap();
        }
    }
    Arc::new(ans)
}
