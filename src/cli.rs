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
    let s1_ax = ax - s1_adx;
    let s1_ay = ay + oay_in;
    println!(
        "p0 price@s1 {} ax {} ay {} k {}",
        decimal::scale(s1_ay, s1_ax),
        s1_ax,
        s1_ay,
        s1_ax * s1_ay
    );
    let s2_ady = unipool::get_y_out(s1_adx, bx, by);
    println!(
        "step 2 s1_adx {} -> s2_ady {} price {}",
        s1_adx,
        s2_ady,
        decimal::scale(s2_ady, s1_adx)
    );
    let s2_bx = bx + s1_adx;
    let s2_by = by - s2_ady;
    println!(
        "p1 price@s2 {} ax {} ay {} k {}",
        decimal::scale(s2_by, s2_bx),
        s2_bx,
        s2_by,
        s2_bx * s2_by
    );

    let profit = s2_ady - oay_in;
    println!("ay_in {oay_in} ay_out {s2_ady} -> profit {profit}");
}
