'use client';

import { useEffect, useRef, useCallback } from 'react';
import { performanceMonitor } from '@/lib/performance';

interface UsePerformanceOptions {
  componentName: string;
  enableLogging?: boolean;
  threshold?: number; // milliseconds
}

export function usePerformance({ 
  componentName, 
  enableLogging = process.env.NODE_ENV === 'development',
  threshold = 16 
}: UsePerformanceOptions) {
  const renderStartTime = useRef<number>(0);

  // Mark render start
  const markRenderStart = useCallback(() => {
    if (enableLogging) {
      renderStartTime.current = performance.now();
      performanceMonitor.markStart(`${componentName}-render`);
    }
  }, [componentName, enableLogging]);

  // Mark render end and log if slow
  const markRenderEnd = useCallback(() => {
    if (enableLogging && renderStartTime.current > 0) {
      const duration = performance.now() - renderStartTime.current;
      performanceMonitor.markEnd(`${componentName}-render`);
      
      if (duration > threshold) {
        performanceMonitor.logSlowRender(componentName, duration);
      }
    }
  }, [componentName, enableLogging, threshold]);

  // Auto-track component lifecycle
  useEffect(() => {
    markRenderStart();
    return markRenderEnd;
  });

  return {
    markRenderStart,
    markRenderEnd,
    measureRender: () => performanceMonitor.measureRender(`${componentName}-render`)
  };
}

export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

export function useThrottle<T extends (...args: any[]) => any>(
  callback: T,
  delay: number
): T {
  const lastExecution = useRef<number>(0);
  const timeoutRef = useRef<NodeJS.Timeout>();

  return useCallback((...args: Parameters<T>) => {
    const now = Date.now();

    if (now - lastExecution.current >= delay) {
      callback(...args);
      lastExecution.current = now;
    } else {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      
      timeoutRef.current = setTimeout(() => {
        callback(...args);
        lastExecution.current = Date.now();
      }, delay - (now - lastExecution.current));
    }
  }, [callback, delay]) as T;
}

import { useState } from 'react';

export function useLocalStorage<T>(
  key: string,
  initialValue: T
): [T, (value: T | ((val: T) => T)) => void] {
  // State to store our value
  const [storedValue, setStoredValue] = useState<T>(() => {
    if (typeof window === 'undefined') {
      return initialValue;
    }
    
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.log(error);
      return initialValue;
    }
  });

  // Return a wrapped version of useState's setter function that persists the new value to localStorage
  const setValue = (value: T | ((val: T) => T)) => {
    try {
      // Allow value to be a function so we have the same API as useState
      const valueToStore = value instanceof Function ? value(storedValue) : value;
      setStoredValue(valueToStore);
      
      if (typeof window !== 'undefined') {
        window.localStorage.setItem(key, JSON.stringify(valueToStore));
      }
    } catch (error) {
      console.log(error);
    }
  };

  return [storedValue, setValue];
}

export function useIntersectionObserver(
  elementRef: React.RefObject<Element>,
  options?: IntersectionObserverInit
) {
  const [isIntersecting, setIsIntersecting] = useState(false);
  const [hasIntersected, setHasIntersected] = useState(false);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        const intersecting = entry.isIntersecting;
        setIsIntersecting(intersecting);
        
        if (intersecting && !hasIntersected) {
          setHasIntersected(true);
        }
      },
      options
    );

    observer.observe(element);

    return () => {
      observer.unobserve(element);
    };
  }, [elementRef, options, hasIntersected]);

  return { isIntersecting, hasIntersected };
}