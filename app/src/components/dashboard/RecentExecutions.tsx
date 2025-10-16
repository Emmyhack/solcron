'use client';

import React from 'react';
import { useDashboardStore } from '@/store/dashboard';
import { formatAddress, formatSOL, formatTimeAgo } from '@/lib/utils';

interface RecentExecutionsProps {
  limit?: number;
}

export function RecentExecutions({ limit = 10 }: RecentExecutionsProps) {
  const { recentExecutions, loading } = useDashboardStore();
  
  const displayExecutions = limit ? recentExecutions.slice(0, limit) : recentExecutions;

  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-12 bg-muted rounded"></div>
          </div>
        ))}
      </div>
    );
  }

  if (displayExecutions.length === 0) {
    return (
      <div className="text-center py-8">
        <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
          <div className="w-8 h-8 bg-gray-300 rounded-full"></div>
        </div>
        <h3 className="text-lg font-medium mb-1">No recent executions</h3>
        <p className="text-muted-foreground text-sm">
          Execution history will appear here as jobs run.
        </p>
      </div>
    );
  }

  const getStatusIcon = (success: boolean) => {
    return success ? (
      <div className="w-2 h-2 bg-success-500 rounded-full"></div>
    ) : (
      <div className="w-2 h-2 bg-error-500 rounded-full"></div>
    );
  };

  const getExecutionTypeColor = (type: string) => {
    switch (type) {
      case 'scheduled':
        return 'bg-blue-500';
      case 'recurring':
        return 'bg-green-500';
      case 'price_trigger':
        return 'bg-yellow-500';
      case 'balance_trigger':
        return 'bg-purple-500';
      case 'custom':
        return 'bg-gray-500';
      default:
        return 'bg-blue-500';
    }
  };

  return (
    <div className="space-y-2">
      {displayExecutions.map((execution) => (
        <div
          key={execution.id}
          className="border rounded-lg p-3 hover:bg-muted/50 transition-colors"
        >
          <div className="flex items-center space-x-3">
            {getStatusIcon(execution.success)}
            
            <div className="flex-1 min-w-0">
              <div className="flex items-center space-x-2 mb-1">
                <div className={`w-3 h-3 rounded-full ${getExecutionTypeColor(execution.jobType)}`}></div>
                <div className="font-medium text-sm truncate">
                  {execution.jobName}
                </div>
              </div>
              
              <div className="text-xs text-muted-foreground">
                Executed by {formatAddress(execution.keeperAddress)}
              </div>
            </div>
            
            <div className="text-right text-xs">
              <div className="font-medium text-success-600">
                +{formatSOL(execution.rewardPaid)}
              </div>
              <div className="text-muted-foreground">
                {formatTimeAgo(execution.timestamp)}
              </div>
            </div>
          </div>
          
          {execution.gasUsed && (
            <div className="mt-2 pt-2 border-t text-xs text-muted-foreground flex justify-between">
              <span>Gas: {execution.gasUsed.toLocaleString()}</span>
              <span>Tx: {formatAddress(execution.transactionHash)}</span>
            </div>
          )}
          
          {!execution.success && execution.errorMessage && (
            <div className="mt-2 pt-2 border-t">
              <div className="text-xs text-error-600 bg-error-50 p-2 rounded">
                Error: {execution.errorMessage}
              </div>
            </div>
          )}
        </div>
      ))}
      
      {limit && recentExecutions.length > limit && (
        <div className="text-center pt-2">
          <button className="text-sm text-primary hover:text-primary/80 transition-colors">
            View all executions →
          </button>
        </div>
      )}
    </div>
  );
}