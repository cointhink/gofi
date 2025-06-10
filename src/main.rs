use postgres::{Client, NoTls};

mod config;

fn main() -> Result<(), postgres::Error> {
    config::CONFIG
        .set(config::read_type(config::FILENAME))
        .unwrap();

    let config = config::CONFIG.get().unwrap();
    let mut db = Client::connect(&config.pg_url, NoTls)?;

    for row in pairs_with(&mut db, "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")? {
        let pool_contract_address_0: &str = row.get(0);
        let pool_contract_address_1: &str = row.get(1);
        let pool_block_0: &str = row.get(2);
        let pool_block_1: &str = row.get(3);

        println!(
            "pool pair {} #{} {} #{}",
            pool_contract_address_0, pool_block_0, pool_contract_address_1, pool_block_1
        );
    }

    Ok(())
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
