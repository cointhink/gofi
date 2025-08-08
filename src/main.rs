#![allow(dead_code)]

use std::cmp;

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
    let geth_url = Url::parse(&config.geth_url).unwrap();
    let pk_signer: PrivateKeySigner = config.eth_priv_key.parse().unwrap();
    let my_address = pk_signer.address();
    let provider = ProviderBuilder::new()
        .wallet(pk_signer)
        .with_gas_estimation()
        .connect_http(geth_url.clone());
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
    let mut matches = simulate(&pairs);
    matches.sort_by(|a, b| b.scaled_profit().partial_cmp(&a.scaled_profit()).unwrap());

    let gas_price_wei = get_gas_price(&provider);
    println!(
        "{} pools make {} pairs. {} matches for {} gas {:.1}",
        pools_count,
        pairs.len(),
        matches.len(),
        config.preferred_base_token,
        decimal::scale(gas_price_wei, 10_u128.pow(9))
    );

    let matches_preferred = matches
        .iter()
        .filter(|mtch| {
            mtch.pair.pool0.pool.coin1.contract_address == config.preferred_coin_token
                && !config
                    .exclude_addresses
                    .contains(&mtch.pair.pool0.pool.contract_address)
                && !config
                    .exclude_addresses
                    .contains(&mtch.pair.pool1.pool.contract_address)
        })
        .collect::<Vec<&Match>>();

    let gas_cost_wei = gas_price_wei * config.tx_gas as u128;
    for r#match in matches_preferred.iter().take(10) {
        println!("{}", r#match.to_string(gas_cost_wei));
    }

    let winners_profitable = matches_preferred
        .into_iter()
        .filter(|mtch| approval(mtch, gas_cost_wei))
        .collect::<Vec<&Match>>();

    if winners_profitable.len() > 0 {
        for winner in winners_profitable[0..1].into_iter() {
            println!("===========================================================");
            maineth(winner, &provider, gas_cost_wei, my_address).unwrap();
        }
    } else {
        println!("no winners over {}", config.minimum_out);
    }

    Ok(())
}

fn approval(m: &Match, gas_cost_wei: u128) -> bool {
    let gas_cost_coin1 =
        unipool::get_y_out(gas_cost_wei, m.pair.pool0.reserve.x, m.pair.pool0.reserve.y);
    m.profit() > gas_cost_coin1
}

#[cfg(test)]
#[test]
fn test_approval() {
    // this much setup means refactoring is needed
    let coin0 = Coin {
        contract_address: "COIN0".to_owned(),
        symbol: "C0".to_owned(),
        decimals: 18,
    };
    let coin1 = Coin {
        contract_address: "COIN1".to_owned(),
        symbol: "C0".to_owned(),
        decimals: 6,
    };
    let m = Match {
        pair: Pair {
            pool0: PoolSnapshot {
                pool: Pool {
                    contract_address: "POOL-A0".to_owned(),
                    coin0: coin0.clone(),
                    coin1: coin1.clone(),
                },
                reserve: Reserve {
                    contract_address: "POOL-A1".to_owned(),
                    x: 37407681086137164,
                    y: 135629089,
                    block_number: 1,
                    block_timestamp: 1,
                },
            },
            pool1: PoolSnapshot {
                pool: Pool {
                    contract_address: "POOL-B0".to_owned(),
                    coin0: coin0.clone(),
                    coin1: coin1.clone(),
                },
                reserve: Reserve {
                    contract_address: "POOL-B1".to_owned(),
                    x: 276578510416029,
                    y: 1320886,
                    block_number: 1,
                    block_timestamp: 1,
                },
            },
        },
        pool0_ay_in: 144457,
        pool0_ax_out: 1,
        pool1_ay_out: 165295, // profit 20838
    };
    let gas_cost_wei = 1;
    assert!(approval(&m, gas_cost_wei));
}

#[tokio::main]
async fn get_gas_price<T: Provider>(provider: T) -> u128 {
    provider.get_gas_price().await.unwrap()
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
async fn maineth<T: Provider>(
    winner: &Match,
    provider: T,
    gas_cost_wei: u128,
    public_key: Address,
) -> Result<(), String> {
    let config = config::CONFIG.get().unwrap();
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

    let eth_balance_start = provider.get_balance(public_key).await.unwrap();
    println!(
        "{} eth: {}",
        public_key,
        format_units(eth_balance_start, 18).unwrap()
    );
    // erc20_allow(&public_key, uniswab.address(), &coin0).await;
    let coin0_balance_start = coin0.balanceOf(public_key).call().await.unwrap();
    println!(
        "{} {}: {}",
        public_key,
        winner.pair.pool0.pool.coin0.symbol,
        Into::<f64>::into(coin0_balance_start) / 10_f64.powi(winner.pair.pool0.pool.coin0.decimals),
    );
    // erc20_allow(&public_key, uniswab.address(), &coin1).await;
    let coin1_balance_start = coin1.balanceOf(public_key).call().await.unwrap();
    println!(
        "{} {}: {}",
        public_key,
        winner.pair.pool0.pool.coin1.symbol,
        Into::<f64>::into(coin1_balance_start) / 10_f64.powi(winner.pair.pool0.pool.coin1.decimals),
    );

    println!("winner: {}", winner.to_string(gas_cost_wei));
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
        let swab_amt = cmp::min(
            coin1_balance_start.saturating_to::<u128>(),
            winner.pool0_ay_in,
        );
        println!(
            "SWAB {} ({}/{}), {}, {}",
            swab_amt,
            winner.pool0_ay_in,
            coin1_balance_start,
            &winner.pair.pool0.pool.contract_address,
            &winner.pair.pool1.pool.contract_address,
        );
        let swab_tx = uniswab.swab(
            U256::from(swab_amt),
            Address::from_slice(&decode(&winner.pair.pool0.pool.contract_address).unwrap()),
            Address::from_slice(&decode(&winner.pair.pool1.pool.contract_address).unwrap()),
        );
        let swab_tx_receipt = swab_tx
            .gas(config.tx_gas)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
        println!("swab tx {}", swab_tx_receipt.transaction_hash);

        let eth_balance_end = provider.get_balance(public_key).await.unwrap();
        println!(
            "{} eth: {} delta {}",
            public_key,
            format_units(eth_balance_end, 18).unwrap(),
            format_units(eth_balance_start - eth_balance_end, 18).unwrap()
        );
        let coin0_balance_end = coin0.balanceOf(public_key).call().await.unwrap();
        println!(
            "{} {}: {} delta {}",
            public_key,
            winner.pair.pool0.pool.coin0.symbol,
            format_units(
                coin0_balance_end,
                winner.pair.pool0.pool.coin0.decimals.to_string()
            )
            .unwrap(),
            format_units(
                coin0_balance_start - coin0_balance_end,
                winner.pair.pool0.pool.coin0.decimals.to_string()
            )
            .unwrap()
        );
        let coin1_balance_end = coin1.balanceOf(public_key).call().await.unwrap();
        println!(
            "{} {}: {} delta {}",
            public_key,
            winner.pair.pool0.pool.coin1.symbol,
            format_units(
                coin1_balance_end,
                winner.pair.pool0.pool.coin1.decimals.to_string()
            )
            .unwrap(),
            format_units(
                coin1_balance_start - coin1_balance_end,
                winner.pair.pool0.pool.coin1.decimals.to_string()
            )
            .unwrap()
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
    if allowance == U256::from(0) {
        let tx = coin
            .approve(*spender_address, U256::MAX)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();
        println!("erc20 allownace tx: {}", hex::encode(tx.transaction_hash));
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
        let d0 = self.pool.coin0.decimals;
        let d1 = self.pool.coin1.decimals;
        let scale = decimal::scale(self.reserve.y, self.reserve.x);
        if d0 > d1 {
            scale * 10.0_f64.powf((d0 - d1) as f64)
        } else {
            scale / 10.0_f64.powf((d1 - d0) as f64)
        }
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
    pub fn to_string(self: &Self, gas_cost_wei: u128) -> String {
        format!(
            "{:0.4}{} profit:{:0.4}{} gas:{:0.4}{} p0:{} #{} p1:{} #{} ",
            self.pool0_ay_in as f64 / 10_f64.powi(self.pair.pool0.pool.coin1.decimals),
            self.pair.pool0.pool.coin1.symbol,
            self.scaled_profit(),
            self.pair.pool0.pool.coin1.symbol,
            decimal::scale(
                unipool::get_y_out(
                    gas_cost_wei,
                    self.pair.pool0.reserve.x,
                    self.pair.pool0.reserve.y
                ),
                10_u128.pow(6)
            ),
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
            Err(_err) => (),
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
        "{}1 price {} {}2 price {} {:.4}",
        if p1 < p2 { "P" } else { "p" },
        p1,
        if p1 > p2 { "P" } else { "p" },
        p2,
        p1 / p2
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
