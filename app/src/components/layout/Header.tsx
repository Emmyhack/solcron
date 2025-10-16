'use client';

import React from 'react';
import { useWallet, WalletMultiButton } from '@/components/providers/WalletProvider';
import { useSolCron } from '@/components/providers/SolCronProvider';
import { formatSOL, formatAddress } from '@/lib/utils';
import { useDashboardStore } from '@/store/dashboard';

export function Header() {
  const { connected, publicKey, balance } = useWallet();
  const { network, loading: solcronLoading, error: solcronError, refreshAll } = useSolCron();
  const { loading, error } = useDashboardStore();

  const isLoading = loading || solcronLoading;
  const hasError = error || solcronError;

  return (
    <header className="bg-white dark:bg-slate-900 border-b border-gray-200 dark:border-slate-700 px-6 py-3">
      <div className="flex items-center justify-between">
        {/* Left side - Status and Brand */}
        <div className="flex items-center space-x-6">
          <div className="flex items-center space-x-3">
            <div className="text-xl font-semibold text-chainlink-blue dark:text-white">
              SolCron
            </div>
            {network && (
              <span className="px-2.5 py-0.5 text-xs font-medium rounded-full bg-chainlink-blue/10 text-chainlink-blue border border-chainlink-blue/20">
                {network}
              </span>
            )}
          </div>
          
          {/* Status indicators */}
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <div className={`status-dot ${connected ? 'status-dot-active' : 'status-dot-inactive'}`} />
              <span className="text-sm text-slate-600 dark:text-slate-300 font-medium">
                {connected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            
            {hasError && (
              <div className="flex items-center space-x-2">
                <div className="status-dot status-dot-error" />
                <span className="text-sm text-red-600 dark:text-red-400 font-medium">
                  Error
                </span>
              </div>
            )}
          </div>
        </div>

        {/* Right side - Controls and Wallet */}
        <div className="flex items-center space-x-4">
          {/* Refresh button */}
          {connected && (
            <button
              onClick={refreshAll}
              disabled={isLoading}
              className="px-4 py-2 text-sm font-medium bg-slate-100 text-slate-700 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 rounded-md transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
              title="Refresh data"
            >
              {isLoading ? 'Refreshing...' : 'Refresh'}
            </button>
          )}

          {/* Balance display */}
          {connected && (
            <div className="text-right bg-slate-50 dark:bg-slate-800 px-3 py-2 rounded-md">
              <div className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                {formatSOL(balance)} SOL
              </div>
              <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                {formatAddress(publicKey?.toBase58() || '')}
              </div>
            </div>
          )}
          
          {/* Wallet connection button */}
          <div className="wallet-adapter-button-trigger">
            <WalletMultiButton />
          </div>
        </div>
      </div>
    </header>
  );
}