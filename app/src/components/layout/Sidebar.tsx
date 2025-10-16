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
    <div className="w-64 bg-card border-r border-border flex flex-col">
      {/* Logo */}
      <div className="p-6 border-b border-border">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 bg-gradient-to-br from-solana-500 to-purple-500 rounded-lg flex items-center justify-center">
            <span className="text-white font-bold text-lg">S</span>
          </div>
          <div>
            <h1 className="font-bold text-lg gradient-text">SolCron</h1>
            <p className="text-xs text-muted-foreground">Automation Platform</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-2">
        {navigation.map((item) => {
          const isActive = pathname === item.href;
          
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                'nav-item',
                isActive ? 'nav-item-active' : 'nav-item-inactive'
              )}
            >
              <div className={`w-3 h-3 rounded-sm mr-3 ${isActive ? 'bg-blue-500' : 'bg-gray-400'}`} />
              <div className="flex-1">
                <div className="font-medium text-sm">{item.title}</div>
                <div className="text-xs opacity-75">{item.description}</div>
              </div>
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-border">
        <div className="bg-muted rounded-lg p-3">
          <div className="flex items-center space-x-2 mb-2">
            <div className="status-dot status-active" />
            <span className="text-xs font-medium">Network Status</span>
          </div>
          <div className="text-xs text-muted-foreground">
            <div>Devnet Connected</div>
            <div>Block: 247,832,156</div>
          </div>
        </div>
      </div>
    </div>
  );
}