'use client';

import React from 'react';
import { useDashboardStore } from '@/store/dashboard';
import { formatAddress, formatSOL, formatPercentage, calculateSuccessRate } from '@/lib/utils';
import { Keeper } from '@/types';

interface KeepersTableProps {
  limit?: number;
}

export function KeepersTable({ limit }: KeepersTableProps) {
  const { keepers, loading } = useDashboardStore();
  
  const displayKeepers = limit ? keepers.slice(0, limit) : keepers;

  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-16 bg-muted rounded"></div>
          </div>
        ))}
      </div>
    );
  }

  if (displayKeepers.length === 0) {
    return (
      <div className="text-center py-8">
        <div className="text-4xl mb-2">🛡️</div>
        <h3 className="text-lg font-medium mb-1">No keepers found</h3>
        <p className="text-muted-foreground text-sm">
          Keepers will appear here once they join the network.
        </p>
      </div>
    );
  }

  const getKeeperStatusBadge = (keeper: Keeper) => {
    if (!keeper.isActive) {
      return (
        <span className="job-status-badge job-status-inactive">
          Inactive
        </span>
      );
    }

    const recentlyActive = (Date.now() / 1000) - keeper.lastExecutionTime < 3600; // 1 hour
    
    return (
      <span className={`job-status-badge ${recentlyActive ? 'job-status-active' : 'job-status-warning'}`}>
        {recentlyActive ? 'Active' : 'Idle'}
      </span>
    );
  };

  const getReputationColor = (score: number) => {
    if (score >= 9000) return 'text-success-600';
    if (score >= 8000) return 'text-warning-600';
    return 'text-error-600';
  };

  return (
    <div className="space-y-3">
      {displayKeepers.map((keeper) => {
        const successRate = calculateSuccessRate(keeper.successfulExecutions, keeper.totalExecutions);
        
        return (
          <div
            key={keeper.address}
            className="border rounded-lg p-4 hover:bg-muted/50 transition-colors cursor-pointer"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2 mb-2">
                  <div className="w-8 h-8 bg-gradient-to-r from-blue-500 to-purple-500 rounded-full flex items-center justify-center">
                    <span className="text-white text-xs font-bold">
                      {keeper.address.slice(0, 2).toUpperCase()}
                    </span>
                  </div>
                  <div>
                    <div className="font-medium text-sm">
                      {formatAddress(keeper.address)}
                    </div>
                    <div className="flex items-center space-x-2">
                      {getKeeperStatusBadge(keeper)}
                    </div>
                  </div>
                </div>
                
                <div className="text-xs text-muted-foreground space-y-1 pl-10">
                  <div>Stake: {formatSOL(keeper.stakeAmount)} SOL</div>
                  <div>
                    Success Rate: {formatPercentage(successRate)} 
                    <span className="ml-1">
                      ({keeper.successfulExecutions}/{keeper.totalExecutions})
                    </span>
                  </div>
                </div>
              </div>
              
              <div className="text-right text-xs space-y-1">
                <div className={`font-medium ${getReputationColor(keeper.reputationScore)}`}>
                  {formatPercentage(keeper.reputationScore / 100)}
                </div>
                <div className="text-muted-foreground">reputation</div>
                <div className="font-medium text-success-600">
                  {formatSOL(keeper.totalEarnings)}
                </div>
                <div className="text-muted-foreground">earned</div>
              </div>
            </div>
            
            {/* Reputation bar */}
            <div className="mt-3 pl-10">
              <div className="w-full bg-muted rounded-full h-1">
                <div
                  className={`h-1 rounded-full transition-all duration-300 ${
                    keeper.reputationScore >= 9000 
                      ? 'bg-success-500' 
                      : keeper.reputationScore >= 8000 
                      ? 'bg-warning-500' 
                      : 'bg-error-500'
                  }`}
                  style={{
                    width: `${(keeper.reputationScore / 10000) * 100}%`
                  }}
                />
              </div>
            </div>
          </div>
        );
      })}
      
      {limit && keepers.length > limit && (
        <div className="text-center pt-2">
          <button className="text-sm text-primary hover:text-primary/80 transition-colors">
            View all {keepers.length} keepers →
          </button>
        </div>
      )}
    </div>
  );
}