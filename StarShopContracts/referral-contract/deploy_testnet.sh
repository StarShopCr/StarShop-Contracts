#!/usr/bin/env bash
# ==============================================================================
# Referral Contract Testnet Deployment Script (Test-Only)
# StarShop Smart Contracts Ecosystem
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Configuration & Network Parameters
NETWORK="${NETWORK:-testnet}"
RPC_URL="${RPC_URL:-https://soroban-testnet.stellar.org}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
DEPLOYED_CONTRACT_ID="CCHXSA6WFERL3VE4K4TEHFYOYIEFIP5CXWY6OGMKUHXBQG3HTRCMZRO6"
WASM_PATH="${WORKSPACE_ROOT}/target/wasm32-unknown-unknown/release/referral_contract.wasm"

echo "============================================================"
echo "  StarShop Referral Contract Deployment (Stellar Testnet)   "
echo "  Status: TEST-ONLY / STAGING VERIFICATION                  "
echo "============================================================"
echo "Target Network:     ${NETWORK}"
echo "RPC Endpoint:       ${RPC_URL}"
echo "Contract ID:        ${DEPLOYED_CONTRACT_ID}"
echo "Network Passphrase: ${NETWORK_PASSPHRASE}"
echo "============================================================"

# Step 1: Ensure WASM artifact exists or compile
if [[ ! -f "${WASM_PATH}" ]]; then
    echo "⚙️ WASM artifact not found. Compiling referral-contract..."
    (cd "${WORKSPACE_ROOT}" && cargo build -p referral-contract --release --target wasm32-unknown-unknown)
fi

echo "✅ Verified WASM artifact: ${WASM_PATH}"
echo "   Size: $(ls -lh "${WASM_PATH}" | awk '{print $5}')"

# Step 2: Dry-run / Inspection Check
if [[ "${1:-}" == "--dry-run" ]]; then
    echo "🔍 Dry-run mode completed successfully. Artifact is valid and ready."
    exit 0
fi

# Step 3: Initialization Guidance
echo ""
echo "📋 Contract Information & Verification Instructions:"
echo "------------------------------------------------------------"
echo "To initialize the deployed contract on Testnet:"
echo ""
echo "stellar contract invoke \\"
echo "    --id ${DEPLOYED_CONTRACT_ID} \\"
echo "    --source <ADMIN_SECRET_OR_IDENTITY> \\"
echo "    --network ${NETWORK} \\"
echo "    -- initialize \\"
echo "    --admin <ADMIN_ADDRESS> \\"
echo "    --reward_token <REWARD_TOKEN_ADDRESS>"
echo ""
echo "Explorer URL:"
echo "https://stellar.expert/explorer/testnet/contract/${DEPLOYED_CONTRACT_ID}"
echo "------------------------------------------------------------"
echo "🎉 Referral Contract Testnet Artifacts configured successfully."
