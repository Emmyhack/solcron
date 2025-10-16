import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { AutomationJob, Keeper, RegistryState, SystemMetrics, JobExecution, DashboardState } from '@/types';

interface DashboardStore extends DashboardState {
  // Actions
  setJobs: (jobs: AutomationJob[]) => void;
  addJob: (job: AutomationJob) => void;
  updateJob: (jobId: string, updates: Partial<AutomationJob>) => void;
  removeJob: (jobId: string) => void;
  
  setKeepers: (keepers: Keeper[]) => void;
  addKeeper: (keeper: Keeper) => void;
  updateKeeper: (address: string, updates: Partial<Keeper>) => void;
  removeKeeper: (address: string) => void;
  
  setRegistry: (registry: RegistryState) => void;
  setMetrics: (metrics: SystemMetrics) => void;
  setRecentExecutions: (executions: JobExecution[]) => void;
  addExecution: (execution: JobExecution) => void;
  
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setSelectedJob: (job: AutomationJob | null) => void;
  setSelectedKeeper: (keeper: Keeper | null) => void;
  
  // Computed values
  getJobById: (jobId: string) => AutomationJob | undefined;
  getKeeperByAddress: (address: string) => Keeper | undefined;
  getActiveJobs: () => AutomationJob[];
  getActiveKeepers: () => Keeper[];
  getTotalBalance: () => number;
  getSuccessRate: () => number;
}

export const useDashboardStore = create<DashboardStore>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    jobs: [],
    keepers: [],
    registry: null,
    metrics: null,
    recentExecutions: [],
    loading: false,
    error: null,
    selectedJob: null,
    selectedKeeper: null,

    // Job actions
    setJobs: (jobs) => set({ jobs }),
    
    addJob: (job) => set((state) => ({
      jobs: [...state.jobs, job]
    })),
    
    updateJob: (jobId, updates) => set((state) => ({
      jobs: state.jobs.map(job => 
        job.jobId === jobId ? { ...job, ...updates } : job
      )
    })),
    
    removeJob: (jobId) => set((state) => ({
      jobs: state.jobs.filter(job => job.jobId !== jobId)
    })),

    // Keeper actions
    setKeepers: (keepers) => set({ keepers }),
    
    addKeeper: (keeper) => set((state) => ({
      keepers: [...state.keepers, keeper]
    })),
    
    updateKeeper: (address, updates) => set((state) => ({
      keepers: state.keepers.map(keeper => 
        keeper.address === address ? { ...keeper, ...updates } : keeper
      )
    })),
    
    removeKeeper: (address) => set((state) => ({
      keepers: state.keepers.filter(keeper => keeper.address !== address)
    })),

    // Registry and metrics actions
    setRegistry: (registry) => set({ registry }),
    setMetrics: (metrics) => set({ metrics }),
    
    setRecentExecutions: (executions) => set({ 
      recentExecutions: executions.slice(0, 100) // Keep only last 100
    }),
    
    addExecution: (execution) => set((state) => ({
      recentExecutions: [execution, ...state.recentExecutions].slice(0, 100)
    })),

    // UI state actions
    setLoading: (loading) => set({ loading }),
    setError: (error) => set({ error }),
    setSelectedJob: (selectedJob) => set({ selectedJob }),
    setSelectedKeeper: (selectedKeeper) => set({ selectedKeeper }),

    // Computed values
    getJobById: (jobId) => {
      const { jobs } = get();
      return jobs.find(job => job.jobId === jobId);
    },

    getKeeperByAddress: (address) => {
      const { keepers } = get();
      return keepers.find(keeper => keeper.address === address);
    },

    getActiveJobs: () => {
      const { jobs } = get();
      return jobs.filter(job => job.isActive);
    },

    getActiveKeepers: () => {
      const { keepers } = get();
      return keepers.filter(keeper => keeper.isActive);
    },

    getTotalBalance: () => {
      const { jobs } = get();
      return jobs.reduce((total, job) => total + job.balance, 0);
    },

    getSuccessRate: () => {
      const { registry } = get();
      if (!registry || registry.totalExecutions === 0) return 100;
      return (registry.successfulExecutions / registry.totalExecutions) * 100;
    }
  }))
);

// Selectors for better performance
export const useJobs = () => useDashboardStore((state) => state.jobs);
export const useKeepers = () => useDashboardStore((state) => state.keepers);
export const useRegistry = () => useDashboardStore((state) => state.registry);
export const useMetrics = () => useDashboardStore((state) => state.metrics);
export const useRecentExecutions = () => useDashboardStore((state) => state.recentExecutions);
export const useLoading = () => useDashboardStore((state) => state.loading);
export const useError = () => useDashboardStore((state) => state.error);
export const useSelectedJob = () => useDashboardStore((state) => state.selectedJob);
export const useSelectedKeeper = () => useDashboardStore((state) => state.selectedKeeper);

// Computed selectors
export const useActiveJobs = () => useDashboardStore((state) => state.getActiveJobs());
export const useActiveKeepers = () => useDashboardStore((state) => state.getActiveKeepers());
export const useTotalBalance = () => useDashboardStore((state) => state.getTotalBalance());
export const useSuccessRate = () => useDashboardStore((state) => state.getSuccessRate());