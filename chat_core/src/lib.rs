pub use models::*;

pub mod error;
pub mod middlewares;
pub mod models;
pub mod utils;

#[cfg(test)]
mod test_util {
    use sqlx::PgPool;
    use sqlx_db_tester::TestPg;

    pub async fn get_test_pool(url: Option<&str>) -> (PgPool, TestPg) {
        let url = url.unwrap_or("postgres://postgres:postgres@localhost:5432");
        let tdb = TestPg::new(url.to_string(), std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // let sqls = include_str!("../fixtures/test.sql").split(';');
        // let mut tx = pool.begin().await.expect("Failed to begin transaction");
        // for sql in sqls {
        //     if sql.trim().is_empty() {
        //         continue;
        //     }
        //     tx.execute(sql).await.expect("Failed to execute sql");
        // }
        // tx.commit().await.expect("Failed to commit transaction");

        (pool, tdb)
    }
}
