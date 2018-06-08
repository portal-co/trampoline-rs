#[macro_use]
extern crate tramp;

use tramp::*;

fn factorial(n: u128) -> u128 {
    fn fac_acc(n: u128, acc: u128) -> Rec<u128> {
        if n > 1 {
            rec_call!(fac_acc(n - 1, acc * n))
        } else {
            rec_ret!(acc)
        }
    }

    trampolin!(fac_acc(n, 1))
}

#[test]
fn test_fac() {
    assert_eq!(factorial(5), 120);
}
