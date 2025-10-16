'use client';

import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';

interface NavItem {
  title: string;
  href: string;
  description: string;
}

const navigation: NavItem[] = [
  {
    title: 'Dashboard',
    href: '/',
    description: 'Overview and metrics'
  },
  {
    title: 'Jobs',
    href: '/jobs',
    description: 'Automation jobs'
  },
  {
    title: 'Keepers',
    href: '/keepers',
    description: 'Keeper network'
  },
  {
    title: 'Analytics',
    href: '/analytics',
    description: 'Performance insights'
  },
  {
    title: 'Create Job',
    href: '/jobs/create',
    description: 'New automation'
  }
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <div className="w-64 bg-white dark:bg-slate-900 border-r border-gray-200 dark:border-slate-700 flex flex-col">
      {/* Logo */}
      <div className="px-6 py-4 border-b border-gray-200 dark:border-slate-700">
        <div className="flex items-center space-x-3">
          <div className="w-8 h-8 bg-chainlink-blue rounded-lg flex items-center justify-center">
            <span className="text-white font-semibold text-sm">S</span>
          </div>
          <div>
            <h1 className="font-semibold text-lg text-slate-900 dark:text-white">SolCron</h1>
            <p className="text-xs text-slate-500 dark:text-slate-400">Solana Automation</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 space-y-1">
        {navigation.map((item) => {
          const isActive = pathname === item.href;
          
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                'nav-item group',
                isActive ? 'nav-item-active' : 'nav-item-inactive'
              )}
            >
              <div className={`w-2 h-2 rounded-full mr-3 transition-colors ${
                isActive ? 'bg-chainlink-blue' : 'bg-slate-400 group-hover:bg-slate-500'
              }`} />
              <div className="flex-1 min-w-0">
                <div className="font-medium text-sm truncate">{item.title}</div>
                <div className="text-xs opacity-70 truncate">{item.description}</div>
              </div>
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-gray-200 dark:border-slate-700">
        <div className="bg-slate-50 dark:bg-slate-800 rounded-lg p-3">
          <div className="flex items-center space-x-2 mb-2">
            <div className="status-dot status-dot-active" />
            <span className="text-xs font-semibold text-slate-700 dark:text-slate-300">Network Status</span>
          </div>
          <div className="text-xs text-slate-500 dark:text-slate-400 space-y-1">
            <div className="flex justify-between">
              <span>Devnet</span>
              <span className="text-green-600 dark:text-green-400">Connected</span>
            </div>
            <div className="flex justify-between font-mono">
              <span>Block:</span>
              <span>247,832,156</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}