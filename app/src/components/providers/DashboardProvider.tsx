'use client';

import React, { createContext, useContext, useEffect } from 'react';
import { useDashboardStore } from '@/store/dashboard';
import { AutomationJob, Keeper, RegistryState } from '@/types';

// Mock data for demonstration
const mockJobs: AutomationJob[] = [
  {
    jobId: '1',
    owner: 'CWEStqD7RBaaSN2TCEm8FASjV6jGb4ss7H8AzwKW4exk',
    targetProgram: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
    targetInstruction: 'harvest_rewards',
    trigger: {
      type: 'time-based',
      interval: 3600, // 1 hour
    },
    gasLimit: 200000,
    balance: 0.05 * 1e9, // 0.05 SOL
    minBalance: 0.001 * 1e9,
    isActive: true,
    executionCount: 24,
    lastExecution: Date.now() / 1000 - 3480, // 58 minutes ago
    createdAt: Date.now() / 1000 - 86400, // 1 day ago
    updatedAt: Date.now() / 1000 - 3480,
  },
  {
    jobId: '2',
    owner: 'CWEStqD7RBaaSN2TCEm8FASjV6jGb4ss7H8AzwKW4exk',
    targetProgram: 'So11111111111111111111111111111111111111112',
    targetInstruction: 'compound_yield',
    trigger: {
      type: 'conditional',
      condition: 'yield > 5%',
    },
    gasLimit: 150000,
    balance: 0.08 * 1e9,
    minBalance: 0.001 * 1e9,
    isActive: true,
    executionCount: 12,
    lastExecution: Date.now() / 1000 - 7200, // 2 hours ago
    createdAt: Date.now() / 1000 - 172800, // 2 days ago
    updatedAt: Date.now() / 1000 - 7200,
  },
  {
    jobId: '3',
    owner: 'CWEStqD7RBaaSN2TCEm8FASjV6jGb4ss7H8AzwKW4exk',
    targetProgram: 'MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac',
    targetInstruction: 'rebalance_portfolio',
    trigger: {
      type: 'log-based',
      eventSignature: 'PriceUpdate(uint256)',
    },
    gasLimit: 300000,
    balance: 0.002 * 1e9, // Low balance
    minBalance: 0.001 * 1e9,
    isActive: false,
    executionCount: 8,
    lastExecution: Date.now() / 1000 - 14400, // 4 hours ago
    createdAt: Date.now() / 1000 - 259200, // 3 days ago
    updatedAt: Date.now() / 1000 - 14400,
  },
];

const mockKeepers: Keeper[] = [
  {
    address: 'KeEPerSaMPLe1111111111111111111111111111111',
    stakeAmount: 5 * 1e9, // 5 SOL
    reputationScore: 9250, // 92.5%
    isActive: true,
    totalExecutions: 156,
    successfulExecutions: 154,
    totalEarnings: 0.234 * 1e9,
    pendingRewards: 0.012 * 1e9,
    lastExecutionTime: Date.now() / 1000 - 120, // 2 minutes ago
    registeredAt: Date.now() / 1000 - 2592000, // 30 days ago
  },
  {
    address: 'KeEPerSaMPLe2222222222222222222222222222222',
    stakeAmount: 3 * 1e9, // 3 SOL
    reputationScore: 8750, // 87.5%
    isActive: true,
    totalExecutions: 98,
    successfulExecutions: 95,
    totalEarnings: 0.156 * 1e9,
    pendingRewards: 0.008 * 1e9,
    lastExecutionTime: Date.now() / 1000 - 300, // 5 minutes ago
    registeredAt: Date.now() / 1000 - 1728000, // 20 days ago
  },
];

const mockRegistry: RegistryState = {
  totalJobs: 3,
  activeJobs: 2,
  totalKeepers: 2,
  activeKeepers: 2,
  totalExecutions: 44,
  successfulExecutions: 42,
  protocolRevenue: 0.044 * 1e9,
  baseFee: 5000, // 5000 lamports
  protocolFeeBps: 500, // 5%
  minStake: 1 * 1e9, // 1 SOL
  nextJobId: 4,
};

interface DashboardContextType {
  initialized: boolean;
}

const DashboardContext = createContext<DashboardContextType>({ initialized: false });

interface DashboardProviderProps {
  children: React.ReactNode;
}

export function DashboardProvider({ children }: DashboardProviderProps) {
  const {
    setJobs,
    setKeepers,
    setRegistry,
    setLoading,
  } = useDashboardStore();

  useEffect(() => {
    // Initialize with mock data
    const initializeDashboard = async () => {
      setLoading(true);
      
      // Simulate API delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const { generateMockJobs, generateMockKeepers, generateMockRegistry, generateMockExecutions } = await import('@/lib/mockData');
      
      // Generate all mock data
      const jobs = generateMockJobs();
      const keepers = generateMockKeepers();
      const registry = generateMockRegistry();
      const executions = generateMockExecutions();
      
      setJobs(jobs);
      setKeepers(keepers);
      setRegistry(registry);
      useDashboardStore.getState().setRecentExecutions(executions);
      
      setLoading(false);
    };

    initializeDashboard();
  }, [setJobs, setKeepers, setRegistry, setLoading]);

  const value: DashboardContextType = {
    initialized: true,
  };

  return (
    <DashboardContext.Provider value={value}>
      {children}
    </DashboardContext.Provider>
  );
}

export function useDashboard(): DashboardContextType {
  const context = useContext(DashboardContext);
  if (!context) {
    throw new Error('useDashboard must be used within a DashboardProvider');
  }
  return context;
}