use std::cmp;

// divive two u128 numbers, leaving as much precision as an f64 can hold
pub(crate) fn scale(num_a: u128, num_b: u128) -> f64 {
    let al2 = num_a.ilog2();
    let bl2 = num_b.ilog2();
    let big = cmp::max(al2, bl2);
    let drop = if big > 52 { big - 52 } else { 0 }; // f64 = 52bit fraction + 11bit exponent
    if drop > al2 || drop > bl2 {
        return 0.0;
    }
    let a = (num_a >> drop) as f64;
    let b = (num_b >> drop) as f64;
    let result = a / b;
    println!(
        "xscale num_a {} ({}) num_b {} ({}) drop {} a {} ({}) b {} ({})  result {}",
        num_a,
        al2,
        num_b,
        bl2,
        drop,
        a,
        a.log2(),
        b,
        b.log2(),
        result
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale() {
        assert_eq!(scale(1_u128, 1_u128), 1.0);
        assert_eq!(scale(10_u128, 10_u128), 1.0);
        assert_eq!(scale(2_u128.pow(127), 2_u128.pow(127)), 1.0);
        assert_eq!(scale(2_u128.pow(52), 1_u128), 4503599627370496.0);
        assert_eq!(scale(2_u128.pow(53), 1_u128), 0.0); // more than a 52 bit difference is a ratio of zero
        assert_eq!(scale(2_u128.pow(53), 2_u128), 4503599627370496.0); // half a 53 bit number is 52 bits
        assert_eq!(scale(10_u128, 1_u128), 10.0);
        assert_eq!(scale(1_u128, 10_u128), 0.1);
    }
}
