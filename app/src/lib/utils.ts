import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

const LAMPORTS_PER_SOL = 1e9;

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatSOL(lamports: number | string, decimals: number = 4): string {
  const sol = typeof lamports === 'string' ? parseFloat(lamports) : lamports;
  return (sol / LAMPORTS_PER_SOL).toFixed(decimals);
}

export function formatLamports(lamports: number): string {
  return lamports.toLocaleString();
}

export function formatAddress(address: string, chars: number = 4): string {
  if (!address) return '';
  if (address.length <= chars * 2) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

export function formatNumber(num: number, decimals: number = 2): string {
  if (num >= 1e9) {
    return (num / 1e9).toFixed(decimals) + 'B';
  }
  if (num >= 1e6) {
    return (num / 1e6).toFixed(decimals) + 'M';
  }
  if (num >= 1e3) {
    return (num / 1e3).toFixed(decimals) + 'K';
  }
  return num.toFixed(decimals);
}

export function formatPercentage(value: number, decimals = 1): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

export function formatTimeAgo(timestamp: number): string {
  const now = Date.now() / 1000;
  const diff = now - timestamp;
  
  if (diff < 60) return `${Math.floor(diff)}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

export function formatDuration(seconds: number): string {
  const units = [
    { name: 'day', seconds: 86400 },
    { name: 'hour', seconds: 3600 },
    { name: 'minute', seconds: 60 },
    { name: 'second', seconds: 1 }
  ];

  for (const unit of units) {
    const count = Math.floor(seconds / unit.seconds);
    if (count > 0) {
      return `${count} ${unit.name}${count !== 1 ? 's' : ''}`;
    }
  }
  
  return '0 seconds';
}

export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

export function getJobStatusColor(isActive: boolean, hasError: boolean = false): string {
  if (hasError) return 'error';
  if (isActive) return 'success';
  return 'inactive';
}

export function getJobStatusText(isActive: boolean, hasError: boolean = false): string {
  if (hasError) return 'Error';
  if (isActive) return 'Active';
  return 'Inactive';
}

export function calculateSuccessRate(successful: number, total: number): number {
  if (total === 0) return 100;
  return (successful / total) * 100;
}

export function generateMockAddress(): string {
  const chars = 'ABCDEFGHJKMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz123456789';
  let result = '';
  for (let i = 0; i < 44; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}