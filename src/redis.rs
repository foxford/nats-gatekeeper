use std::env::var;
use svc_authz::cache::{create_pool, AuthzCache, RedisCache};

pub fn create_redis() -> Option<Box<dyn AuthzCache>> {
    if let Some("1") = var("CACHE_ENABLED").ok().as_deref() {
        let url = var("CACHE_URL").expect("CACHE_URL must be specified");

        let size = var("CACHE_POOL_SIZE")
            .map(|val| {
                val.parse::<u32>()
                    .expect("Error converting CACHE_POOL_SIZE variable into u32")
            })
            .unwrap_or_else(|_| 5);

        let idle_size = var("CACHE_POOL_IDLE_SIZE")
            .map(|val| {
                val.parse::<u32>()
                    .expect("Error converting CACHE_POOL_IDLE_SIZE variable into u32")
            })
            .ok();

        let timeout = var("CACHE_POOL_TIMEOUT")
            .map(|val| {
                val.parse::<u64>()
                    .expect("Error converting CACHE_POOL_TIMEOUT variable into u64")
            })
            .unwrap_or_else(|_| 5);

        let expiration_time = var("CACHE_EXPIRATION_TIME")
            .map(|val| {
                val.parse::<usize>()
                    .expect("Error converting CACHE_EXPIRATION_TIME variable into u64")
            })
            .unwrap_or_else(|_| 300);

        let pool = create_pool(&url, size, idle_size, timeout);
        let cache = Box::new(RedisCache::new(pool, expiration_time)) as Box<dyn AuthzCache>;
        Some(cache)
    } else {
        None
    }
}
