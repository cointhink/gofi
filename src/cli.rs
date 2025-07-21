use std::env;

mod decimal;
mod unipool;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ax = u128::from_str_radix(&args[1], 10).unwrap();
    let ay = u128::from_str_radix(&args[2], 10).unwrap();
    let bx = u128::from_str_radix(&args[3], 10).unwrap();
    let by = u128::from_str_radix(&args[4], 10).unwrap();
    println!("ax {} ay {} bx {} by {}", ax, ay, bx, by);
    let a_price = decimal::scale(ay, ax);
    let b_price = decimal::scale(by, bx);
    println!(
        "pool 0 price {} {}",
        a_price,
        if a_price < b_price { "CHEAP" } else { "" },
    );
    println!(
        "pool 1 price {} {}",
        b_price,
        if a_price > b_price { "CHEAP" } else { "" },
    );
    let oay_in = unipool::optimal_ay_in(ax, ay, bx, by).unwrap();
    let s1_adx = unipool::get_y_out(oay_in, ay, ax);
    println!(
        "step 1 ay_in {} -> s1_adx {} price {}",
        oay_in,
        s1_adx,
        decimal::scale(oay_in, s1_adx)
    );
    let s2_ady = unipool::get_y_out(s1_adx, bx, by);
    println!(
        "step 2 s1_adx {} -> s2_ady {} price {}",
        s1_adx,
        s2_ady,
        decimal::scale(s2_ady, s1_adx)
    );
}
