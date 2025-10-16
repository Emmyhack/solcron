/**
 * SolCron SDK - TypeScript client for Solana automation platform
 * 
 * This SDK provides a comprehensive interface for interacting with the SolCron
 * decentralized automation platform on Solana. It enables developers to:
 * 
 * - Register and manage automation jobs
 * - Monitor job execution and statistics  
 * - Participate as automation keepers
 * - Build automation-enabled dApps
 */

// Main client
export { SolCronClient } from './client';

// Core types
export type {
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
  TriggerType,
  HybridTriggerConfig,
  JobRegisteredEvent,
  JobExecutedEvent,
  JobFundedEvent,
  JobCancelledEvent,
  KeeperRegisteredEvent,
  KeeperSlashedEvent,
} from './types';

// Error classes
export {
  SolCronError,
  JobNotFoundError,
  KeeperNotFoundError,
  InsufficientBalanceError,
  InvalidTriggerError,
  PROGRAM_IDS,
} from './types';

// Utility functions
export {
  solToLamports,
  lamportsToSol,
  getRegistryStatePDA,
  getAutomationJobPDA,
  getKeeperPDA,
  getExecutionRecordPDA,
  serializeTriggerParams,
  deserializeTriggerParams,
  validateTriggerType,
  calculateJobFee,
  estimateJobCost,
  parseErrorFromLogs,
  sleep,
  retry,
  formatSOL,
  formatTimestamp,
  isValidPublicKey,
  getDefaultConfig,
  getProgramId,
  triggerTypeToString,
  LAMPORTS_PER_SOL,
  DEFAULT_COMMITMENT,
  DEFAULT_CLUSTER,
} from './utils';

// Instruction builders
export {
  createInitializeRegistryInstruction,
  createRegisterJobInstruction,
  createFundJobInstruction,
  createCancelJobInstruction,
  createUpdateJobInstruction,
  createRegisterKeeperInstruction,
  createClaimRewardsInstruction,
  createExecuteJobInstruction,
} from './instructions';

// Re-export common Solana types for convenience
export type {
  PublicKey,
  Connection,
  TransactionSignature,
  Commitment,
  ConfirmOptions,
} from '@solana/web3.js';

export { PublicKey, Connection, clusterApiUrl } from '@solana/web3.js';

/**
 * SDK Version
 */
export const VERSION = '0.1.0';

/**
 * Default configuration for different networks
 */
export const DEFAULT_CONFIGS = {
  'mainnet-beta': {
    registryProgramId: PROGRAM_IDS.registry['mainnet-beta'],
    executionProgramId: PROGRAM_IDS.execution['mainnet-beta'],
    cluster: 'mainnet-beta' as const,
    commitment: 'confirmed' as const,
  },
  devnet: {
    registryProgramId: PROGRAM_IDS.registry.devnet,
    executionProgramId: PROGRAM_IDS.execution.devnet,
    cluster: 'devnet' as const,
    commitment: 'confirmed' as const,
  },
  testnet: {
    registryProgramId: PROGRAM_IDS.registry.testnet,
    executionProgramId: PROGRAM_IDS.execution.testnet,
    cluster: 'testnet' as const,
    commitment: 'confirmed' as const,
  },
  localnet: {
    registryProgramId: PROGRAM_IDS.registry.localnet,
    executionProgramId: PROGRAM_IDS.execution.localnet,
    cluster: 'localnet' as const,
    commitment: 'confirmed' as const,
  },
} as const;

/**
 * Convenience function to create a SolCron client for a specific network
 */
export function createSolCronClient(
  connection: Connection,
  network: 'mainnet-beta' | 'devnet' | 'testnet' | 'localnet' = 'mainnet-beta'
): SolCronClient {
  return new SolCronClient(connection, DEFAULT_CONFIGS[network]);
}

/**
 * Convenience function to create a connection and client for a network
 */
export function createSolCronClientForNetwork(
  network: 'mainnet-beta' | 'devnet' | 'testnet' | 'localnet' = 'mainnet-beta',
  rpcUrl?: string
): SolCronClient {
  const { clusterApiUrl } = require('@solana/web3.js');
  
  const connection = new Connection(
    rpcUrl || clusterApiUrl(network === 'localnet' ? 'devnet' : network),
    'confirmed'
  );
  
  return createSolCronClient(connection, network);
}