#!/bin/bash

# SolCron Test Runner Script
# This script sets up the test environment and runs the full test suite

set -e

echo "🔨 SolCron Test Suite Runner"
echo "=============================="

# Check if we're in the right directory
if [ ! -f "Anchor.toml" ]; then
    echo "❌ Error: Please run this script from the SolCron project root"
    exit 1
fi

# Check dependencies
echo "📋 Checking dependencies..."

# Check if Anchor is installed
if ! command -v anchor &> /dev/null; then
    echo "❌ Anchor CLI not found. Please install: https://www.anchor-lang.com/docs/installation"
    exit 1
fi

# Check if Solana is installed  
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Please install: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

# Check if yarn is installed
if ! command -v yarn &> /dev/null; then
    echo "❌ Yarn not found. Please install: https://yarnpkg.com/getting-started/install"
    exit 1
fi

echo "✅ All dependencies found"

# Install Node.js dependencies
echo "📦 Installing Node.js dependencies..."
yarn install

# Check Solana config
echo "🔧 Checking Solana configuration..."
CLUSTER=$(solana config get | grep "RPC URL" | awk '{print $3}')
echo "Current cluster: $CLUSTER"

if [[ "$CLUSTER" != *"localhost"* ]] && [[ "$CLUSTER" != *"127.0.0.1"* ]]; then
    echo "⚠️  Warning: Not using localnet. Setting to localhost for testing..."
    solana config set --url localhost
fi

# Build the programs
echo "🔨 Building Solana programs..."
anchor build

# Generate program keypairs if they don't exist
echo "🔑 Checking program keypairs..."
if [ ! -f "target/deploy/solcron_registry-keypair.json" ]; then
    echo "Generating registry program keypair..."
    anchor keys sync
fi

# Check if local validator is running
echo "🌐 Checking local validator..."
if ! curl -s http://localhost:8899 > /dev/null 2>&1; then
    echo "❌ Local validator not running. Please start it with:"
    echo "   solana-test-validator"
    echo ""
    echo "Or run this script with the --start-validator flag"
    
    if [[ "$1" == "--start-validator" ]]; then
        echo "🚀 Starting local validator in background..."
        solana-test-validator --reset --quiet &
        VALIDATOR_PID=$!
        echo "Validator PID: $VALIDATOR_PID"
        
        # Wait for validator to be ready
        echo "⏳ Waiting for validator to be ready..."
        for i in {1..30}; do
            if curl -s http://localhost:8899 > /dev/null 2>&1; then
                echo "✅ Validator is ready"
                break
            fi
            sleep 2
            echo -n "."
        done
        
        if ! curl -s http://localhost:8899 > /dev/null 2>&1; then
            echo "❌ Validator failed to start"
            kill $VALIDATOR_PID 2>/dev/null || true
            exit 1
        fi
    else
        exit 1
    fi
fi

echo "✅ Local validator is running"

# Deploy programs
echo "🚀 Deploying programs..."
anchor deploy

# Run tests based on arguments
case "${1:-all}" in
    "all"|"--start-validator")
        echo "🧪 Running full test suite..."
        anchor test --skip-deploy
        ;;
    "basic")
        echo "🧪 Running basic unit tests..."
        anchor run test-basic
        ;;
    "registry") 
        echo "🧪 Running registry integration tests..."
        anchor run test-registry
        ;;
    "execution")
        echo "🧪 Running execution engine tests..."
        anchor run test-execution
        ;;
    *)
        echo "Usage: $0 [all|basic|registry|execution|--start-validator]"
        echo ""
        echo "Options:"
        echo "  all         - Run all tests (default)"
        echo "  basic       - Run basic unit tests only"
        echo "  registry    - Run registry integration tests only" 
        echo "  execution   - Run execution engine tests only"
        echo "  --start-validator - Start local validator and run all tests"
        exit 1
        ;;
esac

# Cleanup if we started the validator
if [[ -n "$VALIDATOR_PID" ]]; then
    echo "🧹 Cleaning up validator..."
    kill $VALIDATOR_PID 2>/dev/null || true
fi

echo ""
echo "✅ Tests completed successfully!"
echo "📊 Test results available in the terminal output above"