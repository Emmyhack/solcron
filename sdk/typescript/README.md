# SolCron TypeScript SDK

A comprehensive TypeScript SDK for interacting with SolCron, the decentralized automation platform for Solana.

## Installation

```bash
npm install @solcron/sdk
# or
yarn add @solcron/sdk
```

## Quick Start

```typescript
import { createSolCronClientForNetwork, PublicKey } from '@solcron/sdk';

// Create a client for devnet
const client = createSolCronClientForNetwork('devnet');

// Register a time-based automation job
const result = await client.registerJob({
  targetProgram: new PublicKey('YourProgramId...'),
  targetInstruction: 'harvest',
  triggerType: {
    type: 'time',
    interval: 3600 // Execute every hour
  },
  gasLimit: 200_000,
  initialFunding: 0.1 // 0.1 SOL
}, payerPublicKey);

console.log(`Job registered: ${result.jobId}`);
```

## Features

- **Job Management**: Register, fund, update, and cancel automation jobs
- **Keeper Operations**: Register as a keeper, execute jobs, claim rewards
- **Monitoring**: Get job statistics, execution history, and network stats
- **Multiple Trigger Types**: Time-based, conditional, event-based, and hybrid triggers
- **Type Safety**: Full TypeScript support with comprehensive type definitions
- **Error Handling**: Detailed error types and logging
- **Retry Logic**: Built-in retry mechanisms for network operations

## Trigger Types

### Time-Based Triggers
Execute jobs at regular intervals:

```typescript
const timeBasedJob = await client.registerJob({
  targetProgram: programId,
  targetInstruction: 'compound',
  triggerType: {
    type: 'time',
    interval: 86400 // Daily execution
  },
  gasLimit: 150_000,
  initialFunding: 0.05
}, payer);
```

### Conditional Triggers
Execute when specific conditions are met:

```typescript
const conditionalJob = await client.registerJob({
  targetProgram: programId,
  targetInstruction: 'liquidate',
  triggerType: {
    type: 'conditional',
    condition: 'balance > 1000000' // Execute when balance > 1 SOL
  },
  gasLimit: 300_000,
  initialFunding: 0.2
}, payer);
```

### Event-Based Triggers
Execute when specific events occur:

```typescript
const eventBasedJob = await client.registerJob({
  targetProgram: programId,
  targetInstruction: 'claim',
  triggerType: {
    type: 'log',
    eventSignature: 'RewardAvailable'
  },
  gasLimit: 100_000,
  initialFunding: 0.03
}, payer);
```

### Hybrid Triggers
Combine multiple conditions:

```typescript
const hybridJob = await client.registerJob({
  targetProgram: programId,
  targetInstruction: 'rebalance',
  triggerType: {
    type: 'hybrid',
    config: {
      timeInterval: 7200, // At least every 2 hours
      condition: 'token_balance > 100',
      requireAll: false // OR condition
    }
  },
  gasLimit: 250_000,
  initialFunding: 0.15
}, payer);
```

## Job Management

### Get Job Information
```typescript
const job = await client.getJob(jobId);
console.log(`Job balance: ${job.balance} lamports`);
console.log(`Executions: ${job.executionCount}`);
```

### Fund a Job
```typescript
await client.fundJob(jobId, 0.05, payer); // Add 0.05 SOL
```

### Cancel a Job
```typescript
await client.cancelJob(jobId, owner);
```

### Get Job Statistics
```typescript
const stats = await client.getJobStats(jobId);
console.log(`Success rate: ${stats.successRate * 100}%`);
console.log(`Total fees spent: ${stats.totalFeesSpent} lamports`);
```

## Keeper Operations

### Register as Keeper
```typescript
await client.registerKeeper(
  1.0, // Stake 1 SOL
  keeperPublicKey
);
```

### Get Keeper Stats
```typescript
const keeperStats = await client.getKeeperStats(keeperPublicKey);
console.log(`Reputation: ${keeperStats.reputationScore}/10000`);
console.log(`Total earnings: ${keeperStats.totalEarnings} SOL`);
```

### Claim Rewards
```typescript
await client.claimRewards(keeperPublicKey);
```

## Network Statistics

```typescript
const networkStats = await client.getNetworkStats();
console.log(`Total jobs: ${networkStats.totalJobs}`);
console.log(`Active keepers: ${networkStats.activeKeepers}`);
console.log(`Total executions: ${networkStats.totalExecutions}`);
```

## Error Handling

The SDK provides specific error types for different scenarios:

```typescript
import { 
  SolCronError, 
  JobNotFoundError, 
  InsufficientBalanceError 
} from '@solcron/sdk';

try {
  await client.registerJob(params, payer);
} catch (error) {
  if (error instanceof JobNotFoundError) {
    console.error('Job not found:', error.message);
  } else if (error instanceof InsufficientBalanceError) {
    console.error('Insufficient balance:', error.message);
  } else if (error instanceof SolCronError) {
    console.error('SolCron error:', error.message);
    console.log('Error code:', error.code);
    console.log('Transaction logs:', error.logs);
  }
}
```

## Utilities

### Cost Estimation
```typescript
import { estimateJobCost } from '@solcron/sdk';

const cost = estimateJobCost({
  baseFee: 5000, // lamports
  gasLimit: 200_000,
  executionsPerDay: 24,
  days: 30
});

console.log(`Monthly cost: ${cost.totalCostSOL} SOL`);
```

### Format Utilities
```typescript
import { formatSOL, formatTimestamp, lamportsToSol } from '@solcron/sdk';

console.log(formatSOL(1_000_000_000)); // "1.0000 SOL"
console.log(formatTimestamp(Date.now() / 1000)); // "12/25/2023, 3:30:45 PM"
console.log(lamportsToSol(500_000_000)); // 0.5
```

## Configuration

### Custom Configuration
```typescript
import { SolCronClient, Connection, PublicKey } from '@solcron/sdk';

const connection = new Connection('https://your-rpc-endpoint.com');
const client = new SolCronClient(connection, {
  registryProgramId: new PublicKey('YourRegistryProgramId...'),
  executionProgramId: new PublicKey('YourExecutionProgramId...'),
  cluster: 'mainnet-beta',
  commitment: 'confirmed'
});
```

### Environment-Specific Clients
```typescript
import { createSolCronClientForNetwork } from '@solcron/sdk';

// Development
const devClient = createSolCronClientForNetwork('devnet');

// Production
const mainnetClient = createSolCronClientForNetwork('mainnet-beta');

// Local testing
const localClient = createSolCronClientForNetwork('localnet');
```

## Advanced Usage

### Custom RPC Endpoint
```typescript
const client = createSolCronClientForNetwork(
  'mainnet-beta',
  'https://your-premium-rpc.com'
);
```

### Batch Operations
```typescript
// Register multiple jobs
const jobs = await Promise.all([
  client.registerJob(job1Params, payer),
  client.registerJob(job2Params, payer),
  client.registerJob(job3Params, payer)
]);

console.log(`Created ${jobs.length} jobs`);
```

### Event Monitoring
```typescript
// Monitor job executions (pseudo-code)
const job = await client.getJob(jobId);
let lastExecution = job.lastExecution;

setInterval(async () => {
  const updatedJob = await client.getJob(jobId);
  if (updatedJob.lastExecution > lastExecution) {
    console.log('Job executed!');
    lastExecution = updatedJob.lastExecution;
  }
}, 10000); // Check every 10 seconds
```

## Best Practices

1. **Fund Jobs Appropriately**: Ensure jobs have sufficient balance for expected executions
2. **Set Reasonable Gas Limits**: Balance between execution success and cost
3. **Monitor Job Health**: Regularly check job statistics and balance
4. **Handle Errors Gracefully**: Implement proper error handling and retry logic
5. **Use Appropriate Trigger Types**: Choose the most efficient trigger for your use case

## Examples

See the `examples/` directory for complete working examples:

- Time-based DeFi yield farming automation
- Conditional liquidation bot
- Event-driven reward claiming
- Multi-condition portfolio rebalancing

## Support

- Documentation: [https://docs.solcron.com](https://docs.solcron.com)
- GitHub: [https://github.com/your-org/solcron](https://github.com/your-org/solcron)
- Discord: [https://discord.gg/solcron](https://discord.gg/solcron)

## License

MIT - see [LICENSE](./LICENSE) file for details.