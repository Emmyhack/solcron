'use client';

import React, { useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { MetricsGrid } from './MetricsGrid';
import { JobsTable } from './JobsTable';
import { KeepersTable } from './KeepersTable';
import { RecentExecutions } from './RecentExecutions';
import { SystemHealth } from './SystemHealth';
import { useWallet } from '@/components/providers/WalletProvider';
import { useSolCron } from '@/components/providers/SolCronProvider';

export const DashboardOverview = React.memo(function DashboardOverview() {
  const { connected } = useWallet();
  const { refreshAll, loading, error } = useSolCron();

  // Refresh data on mount and when wallet connects
  useEffect(() => {
    if (connected) {
      refreshAll();
    }
  }, [connected, refreshAll]);

  // Show wallet connection prompt if not connected
  if (!connected) {
    return (
      <div className="min-h-[500px] flex items-center justify-center">
        <Card className="w-full max-w-lg bg-white dark:bg-slate-800 shadow-lg">
          <CardContent className="flex flex-col items-center space-y-6 p-8">
            <div className="w-16 h-16 bg-chainlink-blue/10 rounded-full flex items-center justify-center">
              <div className="w-8 h-8 bg-chainlink-blue rounded-full"></div>
            </div>
            <div className="text-center space-y-3">
              <h3 className="text-xl font-semibold text-slate-900 dark:text-slate-100">
                Connect Your Wallet
              </h3>
              <p className="text-slate-600 dark:text-slate-400 leading-relaxed">
                Connect your Solana wallet to access the SolCron automation platform and manage your jobs.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Show error state if there's an error
  if (error && !loading) {
    return (
      <div className="min-h-[500px] flex items-center justify-center">
        <Card className="w-full max-w-lg border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/10 shadow-lg">
          <CardContent className="flex flex-col items-center space-y-6 p-8">
            <div className="w-16 h-16 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center">
              <div className="w-8 h-8 bg-red-500 rounded-full"></div>
            </div>
            <div className="text-center space-y-3">
              <h3 className="text-xl font-semibold text-red-900 dark:text-red-100">
                Connection Error
              </h3>
              <p className="text-sm text-red-700 dark:text-red-300">
                {error}
              </p>
              <button 
                onClick={refreshAll}
                className="mt-4 px-6 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors duration-200 font-medium"
              >
                Try Again
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* System Health Status */}
      <SystemHealth />
      
      {/* Metrics Grid */}
      <MetricsGrid />
      
      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Active Jobs */}
        <Card className="bg-white dark:bg-slate-800 shadow-sm">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center justify-between text-lg">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 rounded-full bg-chainlink-blue"></div>
                <span className="font-semibold text-slate-900 dark:text-slate-100">Active Jobs</span>
              </div>
              {loading && (
                <div className="w-4 h-4 animate-spin rounded-full border-2 border-chainlink-blue border-t-transparent" />
              )}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <JobsTable limit={5} />
          </CardContent>
        </Card>
        
        {/* Active Keepers */}
        <Card className="bg-white dark:bg-slate-800 shadow-sm">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center justify-between text-lg">
              <div className="flex items-center space-x-3">
                <div className="w-2 h-2 rounded-full bg-green-500"></div>
                <span className="font-semibold text-slate-900 dark:text-slate-100">Active Keepers</span>
              </div>
              {loading && (
                <div className="w-4 h-4 animate-spin rounded-full border-2 border-chainlink-blue border-t-transparent" />
              )}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <KeepersTable limit={5} />
          </CardContent>
        </Card>
      </div>
      
      {/* Recent Activity */}
      <Card className="bg-white dark:bg-slate-800 shadow-sm">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center justify-between text-lg">
            <div className="flex items-center space-x-3">
              <div className="w-2 h-2 rounded-full bg-amber-500"></div>
              <span className="font-semibold text-slate-900 dark:text-slate-100">Recent Executions</span>
            </div>
            {loading && (
              <div className="w-4 h-4 animate-spin rounded-full border-2 border-chainlink-blue border-t-transparent" />
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <RecentExecutions />
        </CardContent>
      </Card>
    </div>
  );
});