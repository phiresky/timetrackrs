use lazy_static::lazy_static;
use sqlx::SqlitePool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/*
https://stackoverflow.com/questions/41665345/borrow-problems-with-compiled-sql-statements
https://stackoverflow.com/questions/32209391/how-to-store-rusqlite-connection-and-statement-objects-in-the-same-struct-in-rus
https://stackoverflow.com/questions/27552670/how-to-store-sqlite-prepared-statements-for-later
*/

#[derive(Clone)]
pub struct CachingIntMap {
    lru: Arc<RwLock<HashMap<String, i64>>>,
    pub conn: SqlitePool,
    get: String,
    put: String,
}

type IntCache = Arc<RwLock<HashMap<String, i64>>>;
impl<'c> CachingIntMap {
    pub async fn new(conn: SqlitePool, table: &str, cols: &str, keycol: &str) -> CachingIntMap {
        lazy_static! {
            static ref LRUS: Arc<RwLock<HashMap<String, IntCache>>> =
                Arc::new(RwLock::new(HashMap::new()));
        }
        CachingIntMap {
            lru: (*LRUS)
                .write()
                .await
                .entry(table.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(HashMap::with_capacity(10_000))))
                .clone(),
            get: format!("select id from {} where {} = ?1", table, keycol),
            put: format!( // the on conflict clause resolves a race condition by returning the existing id
                "insert into {} {} on conflict ({}) do update set id=id returning id",
                table, cols, keycol
            ),
            conn,
        }
    }
    pub async fn get(&self, key: &str) -> i64 {
        let i: Option<i64> = self.lru.write().await.get(key).copied();

        match i {
            Some(i) => i,
            None => {
                let n = match sqlx::query_scalar(&self.get)
                    .bind(key)
                    .fetch_optional(&self.conn)
                    .await
                    .unwrap()
                {
                    Some(n) => n,
                    None => {
                        let q = sqlx::query_scalar(&self.put).bind(key);
                        let ret = q.fetch_one(&self.conn).await.unwrap();
                        ret
                    }
                };

                self.lru.write().await.insert(key.to_string(), n);
                n
            }
        }
    }
    // very shitty code. spent 2 hours figuring out how to do this get method generically
    pub async fn get_bind2(&'c self, key: &'c str, bind1: i64, bind2: i64) -> i64 {
        let i: Option<i64> = self.lru.write().await.get(key).copied();

        match i {
            Some(i) => i,
            None => {
                let n = match sqlx::query_scalar(&self.get)
                    .bind(key)
                    .fetch_optional(&self.conn)
                    .await
                    .unwrap()
                {
                    Some(n) => n,
                    None => {
                        let q = sqlx::query(&self.put).bind(key).bind(bind1).bind(bind2);
                        let ret = q.execute(&self.conn).await.unwrap();
                        ret.last_insert_rowid()
                    }
                };

                self.lru.write().await.insert(key.to_string(), n);
                n
            }
        }
    }
}
