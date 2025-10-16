// Simple in-memory cache with TTL (Time To Live)
interface CacheItem<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

class DataCache {
  private cache = new Map<string, CacheItem<any>>();
  private readonly defaultTTL = 5 * 60 * 1000; // 5 minutes

  set<T>(key: string, data: T, ttl?: number): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl: ttl || this.defaultTTL
    });
  }

  get<T>(key: string): T | null {
    const item = this.cache.get(key);
    if (!item) return null;

    const now = Date.now();
    const isExpired = now - item.timestamp > item.ttl;

    if (isExpired) {
      this.cache.delete(key);
      return null;
    }

    return item.data;
  }

  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  has(key: string): boolean {
    const item = this.cache.get(key);
    if (!item) return false;

    const now = Date.now();
    const isExpired = now - item.timestamp > item.ttl;

    if (isExpired) {
      this.cache.delete(key);
      return false;
    }

    return true;
  }

  // Clean up expired items
  cleanup(): void {
    const now = Date.now();
    const keysToDelete: string[] = [];
    
    this.cache.forEach((item, key) => {
      if (now - item.timestamp > item.ttl) {
        keysToDelete.push(key);
      }
    });
    
    keysToDelete.forEach(key => this.cache.delete(key));
  }

  // Get cache statistics
  getStats(): { size: number; keys: string[] } {
    return {
      size: this.cache.size,
      keys: Array.from(this.cache.keys())
    };
  }
}

// Export singleton instance
export const dataCache = new DataCache();

// Cache keys for different data types
export const CacheKeys = {
  DASHBOARD_METRICS: 'dashboard:metrics',
  JOBS_LIST: 'jobs:list',
  KEEPERS_LIST: 'keepers:list',
  RECENT_EXECUTIONS: 'executions:recent',
  SYSTEM_HEALTH: 'system:health',
  WALLET_BALANCE: 'wallet:balance',
  NETWORK_STATUS: 'network:status'
} as const;

// Cache utilities for specific data types
export const cacheUtils = {
  // Cache dashboard metrics for 2 minutes
  setMetrics: (data: any) => {
    dataCache.set(CacheKeys.DASHBOARD_METRICS, data, 2 * 60 * 1000);
  },

  getMetrics: () => {
    return dataCache.get(CacheKeys.DASHBOARD_METRICS);
  },

  // Cache jobs list for 1 minute
  setJobs: (data: any) => {
    dataCache.set(CacheKeys.JOBS_LIST, data, 60 * 1000);
  },

  getJobs: () => {
    return dataCache.get(CacheKeys.JOBS_LIST);
  },

  // Cache keepers list for 1 minute
  setKeepers: (data: any) => {
    dataCache.set(CacheKeys.KEEPERS_LIST, data, 60 * 1000);
  },

  getKeepers: () => {
    return dataCache.get(CacheKeys.KEEPERS_LIST);
  },

  // Cache recent executions for 30 seconds
  setExecutions: (data: any) => {
    dataCache.set(CacheKeys.RECENT_EXECUTIONS, data, 30 * 1000);
  },

  getExecutions: () => {
    return dataCache.get(CacheKeys.RECENT_EXECUTIONS);
  },

  // Cache system health for 10 seconds
  setSystemHealth: (data: any) => {
    dataCache.set(CacheKeys.SYSTEM_HEALTH, data, 10 * 1000);
  },

  getSystemHealth: () => {
    return dataCache.get(CacheKeys.SYSTEM_HEALTH);
  },

  // Invalidate related caches when data changes
  invalidateJobsData: () => {
    dataCache.delete(CacheKeys.JOBS_LIST);
    dataCache.delete(CacheKeys.DASHBOARD_METRICS);
    dataCache.delete(CacheKeys.RECENT_EXECUTIONS);
  },

  invalidateKeepersData: () => {
    dataCache.delete(CacheKeys.KEEPERS_LIST);
    dataCache.delete(CacheKeys.DASHBOARD_METRICS);
  },

  invalidateAllDashboard: () => {
    Object.values(CacheKeys).forEach(key => {
      dataCache.delete(key);
    });
  }
};

// Auto cleanup expired items every 5 minutes
if (typeof window !== 'undefined') {
  setInterval(() => {
    dataCache.cleanup();
  }, 5 * 60 * 1000);
}