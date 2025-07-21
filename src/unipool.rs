use alloy::primitives::{U256, U512};

// x is the product. y is the money. ay/ax < by/bx means pool a is cheaper than pool b
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
    // k is always positive
    let k1 = U256::from(bx) * fee / (U256::from(fee_points_magnitude));
    let k2 = fee.pow(U256::from(2)) * U256::from(ax) / (fee_points_magnitude.pow(U256::from(2)));
    let k = k1 + k2;
    // a = k^2
    // a is always positive
    let a = k.pow(U256::from(2));
    // b = 2k*ya*xb
    // b is always positive
    let b = k * U256::from(2) * U256::from(ay) * U256::from(bx);
    // c = (ya*xb)^2 - (1-f)^2*xa*ya*xb*yb
    // c1 is always positive
    let c1 = U256::from(ay) * U256::from(ay) * U256::from(bx) * U256::from(bx);
    let c21 = U256::from(ax) * U256::from(ay) * U256::from(bx) * U256::from(by);
    // c2 is always positive
    let c2 = fee.pow(U256::from(2)) * c21 / fee_points_magnitude.pow(U256::from(2));
    if c1 > c2 {
        if c1 < c21 {
            Err("(a,b,c) no arb after fee".to_owned())
        } else {
            Err("(a,b,c) no arb.".to_owned())
        }
    } else {
        let c = c2 - c1; // -c
        Ok((a, b, c))
    }
}

pub fn quadratic_root(pos_a: U256, pos_b: U256, neg_c: U256) -> u128 {
    let a = U512::from(pos_a);
    let b = U512::from(pos_b);
    let c = U512::from(neg_c);
    // delta = b^2 - 4ac
    let d1 = b.pow(U512::from(2));
    // d1 is always positive
    let d2 = U512::from(4) * a * c;
    // d2 is always negative because c is always negative (expressed here as UINT)
    // delta is always postiive because c is always negative
    // -neg + pos = pos + "neg":  b^2 + 4ac
    let delta = d1 + d2;
    // -b +- sqrt(delta) / 2a
    // sqrt(delta) is always larger than b because delta is b^2 plus a value
    //
    let root = (delta.root(2).saturating_sub(b)) / (U512::from(2) * a);
    println!(
        "+a {} ({}) +b {} ({}) -c {} ({}) -> {} ({})",
        a,
        a.log2(),
        b,
        b.log2(),
        c,
        c.log2(),
        root,
        root.log2()
    );
    root.saturating_to::<u128>()
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
        // a 278237260324 b 48739336800000000 -c 2421143007360000000000
        // wolfram alpha: x≈-215543 x≈40371

        let a = U256::from_str_radix("278237260324", 10).unwrap();
        let b = U256::from_str_radix("48739336800000000", 10).unwrap();
        let c = U256::from_str_radix("2421143007360000000000", 10).unwrap();
        let root = quadratic_root(a, b, c);
        assert_eq!(root, 40371);

        // a 7010956849340041661775550609684450681 b 2719085318207604654461411506480024329673136 -c 1695220225124043972868953930979927881452999219
        // wolfram alpha: x≈-388456 x≈622.45
        let a = U256::from_str_radix("7010956849340041661775550609684450681", 10).unwrap();
        let b = U256::from_str_radix("2719085318207604654461411506480024329673136", 10).unwrap();
        let c = U256::from_str_radix("1695220225124043972868953930979927881452999219", 10).unwrap();
        let root = quadratic_root(a, b, c);
        assert_eq!(root, 622);
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
        println!("ay_in {ay_in} ay_out {s2_ady}");

        let profit = s2_ady - ay_in;
        assert_eq!(profit, 18608, "profit");
    }

    #[test]
    fn test_reserves_to_coefficients() {
        //winner: 1.5432USDT profit:0.0286USDT p0:cbc5bde09fb89220e961415d2098b40860fd352a #2025-07-04 19:45:23 UTC p1:5b8fbba724afc16bee3eb0a4af9953fd023dcb09 #2025-07-03 06:01:23
        //winner p0: cbc5bde09fb89220e961415d2098b40860fd352a r0: 98203032335537373 r1: 242910566 block: 22848029 2025-07-04 19:45:23 UTC
        //winner p1: 5b8fbba724afc16bee3eb0a4af9953fd023dcb09 r0: 50774084797862325 r1: 131079784 block: 22836777 2025-07-03 06:01:23 UTC

        let fee_points = 30;
        let ax = 310000;
        let ay = 210000;
        let bx = 220000;
        let by = 320000;
        let (a, b, c) = reserves_to_coefficients(ax, ay, bx, by, fee_points).unwrap();
        assert_eq!(a, U256::from_str_radix("278237260324", 10).unwrap(), "a");
        assert_eq!(
            b,
            U256::from_str_radix("48739336800000000", 10).unwrap(),
            "b"
        );
        assert_eq!(
            c,
            U256::from_str_radix("2421143007360000000000", 10).unwrap(),
            "c"
        );
    }
}
