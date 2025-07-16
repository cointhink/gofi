use std::cmp;

pub(crate) fn scale(num_a: u128, num_b: u128) -> f64 {
    let al2 = num_a.ilog2();
    let bl2 = num_b.ilog2();
    let big = cmp::max(al2, bl2);
    let drop = if big > 52 { big - 52 } else { 0 };
    let a = (num_a >> drop) as f64;
    let b = (num_b >> drop) as f64;
    a / b
}
