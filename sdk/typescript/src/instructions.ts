import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
} from '@solana/web3.js';
import BN from 'bn.js';
import * as borsh from 'borsh';
import {
  TriggerType,
  RegisterJobParams,
  PROGRAM_IDS,
} from './types';
import {
  getRegistryStatePDA,
  getAutomationJobPDA,
  getKeeperPDA,
  getExecutionRecordPDA,
  serializeTriggerParams,
  solToLamports,
  triggerTypeToString,
} from './utils';

/**
 * Instruction discriminators (8-byte hashes of instruction names)
 * These would normally be generated from the IDL
 */
const INSTRUCTION_DISCRIMINATORS = {
  initializeRegistry: Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]),
  registerJob: Buffer.from([42, 45, 246, 145, 189, 50, 16, 47]),
  fundJob: Buffer.from([165, 87, 23, 45, 178, 91, 145, 203]),
  cancelJob: Buffer.from([239, 107, 189, 48, 178, 109, 45, 67]),
  updateJob: Buffer.from([78, 156, 203, 45, 189, 67, 203, 189]),
  registerKeeper: Buffer.from([189, 67, 203, 45, 78, 156, 203, 45]),
  unregisterKeeper: Buffer.from([203, 45, 78, 156, 203, 45, 189, 67]),
  executeJob: Buffer.from([45, 78, 156, 203, 45, 189, 67, 203]),
  claimRewards: Buffer.from([156, 203, 45, 189, 67, 203, 45, 78]),
  slashKeeper: Buffer.from([67, 203, 45, 78, 156, 203, 45, 189]),
  updateRegistryParams: Buffer.from([203, 45, 189, 67, 203, 45, 78, 156]),
};

/**
 * Create instruction to initialize the registry
 */
export function createInitializeRegistryInstruction(
  admin: PublicKey,
  payer: PublicKey,
  baseFee: number,
  minStake: number,
  protocolFeeBps: number,
  treasury: PublicKey,
  programId: PublicKey
): TransactionInstruction {
  const [registryState] = getRegistryStatePDA(programId);

  const data = Buffer.alloc(8 + 32 + 8 + 8 + 2 + 32);
  let offset = 0;
  
  // Discriminator
  INSTRUCTION_DISCRIMINATORS.initializeRegistry.copy(data, offset);
  offset += 8;
  
  // Admin pubkey
  admin.toBuffer().copy(data, offset);
  offset += 32;
  
  // Base fee
  data.writeBigUInt64LE(BigInt(baseFee), offset);
  offset += 8;
  
  // Min stake
  data.writeBigUInt64LE(BigInt(minStake), offset);
  offset += 8;
  
  // Protocol fee bps
  data.writeUInt16LE(protocolFeeBps, offset);
  offset += 2;
  
  // Treasury
  treasury.toBuffer().copy(data, offset);

  return new TransactionInstruction({
    keys: [
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to register a new automation job
 */
export function createRegisterJobInstruction(
  owner: PublicKey,
  params: RegisterJobParams,
  programId: PublicKey
): TransactionInstruction {
  const [registryState] = getRegistryStatePDA(programId);
  
  // We need the next job ID from the registry state to derive the job PDA
  // In a real implementation, you'd fetch this from the chain first
  // For now, we'll use a placeholder approach
  const jobId = Date.now(); // Temporary - should come from registry state
  const [automationJob] = getAutomationJobPDA(jobId, programId);

  const triggerParams = serializeTriggerParams(params.triggerType);
  const initialFunding = solToLamports(params.initialFunding);
  const minBalance = solToLamports(params.minBalance || 0.001);

  // Serialize instruction data
  const data = Buffer.alloc(
    8 + // discriminator
    32 + // target_program
    4 + Buffer.byteLength(params.targetInstruction, 'utf8') + // target_instruction
    1 + triggerParams.length + // trigger_type + variant data
    4 + triggerParams.length + // trigger_params
    8 + // gas_limit
    8 + // min_balance
    8   // initial_funding
  );

  let offset = 0;
  
  // Discriminator
  INSTRUCTION_DISCRIMINATORS.registerJob.copy(data, offset);
  offset += 8;
  
  // Target program
  params.targetProgram.toBuffer().copy(data, offset);
  offset += 32;
  
  // Target instruction (string)
  const instructionBytes = Buffer.from(params.targetInstruction, 'utf8');
  data.writeUInt32LE(instructionBytes.length, offset);
  offset += 4;
  instructionBytes.copy(data, offset);
  offset += instructionBytes.length;
  
  // Trigger type (simplified enum serialization)
  const triggerTypeStr = triggerTypeToString(params.triggerType);
  data.writeUInt8(getTriggerTypeIndex(params.triggerType), offset);
  offset += 1;
  
  // Trigger params
  data.writeUInt32LE(triggerParams.length, offset);
  offset += 4;
  data.set(triggerParams, offset);
  offset += triggerParams.length;
  
  // Gas limit
  data.writeBigUInt64LE(BigInt(params.gasLimit), offset);
  offset += 8;
  
  // Min balance
  data.writeBigUInt64LE(BigInt(minBalance), offset);
  offset += 8;
  
  // Initial funding
  data.writeBigUInt64LE(BigInt(initialFunding), offset);

  return new TransactionInstruction({
    keys: [
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: automationJob, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to fund an existing job
 */
export function createFundJobInstruction(
  owner: PublicKey,
  jobId: number,
  amount: number,
  programId: PublicKey
): TransactionInstruction {
  const [automationJob] = getAutomationJobPDA(jobId, programId);
  const fundingAmount = solToLamports(amount);

  const data = Buffer.alloc(8 + 8);
  let offset = 0;
  
  // Discriminator
  INSTRUCTION_DISCRIMINATORS.fundJob.copy(data, offset);
  offset += 8;
  
  // Amount
  data.writeBigUInt64LE(BigInt(fundingAmount), offset);

  return new TransactionInstruction({
    keys: [
      { pubkey: automationJob, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to cancel a job
 */
export function createCancelJobInstruction(
  owner: PublicKey,
  jobId: number,
  programId: PublicKey
): TransactionInstruction {
  const [registryState] = getRegistryStatePDA(programId);
  const [automationJob] = getAutomationJobPDA(jobId, programId);

  const data = Buffer.alloc(8);
  INSTRUCTION_DISCRIMINATORS.cancelJob.copy(data, 0);

  return new TransactionInstruction({
    keys: [
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: automationJob, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to update job parameters
 */
export function createUpdateJobInstruction(
  owner: PublicKey,
  jobId: number,
  updates: {
    gasLimit?: number;
    minBalance?: number;
    triggerParams?: Uint8Array;
  },
  programId: PublicKey
): TransactionInstruction {
  const [automationJob] = getAutomationJobPDA(jobId, programId);

  // Calculate data size
  let dataSize = 8; // discriminator
  dataSize += 1 + (updates.gasLimit ? 8 : 0); // Option<u64>
  dataSize += 1 + (updates.minBalance ? 8 : 0); // Option<u64>
  dataSize += 1 + (updates.triggerParams ? 4 + updates.triggerParams.length : 0); // Option<Vec<u8>>

  const data = Buffer.alloc(dataSize);
  let offset = 0;
  
  // Discriminator
  INSTRUCTION_DISCRIMINATORS.updateJob.copy(data, offset);
  offset += 8;
  
  // Gas limit (Option<u64>)
  if (updates.gasLimit !== undefined) {
    data.writeUInt8(1, offset); // Some
    offset += 1;
    data.writeBigUInt64LE(BigInt(updates.gasLimit), offset);
    offset += 8;
  } else {
    data.writeUInt8(0, offset); // None
    offset += 1;
  }
  
  // Min balance (Option<u64>)
  if (updates.minBalance !== undefined) {
    data.writeUInt8(1, offset); // Some
    offset += 1;
    data.writeBigUInt64LE(BigInt(solToLamports(updates.minBalance)), offset);
    offset += 8;
  } else {
    data.writeUInt8(0, offset); // None
    offset += 1;
  }
  
  // Trigger params (Option<Vec<u8>>)
  if (updates.triggerParams) {
    data.writeUInt8(1, offset); // Some
    offset += 1;
    data.writeUInt32LE(updates.triggerParams.length, offset);
    offset += 4;
    data.set(updates.triggerParams, offset);
  } else {
    data.writeUInt8(0, offset); // None
  }

  return new TransactionInstruction({
    keys: [
      { pubkey: automationJob, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to register as a keeper
 */
export function createRegisterKeeperInstruction(
  keeperAddress: PublicKey,
  stakeAmount: number,
  programId: PublicKey
): TransactionInstruction {
  const [registryState] = getRegistryStatePDA(programId);
  const [keeper] = getKeeperPDA(keeperAddress, programId);
  const stake = solToLamports(stakeAmount);

  const data = Buffer.alloc(8 + 8);
  let offset = 0;
  
  // Discriminator
  INSTRUCTION_DISCRIMINATORS.registerKeeper.copy(data, offset);
  offset += 8;
  
  // Stake amount
  data.writeBigUInt64LE(BigInt(stake), offset);

  return new TransactionInstruction({
    keys: [
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: keeper, isSigner: false, isWritable: true },
      { pubkey: keeperAddress, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to claim keeper rewards
 */
export function createClaimRewardsInstruction(
  keeperAddress: PublicKey,
  programId: PublicKey
): TransactionInstruction {
  const [keeper] = getKeeperPDA(keeperAddress, programId);

  const data = Buffer.alloc(8);
  INSTRUCTION_DISCRIMINATORS.claimRewards.copy(data, 0);

  return new TransactionInstruction({
    keys: [
      { pubkey: keeper, isSigner: false, isWritable: true },
      { pubkey: keeperAddress, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create instruction to execute a job (typically called by keepers)
 */
export function createExecuteJobInstruction(
  keeper: PublicKey,
  jobId: number,
  targetProgram: PublicKey,
  executionCount: number,
  remainingAccounts: PublicKey[] = [],
  programId: PublicKey
): TransactionInstruction {
  const [registryState] = getRegistryStatePDA(programId);
  const [automationJob] = getAutomationJobPDA(jobId, programId);
  const [keeperAccount] = getKeeperPDA(keeper, programId);
  const [executionRecord] = getExecutionRecordPDA(jobId, executionCount, programId);

  const data = Buffer.alloc(8 + 8);
  let offset = 0;
  
  // Discriminator
  INSTRUCTION_DISCRIMINATORS.executeJob.copy(data, offset);
  offset += 8;
  
  // Job ID
  data.writeBigUInt64LE(BigInt(jobId), offset);

  const keys = [
    { pubkey: registryState, isSigner: false, isWritable: true },
    { pubkey: automationJob, isSigner: false, isWritable: true },
    { pubkey: keeperAccount, isSigner: false, isWritable: true },
    { pubkey: executionRecord, isSigner: false, isWritable: true },
    { pubkey: keeper, isSigner: true, isWritable: true },
    { pubkey: targetProgram, isSigner: false, isWritable: false },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ];

  // Add remaining accounts for the target program call
  for (const account of remainingAccounts) {
    keys.push({ pubkey: account, isSigner: false, isWritable: true });
  }

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Helper function to get trigger type enum index
 */
function getTriggerTypeIndex(triggerType: TriggerType): number {
  switch (triggerType.type) {
    case 'time': return 0;
    case 'conditional': return 1;
    case 'log': return 2;
    case 'hybrid': return 3;
    default:
      throw new Error(`Unknown trigger type: ${(triggerType as any).type}`);
  }
}