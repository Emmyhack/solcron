'use client';

import React from 'react';
import { useDashboardStore } from '@/store/dashboard';

interface DashboardStatsProps {
  className?: string;
}

export function DashboardStats({ className }: DashboardStatsProps) {
  const { 
    jobs, 
    keepers, 
    registry, 
    loading,
    getActiveJobs,
    getActiveKeepers,
    getTotalBalance,
    getSuccessRate
  } = useDashboardStore();

  if (loading) {
    return (
      <div className={`animate-pulse space-y-2 ${className}`}>
        <div className="h-4 bg-muted rounded w-1/2"></div>
        <div className="h-8 bg-muted rounded"></div>
      </div>
    );
  }

  const activeJobs = getActiveJobs().length;
  const activeKeepers = getActiveKeepers().length;
  const totalBalance = getTotalBalance();
  const successRate = getSuccessRate();

  const stats = [
    {
      label: 'Active Jobs',
      value: activeJobs,
      total: jobs.length,
      color: 'blue'
    },
    {
      label: 'Active Keepers', 
      value: activeKeepers,
      total: keepers.length,
      color: 'green'
    },
    {
      label: 'Success Rate',
      value: `${successRate.toFixed(1)}%`,
      color: successRate >= 95 ? 'green' : successRate >= 90 ? 'yellow' : 'red'
    },
    {
      label: 'Total Balance',
      value: `${(totalBalance / 1e9).toFixed(2)} SOL`,
      color: 'purple'
    }
  ];

  return (
    <div className={`grid grid-cols-2 lg:grid-cols-4 gap-4 ${className}`}>
      {stats.map((stat, index) => (
        <div
          key={index}
          className="border rounded-lg p-3 text-center hover:bg-muted/50 transition-colors"
        >
          <div className="text-xs text-muted-foreground mb-1">
            {stat.label}
          </div>
          <div className={`text-lg font-bold text-${stat.color}-600`}>
            {stat.value}
            {stat.total && (
              <span className="text-sm text-muted-foreground font-normal">
                /{stat.total}
              </span>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}