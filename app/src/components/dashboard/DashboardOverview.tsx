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

export function DashboardOverview() {
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
      <div className="min-h-[400px] flex items-center justify-center">
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center space-y-4 pt-6">
            <div className="w-12 h-12 bg-gray-200 rounded-lg flex items-center justify-center">
              <div className="w-6 h-6 bg-gray-400 rounded"></div>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Connect Your Wallet
              </h3>
              <p className="text-sm text-gray-600 dark:text-gray-400 mt-2">
                Connect your Solana wallet to access the SolCron dashboard and manage your automation jobs.
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
      <div className="min-h-[400px] flex items-center justify-center">
        <Card className="w-full max-w-md border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20">
          <CardContent className="flex flex-col items-center space-y-4 pt-6">
            <div className="w-12 h-12 bg-red-200 rounded-lg flex items-center justify-center">
              <div className="w-6 h-6 bg-red-500 rounded-full"></div>
            </div>
            <div className="text-center">
              <h3 className="text-lg font-semibold text-red-900 dark:text-red-100">
                Connection Error
              </h3>
              <p className="text-sm text-red-700 dark:text-red-300 mt-2">
                {error}
              </p>
              <button 
                onClick={refreshAll}
                className="mt-4 px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors"
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
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
                <span>Active Jobs</span>
              </div>
              {loading && <div className="w-4 h-4 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <JobsTable limit={5} />
          </CardContent>
        </Card>
        
        {/* Active Keepers */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                <span>Active Keepers</span>
              </div>
              {loading && <div className="w-4 h-4 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <KeepersTable limit={5} />
          </CardContent>
        </Card>
      </div>
      
      {/* Recent Activity */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 bg-purple-500 rounded-full"></div>
              <span>Recent Executions</span>
            </div>
            {loading && <div className="w-4 h-4 animate-spin rounded-full border-2 border-blue-600 border-t-transparent" />}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <RecentExecutions />
        </CardContent>
      </Card>
    </div>
  );
}