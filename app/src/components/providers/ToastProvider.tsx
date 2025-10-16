'use client';

import React, { createContext, useContext, useState, useCallback } from 'react';

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

interface ToastContextType {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;
  success: (title: string, message?: string) => void;
  error: (title: string, message?: string) => void;
  warning: (title: string, message?: string) => void;
  info: (title: string, message?: string) => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

interface ToastProviderProps {
  children: React.ReactNode;
}

export function ToastProvider({ children }: ToastProviderProps) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback((toast: Omit<Toast, 'id'>) => {
    const id = Math.random().toString(36).substring(2, 9);
    const newToast: Toast = { ...toast, id };
    
    setToasts(prev => [...prev, newToast]);
    
    // Auto remove after duration
    const duration = toast.duration || 5000;
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }, []);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(toast => toast.id !== id));
  }, []);

  const success = useCallback((title: string, message?: string) => {
    addToast({ type: 'success', title, message });
  }, [addToast]);

  const error = useCallback((title: string, message?: string) => {
    addToast({ type: 'error', title, message });
  }, [addToast]);

  const warning = useCallback((title: string, message?: string) => {
    addToast({ type: 'warning', title, message });
  }, [addToast]);

  const info = useCallback((title: string, message?: string) => {
    addToast({ type: 'info', title, message });
  }, [addToast]);

  const value: ToastContextType = {
    toasts,
    addToast,
    removeToast,
    success,
    error,
    warning,
    info,
  };

  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastContainer />
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextType {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}

// Toast Container Component
function ToastContainer() {
  const { toasts, removeToast } = useToast();

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2">
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onRemove={removeToast} />
      ))}
    </div>
  );
}

// Individual Toast Component
interface ToastItemProps {
  toast: Toast;
  onRemove: (id: string) => void;
}

function ToastItem({ toast, onRemove }: ToastItemProps) {
  const getToastStyles = (type: Toast['type']) => {
    const baseStyles = "max-w-md p-4 rounded-lg shadow-lg border backdrop-blur-sm animate-slide-in";
    
    switch (type) {
      case 'success':
        return `${baseStyles} bg-success-50 border-success-200 text-success-800 dark:bg-success-900/20 dark:border-success-800 dark:text-success-200`;
      case 'error':
        return `${baseStyles} bg-error-50 border-error-200 text-error-800 dark:bg-error-900/20 dark:border-error-800 dark:text-error-200`;
      case 'warning':
        return `${baseStyles} bg-warning-50 border-warning-200 text-warning-800 dark:bg-warning-900/20 dark:border-warning-800 dark:text-warning-200`;
      case 'info':
        return `${baseStyles} bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/20 dark:border-blue-800 dark:text-blue-200`;
      default:
        return `${baseStyles} bg-white border-gray-200 text-gray-800 dark:bg-gray-900/20 dark:border-gray-800 dark:text-gray-200`;
    }
  };

  const getIcon = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '⚠';
      case 'info':
        return 'ℹ';
      default:
        return '';
    }
  };

  return (
    <div className={getToastStyles(toast.type)}>
      <div className="flex items-start">
        <div className="flex-shrink-0 text-lg mr-3">
          {getIcon(toast.type)}
        </div>
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium">
            {toast.title}
          </div>
          {toast.message && (
            <div className="text-xs mt-1 opacity-90">
              {toast.message}
            </div>
          )}
        </div>
        <button
          onClick={() => onRemove(toast.id)}
          className="flex-shrink-0 ml-4 text-lg opacity-70 hover:opacity-100 transition-opacity"
        >
          ×
        </button>
      </div>
    </div>
  );
}