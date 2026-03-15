/// ClawMesh 缓存层
/// 
/// 提供高性能的内存缓存

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 全局缓存实例
pub static CACHE: Lazy<ClawMeshCache> = Lazy::new(ClawMeshCache::new);

/// 缓存项
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Option<Instant>,
}

/// ClawMesh 缓存
pub struct ClawMeshCache {
    /// 用户信用缓存
    credit_cache: DashMap<i32, CacheEntry<i32>>,
    /// 用户等级缓存
    tier_cache: DashMap<i32, CacheEntry<String>>,
    /// 统计数据缓存
    stats_cache: DashMap<String, CacheEntry<String>>,
}

impl ClawMeshCache {
    /// 创建新的缓存实例
    pub fn new() -> Self {
        Self {
            credit_cache: DashMap::new(),
            tier_cache: DashMap::new(),
            stats_cache: DashMap::new(),
        }
    }

    /// 获取用户信用（带缓存）
    pub fn get_credit(&self, person_id: i32) -> Option<i32> {
        self.credit_cache.get(&person_id).and_then(|entry| {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    return None;
                }
            }
            Some(entry.value)
        })
    }

    /// 设置用户信用缓存
    pub fn set_credit(&self, person_id: i32, credit: i32, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        self.credit_cache.insert(
            person_id,
            CacheEntry {
                value: credit,
                expires_at,
            },
        );
    }

    /// 使用户信用缓存失效
    pub fn invalidate_credit(&self, person_id: i32) {
        self.credit_cache.remove(&person_id);
    }

    /// 获取用户等级（带缓存）
    pub fn get_tier(&self, person_id: i32) -> Option<String> {
        self.tier_cache.get(&person_id).and_then(|entry| {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    return None;
                }
            }
            Some(entry.value.clone())
        })
    }

    /// 设置用户等级缓存
    pub fn set_tier(&self, person_id: i32, tier: String, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        self.tier_cache.insert(
            person_id,
            CacheEntry {
                value: tier,
                expires_at,
            },
        );
    }

    /// 使用户等级缓存失效
    pub fn invalidate_tier(&self, person_id: i32) {
        self.tier_cache.remove(&person_id);
    }

    /// 获取统计数据（带缓存）
    pub fn get_stats(&self, key: &str) -> Option<String> {
        self.stats_cache.get(key).and_then(|entry| {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    return None;
                }
            }
            Some(entry.value.clone())
        })
    }

    /// 设置统计数据缓存
    pub fn set_stats(&self, key: String, value: String, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        self.stats_cache.insert(
            key,
            CacheEntry {
                value,
                expires_at,
            },
        );
    }

    /// 清除所有缓存
    pub fn clear_all(&self) {
        self.credit_cache.clear();
        self.tier_cache.clear();
        self.stats_cache.clear();
    }

    /// 清除过期缓存
    pub fn cleanup_expired(&self) {
        let now = Instant::now();

        self.credit_cache.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| now <= exp)
        });

        self.tier_cache.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| now <= exp)
        });

        self.stats_cache.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| now <= exp)
        });
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            credit_entries: self.credit_cache.len(),
            tier_entries: self.tier_cache.len(),
            stats_entries: self.stats_cache.len(),
        }
    }
}

impl Default for ClawMeshCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub credit_entries: usize,
    pub tier_entries: usize,
    pub stats_entries: usize,
}

/// 获取全局缓存实例
pub fn get_cache() -> &'static ClawMeshCache {
    &CACHE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_cache() {
        let cache = ClawMeshCache::new();
        
        // 设置缓存
        cache.set_credit(1, 100, Some(Duration::from_secs(60)));
        
        // 获取缓存
        assert_eq!(cache.get_credit(1), Some(100));
        
        // 使缓存失效
        cache.invalidate_credit(1);
        assert_eq!(cache.get_credit(1), None);
    }

    #[test]
    fn test_tier_cache() {
        let cache = ClawMeshCache::new();
        
        cache.set_tier(1, "regular".to_string(), Some(Duration::from_secs(60)));
        assert_eq!(cache.get_tier(1), Some("regular".to_string()));
        
        cache.invalidate_tier(1);
        assert_eq!(cache.get_tier(1), None);
    }

    #[test]
    fn test_stats_cache() {
        let cache = ClawMeshCache::new();
        
        cache.set_stats("global".to_string(), "{}".to_string(), Some(Duration::from_secs(60)));
        assert_eq!(cache.get_stats("global"), Some("{}".to_string()));
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ClawMeshCache::new();
        
        // 设置一个立即过期的缓存
        cache.set_credit(1, 100, Some(Duration::from_millis(1)));
        
        // 等待过期
        std::thread::sleep(Duration::from_millis(10));
        
        // 应该返回 None
        assert_eq!(cache.get_credit(1), None);
    }

    #[test]
    fn test_cache_stats() {
        let cache = ClawMeshCache::new();
        
        cache.set_credit(1, 100, None);
        cache.set_tier(1, "regular".to_string(), None);
        cache.set_stats("key".to_string(), "value".to_string(), None);
        
        let stats = cache.stats();
        assert_eq!(stats.credit_entries, 1);
        assert_eq!(stats.tier_entries, 1);
        assert_eq!(stats.stats_entries, 1);
    }
}
