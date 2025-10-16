import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { assert } from "chai";

describe("SolCron Registry - Full Integration Tests", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  
  const provider = anchor.AnchorProvider.env();
    const registryProgram = anchor.workspace.SolcronRegistry;
  
  // Test accounts
  let admin: Keypair;
  let treasury: Keypair;
  let user1: Keypair;
  let user2: Keypair;
  let keeper1: Keypair;
  let keeper2: Keypair;
  let targetProgram: Keypair;
  
  // PDAs
  let registryState: PublicKey;
  let registryStateBump: number;

  // Helper functions
  const getAutomationJobPDA = (jobId: number): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("job"), new anchor.BN(jobId).toArrayLike(Buffer, "le", 8)],
      registryProgram.programId
    );
  };

  const getKeeperPDA = (keeperAddress: PublicKey): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("keeper"), keeperAddress.toBuffer()],
      registryProgram.programId
    );
  };

  const getExecutionRecordPDA = (jobId: number, executionCount: number): [PublicKey, number] => {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("execution"),
        new anchor.BN(jobId).toArrayLike(Buffer, "le", 8),
        new anchor.BN(executionCount).toArrayLike(Buffer, "le", 8),
      ],
      registryProgram.programId
    );
  };

  before(async () => {
    // Generate test keypairs
    admin = Keypair.generate();
    treasury = Keypair.generate();
    user1 = Keypair.generate();
    user2 = Keypair.generate();
    keeper1 = Keypair.generate();
    keeper2 = Keypair.generate();
    targetProgram = Keypair.generate();

    // Derive registry state PDA
    [registryState, registryStateBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      registryProgram.programId
    );

    // Fund all test accounts
    const accounts = [admin, treasury, user1, user2, keeper1, keeper2];
    for (const account of accounts) {
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          account.publicKey,
          10 * LAMPORTS_PER_SOL
        )
      );
    }
  });

  describe("Registry Initialization", () => {
    it("Should initialize registry with correct parameters", async () => {
      const baseFee = new anchor.BN(5000);
      const minStake = new anchor.BN(LAMPORTS_PER_SOL);
      const protocolFeeBps = 250; // 2.5%

      await registryProgram.methods
        .initializeRegistry(
          admin.publicKey,
          baseFee,
          minStake,
          protocolFeeBps,
          treasury.publicKey
        )
        .accounts({
          registryState: registryState,
          payer: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Verify registry state
      const registryAccount = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registryAccount.admin.toString(), admin.publicKey.toString());
      assert.equal(registryAccount.baseFee.toNumber(), 5000);
      assert.equal(registryAccount.minStake.toNumber(), LAMPORTS_PER_SOL);
      assert.equal(registryAccount.protocolFeeBps, 250);
      assert.equal(registryAccount.treasury.toString(), treasury.publicKey.toString());
      assert.equal(registryAccount.nextJobId.toNumber(), 1);
      assert.equal(registryAccount.totalJobs.toNumber(), 0);
      assert.equal(registryAccount.activeJobs.toNumber(), 0);
    });

    it("Should fail to initialize registry twice", async () => {
      try {
        await registryProgram.methods
          .initializeRegistry(
            admin.publicKey,
            new anchor.BN(5000),
            new anchor.BN(LAMPORTS_PER_SOL),
            250,
            treasury.publicKey
          )
          .accounts({
            registryState: registryState,
            payer: provider.wallet.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        
        assert.fail("Registry should not be initializable twice");
      } catch (error) {
        // Expected to fail
        assert.include(error.toString(), "already in use");
      }
    });
  });

  describe("Job Registration", () => {
    it("Should register a time-based job successfully", async () => {
      const [jobAccount] = getAutomationJobPDA(1);
      
      const triggerType = {
        timeBased: { interval: new anchor.BN(3600) }
      };
      const triggerParams = Buffer.from(JSON.stringify({ interval: 3600 }));

      await registryProgram.methods
        .registerJob(
          targetProgram.publicKey,
          "harvest",
          triggerType,
          triggerParams,
          new anchor.BN(200_000), // gas limit
          new anchor.BN(1_000_000), // min balance (0.001 SOL)
          new anchor.BN(100_000_000) // initial funding (0.1 SOL)
        )
        .accounts({
          registryState: registryState,
          automationJob: jobAccount,
          owner: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Verify job was created
      const job = await registryProgram.account.automationJob.fetch(jobAccount);
      assert.equal(job.jobId.toNumber(), 1);
      assert.equal(job.owner.toString(), user1.publicKey.toString());
      assert.equal(job.targetProgram.toString(), targetProgram.publicKey.toString());
      assert.equal(job.targetInstruction, "harvest");
      assert.equal(job.balance.toNumber(), 100_000_000);
      assert.equal(job.gasLimit.toNumber(), 200_000);
      assert.equal(job.minBalance.toNumber(), 1_000_000);
      assert.isTrue(job.isActive);
      assert.equal(job.executionCount.toNumber(), 0);

      // Verify registry state updated
      const registry = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registry.nextJobId.toNumber(), 2);
      assert.equal(registry.totalJobs.toNumber(), 1);
      assert.equal(registry.activeJobs.toNumber(), 1);
    });

    it("Should register a conditional job successfully", async () => {
      const [jobAccount] = getAutomationJobPDA(2);
      
      const triggerType = {
        conditional: { logic: Buffer.from("balance > 1000000") }
      };
      const triggerParams = Buffer.from(JSON.stringify({ condition: "balance > 1000000" }));

      await registryProgram.methods
        .registerJob(
          targetProgram.publicKey,
          "liquidate",
          triggerType,
          triggerParams,
          new anchor.BN(300_000),
          new anchor.BN(1_000_000),
          new anchor.BN(200_000_000) // 0.2 SOL
        )
        .accounts({
          registryState: registryState,
          automationJob: jobAccount,
          owner: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      const job = await registryProgram.account.automationJob.fetch(jobAccount);
      assert.equal(job.jobId.toNumber(), 2);
      assert.equal(job.targetInstruction, "liquidate");
      assert.equal(job.gasLimit.toNumber(), 300_000);
    });

    it("Should fail to register job with insufficient funding", async () => {
      const [jobAccount] = getAutomationJobPDA(3);
      
      const triggerType = {
        timeBased: { interval: new anchor.BN(3600) }
      };
      const triggerParams = Buffer.from(JSON.stringify({ interval: 3600 }));

      try {
        await registryProgram.methods
          .registerJob(
            targetProgram.publicKey,
            "test",
            triggerType,
            triggerParams,
            new anchor.BN(200_000),
            new anchor.BN(10_000_000), // min balance 0.01 SOL
            new anchor.BN(5_000_000)   // initial funding 0.005 SOL (less than min)
          )
          .accounts({
            registryState: registryState,
            automationJob: jobAccount,
            owner: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();

        assert.fail("Should fail with insufficient balance");
      } catch (error) {
        assert.include(error.toString(), "InsufficientBalance");
      }
    });
  });

  describe("Job Management", () => {
    let jobId: number;
    let jobAccount: PublicKey;

    before(async () => {
      // Register a job for testing
      jobId = 3;
      [jobAccount] = getAutomationJobPDA(jobId);
      
      const triggerType = {
        timeBased: { interval: new anchor.BN(3600) }
      };
      const triggerParams = Buffer.from(JSON.stringify({ interval: 3600 }));

      await registryProgram.methods
        .registerJob(
          targetProgram.publicKey,
          "test_job",
          triggerType,
          triggerParams,
          new anchor.BN(200_000),
          new anchor.BN(1_000_000),
          new anchor.BN(50_000_000) // 0.05 SOL
        )
        .accounts({
          registryState: registryState,
          automationJob: jobAccount,
          owner: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();
    });

    it("Should fund a job successfully", async () => {
      const initialBalance = (await registryProgram.account.automationJob.fetch(jobAccount)).balance;
      const fundingAmount = new anchor.BN(25_000_000); // 0.025 SOL

      await registryProgram.methods
        .fundJob(fundingAmount)
        .accounts({
          automationJob: jobAccount,
          funder: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const job = await registryProgram.account.automationJob.fetch(jobAccount);
      assert.equal(
        job.balance.toNumber(),
        initialBalance.toNumber() + fundingAmount.toNumber()
      );
    });

    it("Should update job parameters", async () => {
      const newGasLimit = new anchor.BN(250_000);
      const newMinBalance = new anchor.BN(2_000_000);

      await registryProgram.methods
        .updateJob(newGasLimit, newMinBalance, null)
        .accounts({
          automationJob: jobAccount,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const job = await registryProgram.account.automationJob.fetch(jobAccount);
      assert.equal(job.gasLimit.toNumber(), 250_000);
      assert.equal(job.minBalance.toNumber(), 2_000_000);
    });

    it("Should cancel a job and refund balance", async () => {
      const initialUserBalance = await provider.connection.getBalance(user1.publicKey);
      const jobBalance = (await registryProgram.account.automationJob.fetch(jobAccount)).balance;

      await registryProgram.methods
        .cancelJob()
        .accounts({
          registryState: registryState,
          automationJob: jobAccount,
          owner: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Check job is inactive
      const job = await registryProgram.account.automationJob.fetch(jobAccount);
      assert.isFalse(job.isActive);
      assert.equal(job.balance.toNumber(), 0);

      // Check registry state updated
      const registry = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registry.activeJobs.toNumber(), 2); // Should decrease by 1
    });

    it("Should fail to update cancelled job", async () => {
      try {
        await registryProgram.methods
          .updateJob(new anchor.BN(300_000), null, null)
          .accounts({
            automationJob: jobAccount,
            owner: user1.publicKey,
          })
          .signers([user1])
          .rpc();

        assert.fail("Should not be able to update cancelled job");
      } catch (error) {
        assert.include(error.toString(), "InvalidJob");
      }
    });
  });

  describe("Keeper Registration", () => {
    it("Should register keeper successfully", async () => {
      const [keeperAccount] = getKeeperPDA(keeper1.publicKey);
      const stakeAmount = new anchor.BN(2 * LAMPORTS_PER_SOL); // 2 SOL

      await registryProgram.methods
        .registerKeeper(stakeAmount)
        .accounts({
          registryState: registryState,
          keeper: keeperAccount,
          keeperAccount: keeper1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([keeper1])
        .rpc();

      const keeper = await registryProgram.account.keeper.fetch(keeperAccount);
      assert.equal(keeper.address.toString(), keeper1.publicKey.toString());
      assert.equal(keeper.stakeAmount.toNumber(), 2 * LAMPORTS_PER_SOL);
      assert.equal(keeper.reputationScore.toNumber(), 5000); // Starting reputation
      assert.isTrue(keeper.isActive);
      assert.equal(keeper.successfulExecutions.toNumber(), 0);
      assert.equal(keeper.failedExecutions.toNumber(), 0);

      // Check registry state
      const registry = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registry.totalKeepers.toNumber(), 1);
      assert.equal(registry.activeKeepers.toNumber(), 1);
    });

    it("Should fail to register keeper with insufficient stake", async () => {
      const [keeperAccount] = getKeeperPDA(keeper2.publicKey);
      const insufficientStake = new anchor.BN(LAMPORTS_PER_SOL / 2); // 0.5 SOL

      try {
        await registryProgram.methods
          .registerKeeper(insufficientStake)
          .accounts({
            registryState: registryState,
            keeper: keeperAccount,
            keeperAccount: keeper2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([keeper2])
          .rpc();

        assert.fail("Should fail with insufficient stake");
      } catch (error) {
        assert.include(error.toString(), "InsufficientStake");
      }
    });

    it("Should register second keeper successfully", async () => {
      const [keeperAccount] = getKeeperPDA(keeper2.publicKey);
      const stakeAmount = new anchor.BN(LAMPORTS_PER_SOL); // 1 SOL (minimum)

      await registryProgram.methods
        .registerKeeper(stakeAmount)
        .accounts({
          registryState: registryState,
          keeper: keeperAccount,
          keeperAccount: keeper2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([keeper2])
        .rpc();

      const registry = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registry.totalKeepers.toNumber(), 2);
      assert.equal(registry.activeKeepers.toNumber(), 2);
    });
  });

  describe("Job Execution", () => {
    let executionJobId: number;
    let executionJobAccount: PublicKey;
    let keeperAccount: PublicKey;

    before(async () => {
      // Register a job for execution testing
      executionJobId = 4;
      [executionJobAccount] = getAutomationJobPDA(executionJobId);
      [keeperAccount] = getKeeperPDA(keeper1.publicKey);
      
      const triggerType = {
        timeBased: { interval: new anchor.BN(1) } // 1 second for testing
      };
      const triggerParams = Buffer.from(JSON.stringify({ interval: 1 }));

      await registryProgram.methods
        .registerJob(
          targetProgram.publicKey,
          "execute_test",
          triggerType,
          triggerParams,
          new anchor.BN(200_000),
          new anchor.BN(1_000_000),
          new anchor.BN(100_000_000) // 0.1 SOL
        )
        .accounts({
          registryState: registryState,
          automationJob: executionJobAccount,
          owner: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();
    });

    it("Should execute job successfully", async () => {
      const [executionRecord] = getExecutionRecordPDA(executionJobId, 0);

      // Wait a bit to ensure time condition is met
      await new Promise(resolve => setTimeout(resolve, 2000));

      const initialKeeperRewards = (await registryProgram.account.keeper.fetch(keeperAccount)).pendingRewards;
      const initialJobBalance = (await registryProgram.account.automationJob.fetch(executionJobAccount)).balance;

      await registryProgram.methods
        .executeJob(new anchor.BN(executionJobId))
        .accounts({
          registryState: registryState,
          automationJob: executionJobAccount,
          keeper: keeperAccount,
          executionRecord: executionRecord,
          keeperAccount: keeper1.publicKey,
          targetProgram: targetProgram.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([keeper1])
        .rpc();

      // Verify job state updated
      const job = await registryProgram.account.automationJob.fetch(executionJobAccount);
      assert.equal(job.executionCount.toNumber(), 1);
      assert.isTrue(job.lastExecution.toNumber() > 0);
      assert.isTrue(job.balance.toNumber() < initialJobBalance.toNumber()); // Fee deducted

      // Verify keeper state updated
      const keeper = await registryProgram.account.keeper.fetch(keeperAccount);
      assert.equal(keeper.successfulExecutions.toNumber(), 1);
      assert.isTrue(keeper.pendingRewards.toNumber() > initialKeeperRewards.toNumber());

      // Verify execution record created
      const execution = await registryProgram.account.executionRecord.fetch(executionRecord);
      assert.equal(execution.jobId.toNumber(), executionJobId);
      assert.equal(execution.keeper.toString(), keeper1.publicKey.toString());
      assert.isTrue(execution.success);

      // Verify registry stats updated
      const registry = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registry.totalExecutions.toNumber(), 1);
    });

    it("Should fail to execute job too soon", async () => {
      const [executionRecord] = getExecutionRecordPDA(executionJobId, 1);

      try {
        await registryProgram.methods
          .executeJob(new anchor.BN(executionJobId))
          .accounts({
            registryState: registryState,
            automationJob: executionJobAccount,
            keeper: keeperAccount,
            executionRecord: executionRecord,
            keeperAccount: keeper1.publicKey,
            targetProgram: targetProgram.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([keeper1])
          .rpc();

        assert.fail("Should fail to execute too soon");
      } catch (error) {
        assert.include(error.toString(), "InvalidTrigger");
      }
    });
  });

  describe("Keeper Rewards", () => {
    let keeperAccount: PublicKey;

    before(async () => {
      [keeperAccount] = getKeeperPDA(keeper1.publicKey);
    });

    it("Should claim keeper rewards", async () => {
      const keeper = await registryProgram.account.keeper.fetch(keeperAccount);
      const pendingRewards = keeper.pendingRewards.toNumber();
      
      if (pendingRewards > 0) {
        const initialBalance = await provider.connection.getBalance(keeper1.publicKey);

        await registryProgram.methods
          .claimRewards()
          .accounts({
            keeper: keeperAccount,
            keeperAccount: keeper1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([keeper1])
          .rpc();

        // Verify rewards were claimed
        const updatedKeeper = await registryProgram.account.keeper.fetch(keeperAccount);
        assert.equal(updatedKeeper.pendingRewards.toNumber(), 0);
        assert.equal(
          updatedKeeper.totalEarnings.toNumber(),
          keeper.totalEarnings.toNumber() + pendingRewards
        );
      }
    });

    it("Should fail to claim rewards when none available", async () => {
      try {
        await registryProgram.methods
          .claimRewards()
          .accounts({
            keeper: keeperAccount,
            keeperAccount: keeper1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([keeper1])
          .rpc();

        assert.fail("Should fail when no rewards to claim");
      } catch (error) {
        assert.include(error.toString(), "NoRewardsToClaim");
      }
    });
  });

  describe("Admin Functions", () => {
    it("Should update registry parameters", async () => {
      const newBaseFee = new anchor.BN(7500);
      const newMinStake = new anchor.BN(2 * LAMPORTS_PER_SOL);
      const newProtocolFeeBps = 300;

      await registryProgram.methods
        .updateRegistryParams(newBaseFee, newMinStake, newProtocolFeeBps)
        .accounts({
          registryState: registryState,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      const registry = await registryProgram.account.registryState.fetch(registryState);
      assert.equal(registry.baseFee.toNumber(), 7500);
      assert.equal(registry.minStake.toNumber(), 2 * LAMPORTS_PER_SOL);
      assert.equal(registry.protocolFeeBps, 300);
    });

    it("Should fail to update parameters as non-admin", async () => {
      try {
        await registryProgram.methods
          .updateRegistryParams(
            new anchor.BN(10000),
            new anchor.BN(LAMPORTS_PER_SOL),
            400
          )
          .accounts({
            registryState: registryState,
            admin: user1.publicKey, // Not the admin
          })
          .signers([user1])
          .rpc();

        assert.fail("Non-admin should not be able to update parameters");
      } catch (error) {
        assert.include(error.toString(), "Unauthorized");
      }
    });

    it("Should slash malicious keeper", async () => {
      const [keeperAccount] = getKeeperPDA(keeper2.publicKey);
      const slashAmount = new anchor.BN(LAMPORTS_PER_SOL / 2); // Slash 0.5 SOL
      const reason = "Test slashing";

      const keeper = await registryProgram.account.keeper.fetch(keeperAccount);
      const initialStake = keeper.stakeAmount.toNumber();

      await registryProgram.methods
        .slashKeeper(keeper2.publicKey, slashAmount, reason)
        .accounts({
          registryState: registryState,
          keeper: keeperAccount,
          admin: admin.publicKey,
          treasury: treasury.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      const slashedKeeper = await registryProgram.account.keeper.fetch(keeperAccount);
      assert.equal(
        slashedKeeper.stakeAmount.toNumber(),
        initialStake - slashAmount.toNumber()
      );
      assert.isTrue(slashedKeeper.reputationScore.toNumber() < keeper.reputationScore.toNumber());
    });
  });
});
