#[macro_use]
extern crate tramp;


// Not the greatest way of computing "is even" or "is odd".
mod oddness {
    use tramp::Rec;

    fn is_even_rec(x: u128) -> Rec<bool> {
        if x > 0 {
            rec_call!(is_odd_rec(x - 1))
        } else {
            rec_ret!(true)
        }
    }

    fn is_odd_rec(x: u128) -> Rec<bool> {
        if x > 0 {
            rec_call!(is_even_rec(x - 1))
        } else {
            rec_ret!(false)
        }
    }

    pub fn is_even(x: u128) -> bool {
        tramp!(is_even_rec(x))
    }

    pub fn is_odd(x: u128) -> bool {
        tramp!(is_odd_rec(x))
    }
}

#[test]
fn test_oddness() {
    for i in 10000..10050 {
        assert_eq!(oddness::is_even(i), i & 1 == 0);
        assert_eq!(oddness::is_odd(i), i & 1 == 1);
    }
}
