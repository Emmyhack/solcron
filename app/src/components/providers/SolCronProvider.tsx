'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { useWallet } from './WalletProvider';
import { SolCronClient, createSolCronClient, detectNetwork } from '../../lib/solcron';
import { useDashboardStore } from '@/store/dashboard';
import { AutomationJob, Keeper, RegistryState } from '@/types';

interface SolCronContextType {
  client: SolCronClient | null;
  network: 'mainnet-beta' | 'devnet' | 'testnet' | 'localnet' | null;
  loading: boolean;
  error: string | null;
  
  // Data refresh methods
  refreshAll: () => Promise<void>;
  refreshJobs: () => Promise<void>;
  refreshKeepers: () => Promise<void>;
  refreshRegistry: () => Promise<void>;
  
  // Transaction methods
  createJob: (params: CreateJobParams) => Promise<string>;
  registerKeeper: (stakeAmount: number) => Promise<string>;
}

interface CreateJobParams {
  targetProgram: string;
  targetInstruction: string;
  triggerType: string;
  triggerParams?: any;
  gasLimit: number;
  minBalance: number;
  initialFunding: number;
}

const SolCronContext = createContext<SolCronContextType | null>(null);

interface SolCronProviderProps {
  children: React.ReactNode;
}

export function SolCronProvider({ children }: SolCronProviderProps) {
  const { connection, publicKey, connected } = useWallet();
  const [client, setClient] = useState<SolCronClient | null>(null);
  const [network, setNetwork] = useState<'mainnet-beta' | 'devnet' | 'testnet' | 'localnet' | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const {
    setJobs,
    setKeepers,
    setRegistry,
    setLoading: setDashboardLoading,
    setError: setDashboardError
  } = useDashboardStore();

  // Initialize client when wallet connects
  useEffect(() => {
    if (connected && connection && publicKey) {
      initializeClient();
    } else {
      setClient(null);
      setNetwork(null);
    }
  }, [connected, connection, publicKey]);

  const initializeClient = async () => {
    try {
      setLoading(true);
      setError(null);

      // Detect network
      const detectedNetwork = await detectNetwork(connection);
      setNetwork(detectedNetwork);

      // Create wallet adapter for client
      const walletAdapter = {
        publicKey,
        signTransaction: async (tx: any) => tx,
        signAllTransactions: async (txs: any[]) => txs,
      };

      // Create SolCron client
      const solcronClient = createSolCronClient(connection, walletAdapter, detectedNetwork);
      setClient(solcronClient);

      // Initial data load
      await refreshAll(solcronClient);

    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to initialize SolCron client';
      setError(errorMsg);
      setDashboardError(errorMsg);
      console.error('Client initialization error:', err);
    } finally {
      setLoading(false);
    }
  };

  const refreshAll = async (clientInstance?: SolCronClient) => {
    const activeClient = clientInstance || client;
    if (!activeClient) return;

    try {
      setDashboardLoading(true);
      setDashboardError(null);

      // Fetch all data in parallel
      const [registryState, jobsData, keepersData] = await Promise.all([
        activeClient.getRegistryState().catch((err: unknown) => {
          console.warn('Failed to fetch registry state:', err);
          return null;
        }),
        activeClient.getAllJobs().catch((err: unknown) => {
          console.warn('Failed to fetch jobs:', err);
          return [];
        }),
        activeClient.getAllKeepers().catch((err: unknown) => {
          console.warn('Failed to fetch keepers:', err);
          return [];
        }),
      ]);

      // Transform and set registry state
      if (registryState) {
        const registry: RegistryState = {
          totalJobs: registryState.totalJobs || 0,
          activeJobs: registryState.activeJobs || 0,
          totalKeepers: registryState.totalKeepers || 0,
          activeKeepers: registryState.activeKeepers || 0,
          totalExecutions: registryState.totalExecutions || 0,
          successfulExecutions: registryState.successfulExecutions || 0,
          protocolRevenue: registryState.protocolRevenue || 0,
          baseFee: registryState.baseFee || 0,
          protocolFeeBps: registryState.protocolFeeBps || 0,
          minStake: registryState.minStake || 0,
          nextJobId: registryState.nextJobId || 0
        };
        setRegistry(registry);
      }

      // Transform and set jobs
      const jobs: AutomationJob[] = jobsData.map((jobData: any, index: number) => ({
        jobId: jobData.account?.jobId || `job_${index}`,
        owner: jobData.account?.owner?.toString() || '',
        targetProgram: jobData.account?.targetProgram?.toString() || '',
        targetInstruction: jobData.account?.targetInstruction || '',
        trigger: {
          type: jobData.account?.triggerType || 'time-based',
          interval: 300 // Default 5 minutes
        },
        gasLimit: jobData.account?.gasLimit || 200000,
        balance: jobData.account?.balance || 0,
        minBalance: jobData.account?.minBalance || 10000000,
        isActive: jobData.account?.isActive ?? true,
        executionCount: jobData.account?.executionCount || 0,
        lastExecution: jobData.account?.lastExecution || Date.now(),
        createdAt: jobData.account?.createdAt || Date.now(),
        updatedAt: jobData.account?.updatedAt || Date.now()
      }));
      setJobs(jobs);

      // Transform and set keepers
      const keepers: Keeper[] = keepersData.map((keeperData: any, index: number) => ({
        address: keeperData.publicKey?.toString() || `keeper_${index}`,
        stakeAmount: keeperData.account?.stakeAmount || 0,
        reputationScore: keeperData.account?.reputationScore || 0,
        isActive: keeperData.account?.isActive ?? true,
        totalExecutions: keeperData.account?.totalExecutions || 0,
        successfulExecutions: keeperData.account?.successfulExecutions || 0,
        totalEarnings: keeperData.account?.totalEarnings || 0,
        pendingRewards: keeperData.account?.pendingRewards || 0,
        lastExecutionTime: keeperData.account?.lastExecutionTime || Date.now(),
        registeredAt: keeperData.account?.registeredAt || Date.now()
      }));
      setKeepers(keepers);

    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to refresh data';
      setDashboardError(errorMsg);
      console.error('Data refresh error:', err);
    } finally {
      setDashboardLoading(false);
    }
  };

  const refreshJobs = useCallback(async () => {
    if (!client) return;
    
    try {
      const jobsData = await client.getAllJobs();
      const jobs: AutomationJob[] = jobsData.map((jobData: any, index: number) => ({
        jobId: jobData.account?.jobId || `job_${index}`,
        owner: jobData.account?.owner?.toString() || '',
        targetProgram: jobData.account?.targetProgram?.toString() || '',
        targetInstruction: jobData.account?.targetInstruction || '',
        trigger: {
          type: jobData.account?.triggerType || 'time-based',
          interval: 300
        },
        gasLimit: jobData.account?.gasLimit || 200000,
        balance: jobData.account?.balance || 0,
        minBalance: jobData.account?.minBalance || 10000000,
        isActive: jobData.account?.isActive ?? true,
        executionCount: jobData.account?.executionCount || 0,
        lastExecution: jobData.account?.lastExecution || Date.now(),
        createdAt: jobData.account?.createdAt || Date.now(),
        updatedAt: jobData.account?.updatedAt || Date.now()
      }));
      setJobs(jobs);
    } catch (err) {
      console.error('Failed to refresh jobs:', err);
    }
  }, [client, setJobs]);

  const refreshKeepers = useCallback(async () => {
    if (!client) return;
    
    try {
      const keepersData = await client.getAllKeepers();
      const keepers: Keeper[] = keepersData.map((keeperData: any, index: number) => ({
        address: keeperData.publicKey?.toString() || `keeper_${index}`,
        stakeAmount: keeperData.account?.stakeAmount || 0,
        reputationScore: keeperData.account?.reputationScore || 0,
        isActive: keeperData.account?.isActive ?? true,
        totalExecutions: keeperData.account?.totalExecutions || 0,
        successfulExecutions: keeperData.account?.successfulExecutions || 0,
        totalEarnings: keeperData.account?.totalEarnings || 0,
        pendingRewards: keeperData.account?.pendingRewards || 0,
        lastExecutionTime: keeperData.account?.lastExecutionTime || Date.now(),
        registeredAt: keeperData.account?.registeredAt || Date.now()
      }));
      setKeepers(keepers);
    } catch (err) {
      console.error('Failed to refresh keepers:', err);
    }
  }, [client, setKeepers]);

  const refreshRegistry = useCallback(async () => {
    if (!client) return;
    
    try {
      const registryState = await client.getRegistryState();
      if (registryState) {
        const registry: RegistryState = {
          totalJobs: registryState.totalJobs || 0,
          activeJobs: registryState.activeJobs || 0,
          totalKeepers: registryState.totalKeepers || 0,
          activeKeepers: registryState.activeKeepers || 0,
          totalExecutions: registryState.totalExecutions || 0,
          successfulExecutions: registryState.successfulExecutions || 0,
          protocolRevenue: registryState.protocolRevenue || 0,
          baseFee: registryState.baseFee || 0,
          protocolFeeBps: registryState.protocolFeeBps || 0,
          minStake: registryState.minStake || 0,
          nextJobId: registryState.nextJobId || 0
        };
        setRegistry(registry);
      }
    } catch (err) {
      console.error('Failed to refresh registry:', err);
    }
  }, [client, setRegistry]);

  const createJob = useCallback(async (params: CreateJobParams): Promise<string> => {
    if (!client) throw new Error('SolCron client not initialized');
    
    try {
      setDashboardLoading(true);
      
      // Convert target program string to PublicKey
      const targetProgramKey = new (await import('@solana/web3.js')).PublicKey(params.targetProgram);
      
      // Encode trigger parameters (simplified for demo)
      const triggerParams = new TextEncoder().encode(JSON.stringify(params.triggerParams || {}));
      
      const txSignature = await client.createJob(
        targetProgramKey,
        params.targetInstruction,
        params.triggerType,
        triggerParams,
        params.gasLimit,
        params.minBalance,
        params.initialFunding
      );
      
      // Refresh jobs after creation
      await refreshJobs();
      
      return txSignature;
    } finally {
      setDashboardLoading(false);
    }
  }, [client, refreshJobs, setDashboardLoading]);

  const registerKeeperMethod = useCallback(async (stakeAmount: number): Promise<string> => {
    if (!client) throw new Error('SolCron client not initialized');
    
    try {
      setDashboardLoading(true);
      
      const txSignature = await client.registerKeeper(stakeAmount);
      
      // Refresh keepers after registration
      await refreshKeepers();
      
      return txSignature;
    } finally {
      setDashboardLoading(false);
    }
  }, [client, refreshKeepers, setDashboardLoading]);

  const value: SolCronContextType = {
    client,
    network,
    loading,
    error,
    refreshAll: () => refreshAll(),
    refreshJobs,
    refreshKeepers,
    refreshRegistry,
    createJob,
    registerKeeper: registerKeeperMethod
  };

  return (
    <SolCronContext.Provider value={value}>
      {children}
    </SolCronContext.Provider>
  );
}

export function useSolCron(): SolCronContextType {
  const context = useContext(SolCronContext);
  if (!context) {
    throw new Error('useSolCron must be used within a SolCronProvider');
  }
  return context;
}