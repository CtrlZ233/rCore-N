use alloc::vec::Vec;
use alloc::vec;
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

    let result = matrix_multiply(n, &a, &a);
}

fn matrix_multiply(n:usize, a1: &[Vec<u64>], a2: &[Vec<u64>]) -> Vec<Vec<u64>>
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
