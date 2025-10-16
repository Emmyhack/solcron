import { PublicKey, SystemProgram } from '@solana/web3.js';
import BN from 'bn.js';
import { TriggerType, PROGRAM_IDS, SolCronConfig } from './types';

/**
 * Constants
 */
export const LAMPORTS_PER_SOL = 1_000_000_000;
export const DEFAULT_COMMITMENT = 'confirmed';
export const DEFAULT_CLUSTER = 'mainnet-beta';

/**
 * Get program ID for the specified program and cluster
 */
export function getProgramId(
  program: 'registry' | 'execution',
  cluster: string = DEFAULT_CLUSTER
): PublicKey {
  const programIds = PROGRAM_IDS[program] as Record<string, PublicKey>;
  return programIds[cluster] || programIds['mainnet-beta'];
}

/**
 * Convert SOL amount to lamports
 */
export function solToLamports(sol: number): number {
  return Math.floor(sol * LAMPORTS_PER_SOL);
}

/**
 * Convert lamports to SOL
 */
export function lamportsToSol(lamports: number | BN): number {
  const amount = typeof lamports === 'number' ? lamports : lamports.toNumber();
  return amount / LAMPORTS_PER_SOL;
}

/**
 * Derive PDA for registry state
 */
export function getRegistryStatePDA(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('registry')],
    programId
  );
}

/**
 * Derive PDA for automation job
 */
export function getAutomationJobPDA(
  jobId: number | BN,
  programId: PublicKey
): [PublicKey, number] {
  const jobIdBuffer = typeof jobId === 'number' 
    ? new BN(jobId).toArrayLike(Buffer, 'le', 8)
    : jobId.toArrayLike(Buffer, 'le', 8);
    
  return PublicKey.findProgramAddressSync(
    [Buffer.from('job'), jobIdBuffer],
    programId
  );
}

/**
 * Derive PDA for keeper account
 */
export function getKeeperPDA(
  keeperAddress: PublicKey,
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('keeper'), keeperAddress.toBuffer()],
    programId
  );
}

/**
 * Derive PDA for execution record
 */
export function getExecutionRecordPDA(
  jobId: number | BN,
  executionCount: number | BN,
  programId: PublicKey
): [PublicKey, number] {
  const jobIdBuffer = typeof jobId === 'number' 
    ? new BN(jobId).toArrayLike(Buffer, 'le', 8)
    : jobId.toArrayLike(Buffer, 'le', 8);
    
  const executionCountBuffer = typeof executionCount === 'number'
    ? new BN(executionCount).toArrayLike(Buffer, 'le', 8)
    : executionCount.toArrayLike(Buffer, 'le', 8);
    
  return PublicKey.findProgramAddressSync(
    [Buffer.from('execution'), jobIdBuffer, executionCountBuffer],
    programId
  );
}

/**
 * Serialize trigger parameters based on trigger type
 */
export function serializeTriggerParams(triggerType: TriggerType): Uint8Array {
  switch (triggerType.type) {
    case 'time':
      const timeParams = {
        interval: triggerType.interval
      };
      return new TextEncoder().encode(JSON.stringify(timeParams));
      
    case 'conditional':
      const conditionalParams = {
        condition: triggerType.condition
      };
      return new TextEncoder().encode(JSON.stringify(conditionalParams));
      
    case 'log':
      const logParams = {
        event_signature: triggerType.eventSignature
      };
      return new TextEncoder().encode(JSON.stringify(logParams));
      
    case 'hybrid':
      const hybridParams = {
        time_interval: triggerType.config.timeInterval,
        condition: triggerType.config.condition,
        event_signature: triggerType.config.eventSignature,
        require_all: triggerType.config.requireAll
      };
      return new TextEncoder().encode(JSON.stringify(hybridParams));
      
    default:
      throw new Error(`Unknown trigger type: ${(triggerType as any).type}`);
  }
}

/**
 * Deserialize trigger parameters from bytes
 */
export function deserializeTriggerParams(
  triggerType: string,
  params: Uint8Array
): TriggerType {
  const decoded = JSON.parse(new TextDecoder().decode(params));
  
  switch (triggerType.toLowerCase()) {
    case 'time':
      return {
        type: 'time',
        interval: decoded.interval
      };
      
    case 'conditional':
      return {
        type: 'conditional',
        condition: decoded.condition
      };
      
    case 'log':
      return {
        type: 'log',
        eventSignature: decoded.event_signature
      };
      
    case 'hybrid':
      return {
        type: 'hybrid',
        config: {
          timeInterval: decoded.time_interval,
          condition: decoded.condition,
          eventSignature: decoded.event_signature,
          requireAll: decoded.require_all
        }
      };
      
    default:
      throw new Error(`Unknown trigger type: ${triggerType}`);
  }
}

/**
 * Validate trigger type parameters
 */
export function validateTriggerType(triggerType: TriggerType): void {
  switch (triggerType.type) {
    case 'time':
      if (triggerType.interval <= 0) {
        throw new Error('Time interval must be positive');
      }
      if (triggerType.interval < 60) {
        throw new Error('Time interval must be at least 60 seconds');
      }
      if (triggerType.interval > 86400 * 30) {
        throw new Error('Time interval cannot exceed 30 days');
      }
      break;
      
    case 'conditional':
      if (!triggerType.condition || triggerType.condition.trim() === '') {
        throw new Error('Condition cannot be empty');
      }
      if (triggerType.condition.length > 500) {
        throw new Error('Condition too long (max 500 characters)');
      }
      break;
      
    case 'log':
      if (!triggerType.eventSignature || triggerType.eventSignature.trim() === '') {
        throw new Error('Event signature cannot be empty');
      }
      break;
      
    case 'hybrid':
      if (!triggerType.config.timeInterval && 
          !triggerType.config.condition && 
          !triggerType.config.eventSignature) {
        throw new Error('Hybrid trigger must have at least one condition');
      }
      
      if (triggerType.config.timeInterval && triggerType.config.timeInterval <= 0) {
        throw new Error('Time interval must be positive');
      }
      break;
      
    default:
      throw new Error(`Invalid trigger type: ${(triggerType as any).type}`);
  }
}

/**
 * Calculate job execution fee
 */
export function calculateJobFee(
  baseFee: number,
  gasLimit: number,
  priorityMultiplier: number = 1.0
): number {
  // Simple fee calculation - in practice this would be more sophisticated
  const computeFee = Math.ceil(gasLimit / 100000) * 1000; // 1000 lamports per 100k compute units
  return Math.ceil((baseFee + computeFee) * priorityMultiplier);
}

/**
 * Estimate job runtime cost
 */
export function estimateJobCost(
  params: {
    baseFee: number;
    gasLimit: number;
    executionsPerDay: number;
    days: number;
    priorityMultiplier?: number;
  }
): {
  feePerExecution: number;
  dailyCost: number;
  totalCost: number;
  totalCostSOL: number;
} {
  const feePerExecution = calculateJobFee(
    params.baseFee,
    params.gasLimit,
    params.priorityMultiplier
  );
  
  const dailyCost = feePerExecution * params.executionsPerDay;
  const totalCost = dailyCost * params.days;
  
  return {
    feePerExecution,
    dailyCost,
    totalCost,
    totalCostSOL: lamportsToSol(totalCost)
  };
}

/**
 * Parse error from transaction logs
 */
export function parseErrorFromLogs(logs: string[]): string | null {
  for (const log of logs) {
    if (log.includes('Error:') || log.includes('Instruction failed')) {
      return log;
    }
    
    // Look for program error codes
    const errorMatch = log.match(/Error Code: (\d+)/);
    if (errorMatch) {
      return `Program error code: ${errorMatch[1]}`;
    }
  }
  
  return null;
}

/**
 * Sleep utility for delays
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Retry utility for async operations
 */
export async function retry<T>(
  operation: () => Promise<T>,
  maxAttempts: number = 3,
  delayMs: number = 1000
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;
      
      if (attempt === maxAttempts) {
        throw lastError;
      }
      
      await sleep(delayMs * attempt); // Exponential backoff
    }
  }
  
  throw lastError!;
}

/**
 * Format display amount with proper decimals
 */
export function formatSOL(
  amount: number | BN,
  decimals: number = 4
): string {
  const sol = typeof amount === 'number' ? amount : lamportsToSol(amount);
  return `${sol.toFixed(decimals)} SOL`;
}

/**
 * Format timestamp for display
 */
export function formatTimestamp(timestamp: number | BN): string {
  const ts = typeof timestamp === 'number' ? timestamp : timestamp.toNumber();
  return new Date(ts * 1000).toLocaleString();
}

/**
 * Check if a PublicKey is a valid Solana address
 */
export function isValidPublicKey(address: string): boolean {
  try {
    new PublicKey(address);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get default configuration
 */
export function getDefaultConfig(cluster?: string): SolCronConfig {
  const clusterName = cluster || DEFAULT_CLUSTER;
  
  return {
    registryProgramId: getProgramId('registry', clusterName),
    executionProgramId: getProgramId('execution', clusterName),
    cluster: clusterName as any,
    commitment: DEFAULT_COMMITMENT as any,
  };
}

/**
 * Convert enum-like trigger type to string for serialization
 */
export function triggerTypeToString(triggerType: TriggerType): string {
  switch (triggerType.type) {
    case 'time':
      return 'TimeBased';
    case 'conditional':
      return 'Conditional';
    case 'log':
      return 'LogTrigger';
    case 'hybrid':
      return 'Hybrid';
    default:
      throw new Error(`Unknown trigger type: ${(triggerType as any).type}`);
  }
}