use alloy::primitives::U256;

pub fn optimal_ay_in(ax: u128, ay: u128, bx: u128, by: u128) -> Result<u128, String> {
    const POOL_FEE_BASIS_POINTS: u8 = 30;
    let (a, b, c) = reserves_to_coefficients(ax, ay, bx, by, POOL_FEE_BASIS_POINTS)?;
    Ok(quadratic_root(a, b, c))
}

pub fn reserves_to_coefficients(
    ax: u128,
    ay: u128,
    bx: u128,
    by: u128,
    fee_points: u8,
) -> Result<(U256, U256, U256), String> {
    let fee_points_magnitude = U256::from(10000);
    let fee = fee_points_magnitude - U256::from(fee_points);
    // k = (1-f)*xb + (1-f)^2*xa
    let k1 = U256::from(bx) * fee / (U256::from(fee_points_magnitude));
    let k2 = fee.pow(U256::from(2)) * U256::from(ax) / (fee_points_magnitude.pow(U256::from(2)));
    let k = k1 + k2;
    // a = k^2
    let a = k.pow(U256::from(2));
    // b = 2k*ya*xb
    let b = k * U256::from(2) * U256::from(ay) * U256::from(bx);
    // c = (ya*xb)^2 - (1-f)^2*xa*ya*xb*yb
    let c1 = U256::from(ay) * U256::from(ay) * U256::from(bx) * U256::from(bx);
    let c21 = U256::from(ax) * U256::from(ay) * U256::from(bx) * U256::from(by);
    let c2 = fee.pow(U256::from(2)) * c21 / fee_points_magnitude.pow(U256::from(2));
    println!("c1 {} ({}) c2 {} ({})", c1, c1.log10(), c2, c2.log10(),);
    if c1 > c2 {
        // let c = c1.saturating_sub(c2);
        // println!("a {} b {} c {}", a, b, c);
        // Ok((a, b, c))
        Err("c of (a,b,c) is positive".to_owned())
    } else {
        let c = c2 - c1;
        println!("a {} b {} -c {}", a, b, c);
        Ok((a, b, c))
    }
}

pub fn quadratic_root(a: U256, b: U256, c: U256) -> u128 {
    // delta = b^2 - 4ac
    let d1 = b.pow(U256::from(2));
    let d2 = U256::from(4) * a * c;
    println!("d1 {} ({}) d2 {} ({})", d1, d1.log10(), d2, d2.log10());
    let delta = d1 + d2;
    // println!("d1 {} d2 {} delta {}", d1, d2, delta);
    // -b +- sqrt(delta) / 2a
    let root1 = (delta.root(2).saturating_sub(b)) / (U256::from(2) * a);
    let root2 = (b + delta.root(2)) / (U256::from(2) * a);
    println!(
        "b {} + delta.root(2) {} / 2*a {} = {}",
        b,
        delta.root(2),
        a * U256::from(2),
        root1
    );
    println!("{},{},{} -> ({}, {})", a, b, c, root1, root2);
    root1.saturating_to::<u128>()
}

pub fn get_y_out(dx: u128, x: u128, y: u128) -> u128 {
    // uniswap v1 paper: (997 * dx * y) / (1000 * x + 997 * dx)
    let big = (U256::from(997) * U256::from(dx) * U256::from(y))
        / (U256::from(1000) * U256::from(x) + U256::from(997) * U256::from(dx));
    big.saturating_to::<u128>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_root() {
        // -b +- sqrt(b^2 - 4ac) / 2a
        // delta = b^2 - 4ac
        // a real root exists when b^2 > 4ac
        // 340,-690,340 = (0.84258, 1.1868) # wolframalpha
        let a = U256::from(1);
        let b = U256::from(338318);
        let c = U256::from(169);
        let root = quadratic_root(a, b, c);
        assert_eq!(338317, root);
    }

    #[test]
    fn test_get_y_out() {
        let dx = 10;
        let x = 100;
        let y = 50;
        assert_eq!(get_y_out(dx, x, y), 4)
    }

    #[test]
    fn test_optimal_ay_in() {
        // let ax = 4000;
        // let ay = 60000;
        // let bx = 3000;
        // let by = 90000;
        let ax = 310000;
        let ay = 210000;
        let bx = 220000;
        let by = 320000;
        println!("ax {} ay {} bx {} by{}", ax, ay, bx, by);
        println!(
            "p1 price@s0 {} k {}",
            (ay * 100 / ax) as f64 / 100.0,
            ax * ay
        );
        println!(
            "p2 price@s0 {} k {}",
            (by * 100 / bx) as f64 / 100.0,
            bx * by
        );
        println!(
            "p1-p2 midpoint {}",
            ((ay * 100 / ax) + ((by * 100 / bx) - (ay * 100 / ax)) / 2) as f64 / 100.0,
        );

        let ay_in = optimal_ay_in(ax, ay, bx, by).unwrap();
        assert_eq!(ay_in, 40371, "ay_in");

        let s1_adx = get_y_out(ay_in, ay, ax);
        println!(
            "p1 sale {} s1_adx {} / ay_in {}",
            (ay_in * 100 / s1_adx) as f64 / 100.0,
            s1_adx,
            ay_in
        );
        let s1_ax = ax - s1_adx;
        let s1_ay = ay + ay_in;
        println!(
            "p1 price@s1 {} ax {} ay {} k {}",
            (s1_ay * 100 / s1_ax) as f64 / 100.0,
            s1_ax,
            s1_ay,
            s1_ax * s1_ay
        );
        let s2_ady = get_y_out(s1_adx, bx, by);
        println!(
            "p2 sale {} s1_adx {} / s2_ady {}",
            (s2_ady * 100 / s1_adx) as f64 / 100.0,
            s1_adx,
            s2_ady
        );
        let s2_bx = bx + s1_adx;
        let s2_by = by - s2_ady;
        println!(
            "p2 price@s2 {} bx {} by {} k {}",
            (s2_by * 100 / s2_bx) as f64 / 100.0,
            s2_bx,
            s2_by,
            s2_bx * s2_by
        );
        // let profit = s2_ady - ay_in;
        println!("ay_in {ay_in} ay_out {s2_ady}");
    }

    #[test]
    fn test_reserves_to_coefficients() {
        //winner: 1.5432USDT profit:0.0286USDT p0:cbc5bde09fb89220e961415d2098b40860fd352a #2025-07-04 19:45:23 UTC p1:5b8fbba724afc16bee3eb0a4af9953fd023dcb09 #2025-07-03 06:01:23
        //winner p0: cbc5bde09fb89220e961415d2098b40860fd352a r0: 98203032335537373 r1: 242910566 block: 22848029 2025-07-04 19:45:23 UTC
        //winner p1: 5b8fbba724afc16bee3eb0a4af9953fd023dcb09 r0: 50774084797862325 r1: 131079784 block: 22836777 2025-07-03 06:01:23 UTC

        let fee_points = 30;
        let ax = 98203032335537373;
        let ay = 242910566;
        let bx = 50774084797862325;
        let by = 131079784;
        let (a, b, c) = reserves_to_coefficients(ax, ay, bx, by, fee_points).unwrap();
        assert_eq!(
            a,
            U256::from_str_radix("21974048225209905743260320346616836", 10).unwrap(),
            "a"
        );
        assert_eq!(
            b,
            U256::from_str_radix("3656567056833261232090410051780886244321400", 10).unwrap(),
            "b"
        );
        assert_eq!(c, U256::from(0), "c");
    }

    #[test]
    fn test_reserves_to_coefficients2() {
        //winner: 1.5432USDT profit:0.0286USDT p0:cbc5bde09fb89220e961415d2098b40860fd352a #2025-07-04 19:45:23 UTC p1:5b8fbba724afc16bee3eb0a4af9953fd023dcb09 #2025-07-03 06:01:23
        //winner p0: cbc5bde09fb89220e961415d2098b40860fd352a r0: 98203032335537373 r1: 242910566 block: 22848029 2025-07-04 19:45:23 UTC
        //winner p1: 5b8fbba724afc16bee3eb0a4af9953fd023dcb09 r0: 50774084797862325 r1: 131079784 block: 22836777 2025-07-03 06:01:23 UTC

        let fee_points = 30;
        let ax = 3000;
        let ay = 2000;
        let bx = 3000;
        let by = 1000;
        let (a, b, c) = reserves_to_coefficients(ax, ay, bx, by, fee_points).unwrap();
        assert_eq!(a, U256::from_str_radix("35676729", 10).unwrap(), "a");
        assert_eq!(b, U256::from_str_radix("71676000000", 10).unwrap(), "b");
        assert_eq!(c, U256::from_str_radix("18107838000000", 10).unwrap(), "c");
    }
}
