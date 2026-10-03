#!/usr/bin/env bash
set -euo pipefail

# Deploy All Contracts to Stellar Testnet
# Usage: ./scripts/deploy_all_testnet.sh
# Requires: stellar CLI installed, source account `alice` funded on testnet

NETWORK="testnet"
SOURCE="alice"
CONTRACTS_ROOT="StarShopContracts"
DEPLOYMENTS_FILE="DEPLOYMENTS.md"

echo "🚀 Starting deployment to Stellar Testnet"
echo "Source: $SOURCE | Network: $NETWORK"

if [ ! -d "$CONTRACTS_ROOT" ]; then
  echo "❌ Contracts directory $CONTRACTS_ROOT not found"
  exit 1
fi

for contract_dir in "$CONTRACTS_ROOT"/*; do
  if [ -d "$contract_dir" ]; then
    contract_name=$(basename "$contract_dir")
    echo ""
    echo "📦 Building $contract_name..."
    (cd "$contract_dir" && stellar contract build)

    wasm_path="target/wasm32-unknown-unknown/release/${contract_name}.wasm"
    
    if [ ! -f "$contract_dir/$wasm_path" ]; then
      echo "⚠️ WASM not found at $contract_dir/$wasm_path, skipping deploy"
      continue
    fi

    echo "🚀 Deploying $contract_name..."
    deploy_output=$(stellar contract deploy \
      --wasm "$contract_dir/$wasm_path" \
      --source "$SOURCE" \
      --network "$NETWORK" \
      --alias "$contract_name-testnet"
    )

    contract_id=$(echo "$deploy_output" | grep -o 'CA[0-9A-Z]*' | head -n1 || true)
    
    if [ -n "$contract_id" ]; then
      echo "✅ $contract_name deployed: $contract_id"
      # Append to DEPLOYMENTS.md - manual update recommended for audit
      echo "- $contract_name: $contract_id"
    else
      echo "❌ Failed to extract contract ID for $contract_name"
    fi
  fi
done

echo ""
echo "✅ Deployment run complete. Update DEPLOYMENTS.md with contract IDs."
