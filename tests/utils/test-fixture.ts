import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolcronRegistry } from "../../target/types/solcron_registry";
import { SolcronExecution } from "../../target/types/solcron_execution";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { assert } from "chai";

// Test fixtures and utilities
export class TestFixture {
  provider: anchor.AnchorProvider;
  registryProgram: any;
  executionProgram: any;
  
  // Test keypairs
  admin: Keypair;
  treasury: Keypair;
  user: Keypair;
  keeper: Keypair;
  targetProgram: Keypair;
  
  // PDAs
  registryState: PublicKey;
  registryStateBump: number;

  constructor() {
    this.provider = anchor.AnchorProvider.env();
    anchor.setProvider(this.provider);

    this.registryProgram = anchor.workspace.SolcronRegistry;
    this.executionProgram = anchor.workspace.SolcronExecution;

    // Generate test keypairs
    this.admin = Keypair.generate();
    this.treasury = Keypair.generate();
    this.user = Keypair.generate();
    this.keeper = Keypair.generate();
    this.targetProgram = Keypair.generate();

    // Derive registry state PDA
    [this.registryState, this.registryStateBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      this.registryProgram.programId
    );
  }

  /**
   * Fund all test accounts
   */
  async fundAccounts() {
    const accounts = [this.admin, this.treasury, this.user, this.keeper];
    
    for (const account of accounts) {
      await this.provider.connection.confirmTransaction(
        await this.provider.connection.requestAirdrop(
          account.publicKey,
          10 * LAMPORTS_PER_SOL
        )
      );
    }
  }

  /**
   * Initialize the registry
   */
  async initializeRegistry() {
    const baseFee = new anchor.BN(5000); // 5000 lamports
    const minStake = new anchor.BN(LAMPORTS_PER_SOL); // 1 SOL
    const protocolFeeBps = 250; // 2.5%

    await this.registryProgram.methods
      .initializeRegistry(
        this.admin.publicKey,
        baseFee,
        minStake,
        protocolFeeBps,
        this.treasury.publicKey
      )
      .accounts({
        registryState: this.registryState,
        payer: this.provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  /**
   * Get automation job PDA
   */
  getAutomationJobPDA(jobId: number): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("job"), new anchor.BN(jobId).toArrayLike(Buffer, "le", 8)],
      this.registryProgram.programId
    );
  }

  /**
   * Get keeper PDA
   */
  getKeeperPDA(keeperAddress: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("keeper"), keeperAddress.toBuffer()],
      this.registryProgram.programId
    );
  }

  /**
   * Get execution record PDA
   */
  getExecutionRecordPDA(jobId: number, executionCount: number): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("execution"),
        new anchor.BN(jobId).toArrayLike(Buffer, "le", 8),
        new anchor.BN(executionCount).toArrayLike(Buffer, "le", 8),
      ],
      this.registryProgram.programId
    );
  }

  /**
   * Create time-based trigger params
   */
  createTimeBasedTrigger(interval: number): any {
    return {
      timeBased: {
        interval: new anchor.BN(interval),
      },
    };
  }

  /**
   * Create conditional trigger params
   */
  createConditionalTrigger(condition: string): any {
    return {
      conditional: {
        logic: Buffer.from(condition),
      },
    };
  }

  /**
   * Register a test job
   */
  async registerTestJob(
    owner: Keypair,
    targetProgram: PublicKey,
    targetInstruction: string,
    triggerType: any,
    gasLimit: number = 200_000,
    initialFunding: number = 0.1 * LAMPORTS_PER_SOL
  ): Promise<{ jobId: number; jobAccount: PublicKey }> {
    // Get next job ID from registry state
    const registryState = await this.registryProgram.account.registryState.fetch(
      this.registryState
    );
    const jobId = registryState.nextJobId.toNumber();

    const [jobAccount] = this.getAutomationJobPDA(jobId);

    const triggerParams = Buffer.from(JSON.stringify({ interval: 3600 })); // Placeholder

    await this.registryProgram.methods
      .registerJob(
        targetProgram,
        targetInstruction,
        triggerType,
        triggerParams,
        new anchor.BN(gasLimit),
        new anchor.BN(1000000), // min balance: 0.001 SOL
        new anchor.BN(initialFunding)
      )
      .accounts({
        registryState: this.registryState,
        automationJob: jobAccount,
        owner: owner.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    return { jobId, jobAccount };
  }

  /**
   * Register a test keeper
   */
  async registerTestKeeper(
    keeper: Keypair,
    stakeAmount: number = LAMPORTS_PER_SOL
  ): Promise<PublicKey> {
    const [keeperAccount] = this.getKeeperPDA(keeper.publicKey);

    await this.registryProgram.methods
      .registerKeeper(new anchor.BN(stakeAmount))
      .accounts({
        registryState: this.registryState,
        keeper: keeperAccount,
        keeperAccount: keeper.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([keeper])
      .rpc();

    return keeperAccount;
  }

  /**
   * Wait for a specified time
   */
  async wait(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Assert account balance
   */
  async assertBalance(
    account: PublicKey,
    expectedBalance: number,
    tolerance: number = 1000
  ) {
    const balance = await this.provider.connection.getBalance(account);
    assert.approximately(
      balance,
      expectedBalance,
      tolerance,
      `Account balance mismatch: expected ~${expectedBalance}, got ${balance}`
    );
  }

  /**
   * Get current timestamp
   */
  getCurrentTimestamp(): number {
    return Math.floor(Date.now() / 1000);
  }

  /**
   * Advance time (for testing time-based triggers)
   */
  async advanceTime(seconds: number) {
    // In a real test environment, you'd use a test validator with time manipulation
    // For now, we'll just wait
    await this.wait(seconds * 1000);
  }
}

// Test data factories
export class TestDataFactory {
  static createTimeBasedJob(interval: number = 3600) {
    return {
      targetInstruction: "test_instruction",
      triggerType: { timeBased: { interval: new anchor.BN(interval) } },
      gasLimit: 200_000,
      initialFunding: 0.1 * LAMPORTS_PER_SOL,
      minBalance: 0.001 * LAMPORTS_PER_SOL,
    };
  }

  static createConditionalJob(condition: string = "balance > 1000000") {
    return {
      targetInstruction: "conditional_instruction", 
      triggerType: { conditional: { logic: Buffer.from(condition) } },
      gasLimit: 300_000,
      initialFunding: 0.2 * LAMPORTS_PER_SOL,
      minBalance: 0.001 * LAMPORTS_PER_SOL,
    };
  }

  static createLogTriggerJob(eventSignature: string = "TestEvent") {
    return {
      targetInstruction: "event_instruction",
      triggerType: { logTrigger: { eventSignature } },
      gasLimit: 150_000,
      initialFunding: 0.05 * LAMPORTS_PER_SOL,
      minBalance: 0.001 * LAMPORTS_PER_SOL,
    };
  }

  static createHybridJob() {
    return {
      targetInstruction: "hybrid_instruction",
      triggerType: { hybrid: {} },
      gasLimit: 250_000,
      initialFunding: 0.15 * LAMPORTS_PER_SOL,
      minBalance: 0.001 * LAMPORTS_PER_SOL,
    };
  }
}

// Custom assertions
export class TestAssertions {
  static async assertJobState(
    fixture: TestFixture,
    jobAccount: PublicKey,
    expectedState: Partial<any>
  ) {
    const job = await fixture.registryProgram.account.automationJob.fetch(jobAccount);
    
    if (expectedState.isActive !== undefined) {
      assert.equal(job.isActive, expectedState.isActive, "Job active state mismatch");
    }
    
    if (expectedState.executionCount !== undefined) {
      assert.equal(
        job.executionCount.toNumber(),
        expectedState.executionCount,
        "Execution count mismatch"
      );
    }
    
    if (expectedState.balance !== undefined) {
      assert.approximately(
        job.balance.toNumber(),
        expectedState.balance,
        1000,
        "Job balance mismatch"
      );
    }
  }

  static async assertKeeperState(
    fixture: TestFixture,
    keeperAccount: PublicKey,
    expectedState: Partial<any>
  ) {
    const keeper = await fixture.registryProgram.account.keeper.fetch(keeperAccount);
    
    if (expectedState.isActive !== undefined) {
      assert.equal(keeper.isActive, expectedState.isActive, "Keeper active state mismatch");
    }
    
    if (expectedState.successfulExecutions !== undefined) {
      assert.equal(
        keeper.successfulExecutions.toNumber(),
        expectedState.successfulExecutions,
        "Successful executions mismatch"
      );
    }
    
    if (expectedState.reputationScore !== undefined) {
      assert.equal(
        keeper.reputationScore.toNumber(),
        expectedState.reputationScore,
        "Reputation score mismatch"
      );
    }
  }

  static async assertRegistryState(
    fixture: TestFixture,
    expectedState: Partial<any>
  ) {
    const registry = await fixture.registryProgram.account.registryState.fetch(
      fixture.registryState
    );
    
    if (expectedState.totalJobs !== undefined) {
      assert.equal(
        registry.totalJobs.toNumber(),
        expectedState.totalJobs,
        "Total jobs mismatch"
      );
    }
    
    if (expectedState.activeJobs !== undefined) {
      assert.equal(
        registry.activeJobs.toNumber(),
        expectedState.activeJobs,
        "Active jobs mismatch"
      );
    }
    
    if (expectedState.totalKeepers !== undefined) {
      assert.equal(
        registry.totalKeepers.toNumber(),
        expectedState.totalKeepers,
        "Total keepers mismatch"
      );
    }
  }

  static assertTransactionLogs(logs: string[], expectedMessages: string[]) {
    for (const message of expectedMessages) {
      const found = logs.some(log => log.includes(message));
      assert.isTrue(found, `Expected log message not found: ${message}`);
    }
  }
}