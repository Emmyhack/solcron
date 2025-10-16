import { AutomationJob, Keeper, JobExecution, RegistryState } from '@/types';
import { generateMockAddress } from '@/lib/utils';

// Mock job data
export function generateMockJobs(): AutomationJob[] {
  const currentTime = Date.now() / 1000;
  
  return [
    {
      jobId: 'job_001',
      owner: generateMockAddress(),
      targetProgram: generateMockAddress(),
      targetInstruction: 'swap_exact_tokens_for_tokens',
      trigger: {
        type: 'time-based',
        interval: 604800
      },
      gasLimit: 200000,
      balance: 50000000000, // 50 SOL
      minBalance: 1000000000, // 1 SOL
      isActive: true,
      executionCount: 12,
      lastExecution: currentTime - 86400,
      createdAt: currentTime - 86400 * 7,
      updatedAt: currentTime - 86400
    },
    {
      jobId: 'job_002',
      owner: generateMockAddress(),
      targetProgram: generateMockAddress(),
      targetInstruction: 'liquidate_position',
      trigger: {
        type: 'conditional',
        condition: 'price_below',
        parameters: { 
          priceSource: 'pyth',
          targetPrice: 45000,
          currentPrice: 43500
        }
      },
      gasLimit: 250000,
      balance: 25000000000, // 25 SOL
      minBalance: 5000000000, // 5 SOL
      isActive: true,
      executionCount: 3,
      lastExecution: currentTime - 3600,
      createdAt: currentTime - 86400 * 14,
      updatedAt: currentTime - 3600
    },
    {
      jobId: 'job_003',
      owner: generateMockAddress(),
      targetProgram: generateMockAddress(),
      targetInstruction: 'harvest_rewards',
      trigger: {
        type: 'conditional',
        condition: 'balance_above',
        parameters: {
          tokenAddress: generateMockAddress(),
          minBalance: 1000000000
        }
      },
      gasLimit: 150000,
      balance: 15000000000, // 15 SOL
      minBalance: 1000000000, // 1 SOL
      isActive: true,
      executionCount: 8,
      lastExecution: currentTime - 7200,
      createdAt: currentTime - 86400 * 3,
      updatedAt: currentTime - 7200
    },
    {
      jobId: 'job_004',
      owner: generateMockAddress(),
      targetProgram: generateMockAddress(),
      targetInstruction: 'market_sell',
      trigger: {
        type: 'conditional',
        condition: 'price_below',
        parameters: { 
          priceSource: 'pyth',
          targetPrice: 40000,
          currentPrice: 43500
        }
      },
      gasLimit: 180000,
      balance: 5000000000, // 5 SOL
      minBalance: 500000000, // 0.5 SOL
      isActive: false,
      executionCount: 0,
      lastExecution: 0,
      createdAt: currentTime - 86400 * 21,
      updatedAt: currentTime - 86400 * 21
    }
  ];
}

// Mock keeper data
export function generateMockKeepers(): Keeper[] {
  const currentTime = Date.now() / 1000;
  
  return [
    {
      address: generateMockAddress(),
      stakeAmount: 100000000000, // 100 SOL
      reputationScore: 9850,
      isActive: true,
      totalExecutions: 245,
      successfulExecutions: 241,
      totalEarnings: 1250000000, // 1.25 SOL
      pendingRewards: 50000000, // 0.05 SOL
      lastExecutionTime: currentTime - 300,
      registeredAt: currentTime - 86400 * 30
    },
    {
      address: generateMockAddress(),
      stakeAmount: 75000000000, // 75 SOL
      reputationScore: 9200,
      isActive: true,
      totalExecutions: 189,
      successfulExecutions: 185,
      totalEarnings: 945000000, // 0.945 SOL
      pendingRewards: 32000000, // 0.032 SOL
      lastExecutionTime: currentTime - 1800,
      registeredAt: currentTime - 86400 * 45
    },
    {
      address: generateMockAddress(),
      stakeAmount: 50000000000, // 50 SOL
      reputationScore: 7800,
      isActive: false,
      totalExecutions: 67,
      successfulExecutions: 62,
      totalEarnings: 335000000, // 0.335 SOL
      pendingRewards: 15000000, // 0.015 SOL
      lastExecutionTime: currentTime - 86400,
      registeredAt: currentTime - 86400 * 15
    }
  ];
}

// Mock execution history
export function generateMockExecutions(): JobExecution[] {
  const currentTime = Date.now() / 1000;
  const jobs = ['DCA BTC Purchase', 'Liquidation Monitor', 'Yield Farm Harvest', 'Stop Loss Order'];
  const types = ['recurring', 'price_trigger', 'balance_trigger', 'scheduled'];
  const keepers = [
    generateMockAddress(),
    generateMockAddress(),
    generateMockAddress()
  ];
  
  return Array.from({ length: 20 }, (_, i) => {
    const timestamp = currentTime - (i * 3600); // Each execution 1 hour apart
    const success = Math.random() > 0.05; // 95% success rate
    const job = jobs[Math.floor(Math.random() * jobs.length)];
    const keeper = keepers[Math.floor(Math.random() * keepers.length)];
    
    return {
      id: `exec_${String(i + 1).padStart(3, '0')}`,
      jobId: `job_${String(Math.floor(Math.random() * 4) + 1).padStart(3, '0')}`,
      jobName: job,
      jobType: types[Math.floor(Math.random() * types.length)],
      executionId: `exec_${Date.now()}_${i}`,
      keeper: keeper,
      keeperAddress: keeper,
      timestamp: timestamp,
      gasUsed: Math.floor(Math.random() * 50000) + 20000,
      feeCharged: Math.floor(Math.random() * 5000000) + 1000000,
      rewardPaid: Math.floor(Math.random() * 8000000) + 2000000,
      success: success,
      errorMessage: success ? undefined : 'Insufficient balance for execution',
      transactionSignature: generateMockAddress(),
      transactionHash: generateMockAddress()
    };
  });
}

// Mock registry state
export function generateMockRegistry(): RegistryState {
  const jobs = generateMockJobs();
  const keepers = generateMockKeepers();
  const executions = generateMockExecutions();
  
  const totalExecutions = executions.length;
  const successfulExecutions = executions.filter(e => e.success).length;
  
  return {
    totalJobs: jobs.length,
    activeJobs: jobs.filter(j => j.isActive).length,
    totalKeepers: keepers.length,
    activeKeepers: keepers.filter(k => k.isActive).length,
    totalExecutions: totalExecutions,
    successfulExecutions: successfulExecutions,
    protocolRevenue: executions.reduce((sum, exec) => sum + exec.feeCharged, 0),
    baseFee: 1000000, // 0.001 SOL
    protocolFeeBps: 250, // 2.5%
    minStake: 10000000000, // 10 SOL
    nextJobId: jobs.length + 1
  };
}