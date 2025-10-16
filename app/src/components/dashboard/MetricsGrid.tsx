'use client';

import React from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { useDashboardStore } from '@/store/dashboard';
import { formatSOL, formatNumber, formatPercentage } from '@/lib/utils';

export const MetricsGrid = React.memo(function MetricsGrid() {
  const { jobs, keepers, registry } = useDashboardStore();

  const metrics = React.useMemo(() => {
    const activeJobs = jobs.filter(job => job.isActive);
    const activeKeepers = keepers.filter(keeper => keeper.isActive);
    const totalBalance = jobs.reduce((sum, job) => sum + job.balance, 0);
    const successRate = registry ? 
      registry.totalExecutions > 0 ? (registry.successfulExecutions / registry.totalExecutions) * 100 : 100 
      : 100;

    return {
      totalJobs: jobs.length,
      activeJobs: activeJobs.length,
      totalKeepers: keepers.length,
      activeKeepers: activeKeepers.length,
      totalBalance,
      successRate,
      totalExecutions: registry?.totalExecutions || 0,
      protocolRevenue: registry?.protocolRevenue || 0,
    };
  }, [jobs, keepers, registry]);

  const metricCards = [
    {
      title: 'Active Jobs',
      value: metrics.activeJobs.toString(),
      subtitle: `of ${metrics.totalJobs} total`,
      color: 'text-blue-600',
      bgColor: 'bg-blue-100',
      trend: metrics.activeJobs > 0 ? '+12%' : '0%',
    },
    {
      title: 'Active Keepers',
      value: metrics.activeKeepers.toString(),
      subtitle: `of ${metrics.totalKeepers} total`,
      color: 'text-success-600',
      bgColor: 'bg-green-100',
      trend: '+8%',
    },
    {
      title: 'Total Balance',
      value: `${formatSOL(metrics.totalBalance)}`,
      subtitle: 'SOL in jobs',
      color: 'text-solana-600',
      bgColor: 'bg-purple-100',
      trend: '+15%',
    },
    {
      title: 'Success Rate',
      value: formatPercentage(metrics.successRate),
      subtitle: `${metrics.totalExecutions} executions`,
      color: 'text-success-600',
      bgColor: 'bg-green-100',
      trend: '+2%',
    },
    {
      title: 'Total Executions',
      value: formatNumber(metrics.totalExecutions, 0),
      subtitle: 'all time',
      color: 'text-purple-600',
      bgColor: 'bg-purple-100',
      trend: '+24%',
    },
    {
      title: 'Protocol Revenue',
      value: `${formatSOL(metrics.protocolRevenue)}`,
      subtitle: 'SOL earned',
      color: 'text-amber-600',
      bgColor: 'bg-amber-100',
      trend: '+18%',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
      {metricCards.map((metric, index) => (
        <Card key={index} className="metric-card group">
          <CardContent className="p-5">
            <div className="flex items-center justify-between mb-3">
              <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 uppercase tracking-wider">
                {metric.title}
              </div>
              <span className="status-badge status-badge-success text-xs">
                {metric.trend}
              </span>
            </div>
            
            <div className="space-y-2">
              <div className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight">
                {metric.value}
              </div>
              <div className="text-sm text-slate-500 dark:text-slate-400">
                {metric.subtitle}
              </div>
            </div>
            
            {/* Progress indicator */}
            <div className="mt-3">
              <div className="w-full bg-slate-200 dark:bg-slate-700 rounded-full h-1">
                <div 
                  className="h-1 rounded-full bg-chainlink-blue transition-all duration-300"
                  style={{ width: `${Math.min(100, (index + 1) * 15)}%` }}
                ></div>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
});