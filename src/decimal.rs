use std::cmp;

pub(crate) fn scale(num_a: u128, num_b: u128) -> f64 {
    let al2 = num_a.ilog2();
    let bl2 = num_b.ilog2();
    let big = cmp::max(al2, bl2);
    let drop = if big > 52 { big - 52 } else { 0 };
    let a = (num_a >> drop) as f64;
    let b = (num_b >> drop) as f64;
    let result = a / b;
    println!(
        "xscale a {} b {} a/b {} result {}",
        num_a,
        num_b,
        (num_a * 1000 / num_b) as f64 / 1000.0,
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
        assert_eq!(scale(10_u128, 1_u128), 10.0);
        assert_eq!(scale(1_u128, 10_u128), 0.1);
    }
}
