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
        "a nieve price {} ({}) {} b nieve price {} ({}) {}",
        a_price,
        a_price.log10(),
        if a_price < b_price { "CHEAP" } else { "" },
        b_price,
        b_price.log10(),
        if a_price > b_price { "CHEAP" } else { "" },
    );
    let _optimal = unipool::optimal_ay_in(ax, ay, bx, by).unwrap();
}
