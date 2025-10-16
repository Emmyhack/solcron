'use client';

import React from 'react';
import { useDashboardStore } from '@/store/dashboard';
import { formatAddress, formatSOL, formatTimeAgo } from '@/lib/utils';

interface RecentExecutionsProps {
  limit?: number;
}

export const RecentExecutions = React.memo(function RecentExecutions({ limit = 10 }: RecentExecutionsProps) {
  const { recentExecutions, loading } = useDashboardStore();
  
  const displayExecutions = limit ? recentExecutions.slice(0, limit) : recentExecutions;

  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-12 bg-slate-200 dark:bg-slate-700 rounded-lg"></div>
          </div>
        ))}
      </div>
    );
  }

  if (displayExecutions.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="w-16 h-16 mx-auto mb-4 bg-slate-100 dark:bg-slate-800 rounded-full flex items-center justify-center">
          <div className="w-8 h-8 bg-slate-300 dark:bg-slate-600 rounded-full"></div>
        </div>
        <h3 className="text-lg font-semibold mb-1 text-slate-900 dark:text-slate-100">No Recent Executions</h3>
        <p className="text-slate-500 dark:text-slate-400 text-sm">
          Execution history will appear here as automation jobs run.
        </p>
      </div>
    );
  }

  const getStatusIcon = React.useCallback((success: boolean) => {
    return success ? (
      <div className="status-dot status-dot-active"></div>
    ) : (
      <div className="status-dot status-dot-error"></div>
    );
  }, []);

  const getExecutionTypeColor = React.useCallback((type: string) => {
    switch (type) {
      case 'scheduled':
        return 'bg-chainlink-blue';
      case 'recurring':
        return 'bg-green-500';
      case 'price_trigger':
        return 'bg-amber-500';
      case 'balance_trigger':
        return 'bg-purple-500';
      case 'custom':
        return 'bg-slate-500';
      default:
        return 'bg-chainlink-blue';
    }
  }, []);

  return (
    <div className="space-y-2">
      {displayExecutions.map((execution) => (
        <div
          key={execution.id}
          className="bg-white dark:bg-slate-800 border border-gray-200 dark:border-slate-700 rounded-lg p-4 hover:bg-slate-50 dark:hover:bg-slate-700/50 transition-colors duration-200"
        >
          <div className="flex items-center space-x-3">
            {getStatusIcon(execution.success)}
            
            <div className="flex-1 min-w-0">
              <div className="flex items-center space-x-2 mb-2">
                <div className={`w-2 h-2 rounded-full ${getExecutionTypeColor(execution.jobType)}`}></div>
                <div className="font-semibold text-sm text-slate-900 dark:text-slate-100 truncate">
                  {execution.jobName}
                </div>
              </div>
              
              <div className="text-xs text-slate-500 dark:text-slate-400">
                Executed by <span className="font-mono">{formatAddress(execution.keeperAddress)}</span>
              </div>
            </div>
            
            <div className="text-right text-xs">
              <div className="font-semibold text-green-600 dark:text-green-400">
                +{formatSOL(execution.rewardPaid)} SOL
              </div>
              <div className="text-slate-500 dark:text-slate-400 mt-1">
                {formatTimeAgo(execution.timestamp)}
              </div>
            </div>
          </div>
          
          {execution.gasUsed && (
            <div className="mt-3 pt-3 border-t border-gray-200 dark:border-slate-600 text-xs text-slate-500 dark:text-slate-400 flex justify-between">
              <span>Gas Used: <span className="font-mono">{execution.gasUsed.toLocaleString()}</span></span>
              <span>Tx: <span className="font-mono">{formatAddress(execution.transactionHash)}</span></span>
            </div>
          )}
          
          {!execution.success && execution.errorMessage && (
            <div className="mt-3 pt-3 border-t border-gray-200 dark:border-slate-600">
              <div className="text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-2 rounded">
                <span className="font-medium">Error:</span> {execution.errorMessage}
              </div>
            </div>
          )}
        </div>
      ))}
      
      {limit && recentExecutions.length > limit && (
        <div className="text-center pt-4">
          <button className="text-sm text-chainlink-blue hover:text-chainlink-blue/80 font-medium transition-colors duration-200 hover:underline">
            View All Executions →
          </button>
        </div>
      )}
    </div>
  );
});