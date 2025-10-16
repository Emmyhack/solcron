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
    <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4">
      <div className="flex items-center justify-between">
        {/* Left side - Status and Brand */}
        <div className="flex items-center space-x-6">
          <div className="flex items-center space-x-2">
            <div className="text-2xl font-bold bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
              SolCron
            </div>
            {network && (
              <span className="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                {network}
              </span>
            )}
          </div>
          
          {/* Status indicators */}
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-500' : 'bg-gray-400'}`} />
              <span className="text-sm text-gray-600 dark:text-gray-300">
                {connected ? 'Wallet Connected' : 'Wallet Disconnected'}
              </span>
            </div>
            
            {hasError && (
              <div className="flex items-center space-x-2">
                <div className="w-2 h-2 rounded-full bg-red-500" />
                <span className="text-sm text-red-600 dark:text-red-400">
                  {error || solcronError}
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
              className="px-3 py-1 text-sm bg-blue-100 text-blue-700 hover:bg-blue-200 dark:bg-blue-900 dark:text-blue-200 dark:hover:bg-blue-800 rounded transition-colors disabled:opacity-50"
              title="Refresh data"
            >
              {isLoading ? 'Refreshing...' : 'Refresh'}
            </button>
          )}

          {/* Balance display */}
          {connected && (
            <div className="text-right">
              <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                {formatSOL(balance)} SOL
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
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