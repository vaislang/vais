//! 인라인 캐싱 (Inline Caching)
//!
//! 함수 호출 지점의 타겟을 캐싱하여 간접 호출 오버헤드를 제거합니다.
//! - Monomorphic IC: 단일 타겟
//! - Polymorphic IC: 여러 타겟 (최대 4개)
//! - Megamorphic: 캐시 포기, 일반 디스패치

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// IC 상태
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ICState {
    /// 초기화되지 않음
    Uninitialized,
    /// 단일 타겟 (가장 빠름)
    Monomorphic,
    /// 여러 타겟 (최대 4개)
    Polymorphic,
    /// 캐시 포기 (느림)
    Megamorphic,
}

/// 캐시 엔트리
#[derive(Debug)]
pub struct CacheEntry {
    /// 타겟 함수 이름
    pub target: String,
    /// 타겟 함수 포인터 (JIT 컴파일된 경우)
    pub fn_ptr: Option<*const u8>,
    /// 히트 카운트
    pub hits: AtomicU64,
}

impl Clone for CacheEntry {
    fn clone(&self) -> Self {
        Self {
            target: self.target.clone(),
            fn_ptr: self.fn_ptr,
            hits: AtomicU64::new(self.hits.load(Ordering::Relaxed)),
        }
    }
}

impl CacheEntry {
    pub fn new(target: String) -> Self {
        Self {
            target,
            fn_ptr: None,
            hits: AtomicU64::new(0),
        }
    }

    pub fn with_ptr(target: String, fn_ptr: *const u8) -> Self {
        Self {
            target,
            fn_ptr: Some(fn_ptr),
            hits: AtomicU64::new(0),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }
}

// *const u8를 안전하게 공유하기 위한 래퍼
unsafe impl Send for CacheEntry {}
unsafe impl Sync for CacheEntry {}

/// 인라인 캐시
#[derive(Debug)]
pub struct InlineCache {
    /// 캐시 ID
    pub id: u64,
    /// 호출 지점 (함수 이름 + 오프셋)
    pub call_site: String,
    /// 현재 상태
    state: AtomicUsize, // ICState as usize
    /// 캐시 엔트리들 (최대 4개)
    entries: Vec<CacheEntry>,
    /// 총 호출 횟수
    total_calls: AtomicU64,
    /// 캐시 미스 횟수
    misses: AtomicU64,
}

impl InlineCache {
    pub fn new(id: u64, call_site: String) -> Self {
        Self {
            id,
            call_site,
            state: AtomicUsize::new(ICState::Uninitialized as usize),
            entries: Vec::with_capacity(4),
            total_calls: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// 현재 상태 조회
    pub fn state(&self) -> ICState {
        match self.state.load(Ordering::Relaxed) {
            0 => ICState::Uninitialized,
            1 => ICState::Monomorphic,
            2 => ICState::Polymorphic,
            _ => ICState::Megamorphic,
        }
    }

    /// 타겟 조회 (캐시 히트 시 Some 반환)
    pub fn lookup(&self, target: &str) -> Option<&CacheEntry> {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        for entry in &self.entries {
            if entry.target == target {
                entry.record_hit();
                return Some(entry);
            }
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// 타겟 추가/업데이트
    pub fn update(&mut self, target: String, fn_ptr: Option<*const u8>) {
        // 이미 존재하는지 확인
        for entry in &mut self.entries {
            if entry.target == target {
                if fn_ptr.is_some() {
                    entry.fn_ptr = fn_ptr;
                }
                return;
            }
        }

        // 새 엔트리 추가
        let entry = if let Some(ptr) = fn_ptr {
            CacheEntry::with_ptr(target, ptr)
        } else {
            CacheEntry::new(target)
        };

        match self.entries.len() {
            0 => {
                self.entries.push(entry);
                self.state.store(ICState::Monomorphic as usize, Ordering::Relaxed);
            }
            1..=3 => {
                self.entries.push(entry);
                self.state.store(ICState::Polymorphic as usize, Ordering::Relaxed);
            }
            _ => {
                // 4개 초과 - megamorphic으로 전환
                self.state.store(ICState::Megamorphic as usize, Ordering::Relaxed);
                // 가장 적게 사용된 엔트리 교체
                if let Some(min_idx) = self.entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.hits.load(Ordering::Relaxed))
                    .map(|(i, _)| i)
                {
                    self.entries[min_idx] = entry;
                }
            }
        }
    }

    /// 캐시 히트율
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_calls.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (total - misses) as f64 / total as f64
        }
    }

    /// 통계 문자열
    pub fn stats_string(&self) -> String {
        let total = self.total_calls.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let hit_rate = self.hit_rate() * 100.0;

        format!(
            "IC[{}] {} - {:?}, entries: {}, calls: {}, misses: {}, hit rate: {:.1}%",
            self.id,
            self.call_site,
            self.state(),
            self.entries.len(),
            total,
            misses,
            hit_rate
        )
    }
}

/// 인라인 캐시 매니저
pub struct ICManager {
    /// 호출 지점별 캐시
    caches: HashMap<String, InlineCache>,
    /// 다음 캐시 ID
    next_id: AtomicU64,
}

impl ICManager {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// 캐시 가져오기 또는 생성
    pub fn get_or_create(&mut self, call_site: &str) -> &mut InlineCache {
        if !self.caches.contains_key(call_site) {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            self.caches.insert(
                call_site.to_string(),
                InlineCache::new(id, call_site.to_string()),
            );
        }
        self.caches.get_mut(call_site).unwrap()
    }

    /// 캐시 조회
    pub fn get(&self, call_site: &str) -> Option<&InlineCache> {
        self.caches.get(call_site)
    }

    /// 함수 포인터 업데이트 (JIT 컴파일 후)
    pub fn update_fn_ptr(&mut self, func_name: &str, fn_ptr: *const u8) {
        for cache in self.caches.values_mut() {
            for entry in &mut cache.entries {
                if entry.target == func_name {
                    entry.fn_ptr = Some(fn_ptr);
                }
            }
        }
    }

    /// 전체 통계
    pub fn print_stats(&self) {
        println!("\n=== Inline Cache Statistics ===");

        let mut caches: Vec<_> = self.caches.values().collect();
        caches.sort_by(|a, b| {
            b.total_calls.load(Ordering::Relaxed)
                .cmp(&a.total_calls.load(Ordering::Relaxed))
        });

        for cache in caches.iter().take(20) {
            println!("  {}", cache.stats_string());
        }

        // 요약
        let total_caches = self.caches.len();
        let monomorphic = self.caches.values()
            .filter(|c| c.state() == ICState::Monomorphic)
            .count();
        let polymorphic = self.caches.values()
            .filter(|c| c.state() == ICState::Polymorphic)
            .count();
        let megamorphic = self.caches.values()
            .filter(|c| c.state() == ICState::Megamorphic)
            .count();

        println!("\nSummary:");
        println!("  Total caches: {}", total_caches);
        println!("  Monomorphic: {} ({:.1}%)", monomorphic, monomorphic as f64 / total_caches as f64 * 100.0);
        println!("  Polymorphic: {} ({:.1}%)", polymorphic, polymorphic as f64 / total_caches as f64 * 100.0);
        println!("  Megamorphic: {} ({:.1}%)", megamorphic, megamorphic as f64 / total_caches as f64 * 100.0);

        let avg_hit_rate: f64 = self.caches.values()
            .map(|c| c.hit_rate())
            .sum::<f64>() / total_caches as f64;
        println!("  Average hit rate: {:.1}%", avg_hit_rate * 100.0);
    }

    /// 모든 캐시 초기화
    pub fn clear(&mut self) {
        self.caches.clear();
    }
}

impl Default for ICManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 빠른 함수 호출을 위한 디스패치 테이블
pub struct DispatchTable {
    /// 함수 이름 -> 함수 포인터
    entries: HashMap<String, *const u8>,
}

impl DispatchTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// 함수 등록
    pub fn register(&mut self, name: String, ptr: *const u8) {
        self.entries.insert(name, ptr);
    }

    /// 함수 조회
    pub fn lookup(&self, name: &str) -> Option<*const u8> {
        self.entries.get(name).copied()
    }

    /// 함수 제거
    pub fn remove(&mut self, name: &str) -> Option<*const u8> {
        self.entries.remove(name)
    }

    /// 등록된 함수 수
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 비어있는지 확인
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for DispatchTable {
    fn default() -> Self {
        Self::new()
    }
}

// DispatchTable의 포인터를 안전하게 공유
unsafe impl Send for DispatchTable {}
unsafe impl Sync for DispatchTable {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ic_state_transitions() {
        let mut ic = InlineCache::new(1, "test_site".to_string());

        assert_eq!(ic.state(), ICState::Uninitialized);

        // 첫 번째 타겟 추가 -> Monomorphic
        ic.update("func_a".to_string(), None);
        assert_eq!(ic.state(), ICState::Monomorphic);

        // 두 번째 타겟 추가 -> Polymorphic
        ic.update("func_b".to_string(), None);
        assert_eq!(ic.state(), ICState::Polymorphic);

        // 세 번째, 네 번째 -> 여전히 Polymorphic
        ic.update("func_c".to_string(), None);
        ic.update("func_d".to_string(), None);
        assert_eq!(ic.state(), ICState::Polymorphic);

        // 다섯 번째 -> Megamorphic
        ic.update("func_e".to_string(), None);
        assert_eq!(ic.state(), ICState::Megamorphic);
    }

    #[test]
    fn test_ic_lookup() {
        let mut ic = InlineCache::new(1, "test".to_string());

        ic.update("target_func".to_string(), None);

        // 캐시 히트
        assert!(ic.lookup("target_func").is_some());

        // 캐시 미스
        assert!(ic.lookup("other_func").is_none());

        // 히트율 확인
        assert!(ic.hit_rate() > 0.0);
    }

    #[test]
    fn test_ic_manager() {
        let mut manager = ICManager::new();

        // 캐시 생성
        let cache = manager.get_or_create("call_site_1");
        cache.update("func_a".to_string(), None);

        // 조회
        let cache = manager.get("call_site_1").unwrap();
        assert_eq!(cache.state(), ICState::Monomorphic);

        // 함수 포인터 업데이트
        let fake_ptr = 0x1000 as *const u8;
        manager.update_fn_ptr("func_a", fake_ptr);

        let cache = manager.get("call_site_1").unwrap();
        let entry = cache.lookup("func_a").unwrap();
        assert_eq!(entry.fn_ptr, Some(fake_ptr));
    }

    #[test]
    fn test_dispatch_table() {
        let mut table = DispatchTable::new();

        let ptr1 = 0x1000 as *const u8;
        let ptr2 = 0x2000 as *const u8;

        table.register("func_a".to_string(), ptr1);
        table.register("func_b".to_string(), ptr2);

        assert_eq!(table.lookup("func_a"), Some(ptr1));
        assert_eq!(table.lookup("func_b"), Some(ptr2));
        assert_eq!(table.lookup("func_c"), None);

        assert_eq!(table.len(), 2);
    }
}
