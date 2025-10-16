import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, AccountMeta } from "@solana/web3.js";
import { assert } from "chai";
import { SolcronExecution } from "../target/types/solcron_execution";

// Use the generated SolcronExecution IDL type

describe("SolCron Execution Engine Tests", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  
  const provider = anchor.AnchorProvider.env();
  const executionProgram = anchor.workspace.SolcronExecution;
  
  // Test accounts
  let executionAuthority: Keypair;
  let targetProgram: Keypair;
  let user: Keypair;
  
  before(async () => {
    // Generate test keypairs
    executionAuthority = Keypair.generate();
    targetProgram = Keypair.generate();
    user = Keypair.generate();

    // Fund test accounts
    const accounts = [executionAuthority, user];
    for (const account of accounts) {
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          account.publicKey,
          10 * LAMPORTS_PER_SOL
        )
      );
    }
  });

  describe("Cross-Program Invocation", () => {
    it("Should execute CPI call successfully", async () => {
      // Create a simple instruction to the system program (transfer)
      const instruction = SystemProgram.transfer({
        fromPubkey: user.publicKey,
        toPubkey: targetProgram.publicKey,
        lamports: 1000000, // 0.001 SOL
      });

      // Convert instruction to the format expected by the program
      const accountMetas: AccountMeta[] = instruction.keys.map((key) => ({
        pubkey: key.pubkey,
        isSigner: key.isSigner,
        isWritable: key.isWritable,
      }));

      await executionProgram.methods
        .executeCpiCall(
          targetProgram.publicKey, // In this case, system program
          instruction.data,
          accountMetas
        )
        .accounts({
          executionAuthority: executionAuthority.publicKey,
          targetProgram: SystemProgram.programId,
        })
        .remainingAccounts([
          {
            pubkey: user.publicKey,
            isSigner: true,
            isWritable: true,
          },
          {
            pubkey: targetProgram.publicKey,
            isSigner: false,
            isWritable: true,
          },
        ])
        .signers([executionAuthority, user])
        .rpc();

      // Verify the transfer occurred
      const targetBalance = await provider.connection.getBalance(targetProgram.publicKey);
      assert.equal(targetBalance, 1000000);
    });

    it("Should execute signed CPI call successfully", async () => {
      // Create seeds for PDA signing
      const seeds = [Buffer.from("execution_authority")];
      const [pdaAuthority] = PublicKey.findProgramAddressSync(
        seeds,
        executionProgram.programId
      );

      // Fund the PDA authority
      await provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          pdaAuthority,
          2 * LAMPORTS_PER_SOL
        )
      );

      const recipient = Keypair.generate();

      // Create a transfer instruction from PDA to recipient
      const instruction = SystemProgram.transfer({
        fromPubkey: pdaAuthority,
        toPubkey: recipient.publicKey,
        lamports: 500000, // 0.0005 SOL
      });

      const accountMetas: AccountMeta[] = instruction.keys.map((key) => ({
        pubkey: key.pubkey,
        isSigner: key.isSigner,
        isWritable: key.isWritable,
      }));

      await executionProgram.methods
        .executeCpiCallWithSeeds(
          SystemProgram.programId,
          instruction.data,
          accountMetas,
          seeds.map(seed => Array.from(seed)) // Seeds for signing
        )
        .accounts({
          executionAuthority: executionAuthority.publicKey,
          targetProgram: SystemProgram.programId,
        })
        .remainingAccounts([
          {
            pubkey: pdaAuthority,
            isSigner: false, // Will be signed by PDA
            isWritable: true,
          },
          {
            pubkey: recipient.publicKey,
            isSigner: false,
            isWritable: true,
          },
        ])
        .signers([executionAuthority])
        .rpc();

      // Verify the transfer occurred
      const recipientBalance = await provider.connection.getBalance(recipient.publicKey);
      assert.equal(recipientBalance, 500000);
    });

    it("Should fail CPI call with invalid authority", async () => {
      const invalidAuthority = Keypair.generate();
      
      const instruction = SystemProgram.transfer({
        fromPubkey: user.publicKey,
        toPubkey: targetProgram.publicKey,
        lamports: 1000000,
      });

      const accountMetas: AccountMeta[] = instruction.keys.map((key) => ({
        pubkey: key.pubkey,
        isSigner: key.isSigner,
        isWritable: key.isWritable,
      }));

      try {
        await executionProgram.methods
          .executeCpiCall(
            SystemProgram.programId,
            instruction.data,
            accountMetas
        )
          .accounts({
            executionAuthority: invalidAuthority.publicKey,
            targetProgram: SystemProgram.programId,
          })
          .remainingAccounts([
            {
              pubkey: user.publicKey,
              isSigner: true,
              isWritable: true,
            },
            {
              pubkey: targetProgram.publicKey,
              isSigner: false,
              isWritable: true,
            },
          ])
          .signers([invalidAuthority, user])
          .rpc();

        assert.fail("Should fail with unauthorized access");
      } catch (error) {
        assert.include(error.toString(), "Unauthorized");
      }
    });

    it("Should fail with invalid instruction data", async () => {
      const invalidData = Buffer.from([0, 1, 2, 3]); // Invalid instruction data
      
      try {
        await executionProgram.methods
          .executeCpiCall(
            SystemProgram.programId,
            invalidData,
            []
          )
          .accounts({
            executionAuthority: executionAuthority.publicKey,
            targetProgram: SystemProgram.programId,
          })
          .signers([executionAuthority])
          .rpc();

        assert.fail("Should fail with invalid instruction data");
      } catch (error) {
        // Expected to fail due to invalid instruction format
        assert.include(error.toString().toLowerCase(), "invalid");
      }
    });
  });

  describe("Error Handling", () => {
    it("Should handle target program errors gracefully", async () => {
      // Try to transfer more than available (should fail)
      const instruction = SystemProgram.transfer({
        fromPubkey: targetProgram.publicKey,
        toPubkey: user.publicKey,
        lamports: 100 * LAMPORTS_PER_SOL, // More than available
      });

      const accountMetas: AccountMeta[] = instruction.keys.map((key) => ({
        pubkey: key.pubkey,
        isSigner: key.isSigner,
        isWritable: key.isWritable,
      }));

      try {
        await executionProgram.methods
          .executeCpiCall(
            SystemProgram.programId,
            instruction.data,
            accountMetas
          )
          .accounts({
            executionAuthority: executionAuthority.publicKey,
            targetProgram: SystemProgram.programId,
          })
          .remainingAccounts([
            {
              pubkey: targetProgram.publicKey,
              isSigner: true,
              isWritable: true,
            },
            {
              pubkey: user.publicKey,
              isSigner: false,
              isWritable: true,
            },
          ])
          .signers([executionAuthority, targetProgram])
          .rpc();

        assert.fail("Should fail due to insufficient funds");
      } catch (error) {
        // Expected to fail
        assert.include(error.toString().toLowerCase(), "insufficient");
      }
    });
  });
});