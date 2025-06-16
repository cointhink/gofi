use postgres::{Client, NoTls};

mod config;

const WETH: &str = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

macro_rules! sql_field {
    ($name:expr, $digit:expr) => {
        format!($name, $digit).as_str()
    };
}

struct Pool {
    contract_address: String,
    coin0: Coin,
    coin1: Coin,
}

impl Pool {
    pub fn from_pair_row(db: &mut postgres::Client, row: &postgres::Row, pool_digit: &str) -> Pool {
        let pool_contract_address_0: &str = row.get(sql_field!("p{}_contract_address", pool_digit));
        let pool_token0: &str = row.get(sql_field!("p{}_token0", pool_digit));
        let pool_token1: &str = row.get(sql_field!("p{}_token1", pool_digit));
        Pool {
            contract_address: pool_contract_address_0.to_owned(),
            coin0: coin(db, pool_token0),
            coin1: coin(db, pool_token1),
        }
    }
}

struct Coin {
    contract_address: String,
    symbol: String,
    decimals: i32,
}

struct Reserve {
    contract_address: String,
    x: u128,
    y: u128,
    block: u32,
}

impl Reserve {
    pub fn from_pair_row(row: &postgres::Row, pool_digit: &str) -> Reserve {
        let pool_contract_address: &str = row.get(sql_field!("p{}_contract_address", pool_digit));
        let pool_digits_x: &str = row.get(sql_field!("qty_x{}", pool_digit));
        let pool_x = u128::from_str_radix(pool_digits_x, 10).unwrap();
        let pool_digits_y: &str = row.get(sql_field!("qty_y{}", pool_digit));
        let pool_y = u128::from_str_radix(pool_digits_y, 10).unwrap();
        let pool_block: i32 = row.get(sql_field!("p{}_block_number", pool_digit));
        Reserve {
            contract_address: pool_contract_address.to_owned(),
            x: pool_x,
            y: pool_y,
            block: pool_block as u32,
        }
    }
}

struct PoolSnapshot {
    pool: Pool,
    reserve: Reserve,
}

struct Pair {
    pool0: PoolSnapshot,
    pool1: PoolSnapshot,
}

impl Pair {
    pub fn from_pair_row(db: &mut postgres::Client, row: &postgres::Row) -> Pair {
        let pool0 = PoolSnapshot {
            pool: Pool::from_pair_row(db, row, "1"),
            reserve: Reserve::from_pair_row(row, "1"),
        };

        let pool1 = PoolSnapshot {
            pool: Pool::from_pair_row(db, row, "2"),
            reserve: Reserve::from_pair_row(row, "2"),
        };

        Pair { pool0, pool1 }
    }
}

struct Match {
    pair: Pair,
    pool0_ay_in: u128,
    pool0_ax_out: u128,
    pool1_ay_out: u128,
}

impl Match {
    pub fn to_string(self: &Self) -> String {
        format!(
            "{:0.4}{} {:0.4}{} pool pair {} #{} x:{} {} #{} x:{}",
            self.pool0_ay_in as f64 / 10_f64.powi(self.pair.pool0.pool.coin1.decimals),
            self.pair.pool0.pool.coin1.symbol,
            self.profit() as f64 / 10_f64.powi(self.pair.pool0.pool.coin1.decimals),
            self.pair.pool0.pool.coin1.symbol,
            self.pair.pool0.pool.contract_address,
            self.pair.pool0.reserve.block,
            self.pair.pool0.reserve.x,
            self.pair.pool1.pool.contract_address,
            self.pair.pool1.reserve.block,
            self.pair.pool1.reserve.x,
        )
    }

    pub fn profit(self: &Self) -> u128 {
        self.pool1_ay_out - self.pool0_ay_in
    }
}

fn main() -> Result<(), postgres::Error> {
    config::CONFIG
        .set(config::read_type(config::FILENAME))
        .unwrap();

    let config = config::CONFIG.get().unwrap();
    let mut db = Client::connect(&config.pg_url, NoTls)?;
    println!("gofi {} eth 0x{}", config::FILENAME, config.public_key());

    let pools_count = rows_count(&mut db, "pools");
    let pairs = pairs_with(&mut db, WETH)?;
    let matches = matches(&mut db, &pairs);
    println!(
        "{} pools make {} pairs and {} matches for {}",
        pools_count,
        pairs.len(),
        matches.len(),
        WETH
    );

    for r#match in matches {
        println!("{}", r#match.to_string())
    }

    Ok(())
}

fn matches(mut db: &mut postgres::Client, pairs: &Vec<postgres::Row>) -> Vec<Match> {
    let mut matches = vec![];
    for row in pairs {
        let pair = Pair::from_pair_row(db, &row);

        // f(b) - f(a) == 0
        let oay_in = optimal_ay_in(
            pair.pool0.reserve.x,
            pair.pool0.reserve.y,
            pair.pool1.reserve.x,
            pair.pool1.reserve.y,
        );

        if oay_in > 0.0 {
            // trade simulation
            let s1_adx = get_y_out(oay_in as u128, pair.pool0.reserve.y, pair.pool0.reserve.x);
            let s2_ady = get_y_out(s1_adx, pair.pool1.reserve.x, pair.pool1.reserve.y);
            // let profit = s2_ady - oay_in as u128;
            let r#match = Match {
                pair,
                pool0_ay_in: oay_in as u128,
                pool0_ax_out: s1_adx,
                pool1_ay_out: s2_ady,
            };
            matches.push(r#match);
        }
    }
    matches
}
fn pool(db: &mut postgres::Client, contract_address_in: &str) -> Pool {
    let sql = "SELECT * from pools where contract_address = $1";
    let rows = db.query(sql, &[&contract_address_in]).unwrap();
    let contract_address = rows[0].get::<_, String>("contract_address");
    let token0 = rows[0].get::<_, String>("token0");
    let coin0 = coin(db, &token0);
    let token1 = rows[0].get::<_, String>("token1");
    let coin1 = coin(db, &token1);

    Pool {
        contract_address,
        coin0,
        coin1,
    }
}

fn coin(db: &mut postgres::Client, contract_address: &str) -> Coin {
    let sql = "SELECT * from coins where contract_address = $1";
    let rows = db.query(sql, &[&contract_address]).unwrap();
    let row = &rows[0];
    Coin {
        contract_address: row.get::<_, String>("contract_address"),
        symbol: row.get::<_, String>("symbol"),
        decimals: row.get::<_, i32>("decimals"),
    }
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
                     p1.token0 as p1_token0,
                     p1.token0 as p1_token1,
                     p2.contract_address as p2_contract_address,
                     p2.token0 as p2_token0,
                     p2.token0 as p2_token1,
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
