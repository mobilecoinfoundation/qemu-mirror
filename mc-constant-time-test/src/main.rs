use rand_core::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

fn check_bad(a: &[u32], b: &[u32]) -> Choice {
    for i in 0..(a.len() - 1) {
        if a[i] != b[i] {
            return Choice::from(0);
        }
    }
    return Choice::from(1);
}

fn check_good(a: &[u32], b: &[u32]) -> Choice {
    a.ct_eq(b)
}

fn main_old() {
    let mut a = [0; 16];
    let b = [0; 16];
    a[0] = std::env::args().nth(1).unwrap().parse().unwrap();

    println!("bad:{:?}", check_bad(&a, &b));
    println!("good:{:?}", check_good(&a, &b));
}

fn main() {
    let words = ["zero", "one"];
    let mut a = [0u32; 2];
    let b = [0u32; 2];
    let mut rng = Hc128Rng::from_seed([42u8; 32]);
    a[0] = rng.next_u32();
    let result_good = check_good(&a, &b);
    let result_bad = check_bad(&a, &b);
    //println!("result_good:{:?}", result_good);
    //println!("result_bad:{:?}", result_bad);
    // let word = words[u8::conditional_select(&0u8, &1u8, result) as usize];
    // println!("word:{:?}", word);
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_core::{RngCore, SeedableRng};
    use rand_hc::Hc128Rng;

    #[test]
    fn test_bad() {
        let mut rng = Hc128Rng::from_seed([7u8; 32]);
        let mut a = [0; 2];
        let b = [0; 2];
        a[0] = rng.next_u32();
        let result = check_bad(&a, &b);
        let out = u8::conditional_select(&0u8, &1u8, result);
        println!("bad:{:?}", out);
    }
    #[test]
    fn test_good() {
        let mut rng = Hc128Rng::from_seed([7u8; 32]);
        let mut a = [0; 2];
        let b = [0; 2];
        a[0] = rng.next_u32();
        let result = check_good(&a, &b);
        let out = u8::conditional_select(&0u8, &1u8, result);
        println!("bad:{:?}", out);
    }
}
