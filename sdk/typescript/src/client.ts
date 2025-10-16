import {
  Connection,
  PublicKey,
  Transaction,
  TransactionSignature,
  Commitment,
  ConfirmOptions,
  AccountInfo,
} from '@solana/web3.js';
import BN from 'bn.js';
import {
  AutomationJob,
  Keeper,
  RegistryState,
  ExecutionRecord,
  RegisterJobParams,
  CreateJobResult,
  TransactionResult,
  SolCronConfig,
  JobStats,
  KeeperStats,
  NetworkStats,
  SolCronError,
  JobNotFoundError,
  KeeperNotFoundError,
  PROGRAM_IDS,
} from './types';
import {
  getRegistryStatePDA,
  getAutomationJobPDA,
  getKeeperPDA,
  getExecutionRecordPDA,
  getProgramId,
  lamportsToSol,
  formatSOL,
  getDefaultConfig,
  retry,
  validateTriggerType,
} from './utils';
import {
  createRegisterJobInstruction,
  createFundJobInstruction,
  createCancelJobInstruction,
  createUpdateJobInstruction,
  createRegisterKeeperInstruction,
  createClaimRewardsInstruction,
  createExecuteJobInstruction,
} from './instructions';

/**
 * Main SolCron SDK client for interacting with the automation platform
 */
export class SolCronClient {
  private connection: Connection;
  private config: Required<SolCronConfig>;

  constructor(connection: Connection, config?: SolCronConfig) {
    this.connection = connection;
    this.config = {
      ...getDefaultConfig(),
      ...config,
    } as Required<SolCronConfig>;
  }

  /**
   * Get the connection instance
   */
  getConnection(): Connection {
    return this.connection;
  }

  /**
   * Get the current configuration
   */
  getConfig(): Required<SolCronConfig> {
    return this.config;
  }

  /**
   * Register a new automation job
   */
  async registerJob(
    params: RegisterJobParams,
    payer: PublicKey,
    confirmOptions?: ConfirmOptions
  ): Promise<CreateJobResult> {
    // Validate parameters
    validateTriggerType(params.triggerType);
    
    if (params.gasLimit <= 0 || params.gasLimit > 1_400_000) {
      throw new SolCronError('Gas limit must be between 1 and 1,400,000');
    }
    
    if (params.initialFunding < 0.001) {
      throw new SolCronError('Initial funding must be at least 0.001 SOL');
    }

    // Get next job ID from registry state
    const registryState = await this.getRegistryState();
    const jobId = registryState.nextJobId.toNumber();

    // Create instruction
    const instruction = createRegisterJobInstruction(
      payer,
      params,
      this.config.registryProgramId
    );

    // Create and send transaction
    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [payer], confirmOptions);

    // Derive job account address
    const [jobAccount] = getAutomationJobPDA(jobId, this.config.registryProgramId);

    return {
      jobId,
      signature,
      jobAccount,
    };
  }

  /**
   * Fund an existing job with additional SOL
   */
  async fundJob(
    jobId: number,
    amount: number,
    payer: PublicKey,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionResult> {
    if (amount <= 0) {
      throw new SolCronError('Funding amount must be positive');
    }

    const instruction = createFundJobInstruction(
      payer,
      jobId,
      amount,
      this.config.registryProgramId
    );

    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [payer], confirmOptions);

    return {
      signature,
      success: true,
    };
  }

  /**
   * Cancel a job and withdraw remaining funds
   */
  async cancelJob(
    jobId: number,
    owner: PublicKey,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionResult> {
    const instruction = createCancelJobInstruction(
      owner,
      jobId,
      this.config.registryProgramId
    );

    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [owner], confirmOptions);

    return {
      signature,
      success: true,
    };
  }

  /**
   * Update job parameters
   */
  async updateJob(
    jobId: number,
    updates: {
      gasLimit?: number;
      minBalance?: number;
      triggerParams?: Uint8Array;
    },
    owner: PublicKey,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionResult> {
    const instruction = createUpdateJobInstruction(
      owner,
      jobId,
      updates,
      this.config.registryProgramId
    );

    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [owner], confirmOptions);

    return {
      signature,
      success: true,
    };
  }

  /**
   * Register as a keeper
   */
  async registerKeeper(
    stakeAmount: number,
    keeper: PublicKey,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionResult> {
    const registryState = await this.getRegistryState();
    const minStakeSOL = lamportsToSol(registryState.minStake.toNumber());
    
    if (stakeAmount < minStakeSOL) {
      throw new SolCronError(`Stake amount must be at least ${formatSOL(minStakeSOL)}`);
    }

    const instruction = createRegisterKeeperInstruction(
      keeper,
      stakeAmount,
      this.config.registryProgramId
    );

    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [keeper], confirmOptions);

    return {
      signature,
      success: true,
    };
  }

  /**
   * Claim keeper rewards
   */
  async claimRewards(
    keeper: PublicKey,
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionResult> {
    const instruction = createClaimRewardsInstruction(
      keeper,
      this.config.registryProgramId
    );

    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [keeper], confirmOptions);

    return {
      signature,
      success: true,
    };
  }

  /**
   * Execute a job (typically called by keepers)
   */
  async executeJob(
    jobId: number,
    keeper: PublicKey,
    remainingAccounts: PublicKey[] = [],
    confirmOptions?: ConfirmOptions
  ): Promise<TransactionResult> {
    // Get job details to find target program
    const job = await this.getJob(jobId);
    if (!job) {
      throw new JobNotFoundError(jobId);
    }

    // Get current execution count for PDA derivation
    const executionCount = job.executionCount.toNumber();

    const instruction = createExecuteJobInstruction(
      keeper,
      jobId,
      job.targetProgram,
      executionCount,
      remainingAccounts,
      this.config.registryProgramId
    );

    const transaction = new Transaction().add(instruction);
    const signature = await this.sendTransaction(transaction, [keeper], confirmOptions);

    return {
      signature,
      success: true,
    };
  }

  /**
   * Get job details by ID
   */
  async getJob(jobId: number): Promise<AutomationJob | null> {
    const [jobAccount] = getAutomationJobPDA(jobId, this.config.registryProgramId);
    
    try {
      const accountInfo = await this.connection.getAccountInfo(
        jobAccount,
        this.config.commitment
      );
      
      if (!accountInfo) {
        return null;
      }
      
      return this.deserializeAutomationJob(accountInfo.data);
    } catch (error) {
      console.warn(`Failed to fetch job ${jobId}:`, error);
      return null;
    }
  }

  /**
   * Get all jobs owned by a specific address
   */
  async getAllJobs(owner?: PublicKey): Promise<AutomationJob[]> {
    // In a real implementation, this would use getProgramAccounts with filters
    // For now, we'll implement a basic version that requires knowing job IDs
    const jobs: AutomationJob[] = [];
    
    // This is a simplified implementation - in practice you'd either:
    // 1. Use getProgramAccounts with owner filter
    // 2. Maintain an off-chain index
    // 3. Use event logs to discover jobs
    
    console.warn('getAllJobs: This method requires an off-chain index for efficiency');
    return jobs;
  }

  /**
   * Get keeper information
   */
  async getKeeper(address: PublicKey): Promise<Keeper | null> {
    const [keeperAccount] = getKeeperPDA(address, this.config.registryProgramId);
    
    try {
      const accountInfo = await this.connection.getAccountInfo(
        keeperAccount,
        this.config.commitment
      );
      
      if (!accountInfo) {
        return null;
      }
      
      return this.deserializeKeeper(accountInfo.data);
    } catch (error) {
      console.warn(`Failed to fetch keeper ${address.toString()}:`, error);
      return null;
    }
  }

  /**
   * Get registry state information
   */
  async getRegistryState(): Promise<RegistryState> {
    const [registryAccount] = getRegistryStatePDA(this.config.registryProgramId);
    
    const accountInfo = await this.connection.getAccountInfo(
      registryAccount,
      this.config.commitment
    );
    
    if (!accountInfo) {
      throw new SolCronError('Registry not initialized');
    }
    
    return this.deserializeRegistryState(accountInfo.data);
  }

  /**
   * Get execution history for a job
   */
  async getExecutionHistory(
    jobId: number,
    limit: number = 10
  ): Promise<ExecutionRecord[]> {
    // This would require either:
    // 1. Querying multiple execution record PDAs
    // 2. Using an off-chain indexer
    // 3. Parsing transaction logs
    
    const records: ExecutionRecord[] = [];
    
    // Try to fetch recent execution records
    for (let i = 0; i < limit; i++) {
      try {
        const [executionAccount] = getExecutionRecordPDA(
          jobId,
          i,
          this.config.registryProgramId
        );
        
        const accountInfo = await this.connection.getAccountInfo(executionAccount);
        if (accountInfo) {
          const record = this.deserializeExecutionRecord(accountInfo.data);
          records.push(record);
        }
      } catch (error) {
        // Execution record doesn't exist, continue
        continue;
      }
    }
    
    return records.reverse(); // Most recent first
  }

  /**
   * Get job statistics
   */
  async getJobStats(jobId: number): Promise<JobStats> {
    const job = await this.getJob(jobId);
    if (!job) {
      throw new JobNotFoundError(jobId);
    }

    const executions = await this.getExecutionHistory(jobId, 100);
    const successfulExecutions = executions.filter(e => e.success).length;
    const failedExecutions = executions.filter(e => !e.success).length;
    
    const totalGasUsed = executions.reduce((sum, e) => sum + e.gasUsed.toNumber(), 0);
    const totalFeesSpent = executions.reduce((sum, e) => sum + e.feePaid.toNumber(), 0);

    return {
      totalExecutions: job.executionCount.toNumber(),
      successfulExecutions,
      failedExecutions,
      successRate: job.executionCount.toNumber() > 0 
        ? successfulExecutions / job.executionCount.toNumber() 
        : 0,
      avgGasUsed: executions.length > 0 ? totalGasUsed / executions.length : 0,
      totalFeesSpent,
      lastExecution: job.lastExecution.toNumber() > 0 
        ? new Date(job.lastExecution.toNumber() * 1000) 
        : undefined,
    };
  }

  /**
   * Get keeper statistics
   */
  async getKeeperStats(address: PublicKey): Promise<KeeperStats> {
    const keeper = await this.getKeeper(address);
    if (!keeper) {
      throw new KeeperNotFoundError(address.toString());
    }

    const totalExecutions = keeper.successfulExecutions.add(keeper.failedExecutions).toNumber();
    const successRate = totalExecutions > 0 
      ? keeper.successfulExecutions.toNumber() / totalExecutions 
      : 0;

    return {
      address: address.toString(),
      totalExecutions,
      successfulExecutions: keeper.successfulExecutions.toNumber(),
      failedExecutions: keeper.failedExecutions.toNumber(),
      successRate,
      totalEarnings: lamportsToSol(keeper.totalEarnings.toNumber()),
      reputationScore: keeper.reputationScore.toNumber(),
      isActive: keeper.isActive,
      lastActive: keeper.lastActive.toNumber() > 0 
        ? new Date(keeper.lastActive.toNumber() * 1000) 
        : undefined,
    };
  }

  /**
   * Get network-wide statistics
   */
  async getNetworkStats(): Promise<NetworkStats> {
    const registry = await this.getRegistryState();

    return {
      totalJobs: registry.totalJobs.toNumber(),
      activeJobs: registry.activeJobs.toNumber(),
      totalKeepers: registry.totalKeepers.toNumber(),
      activeKeepers: registry.activeKeepers.toNumber(),
      totalExecutions: registry.totalExecutions.toNumber(),
      avgExecutionFee: lamportsToSol(registry.baseFee.toNumber()),
      totalValueLocked: 0, // Would need to sum all job balances
    };
  }

  /**
   * Send and confirm transaction with retry logic
   */
  private async sendTransaction(
    transaction: Transaction,
    signers: PublicKey[],
    options?: ConfirmOptions
  ): Promise<TransactionSignature> {
    const commitment = options?.commitment || this.config.commitment;
    
    return retry(async () => {
      const { blockhash } = await this.connection.getLatestBlockhash(commitment);
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = signers[0];

      // In a real implementation, you'd need actual Keypair signers
      // This is just showing the structure
      const signature = await this.connection.sendRawTransaction(
        transaction.serialize(),
        {
          skipPreflight: false,
          preflightCommitment: commitment,
        }
      );

      await this.connection.confirmTransaction(signature, commitment);
      return signature;
    }, 3, 1000);
  }

  /**
   * Deserialize automation job account data
   */
  private deserializeAutomationJob(data: Buffer): AutomationJob {
    // This is a simplified deserialization
    // In a real implementation, you'd use the proper Anchor/Borsh deserialization
    
    let offset = 8; // Skip discriminator
    
    const jobId = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    
    const owner = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    
    // ... continue deserializing all fields
    
    // Placeholder return - implement proper deserialization
    return {
      jobId,
      owner,
      targetProgram: PublicKey.default,
      targetInstruction: '',
      triggerType: { type: 'time', interval: 3600 },
      triggerParams: new Uint8Array(),
      balance: new BN(0),
      gasLimit: new BN(0),
      minBalance: new BN(0),
      isActive: true,
      createdAt: new BN(0),
      lastExecution: new BN(0),
      executionCount: new BN(0),
      failedCount: new BN(0),
      bump: 0,
    };
  }

  /**
   * Deserialize keeper account data
   */
  private deserializeKeeper(data: Buffer): Keeper {
    // Placeholder implementation
    return {
      address: PublicKey.default,
      stakeAmount: new BN(0),
      reputationScore: new BN(0),
      successfulExecutions: new BN(0),
      failedExecutions: new BN(0),
      totalEarnings: new BN(0),
      pendingRewards: new BN(0),
      isActive: true,
      registeredAt: new BN(0),
      lastActive: new BN(0),
      bump: 0,
    };
  }

  /**
   * Deserialize registry state account data
   */
  private deserializeRegistryState(data: Buffer): RegistryState {
    // Placeholder implementation
    return {
      admin: PublicKey.default,
      totalJobs: new BN(0),
      activeJobs: new BN(0),
      totalKeepers: new BN(0),
      activeKeepers: new BN(0),
      totalExecutions: new BN(0),
      baseFee: new BN(5000),
      minStake: new BN(1000000000),
      protocolFeeBps: 250,
      treasury: PublicKey.default,
      nextJobId: new BN(1),
      bump: 0,
    };
  }

  /**
   * Deserialize execution record account data
   */
  private deserializeExecutionRecord(data: Buffer): ExecutionRecord {
    // Placeholder implementation
    return {
      jobId: new BN(0),
      keeper: PublicKey.default,
      timestamp: new BN(0),
      success: true,
      gasUsed: new BN(0),
      feePaid: new BN(0),
      bump: 0,
    };
  }
}