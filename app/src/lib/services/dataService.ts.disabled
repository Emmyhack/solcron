import { Connection } from '@solana/web3.js';
import { SolCronClient } from '../solcron';
import { useDashboardStore } from '../../store/dashboard';
import { AutomationJob, Keeper, RegistryState, SystemMetrics, JobExecution } from '@/types';
import { generateMockJobs, generateMockKeepers, generateMockRegistry, generateMockExecutions } from '../mockData';

export class DataService {
  private solcronClient: SolCronClient;
  private connection: Connection;

  constructor(connection: Connection, wallet?: any) {
    this.connection = connection;
    this.solcronClient = new SolCronClient(connection, wallet);
  }

  async loadDashboardData(): Promise<void> {
    const store = useDashboardStore.getState();
    
    try {
      store.setLoading(true);
      store.setError(null);

      // Check if wallet is connected
      if (!this.solcronClient.isConnected()) {
        console.log('Wallet not connected, using mock data');
        await this.loadMockData();
        return;
      }

      console.log('Loading real blockchain data...');
      
      // Load real data from blockchain
      const [jobs, keepers, executionHistory] = await Promise.allSettled([
        this.solcronClient.getJobs(),
        this.solcronClient.getKeepers(),
        this.solcronClient.getExecutionHistory()
      ]);

      // Process jobs
      if (jobs.status === 'fulfilled') {
        const processedJobs = await this.processJobsData(jobs.value);
        store.setJobs(processedJobs);
      } else {
        console.warn('Failed to load jobs:', jobs.reason);
        store.setJobs(mockJobs); // Fallback to mock data
      }

      // Process keepers
      if (keepers.status === 'fulfilled') {
        const processedKeepers = await this.processKeepersData(keepers.value);
        store.setKeepers(processedKeepers);
      } else {
        console.warn('Failed to load keepers:', keepers.reason);
        store.setKeepers(mockKeepers); // Fallback to mock data
      }

      // Process execution history
      if (executionHistory.status === 'fulfilled') {
        const processedExecutions = await this.processExecutionHistoryData(executionHistory.value);
        store.setRecentExecutions(processedExecutions);
      } else {
        console.warn('Failed to load execution history:', executionHistory.reason);
        store.setRecentExecutions(mockRecentExecutions); // Fallback to mock data
      }

      // Calculate and set derived data
      await this.calculateMetrics();
      await this.loadRegistryState();

    } catch (error) {
      console.error('Error loading dashboard data:', error);
      store.setError('Failed to load dashboard data');
      // Fallback to mock data on error
      await this.loadMockData();
    } finally {
      store.setLoading(false);
    }
  }

  private async loadMockData(): Promise<void> {
    const store = useDashboardStore.getState();
    
    // Simulate loading delay
    await new Promise(resolve => setTimeout(resolve, 500));
    
    store.setJobs(generateMockJobs());
    store.setKeepers(generateMockKeepers());
    store.setRegistry(generateMockRegistry());
    store.setRecentExecutions(generateMockExecutions());
  }

  private async processJobsData(rawJobs: any[]): Promise<AutomationJob[]> {
    // Convert blockchain job data to our AutomationJob type
    return rawJobs.map(job => ({
      jobId: job.id || `job_${Date.now()}`,
      owner: job.owner || '',
      targetProgram: job.targetProgram || '',
      targetInstruction: job.instruction || '',
      trigger: {
        type: 'time-based' as const,
        interval: 3600 // 1 hour default
      },
      gasLimit: job.gasLimit || 200000,
      balance: job.balance || 0,
      minBalance: job.minBalance || 1000000, // 0.001 SOL
      isActive: job.isActive !== undefined ? job.isActive : true,
      createdAt: job.createdAt ? Math.floor(job.createdAt.getTime() / 1000) : Math.floor(Date.now() / 1000),
      lastExecutionTime: job.lastExecution ? Math.floor(job.lastExecution.getTime() / 1000) : null,
      executionCount: job.executions || 0
    }));
  }

  private async processKeepersData(rawKeepers: any[]): Promise<Keeper[]> {
    // Convert blockchain keeper data to our Keeper type
    return rawKeepers.map(keeper => ({
      address: keeper.address || keeper.id,
      stake: keeper.stake || 0,
      isActive: keeper.isActive !== undefined ? keeper.isActive : true,
      reputation: keeper.reputation || 100,
      totalExecutions: keeper.totalExecutions || 0,
      successRate: keeper.successRate || 100,
      joinedAt: keeper.joinedAt ? new Date(keeper.joinedAt) : new Date(),
      balance: keeper.balance || 0,
      assignedJobs: keeper.assignedJobs || 0,
      earnings: keeper.earnings || 0,
      status: keeper.isActive ? 'active' : 'inactive'
    }));
  }

  private async processExecutionHistoryData(rawExecutions: any[]): Promise<JobExecution[]> {
    // Convert blockchain execution data to our JobExecution type
    return rawExecutions.map(execution => ({
      id: execution.id || `exec_${Date.now()}_${Math.random()}`,
      jobId: execution.jobId || '',
      keeperId: execution.keeperId || '',
      timestamp: execution.timestamp ? new Date(execution.timestamp) : new Date(),
      success: execution.success !== undefined ? execution.success : true,
      gasUsed: execution.gasUsed || 0,
      gasPrice: execution.gasPrice || 0,
      reward: execution.reward || 0,
      error: execution.error || null,
      transactionHash: execution.transactionHash || '',
      blockNumber: execution.blockNumber || 0
    }));
  }

  private async calculateMetrics(): Promise<void> {
    const store = useDashboardStore.getState();
    const { jobs, keepers, recentExecutions } = store;

    // Calculate system metrics based on loaded data
    const totalJobs = jobs.length;
    const activeJobs = jobs.filter(job => job.isActive).length;
    const totalKeepers = keepers.length;
    const activeKeepers = keepers.filter(keeper => keeper.isActive).length;
    const totalExecutions = recentExecutions.length;
    const successfulExecutions = recentExecutions.filter(exec => exec.success).length;

    const metrics: SystemMetrics = {
      totalJobs,
      activeJobs,
      totalKeepers,
      activeKeepers,
      totalExecutions,
      successfulExecutions,
      totalVolume: jobs.reduce((sum, job) => sum + job.balance, 0),
      averageExecutionTime: this.calculateAverageExecutionTime(recentExecutions),
      uptime: 99.9, // TODO: Calculate actual uptime
      networkFees: recentExecutions.reduce((sum, exec) => sum + (exec.gasUsed * exec.gasPrice), 0),
      period: '24h'
    };

    store.setMetrics(metrics);
  }

  private calculateAverageExecutionTime(executions: JobExecution[]): number {
    if (executions.length === 0) return 0;
    // For now, return a mock average (in ms)
    // TODO: Calculate based on actual execution times when available
    return 2500;
  }

  private async loadRegistryState(): Promise<void> {
    const store = useDashboardStore.getState();
    
    // TODO: Load actual registry state from blockchain
    // For now, calculate from loaded data
    const { jobs, keepers, recentExecutions } = store;
    
    const registryState: RegistryState = {
      totalJobs: jobs.length,
      activeJobs: jobs.filter(job => job.isActive).length,
      totalKeepers: keepers.length,
      activeKeepers: keepers.filter(keeper => keeper.isActive).length,
      totalExecutions: recentExecutions.length,
      successfulExecutions: recentExecutions.filter(exec => exec.success).length,
      totalVolume: jobs.reduce((sum, job) => sum + job.balance, 0),
      baseFee: 0.001, // SOL
      treasury: 0, // TODO: Get from blockchain
      lastUpdated: new Date()
    };

    store.setRegistry(registryState);
  }

  // Job management methods
  async createJob(jobData: Partial<AutomationJob>): Promise<string> {
    try {
      const jobId = await this.solcronClient.createJob(jobData);
      
      // Refresh jobs data after creation
      await this.loadJobs();
      
      return jobId;
    } catch (error) {
      console.error('Failed to create job:', error);
      throw error;
    }
  }

  async updateJob(jobId: string, updates: Partial<AutomationJob>): Promise<void> {
    try {
      await this.solcronClient.updateJob(jobId, updates);
      
      // Refresh jobs data after update
      await this.loadJobs();
    } catch (error) {
      console.error('Failed to update job:', error);
      throw error;
    }
  }

  async deleteJob(jobId: string): Promise<void> {
    try {
      await this.solcronClient.deleteJob(jobId);
      
      // Refresh jobs data after deletion
      await this.loadJobs();
    } catch (error) {
      console.error('Failed to delete job:', error);
      throw error;
    }
  }

  // Keeper management methods
  async registerKeeper(stake: number): Promise<void> {
    try {
      await this.solcronClient.registerKeeper(stake);
      
      // Refresh keepers data after registration
      await this.loadKeepers();
    } catch (error) {
      console.error('Failed to register keeper:', error);
      throw error;
    }
  }

  // Individual data loading methods
  private async loadJobs(): Promise<void> {
    try {
      const jobs = await this.solcronClient.getJobs();
      const processedJobs = await this.processJobsData(jobs);
      useDashboardStore.getState().setJobs(processedJobs);
    } catch (error) {
      console.error('Failed to load jobs:', error);
    }
  }

  private async loadKeepers(): Promise<void> {
    try {
      const keepers = await this.solcronClient.getKeepers();
      const processedKeepers = await this.processKeepersData(keepers);
      useDashboardStore.getState().setKeepers(processedKeepers);
    } catch (error) {
      console.error('Failed to load keepers:', error);
    }
  }

  // Utility methods
  getSolCronClient(): SolCronClient {
    return this.solcronClient;
  }

  updateWallet(wallet: any): void {
    this.solcronClient = new SolCronClient(this.connection, wallet);
  }
}