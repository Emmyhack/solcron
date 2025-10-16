# SolCron Testing Framework

This directory contains comprehensive tests for the SolCron automation platform, covering all major components and integration scenarios.

## Test Structure

### Core Test Files

- **`basic.ts`** - Unit tests for data structures, economic models, and utility functions
- **`solcron.ts`** - Full integration tests for the registry program 
- **`execution.ts`** - Tests for the execution engine and cross-program invocation
- **`utils/test-fixture.ts`** - Test infrastructure and helper utilities

## Test Categories

### 1. Registry Tests (`solcron.ts`)

**Registry Initialization**
- ✅ Initialize registry with correct parameters
- ✅ Prevent double initialization
- ✅ Validate admin permissions

**Job Registration**
- ✅ Register time-based automation jobs
- ✅ Register conditional automation jobs  
- ✅ Register log-based automation jobs
- ✅ Validate job parameters and funding
- ✅ Handle insufficient funding errors

**Job Management**
- ✅ Fund existing jobs with additional SOL
- ✅ Update job parameters (gas limits, thresholds)
- ✅ Cancel jobs and refund remaining balance
- ✅ Prevent operations on cancelled jobs

**Keeper Registration**
- ✅ Register keepers with proper staking
- ✅ Validate minimum stake requirements
- ✅ Track keeper reputation and statistics
- ✅ Handle multiple keeper scenarios

**Job Execution**
- ✅ Execute jobs when trigger conditions are met
- ✅ Prevent premature execution attempts
- ✅ Update job state after execution
- ✅ Distribute rewards to keepers
- ✅ Record execution history

**Keeper Rewards**
- ✅ Calculate and distribute execution rewards
- ✅ Claim pending rewards
- ✅ Handle protocol fee collection
- ✅ Update keeper statistics

**Admin Functions**
- ✅ Update registry parameters
- ✅ Slash malicious keepers
- ✅ Emergency pause functionality
- ✅ Access control validation

### 2. Execution Engine Tests (`execution.ts`)

**Cross-Program Invocation**
- ✅ Execute regular CPI calls
- ✅ Execute signed CPI calls with PDAs
- ✅ Handle account metadata properly
- ✅ Validate execution authority

**Error Handling**
- ✅ Handle invalid authority errors
- ✅ Handle malformed instruction data
- ✅ Handle target program errors gracefully
- ✅ Validate account permissions

### 3. Basic Unit Tests (`basic.ts`)

**Data Structures**
- ✅ PDA derivation for all account types
- ✅ Trigger type validation and serialization
- ✅ Account size calculations
- ✅ Reputation scoring algorithms

**Economic Models**
- ✅ Fee calculation and distribution
- ✅ Minimum balance validation
- ✅ Staking requirement enforcement
- ✅ Reward distribution logic

**Time and Trigger Logic**
- ✅ Time-based trigger validation
- ✅ Conditional trigger evaluation
- ✅ Log-based trigger analysis
- ✅ Hybrid trigger combinations

**Error Handling**
- ✅ Account ownership validation
- ✅ PDA derivation error handling
- ✅ Numeric range validation
- ✅ Access control checks

**Integration Scenarios**
- ✅ Complete job lifecycle simulation
- ✅ Multi-keeper reward distribution
- ✅ Economic incentive modeling

## Running Tests

### All Tests
```bash
yarn test
# or
anchor test
```

### Individual Test Suites
```bash
# Basic unit tests
yarn test:basic

# Registry integration tests  
yarn test:registry

# Execution engine tests
yarn test:execution
```

### Local Development
```bash
# Start local validator (separate terminal)
solana-test-validator

# Deploy programs
anchor deploy

# Run tests
anchor test --skip-deploy
```

## Test Configuration

### Environment Setup
- Uses Anchor testing framework with Mocha/Chai
- Configured for localnet testing by default
- Automatic account funding via airdrops
- Comprehensive error assertion testing

### Test Accounts
Each test suite creates fresh keypairs for:
- Registry admin
- Treasury account  
- Job owners (users)
- Keeper nodes
- Target programs for testing

### Assertions
- Account state validation
- Balance change verification  
- Event emission testing
- Error condition handling
- Cross-program interaction validation

## Test Data

### Sample Job Types

**Time-Based Job**
```typescript
{
  targetProgram: "harvest_program", 
  targetInstruction: "harvest_rewards",
  triggerType: { timeBased: { interval: 3600 } }, // 1 hour
  gasLimit: 200_000,
  minBalance: 1_000_000, // 0.001 SOL
  initialFunding: 100_000_000 // 0.1 SOL
}
```

**Conditional Job**  
```typescript
{
  targetProgram: "liquidation_program",
  targetInstruction: "liquidate_position", 
  triggerType: { conditional: { logic: "collateral_ratio < 1.2" } },
  gasLimit: 300_000,
  minBalance: 5_000_000, // 0.005 SOL
  initialFunding: 200_000_000 // 0.2 SOL
}
```

**Log-Based Job**
```typescript
{
  targetProgram: "oracle_program",
  targetInstruction: "update_price",
  triggerType: { 
    logBased: { 
      programId: "price_feed_program",
      eventFilter: "price_deviation > 1%" 
    }
  },
  gasLimit: 150_000,
  minBalance: 2_000_000, // 0.002 SOL
  initialFunding: 150_000_000 // 0.15 SOL
}
```

## Expected Test Results

### Success Metrics
- **Registry Operations**: 100% success rate for valid operations
- **Job Lifecycle**: Complete job registration → execution → completion
- **Keeper Management**: Proper staking, rewards, and slashing
- **Error Handling**: Appropriate failures for invalid inputs
- **Economic Model**: Accurate fee calculation and distribution

### Performance Benchmarks
- **Job Registration**: < 1 second per job
- **Keeper Registration**: < 0.5 seconds per keeper  
- **Job Execution**: < 2 seconds per execution
- **Reward Claims**: < 0.5 seconds per claim

### Coverage Goals
- **Instruction Coverage**: 100% of all program instructions
- **Error Path Coverage**: All error conditions tested
- **Integration Coverage**: End-to-end workflow validation
- **Edge Case Coverage**: Boundary conditions and limits

## Troubleshooting

### Common Issues

**Program Deployment Errors**
```bash
# Clean and rebuild
anchor clean
anchor build
anchor deploy
```

**Account Funding Issues**
```bash
# Check localnet is running
solana cluster-info

# Check wallet balance
solana balance
```

**Test Timeout Issues**
```bash
# Increase timeout in Anchor.toml
test = "yarn run ts-mocha -p ./tsconfig.json -t 2000000 tests/**/*.ts"
```

### Debug Mode
```bash
# Run with verbose logging
ANCHOR_LOG=debug anchor test

# Run specific test with logging
DEBUG=* yarn test:basic
```

## Contributing

When adding new tests:
1. Follow the existing test structure and naming conventions
2. Include both positive and negative test cases  
3. Add comprehensive assertions for state changes
4. Update this README with new test descriptions
5. Ensure all tests pass before submitting PRs

## Integration with CI/CD

These tests are designed to integrate with continuous integration:

```yaml
# Example GitHub Actions workflow
- name: Run Solana Tests
  run: |
    anchor test --provider.cluster localnet
    
- name: Generate Test Report
  run: |
    yarn test --reporter json > test-results.json
```