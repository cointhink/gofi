use postgres::{Client, NoTls};

mod config;

const WETH: &str = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

fn main() -> Result<(), postgres::Error> {
    config::CONFIG
        .set(config::read_type(config::FILENAME))
        .unwrap();

    let config = config::CONFIG.get().unwrap();
    let mut db = Client::connect(&config.pg_url, NoTls)?;
    println!("gofi {} eth 0x{}", config::FILENAME, config.public_key());

    let pools_count = rows_count(&mut db, "pools");
    let pairs = pairs_with(&mut db, WETH)?;
    println!(
        "{} pools make {} pairs for {}",
        pools_count,
        pairs.len(),
        WETH
    );

    for row in pairs {
        let pool_contract_address_0: &str = row.get("p1_contract_address");
        let pool_contract_address_1: &str = row.get("p2_contract_address");
        let pool_block_0: i32 = row.get("p1_block_number");
        let pool_block_1: i32 = row.get("p2_block_number");
        let pool_0 = pool(&mut db, pool_contract_address_0);
        let coin_ay = coin(&mut db, &pool_0.2);
        // let pool_1 = pool(&mut db, pool_contract_address_1);
        let reserves_0 = reserves_for(&mut db, pool_contract_address_0, pool_block_0);
        let reserves_1 = reserves_for(&mut db, pool_contract_address_1, pool_block_1);

        // f(b) - f(a) == 0
        let oay_in = optimal_ay_in(reserves_0.0, reserves_0.1, reserves_1.0, reserves_1.1);

        if oay_in > 0.0 {
            // trade simulation
            let s1_adx = get_y_out(oay_in as u128, reserves_0.1, reserves_0.0);
            let s2_ady = get_y_out(s1_adx, reserves_1.0, reserves_1.1);
            let profit = s2_ady - oay_in as u128;

            println!(
                "{:0.4}{} {:0.4}{} pool pair {} #{} x:{} {} #{} x:{}",
                oay_in / 10_f64.powi(coin_ay.2),
                coin_ay.1,
                profit as f64 / 10_f64.powi(coin_ay.2),
                coin_ay.1,
                pool_contract_address_0,
                pool_block_0,
                reserves_0.0,
                pool_contract_address_1,
                pool_block_1,
                reserves_1.0,
            );
        }
    }

    Ok(())
}

fn pool(db: &mut postgres::Client, contract_address: &str) -> (String, String, String) {
    let sql = "SELECT * from pools where contract_address = $1";
    let rows = db.query(sql, &[&contract_address]).unwrap();
    let contract_address_row = rows[0].get::<_, String>("contract_address");
    let token0 = rows[0].get::<_, String>("token0");
    let token1 = rows[0].get::<_, String>("token1");

    (contract_address_row, token0, token1)
}

fn coin(db: &mut postgres::Client, contract_address: &str) -> (String, String, i32) {
    let sql = "SELECT * from coins where contract_address = $1";
    let rows = db.query(sql, &[&contract_address]).unwrap();
    let contract_address_row = rows[0].get::<_, String>("contract_address");
    let symbol = rows[0].get::<_, String>("symbol");
    let decimals = rows[0].get::<_, i32>("decimals");
    (contract_address_row, symbol, decimals)
}

fn rows_count(db: &mut postgres::Client, table_name: &str) -> i64 {
    let sql = format!("SELECT count(*) from {}", table_name);
    let rows = db.query(&sql, &[]).unwrap();
    rows[0].get::<_, i64>("count")
}

fn reserves_for(db: &mut postgres::Client, token: &str, block_number: i32) -> (u128, u128, i32) {
    let sql = "SELECT * from reserves where contract_address = $1 and block_number = $2 order by block_number desc limit 1";
    let rows = db.query(sql, &[&token, &block_number]).unwrap();
    let row = &rows[0];
    let digits_x: &str = row.get::<_, &str>("x");
    let digits_y: &str = row.get::<_, &str>("y");
    let block_number = row.get::<_, i32>("block_number");
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

pub fn get_y_out(dx: u128, x: u128, y: u128) -> u128 {
    (997 * dx * y) / (1000 * x + 997 * dx)
}
