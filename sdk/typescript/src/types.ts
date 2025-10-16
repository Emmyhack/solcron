import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

/**
 * Trigger type configuration for automation jobs
 */
export type TriggerType = 
  | { type: 'time'; interval: number }
  | { type: 'conditional'; condition: string }
  | { type: 'log'; eventSignature: string }
  | { type: 'hybrid'; config: HybridTriggerConfig };

export interface HybridTriggerConfig {
  timeInterval?: number;
  condition?: string;
  eventSignature?: string;
  requireAll?: boolean; // true = AND, false = OR
}

/**
 * Parameters for registering a new automation job
 */
export interface RegisterJobParams {
  targetProgram: PublicKey;
  targetInstruction: string;
  triggerType: TriggerType;
  gasLimit: number;
  initialFunding: number; // in SOL
  minBalance?: number; // in SOL, defaults to 0.001 SOL
}

/**
 * Automation job account data
 */
export interface AutomationJob {
  jobId: BN;
  owner: PublicKey;
  targetProgram: PublicKey;
  targetInstruction: string;
  triggerType: TriggerType;
  triggerParams: Uint8Array;
  balance: BN;
  gasLimit: BN;
  minBalance: BN;
  isActive: boolean;
  createdAt: BN;
  lastExecution: BN;
  executionCount: BN;
  failedCount: BN;
  bump: number;
}

/**
 * Keeper account data
 */
export interface Keeper {
  address: PublicKey;
  stakeAmount: BN;
  reputationScore: BN;
  successfulExecutions: BN;
  failedExecutions: BN;
  totalEarnings: BN;
  pendingRewards: BN;
  isActive: boolean;
  registeredAt: BN;
  lastActive: BN;
  bump: number;
}

/**
 * Registry state account data
 */
export interface RegistryState {
  admin: PublicKey;
  totalJobs: BN;
  activeJobs: BN;
  totalKeepers: BN;
  activeKeepers: BN;
  totalExecutions: BN;
  baseFee: BN;
  minStake: BN;
  protocolFeeBps: number;
  treasury: PublicKey;
  nextJobId: BN;
  bump: number;
}

/**
 * Execution record data
 */
export interface ExecutionRecord {
  jobId: BN;
  keeper: PublicKey;
  timestamp: BN;
  success: boolean;
  gasUsed: BN;
  feePaid: BN;
  errorCode?: number;
  bump: number;
}

/**
 * Job execution statistics
 */
export interface JobStats {
  totalExecutions: number;
  successfulExecutions: number;
  failedExecutions: number;
  successRate: number;
  avgGasUsed: number;
  totalFeesSpent: number;
  lastExecution?: Date;
  nextEstimatedExecution?: Date;
}

/**
 * Keeper performance statistics
 */
export interface KeeperStats {
  address: string;
  totalExecutions: number;
  successfulExecutions: number;
  failedExecutions: number;
  successRate: number;
  totalEarnings: number;
  reputationScore: number;
  isActive: boolean;
  lastActive?: Date;
}

/**
 * Network-wide statistics
 */
export interface NetworkStats {
  totalJobs: number;
  activeJobs: number;
  totalKeepers: number;
  activeKeepers: number;
  totalExecutions: number;
  avgExecutionFee: number;
  totalValueLocked: number; // Total SOL locked in jobs
}

/**
 * Job creation result
 */
export interface CreateJobResult {
  jobId: number;
  signature: string;
  jobAccount: PublicKey;
}

/**
 * Transaction result
 */
export interface TransactionResult {
  signature: string;
  success: boolean;
  error?: string;
}

/**
 * SDK configuration options
 */
export interface SolCronConfig {
  registryProgramId?: PublicKey;
  executionProgramId?: PublicKey;
  cluster?: 'mainnet-beta' | 'testnet' | 'devnet' | 'localnet';
  commitment?: 'processed' | 'confirmed' | 'finalized';
}

/**
 * Event types emitted by the program
 */
export interface JobRegisteredEvent {
  jobId: BN;
  owner: PublicKey;
  targetProgram: PublicKey;
  initialFunding: BN;
}

export interface JobExecutedEvent {
  jobId: BN;
  keeper: PublicKey;
  success: boolean;
  feePaid: BN;
  gasUsed: BN;
}

export interface JobFundedEvent {
  jobId: BN;
  amount: BN;
  newBalance: BN;
}

export interface JobCancelledEvent {
  jobId: BN;
  owner: PublicKey;
  refundedAmount: BN;
}

export interface KeeperRegisteredEvent {
  address: PublicKey;
  stakeAmount: BN;
}

export interface KeeperSlashedEvent {
  keeper: PublicKey;
  slashAmount: BN;
  reason: string;
  newStake: BN;
  newReputation: BN;
}

/**
 * Program IDs for different networks
 */
export const PROGRAM_IDS = {
  registry: {
    'mainnet-beta': new PublicKey('Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'),
    'devnet': new PublicKey('Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'),
    'testnet': new PublicKey('Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'),
    'localnet': new PublicKey('Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS'),
  },
  execution: {
    'mainnet-beta': new PublicKey('ExecNqpXiPPjs7m5wbuTCxZE8PJzgdW2cWEw23kcKJKm'),
    'devnet': new PublicKey('ExecNqpXiPPjs7m5wbuTCxZE8PJzgdW2cWEw23kcKJKm'),
    'testnet': new PublicKey('ExecNqpXiPPjs7m5wbuTCxZE8PJzgdW2cWEw23kcKJKm'),
    'localnet': new PublicKey('ExecNqpXiPPjs7m5wbuTCxZE8PJzgdW2cWEw23kcKJKm'),
  }
} as const;

/**
 * Error types
 */
export class SolCronError extends Error {
  constructor(message: string, public code?: number, public logs?: string[]) {
    super(message);
    this.name = 'SolCronError';
  }
}

export class JobNotFoundError extends SolCronError {
  constructor(jobId: number) {
    super(`Job with ID ${jobId} not found`);
    this.name = 'JobNotFoundError';
  }
}

export class KeeperNotFoundError extends SolCronError {
  constructor(address: string) {
    super(`Keeper with address ${address} not found`);
    this.name = 'KeeperNotFoundError';
  }
}

export class InsufficientBalanceError extends SolCronError {
  constructor(required: number, available: number) {
    super(`Insufficient balance: required ${required}, available ${available}`);
    this.name = 'InsufficientBalanceError';
  }
}

export class InvalidTriggerError extends SolCronError {
  constructor(reason: string) {
    super(`Invalid trigger configuration: ${reason}`);
    this.name = 'InvalidTriggerError';
  }
}