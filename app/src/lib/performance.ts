import { lazy } from 'react';

// Lazy load components for better performance
export const LazyJobsTable = lazy(() => import('@/components/dashboard/JobsTable').then(m => ({ default: m.JobsTable })));
export const LazyKeepersTable = lazy(() => import('@/components/dashboard/KeepersTable').then(m => ({ default: m.KeepersTable })));
export const LazySystemHealth = lazy(() => import('@/components/dashboard/SystemHealth').then(m => ({ default: m.SystemHealth })));

// Performance monitoring utilities
export const performanceMonitor = {
  // Mark performance milestones
  markStart: (name: string) => {
    if (typeof window !== 'undefined' && window.performance) {
      window.performance.mark(`${name}-start`);
    }
  },

  markEnd: (name: string) => {
    if (typeof window !== 'undefined' && window.performance) {
      window.performance.mark(`${name}-end`);
      window.performance.measure(name, `${name}-start`, `${name}-end`);
    }
  },

  // Measure component render times
  measureRender: (componentName: string) => {
    if (typeof window !== 'undefined' && window.performance) {
      const entries = window.performance.getEntriesByName(componentName);
      return entries.length > 0 ? entries[entries.length - 1].duration : 0;
    }
    return 0;
  },

  // Log slow renders (> 16ms for 60fps)
  logSlowRender: (componentName: string, duration: number) => {
    if (duration > 16) {
      console.warn(`Slow render detected in ${componentName}: ${duration.toFixed(2)}ms`);
    }
  }
};

// Debounce utility for expensive operations
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number,
  immediate?: boolean
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;
  
  return function executedFunction(...args: Parameters<T>) {
    const later = () => {
      timeout = null;
      if (!immediate) func(...args);
    };
    
    const callNow = immediate && !timeout;
    
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(later, wait);
    
    if (callNow) func(...args);
  };
}

// Throttle utility for scroll/resize events
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  
  return function executedFunction(...args: Parameters<T>) {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

// Memoization utility for expensive calculations
export function memoize<T extends (...args: any[]) => any>(
  fn: T,
  getKey?: (...args: Parameters<T>) => string
): T {
  const cache = new Map();
  
  return ((...args: Parameters<T>) => {
    const key = getKey ? getKey(...args) : JSON.stringify(args);
    
    if (cache.has(key)) {
      return cache.get(key);
    }
    
    const result = fn(...args);
    cache.set(key, result);
    
    // Clear cache if it gets too large
    if (cache.size > 100) {
      const firstKey = cache.keys().next().value;
      cache.delete(firstKey);
    }
    
    return result;
  }) as T;
}

// Image optimization utilities
export const imageOptimization = {
  // Preload critical images
  preloadImage: (src: string): Promise<void> => {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve();
      img.onerror = reject;
      img.src = src;
    });
  },

  // Lazy load images with intersection observer
  lazyLoadImage: (element: HTMLImageElement, src: string) => {
    if ('IntersectionObserver' in window) {
      const observer = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (entry.isIntersecting) {
              element.src = src;
              observer.unobserve(element);
            }
          });
        },
        { threshold: 0.1 }
      );
      observer.observe(element);
    } else {
      // Fallback for older browsers
      element.src = src;
    }
  }
};

// Bundle size optimization
export const bundleOptimization = {
  // Dynamic imports for code splitting
  loadModule: async <T>(importFn: () => Promise<{ default: T }>): Promise<T> => {
    try {
      const module = await importFn();
      return module.default;
    } catch (error) {
      console.error('Failed to load module:', error);
      throw error;
    }
  },

  // Check if feature is supported before loading polyfills
  loadPolyfillIfNeeded: async (feature: string, polyfillLoader: () => Promise<any>) => {
    if (!(feature in window)) {
      await polyfillLoader();
    }
  }
};