use postgres::{Client, NoTls};

mod config;

fn main() -> Result<(), postgres::Error> {
    config::CONFIG
        .set(config::read_type(config::FILENAME))
        .unwrap();

    let config = config::CONFIG.get().unwrap();
    let mut db = Client::connect(&config.pg_url, NoTls)?;

    let rows = pairs_with(&mut db, "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")?;
    println!("{} rows", rows.len());
    
    for row in rows {
        let pool_contract_address_0: &str = row.get(0);
        let pool_contract_address_1: &str = row.get(1);
        let pool_block_0: &str = row.get(2);
        let pool_block_1: &str = row.get(3);
        let reserves_0 = reserves_for(&mut db, pool_contract_address_0);
        let reserves_1 = reserves_for(&mut db, pool_contract_address_1);
        let oay_in_fwd = optimal_ay_in(reserves_0.0, reserves_0.1, reserves_1.0, reserves_1.1);
        let oay_in_rev = optimal_ay_in(reserves_1.0, reserves_1.1, reserves_0.0, reserves_0.1);

        println!(
            "pool pair {} #{} x:{} {} #{} x:{} oay_in:{} oay_in_rev:{}",
            pool_contract_address_0,
            pool_block_0,
            reserves_0.0,
            pool_contract_address_1,
            pool_block_1,
            reserves_1.0,
            oay_in_fwd,
            oay_in_rev
        );
    }

    Ok(())
}

fn reserves_for(db: &mut postgres::Client, token: &str) -> (u128, u128, i32) {
    let sql = "SELECT x,y,block_number from reserves where contract_address = $1 order by block_number desc limit 1";
    let rows = db.query(sql, &[&token]).unwrap();
    let digits_x: &str = rows[0].get::<_, &str>("x");
    let digits_y: &str = rows[0].get::<_, &str>("y");
    let block_number = rows[0].get::<_, i32>("block_number");
    let x = u128::from_str_radix(digits_x, 10).unwrap();
    let y = u128::from_str_radix(digits_y, 10).unwrap();
    (x, y, block_number)
}

fn pairs_with(
    db: &mut postgres::Client,
    base_token: &str,
) -> Result<Vec<postgres::Row>, postgres::Error> {
    let sql = "WITH latest_reserves AS
              (SELECT contract_address, block_number, x,y, ROW_NUMBER() OVER(PARTITION BY contract_address ORDER BY block_number)
                FROM reserves ORDER BY contract_address, block_number)
              SELECT p1.contract_address as p1_contract_address,
                    p2.contract_address as p2_contract_address,
                    lrp1.x as qty_x1, lrp2.x AS qty_x2, lrp1.block_number AS p1_block_number,
                              lrp1.y as qty_y1, lrp2.y AS qty_y2, lrp2.block_number AS p2_block_number,
                    ABS((lrp1.x::decimal/lrp1.y::decimal) - (lrp2.x::decimal/lrp2.y::decimal))::float8 as spread,
                    (least(lrp1.x::decimal , lrp2.x::decimal ) *
                       ABS((lrp1.x::decimal/lrp1.y::decimal) - (lrp2.x::decimal/lrp2.y::decimal)))::float8 as value
              FROM pools AS p1
              JOIN pools AS p2 ON p1.token0 = p2.token0 AND p1.token1 = p2.token1 AND p1.contract_address != p2.contract_address AND p1.token0 = $1
              JOIN latest_reserves AS lrp1 ON p1.contract_address = lrp1.contract_address AND lrp1.row_number = 1
              JOIN latest_reserves AS lrp2 ON p2.contract_address = lrp2.contract_address AND lrp2.row_number = 1
              ORDER BY value desc";

    db.query(sql, &[&base_token])
}

pub fn optimal_ay_in(ax: u128, ay: u128, bx: u128, by: u128) -> f64 {
    const POOL_FEE: f64 = 1.0 - 0.003;
    let k = POOL_FEE * bx as f64 + POOL_FEE.powi(2) * ax as f64;
    let a = k.powi(2);
    let b = 2.0 * k * ay as f64 * bx as f64;
    let c = (ay as f64 * bx as f64).powi(2)
        - POOL_FEE.powi(2) * ax as f64 * bx as f64 * ay as f64 * by as f64;
    quadratic_root(a, b, c)
}

pub fn quadratic_root(a: f64, b: f64, c: f64) -> f64 {
    let d = b.powi(2) - 4.0 * a * c;
    if d > 0.0 {
        return (-b + d.sqrt()) / (2.0 * a);
    } else {
        return 0.0;
    }
}
