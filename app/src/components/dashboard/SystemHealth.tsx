'use client';

import React from 'react';
import { useDashboardStore } from '@/store/dashboard';
import { formatPercentage } from '@/lib/utils';

export function SystemHealth() {
  const { registry, loading, getSuccessRate } = useDashboardStore();
  
  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-8 bg-muted rounded"></div>
          </div>
        ))}
      </div>
    );
  }

  if (!registry) {
    return (
      <div className="text-center py-8">
        <div className="text-4xl mb-2">🔄</div>
        <h3 className="text-lg font-medium mb-1">Loading system status</h3>
        <p className="text-muted-foreground text-sm">
          Please wait while we check system health...
        </p>
      </div>
    );
  }

  const successRate = getSuccessRate();
  const keeperUtilization = registry.activeKeepers / registry.totalKeepers;
  const networkLoad = Math.min(registry.totalExecutions / 1000, 1); // Normalize to 1000 executions

  const getHealthStatus = (value: number, thresholds: { good: number, warning: number }) => {
    if (value >= thresholds.good) return { status: 'healthy', color: 'success' };
    if (value >= thresholds.warning) return { status: 'warning', color: 'warning' };
    return { status: 'critical', color: 'error' };
  };

  const successRateHealth = getHealthStatus(successRate, { good: 95, warning: 90 });
  const keeperHealth = getHealthStatus(keeperUtilization * 100, { good: 50, warning: 25 });
  const networkHealth = getHealthStatus((1 - networkLoad) * 100, { good: 70, warning: 50 });

  const healthItems = [
    {
      label: 'Execution Success Rate',
      value: formatPercentage(successRate / 100),
      health: successRateHealth,
      description: 'Job execution reliability'
    },
    {
      label: 'Keeper Network',
      value: `${registry.activeKeepers}/${registry.totalKeepers}`,
      health: keeperHealth,
      description: 'Active keeper coverage'
    },
    {
      label: 'Network Load',
      value: formatPercentage(networkLoad),
      health: networkHealth,
      description: 'System capacity utilization'
    }
  ];

  const overallHealth = Math.min(successRate, keeperUtilization * 100, (1 - networkLoad) * 100);
  const overallStatus = getHealthStatus(overallHealth, { good: 80, warning: 60 });

  return (
    <div className="space-y-4">
      {/* Overall Health Status */}
      <div className="border rounded-lg p-4 bg-gradient-to-r from-blue-50 to-purple-50">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-medium">Overall System Health</h3>
            <p className="text-sm text-muted-foreground">
              Real-time network status
            </p>
          </div>
          <div className="text-right">
            <div className={`text-2xl font-bold text-${overallStatus.color}-600`}>
              {formatPercentage(overallHealth / 100)}
            </div>
            <div className={`text-sm capitalize text-${overallStatus.color}-600`}>
              {overallStatus.status}
            </div>
          </div>
        </div>
        
        <div className="mt-3">
          <div className="w-full bg-muted rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all duration-300 bg-${overallStatus.color}-500`}
              style={{
                width: `${overallHealth}%`
              }}
            />
          </div>
        </div>
      </div>

      {/* Individual Health Metrics */}
      <div className="space-y-3">
        {healthItems.map((item, index) => (
          <div key={index} className="border rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <div>
                <div className="font-medium text-sm">{item.label}</div>
                <div className="text-xs text-muted-foreground">{item.description}</div>
              </div>
              <div className="text-right">
                <div className={`font-medium text-${item.health.color}-600`}>
                  {item.value}
                </div>
                <div className={`text-xs capitalize text-${item.health.color}-600`}>
                  {item.health.status}
                </div>
              </div>
            </div>
            
            <div className="w-full bg-muted rounded-full h-1">
              <div
                className={`h-1 rounded-full transition-all duration-300 bg-${item.health.color}-500`}
                style={{
                  width: `${item.health.status === 'healthy' ? 100 : 
                          item.health.status === 'warning' ? 70 : 40}%`
                }}
              />
            </div>
          </div>
        ))}
      </div>

      {/* Quick Actions */}
      <div className="border rounded-lg p-3 bg-muted/30">
        <h4 className="font-medium text-sm mb-2">Quick Actions</h4>
        <div className="flex flex-wrap gap-2">
          <button className="text-xs bg-primary text-primary-foreground px-2 py-1 rounded hover:bg-primary/90 transition-colors">
            Refresh Status
          </button>
          <button className="text-xs bg-secondary text-secondary-foreground px-2 py-1 rounded hover:bg-secondary/90 transition-colors">
            View Logs
          </button>
          <button className="text-xs bg-secondary text-secondary-foreground px-2 py-1 rounded hover:bg-secondary/90 transition-colors">
            System Config
          </button>
        </div>
      </div>
    </div>
  );
}