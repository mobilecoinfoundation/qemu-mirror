use subtle::{Choice, ConstantTimeEq};

fn check_bad(a: &[u32], b: &[u32]) -> Choice{
  for i in 0..(a.len()-1) {
    if a[i] != b[i]
    {
      return Choice::from(0);
    }
  }
  return Choice::from(1);
}

fn check_good(a: &[u32], b: &[u32]) -> Choice {
    a.ct_eq(b)
}

fn main() {
    let mut a = [0; 16];
    let b = [0; 16];
    a[0] = std::env::args().nth(1).unwrap().parse().unwrap();

    
    println!("bad:{:?}", check_bad(&a, &b));
    println!("good:{:?}", check_good(&a, &b));
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_hc::Hc128Rng;
    use rand_core::{RngCore, SeedableRng};


    #[test]
    fn test_fixed() {
        let mut rng = Hc128Rng::from_seed([7u8; 32]);
        let mut a = [0; 16];
        let b = [0; 16];
        a[0] = rng.next_u32();

        println!("bad:{:?}", check_bad(&a, &b));
        println!("good:{:?}", check_good(&a, &b));
        assert_eq!(check_bad(&a, &b).unwrap_u8(), check_good(&a, &b).unwrap_u8());
    }
}