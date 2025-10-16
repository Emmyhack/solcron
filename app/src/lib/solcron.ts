import { Connection, PublicKey, Transaction } from '@solana/web3.js';
import { Program, AnchorProvider, Idl, setProvider } from '@coral-xyz/anchor';

export interface AutomationJob {
  id: string;
  name: string;
  description: string;
  schedule: string;
  targetProgram: string;
  instruction: string;
  parameters: any[];
  isActive: boolean;
  nextExecution: Date;
  createdAt: Date;
  executions: number;
}

export interface Keeper {
  id: string;
  address: string;
  stake: number;
  isActive: boolean;
  reputation: number;
  totalExecutions: number;
  successRate: number;
  joinedAt: Date;
}

export interface ExecutionRecord {
  id: string;
  jobId: string;
  keeperId: string;
  timestamp: Date;
  success: boolean;
  gasUsed: number;
  error?: string;
}

export class SolCronClient {
  private connection: Connection;
  private provider: AnchorProvider | null = null;
  private registryProgram: Program<Idl> | null = null;
  private executionProgram: Program<Idl> | null = null;
  
  // Program IDs - these should match your deployed programs
  private readonly REGISTRY_PROGRAM_ID = new PublicKey('11111111111111111111111111111112'); // Replace with actual program ID
  private readonly EXECUTION_PROGRAM_ID = new PublicKey('11111111111111111111111111111112'); // Replace with actual program ID

  constructor(connection: Connection, wallet?: any) {
    this.connection = connection;
    
    if (wallet) {
      this.provider = new AnchorProvider(connection, wallet, {});
      setProvider(this.provider);
      
      // Initialize programs with real IDL
      try {
        // Note: In a real implementation, you would load the IDL from the deployed program
        // For now, we'll use the imported types but implement basic functionality
        console.log('SolCron client initialized with wallet');
      } catch (error) {
        console.warn('Failed to initialize programs:', error);
      }
    }
  }

  // Registry operations
  async getJobs(): Promise<AutomationJob[]> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real blockchain data fetching
      // For now, return empty array as we transition from mock data
      console.log('Fetching jobs from blockchain...');
      return [];
      
    } catch (error) {
      console.error('Failed to fetch jobs:', error);
      return [];
    }
  }

  async createJob(
    targetProgram: PublicKey | string | Partial<AutomationJob>, 
    targetInstruction?: string,
    triggerType?: string,
    triggerParams?: Uint8Array,
    gasLimit?: number,
    minBalance?: number,
    initialFunding?: number
  ): Promise<string> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // Handle both single object parameter and multiple parameters
      if (typeof targetProgram === 'object' && 'jobId' in (targetProgram as any)) {
        // Single object parameter (Partial<AutomationJob>)
        console.log('Creating job on blockchain:', targetProgram);
      } else {
        // Multiple parameters
        console.log('Creating job on blockchain:', {
          targetProgram,
          targetInstruction,
          triggerType,
          triggerParams,
          gasLimit,
          minBalance,
          initialFunding
        });
      }
      
      // TODO: Implement real job creation on blockchain
      // Return a mock job ID for now
      return `job_${Date.now()}`;
      
    } catch (error) {
      console.error('Failed to create job:', error);
      throw error;
    }
  }

  async updateJob(jobId: string, updates: Partial<AutomationJob>): Promise<void> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real job update on blockchain
      console.log('Updating job on blockchain:', jobId, updates);
      
    } catch (error) {
      console.error('Failed to update job:', error);
      throw error;
    }
  }

  async deleteJob(jobId: string): Promise<void> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real job deletion on blockchain
      console.log('Deleting job on blockchain:', jobId);
      
    } catch (error) {
      console.error('Failed to delete job:', error);
      throw error;
    }
  }

  // Keeper operations
  async getKeepers(): Promise<Keeper[]> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real keeper data fetching
      console.log('Fetching keepers from blockchain...');
      return [];
      
    } catch (error) {
      console.error('Failed to fetch keepers:', error);
      return [];
    }
  }

  async registerKeeper(stake: number): Promise<string> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real keeper registration
      console.log('Registering keeper on blockchain with stake:', stake);
      
      // Return a mock transaction signature
      return `txsig_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
    } catch (error) {
      console.error('Failed to register keeper:', error);
      throw error;
    }
  }

  // Execution operations
  async getExecutionHistory(jobId?: string): Promise<ExecutionRecord[]> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real execution history fetching
      console.log('Fetching execution history from blockchain...', { jobId });
      return [];
      
    } catch (error) {
      console.error('Failed to fetch execution history:', error);
      return [];
    }
  }

  // Aliases for compatibility with existing provider
  async getAllJobs(): Promise<AutomationJob[]> {
    return this.getJobs();
  }

  async getAllKeepers(): Promise<Keeper[]> {
    return this.getKeepers();
  }

  async getRegistryState(): Promise<any> {
    try {
      if (!this.provider) {
        throw new Error('Wallet not connected');
      }
      
      // TODO: Implement real registry state fetching
      console.log('Fetching registry state from blockchain...');
      return {
        totalJobs: 0,
        activeJobs: 0,
        totalKeepers: 0,
        activeKeepers: 0,
        totalExecutions: 0,
        successfulExecutions: 0,
        baseFee: 0.001,
        treasury: 0,
        lastUpdated: new Date()
      };
      
    } catch (error) {
      console.error('Failed to get registry state:', error);
      throw error;
    }
  }

  // Utility methods
  async getBalance(address: string): Promise<number> {
    try {
      const balance = await this.connection.getBalance(new PublicKey(address));
      return balance / 1e9; // Convert lamports to SOL
    } catch (error) {
      console.error('Failed to get balance:', error);
      return 0;
    }
  }

  isConnected(): boolean {
    return this.provider !== null;
  }

  getConnection(): Connection {
    return this.connection;
  }
}

// Utility functions
export function createSolCronClient(
  connection: Connection, 
  wallet: any, 
  network: string
): SolCronClient {
  return new SolCronClient(connection, wallet);
}

export async function detectNetwork(connection: Connection): Promise<'mainnet-beta' | 'devnet' | 'testnet' | 'localnet'> {
  try {
    const genesisHash = await connection.getGenesisHash();
    
    // Check for common network genesis hashes
    if (genesisHash === '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d') {
      return 'mainnet-beta';
    } else if (genesisHash === 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG') {
      return 'devnet';
    } else if (genesisHash === '4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY') {
      return 'testnet';
    } else {
      return 'localnet';
    }
  } catch (error) {
    console.warn('Failed to detect network, defaulting to devnet:', error);
    return 'devnet';
  }
}