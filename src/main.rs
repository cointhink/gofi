#![allow(dead_code)]

use alloy::{
    primitives::{Address, U256, utils::format_units},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
    transports::http::reqwest::Url,
};
use chrono::DateTime;
use hex::decode;
use postgres::{Client, NoTls};

mod config;
mod decimal;
mod unipool;

macro_rules! sql_field {
    ($name:expr, $digit:expr) => {
        format!($name, $digit).as_str()
    };
}

fn main() -> Result<(), postgres::Error> {
    config::CONFIG
        .set(config::read_type(config::FILENAME))
        .unwrap();
    tracing_subscriber::fmt::init();

    let config = config::CONFIG.get().unwrap();
    let mut db = Client::connect(&config.pg_url, NoTls)?;
    println!(
        "gofi config:{} eth:0x{}",
        config::FILENAME,
        config.public_key()
    );

    let pools_count = rows_count(&mut db, "pools");
    println!(
        "sql finding pairs from {} pools where token0 = {}",
        pools_count, &config.preferred_base_token
    );
    let pairs = pairs_with(&mut db, &config.preferred_base_token)?;
    println!("{} pairs found", pairs.len());
    let matches = simulate(&pairs);
    println!(
        "{} pools make {} pairs and {} matches for {}",
        pools_count,
        pairs.len(),
        matches.len(),
        config.preferred_base_token
    );

    for r#match in &matches {
        println!("{}", r#match.to_string());
        let p1 = r#match.pair.pool0.price();
        let p2 = r#match.pair.pool1.price();
        println!(
            "{} {} {} {} p1 {} (d{}/d{}) n{} p2 {} {} ",
            r#match.pair.pool0.reserve.x,
            r#match.pair.pool0.reserve.y,
            r#match.pair.pool1.reserve.x,
            r#match.pair.pool1.reserve.y,
            p1,
            r#match.pair.pool0.pool.coin0.decimals,
            r#match.pair.pool0.pool.coin1.decimals,
            r#match.pair.pool0.reserve.x / r#match.pair.pool0.reserve.y,
            p2,
            if p1 < p2 {
                "POOL1 CHEAPER"
            } else {
                "POOL2 CHEAPER"
            }
        );
    }

    let limit = 0.02;
    let winners = matches
        .into_iter()
        .filter(|m| {
            let scaled_profit = m.scaled_profit();
            m.pair.pool0.pool.coin1.contract_address == config.preferred_coin_token
                && scaled_profit > limit
                && scaled_profit < 2.0
        })
        .collect::<Vec<Match>>();

    if winners.len() > 0 {
        for winner in winners[0..1].into_iter() {
            println!("===========================================================");
            maineth(winner).unwrap();
        }
    } else {
        println!("no winners over {}", limit);
    }

    Ok(())
}

sol!(
    #[sol(rpc)]
    UniswapV2Pair,
    "sol-abi/UniswapV2Pair.json"
);
sol!(
    #[sol(rpc)]
    ERC20,
    "sol-abi/ERC20.json"
);
sol!(
    #[sol(rpc)]
    UniSwab,
    "ethereum/artifacts/UniSwab.abi"
);

#[tokio::main]
async fn maineth(winner: &Match) -> Result<(), String> {
    let config = config::CONFIG.get().unwrap();
    let geth_url = Url::parse(&config.geth_url).unwrap();
    let pk_signer: PrivateKeySigner = config.eth_priv_key.parse().unwrap();
    let public_key = pk_signer.address();
    let provider = ProviderBuilder::new()
        .wallet(pk_signer)
        .with_gas_estimation()
        .connect_http(geth_url.clone());
    let uniswab = UniSwab::new(config.uniswab.parse().unwrap(), &provider);
    let coin0 = ERC20::new(
        winner
            .pair
            .pool0
            .pool
            .coin0
            .contract_address
            .parse()
            .unwrap(),
        &provider,
    );
    let coin1 = ERC20::new(
        winner
            .pair
            .pool0
            .pool
            .coin1
            .contract_address
            .parse()
            .unwrap(),
        &provider,
    );

    println!(
        "{} eth: {}",
        public_key,
        format_units(provider.get_balance(public_key).await.unwrap(), 18).unwrap()
    );
    erc20_allow(&public_key, uniswab.address(), &coin0).await;
    println!(
        "{} {}: {}",
        public_key,
        winner.pair.pool0.pool.coin0.symbol,
        Into::<f64>::into(coin0.balanceOf(public_key).call().await.unwrap())
            / 10_f64.powi(winner.pair.pool0.pool.coin0.decimals),
    );
    erc20_allow(&public_key, uniswab.address(), &coin1).await;
    println!(
        "{} {}: {}",
        public_key,
        winner.pair.pool0.pool.coin1.symbol,
        Into::<f64>::into(coin1.balanceOf(public_key).call().await.unwrap())
            / 10_f64.powi(winner.pair.pool0.pool.coin1.decimals),
    );

    println!("winner: {}", winner.to_string());
    println!(
        "winner p0: {} r0: {} r1: {} block: {} {}",
        winner.pair.pool0.pool.contract_address,
        winner.pair.pool0.reserve.x,
        winner.pair.pool0.reserve.y,
        winner.pair.pool0.reserve.block_number,
        winner.pair.pool0.reserve.block_time_str(),
    );
    println!(
        "winner p1: {} r0: {} r1: {} block: {} {}",
        winner.pair.pool1.pool.contract_address,
        winner.pair.pool1.reserve.x,
        winner.pair.pool1.reserve.y,
        winner.pair.pool1.reserve.block_number,
        winner.pair.pool1.reserve.block_time_str(),
    );

    // let uniswab = UniSwab::new(config.uniswab.parse().unwrap(), &provider);
    let pool0 = UniswapV2Pair::new(
        winner.pair.pool0.pool.contract_address.parse().unwrap(),
        &provider,
    );
    let pool1 = UniswapV2Pair::new(
        winner.pair.pool1.pool.contract_address.parse().unwrap(),
        &provider,
    );

    let (r00, r01, btime0) = pool0.getReserves().call().await.unwrap().into();
    let btime0_str = DateTime::from_timestamp(btime0 as i64, 0).unwrap();
    println!(
        "fresh p0: {} r0: {} r1: {} btime: {} {}",
        winner.pair.pool0.pool.contract_address, r00, r01, btime0, btime0_str
    );
    let (r10, r11, btime1) = pool1.getReserves().call().await.unwrap().into();
    let btime1_str = DateTime::from_timestamp(btime1 as i64, 0).unwrap();
    println!(
        "fresh p1: {} r0: {} r1: {} btime: {} {}",
        winner.pair.pool1.pool.contract_address, r10, r11, btime1, btime1_str
    );
    println!("winner profit: {}", winner.scaled_profit());
    let fresh_pair = Pair {
        pool0: PoolSnapshot {
            pool: winner.pair.pool0.pool.clone(),
            reserve: Reserve {
                contract_address: "0x00".to_owned(),
                x: r00.to(),
                y: r01.to(),
                block_number: 0,
                block_timestamp: btime0,
            },
        },
        pool1: PoolSnapshot {
            pool: winner.pair.pool1.pool.clone(),
            reserve: Reserve {
                contract_address: "0x01".to_owned(),
                x: r10.to(),
                y: r11.to(),
                block_number: 1,
                block_timestamp: btime1,
            },
        },
    };
    let fresh_match = trade_simulate(fresh_pair)?;
    println!("fresh profit: {}", fresh_match.scaled_profit());

    if winner.pair.pool0.reserve.x == fresh_match.pair.pool0.reserve.x
        && winner.pair.pool0.reserve.y == fresh_match.pair.pool0.reserve.y
        && winner.pair.pool1.reserve.x == fresh_match.pair.pool1.reserve.x
        && winner.pair.pool1.reserve.y == fresh_match.pair.pool1.reserve.y
    {
        println!(
            "SWAB {}, {}, {}",
            winner.pool0_ay_in,
            &winner.pair.pool0.pool.contract_address,
            &winner.pair.pool1.pool.contract_address,
        );
        let swab_tx = uniswab.swab(
            U256::from(winner.pool0_ay_in),
            Address::from_slice(&decode(&winner.pair.pool0.pool.contract_address).unwrap()),
            Address::from_slice(&decode(&winner.pair.pool1.pool.contract_address).unwrap()),
        );
        let swab_tx_receipt = swab_tx.send().await.unwrap().get_receipt().await.unwrap();
        println!("swab tx {}", swab_tx_receipt.transaction_hash);

        println!(
            "{} eth: {}",
            public_key,
            format_units(provider.get_balance(public_key).await.unwrap(), 18).unwrap()
        );
        println!(
            "{} {}: {}",
            public_key,
            winner.pair.pool0.pool.coin0.symbol,
            Into::<f64>::into(coin0.balanceOf(public_key).call().await.unwrap())
                / 10_f64.powi(winner.pair.pool0.pool.coin0.decimals),
        );
        erc20_allow(&public_key, uniswab.address(), &coin1).await;
        println!(
            "{} {}: {}",
            public_key,
            winner.pair.pool0.pool.coin1.symbol,
            Into::<f64>::into(coin1.balanceOf(public_key).call().await.unwrap())
                / 10_f64.powi(winner.pair.pool0.pool.coin1.decimals),
        );
        Ok(())
    } else {
        Err("swap aborted. freshness check failed".to_owned())
    }
}

async fn erc20_allow<T: Provider>(
    owner_address: &Address,
    spender_address: &Address,
    coin: &ERC20::ERC20Instance<T>,
) {
    let allowance = coin
        .allowance(*owner_address, *spender_address)
        .call()
        .await
        .unwrap();
    println!(
        "erc20: {} owner:{} spender: {} allowance: {}",
        coin.address(),
        owner_address,
        spender_address,
        allowance,
    );
    if allowance == U256::from(0) {
        let tx = coin
            .approve(*spender_address, U256::MAX)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
        println!("weth allownace tx: {}", hex::encode(tx.transaction_hash));
    }
}

#[derive(Clone)]
struct Pool {
    contract_address: String,
    coin0: Coin,
    coin1: Coin,
}

impl Pool {
    pub fn from_pair_row(row: &postgres::Row, pool_digit: &str) -> Pool {
        let pool_contract_address_0: &str = row.get(sql_field!("p{}_contract_address", pool_digit));
        Pool {
            contract_address: pool_contract_address_0.to_owned(),
            coin0: Coin::from_pair_row(row, pool_digit, "0"),
            coin1: Coin::from_pair_row(row, pool_digit, "1"),
        }
    }
}

#[derive(Clone)]
struct Coin {
    contract_address: String,
    symbol: String,
    decimals: i32,
}

impl Coin {
    pub fn from_pair_row(row: &postgres::Row, pool_digit: &str, token_digit: &str) -> Coin {
        let contract_address = row.get(format!("p{}_token{}", pool_digit, token_digit).as_str());
        let symbol = row.get(format!("p{}_token{}_symbol", pool_digit, token_digit).as_str());
        let decimals = row.get(format!("p{}_token{}_decimals", pool_digit, token_digit).as_str());
        Coin {
            contract_address,
            symbol,
            decimals,
        }
    }
}

struct Reserve {
    contract_address: String,
    x: u128,
    y: u128,
    block_number: u32,
    block_timestamp: u32,
}

impl Reserve {
    pub fn from_pair_row(row: &postgres::Row, pool_digit: &str) -> Reserve {
        let pool_contract_address: &str = row.get(sql_field!("p{}_contract_address", pool_digit));
        let pool_digits_x: &str = row.get(sql_field!("qty_x{}", pool_digit));
        let pool_x = u128::from_str_radix(pool_digits_x, 10).unwrap();
        let pool_digits_y: &str = row.get(sql_field!("qty_y{}", pool_digit));
        let pool_y = u128::from_str_radix(pool_digits_y, 10).unwrap();
        let pool_block: i32 = row.get(sql_field!("p{}_block_number", pool_digit));
        let pool_timestamp: i32 = row.get(sql_field!("p{}_block_timestamp", pool_digit));
        Reserve {
            contract_address: pool_contract_address.to_owned(),
            x: pool_x,
            y: pool_y,
            block_number: pool_block as u32,
            block_timestamp: pool_timestamp as u32,
        }
    }
    pub fn block_time_str(self: &Self) -> String {
        DateTime::from_timestamp(self.block_timestamp as i64, 0)
            .unwrap()
            .to_string()
    }
}

struct PoolSnapshot {
    pool: Pool,
    reserve: Reserve,
}

impl PoolSnapshot {
    fn price(self: &Self) -> f64 {
        decimal::scale(self.reserve.y, self.reserve.x)
    }
}

struct Pair {
    pool0: PoolSnapshot,
    pool1: PoolSnapshot,
}

impl Pair {
    pub fn from_pair_row(row: &postgres::Row) -> Pair {
        let pool0 = PoolSnapshot {
            pool: Pool::from_pair_row(row, "1"),
            reserve: Reserve::from_pair_row(row, "1"),
        };

        let pool1 = PoolSnapshot {
            pool: Pool::from_pair_row(row, "2"),
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
            "{:0.4}{} profit:{:0.4}{} p0:{} #{} p1:{} #{} ",
            self.pool0_ay_in as f64 / 10_f64.powi(self.pair.pool0.pool.coin1.decimals),
            self.pair.pool0.pool.coin1.symbol,
            self.scaled_profit(),
            self.pair.pool0.pool.coin1.symbol,
            self.pair.pool0.pool.contract_address,
            self.pair.pool0.reserve.block_time_str(),
            //self.pair.pool0.reserve.x as f64 / 10_f64.powi(self.pair.pool0.pool.coin0.decimals),
            //self.pair.pool0.reserve.y as f64 / 10_f64.powi(self.pair.pool0.pool.coin1.decimals),
            self.pair.pool1.pool.contract_address,
            self.pair.pool1.reserve.block_time_str(),
            //self.pair.pool1.reserve.x as f64 / 10_f64.powi(self.pair.pool1.pool.coin0.decimals),
            //self.pair.pool1.reserve.y as f64 / 10_f64.powi(self.pair.pool1.pool.coin1.decimals),
        )
    }

    pub fn profit(self: &Self) -> u128 {
        self.pool1_ay_out.saturating_sub(self.pool0_ay_in)
    }
    pub fn scaled_profit(self: &Self) -> f64 {
        self.profit() as f64 / 10_f64.powi(self.pair.pool0.pool.coin1.decimals)
    }
}

fn simulate(pairs: &Vec<postgres::Row>) -> Vec<Match> {
    let mut matches = vec![];
    for row in pairs {
        let pair = Pair::from_pair_row(&row);
        match trade_simulate(pair) {
            Ok(r#match) => matches.push(r#match),
            Err(err) => println!("{}", err),
        }
    }
    matches
}

fn trade_simulate(pair: Pair) -> Result<Match, String> {
    let ax = pair.pool0.reserve.x;
    let ay = pair.pool0.reserve.y;
    let bx = pair.pool1.reserve.x;
    let by = pair.pool1.reserve.y;
    let p1 = pair.pool0.price();
    let p2 = pair.pool1.price();
    println!(
        "{}1 price@s0 {} ax {} ay {} k {}",
        if p1 < p2 { "P" } else { "p" },
        p1,
        ax,
        ay,
        U256::from(ax) * U256::from(ay),
    );
    println!(
        "{}2 price@s0 {} bx {} by {} k {}",
        if p1 > p2 { "P" } else { "p" },
        p2,
        bx,
        by,
        U256::from(bx) * U256::from(by),
    );

    // f(b) - f(a) == 0
    let oay_in = unipool::optimal_ay_in(ax, ay, bx, by)?;

    // trade simulation
    let s1_adx = unipool::get_y_out(oay_in, pair.pool0.reserve.y, pair.pool0.reserve.x);
    let s2_ady = unipool::get_y_out(s1_adx, pair.pool1.reserve.x, pair.pool1.reserve.y);
    // let profit = s2_ady - oay_in as u128;

    Ok(Match {
        pair,
        pool0_ay_in: oay_in,
        pool0_ax_out: s1_adx,
        pool1_ay_out: s2_ady,
    })
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
              (SELECT contract_address, block_number, x,y, ROW_NUMBER() OVER(PARTITION BY contract_address ORDER BY block_number desc)
                FROM reserves ORDER BY contract_address, block_number)
              SELECT p1.contract_address as p1_contract_address,
                     p1.token0 as p1_token0,
                     p1.token1 as p1_token1,
                     p2.contract_address as p2_contract_address,
                     p2.token0 as p2_token0,
                     p2.token1 as p2_token1,
                     p1c0.symbol as p1_token0_symbol,
                     p1c1.symbol as p1_token1_symbol,
                     p2c0.symbol as p2_token0_symbol,
                     p2c1.symbol as p2_token1_symbol,
                     p1c0.decimals as p1_token0_decimals,
                     p1c1.decimals as p1_token1_decimals,
                     p2c0.decimals as p2_token0_decimals,
                     p2c1.decimals as p2_token1_decimals,
                     lrp1.x as qty_x1, lrp2.x AS qty_x2, lrp1.block_number AS p1_block_number,
                     lrp1.y as qty_y1, lrp2.y AS qty_y2, lrp2.block_number AS p2_block_number,
                     lrp1b.timestamp as p1_block_timestamp,
                     lrp2b.timestamp as p2_block_timestamp,
                     ((lrp1.x::decimal/lrp1.y::decimal) - (lrp2.x::decimal/lrp2.y::decimal))::float8 as spread,
                     (least(lrp1.x::decimal , lrp2.x::decimal ) *
                       ((lrp1.x::decimal/lrp1.y::decimal) - (lrp2.x::decimal/lrp2.y::decimal)))::float8 as value
              FROM pools AS p1
              JOIN pools AS p2 ON p1.token0 = p2.token0 AND p1.token1 = p2.token1 AND p1.contract_address != p2.contract_address AND p1.token0 = $1
              JOIN latest_reserves AS lrp1 ON p1.contract_address = lrp1.contract_address AND lrp1.row_number = 1
              JOIN latest_reserves AS lrp2 ON p2.contract_address = lrp2.contract_address AND lrp2.row_number = 1
              JOIN blocks as lrp1b ON lrp1b.number = lrp1.block_number
              JOIN blocks as lrp2b ON lrp2b.number = lrp2.block_number
              JOIN coins as p1c0 ON p1c0.contract_address = p1.token0
              JOIN coins as p1c1 ON p1c1.contract_address = p1.token1
              JOIN coins as p2c0 ON p2c0.contract_address = p2.token0
              JOIN coins as p2c1 ON p2c1.contract_address = p2.token1
              ORDER BY value desc";

    db.query(sql, &[&base_token])
}
