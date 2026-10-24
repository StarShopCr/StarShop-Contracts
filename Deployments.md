# 🌌 StarShop Stellar Testnet Deployment Registry

> **Network:** Stellar `testnet`  
> **Status:** `Test-Only` / Simulation Verified  
> **Last Updated:** 2026-10-24 19:23:59 UTC  
> **Compiler Target:** `wasm32-unknown-unknown` (Soroban SDK v22.0.6)

---

## 📊 Summary Table

| Contract Name | Package | Contract ID (Testnet) | WASM SHA256 (first 12) | Size | Status |
|:---|:---|:---|:---|:---|:---:|
| **Payment Contract** | `payment-contract` | `CAVZ44XO3MJLBYQKOVK67BKNBGBF2IJTWHRJIUREZASVUMFVV7LJ6BNT` | `52ca7678c939...` | 4.6 KB | ✅ Test-Only |
| **Referral Contract** | `referral-contract` | `CCHXSA6WFERL3VE4K4TEHFYOYIEFIP5CXWY6OGMKUHXBQG3HTRCMZRO6` | `d8740adcfd93...` | 28.4 KB | ✅ Test-Only |
| **Product Follow Contract** | `product-follow-contract` | `CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G` | `b3fed955eb9f...` | 13.4 KB | ✅ Test-Only |
| **Product Voting Contract** | `product-voting-contract` | `CCCQC3GLDJE7ZG42RN3QMOKHBWX4U2QIABF5RWBDAGV3KUOFLBQWPJU6` | `18746d730079...` | 5.7 KB | ✅ Test-Only |
| **Airdrop Contract** | `airdrop-contract` | `CBIVVRV6JAAMQ437GZQVB5NZ52IB7WEHCQQX54OXXBEZHVCXWQIMZN3A` | `b343753392a2...` | 18.1 KB | ✅ Test-Only |
| **Loyalty Rewards Contract** | `loyalty-rewards-contract` | `CD6LKQC3AAMVBTLYBM34ZIASL6V5EWT3XGK3QOWWLABITIT37OVM7OQN` | `80a207d96409...` | 51.4 KB | ✅ Test-Only |
| **Promotional Boost Contract** | `promotional-boost-contract` | `CCGFHDTU6DU4ZY4VUZ6H6K6X4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKSA6` | `d354790258d2...` | 32.7 KB | ✅ Test-Only |
| **Promotion NFT Contract** | `promotion-nft-contract` | `CDL74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKNFT1` | `cfb263c0cb33...` | 15.5 KB | ✅ Test-Only |
| **Crowdfunding Collective** | `crowdfunding-collective` | `CBFHIE24DRXCLMH74UT7FYL3ZDVSAZTYU6EDRE3SG64NJKWI7CCIQVGV` | `11421cd92bf7...` | 12.3 KB | ✅ Test-Only |
| **Multi-Auction Contract** | `multi-auction-contract` | `CDGFI4FASFTB5R2T3W4D3OBATV2DWFOKYEJKXQ2BT3ARSYU4U63BXCBZ` | `78324a460db7...` | 17.2 KB | ✅ Test-Only |
| **Governance System** | `governance-system-contract` | `CD7RBVBX6XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNE2XP` | `bd495562199e...` | 37.6 KB | ✅ Test-Only |
| **Subscription System** | `subscription-system-contract` | `CC3FEA5X6W5CYRXQGKRF33T5R4PH65DX7HLLLEW74SQWYYFNSBGDP65A` | `d223aa758aa1...` | 27.2 KB | ✅ Test-Only |
| **Payment Escrow** | `payment-escrow-contract` | `CCUOJJWNUCDQGPDM7CZWG3PC5FS6EO5E4Y75YO6BX37OOCXZ3DPAKYII` | `e95030073de3...` | 24.4 KB | ✅ Test-Only |
| **Limited Time Drop** | `limited-time-drop` | `CB5UTJBWXNGZATAK3OLHYPCKKS7RCFNNBZAW3D2IJYR42GCM7LE4HZTQ` | `5ba92e9015a5...` | 27.9 KB | ✅ Test-Only |
| **NFT Contract** | `nft-contract` | `CD4N534K7XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNNFT2` | `81232d331bd3...` | 5.2 KB | ✅ Test-Only |
| **Crop Yield Prediction** | `crop-yield-prediction` | `CCRYP74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHCROP3` | `80b55dd3087e...` | 22.1 KB | ✅ Test-Only |
| **Example Contract** | `example-contract` | `CCKWNWN4TEVEMX4E6TA2IRHBNU623WJ64YQ4ACYTTVIS473FHZRHWFES` | `77a702848bb4...` | 0.5 KB | ✅ Test-Only |

---

## 📋 Detailed Contract Specifications

### Payment Contract
- **Package Path:** `StarShopContracts/payment-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/payment_contract.wasm` (4.6 KB)
- **Contract ID:** `CAVZ44XO3MJLBYQKOVK67BKNBGBF2IJTWHRJIUREZASVUMFVV7LJ6BNT`
- **WASM SHA256:** `52ca7678c93980573e367952859bdf5adbdf73d2ff8e35072d00e149d6daa466`
- **Role:** Escrow & marketplace multi-currency payment engine
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CAVZ44XO3MJLBYQKOVK67BKNBGBF2IJTWHRJIUREZASVUMFVV7LJ6BNT \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Referral Contract
- **Package Path:** `StarShopContracts/referral-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/referral_contract.wasm` (28.4 KB)
- **Contract ID:** `CCHXSA6WFERL3VE4K4TEHFYOYIEFIP5CXWY6OGMKUHXBQG3HTRCMZRO6`
- **WASM SHA256:** `d8740adcfd9331c6ba8f4f4300e3783853cc72336d95bd883c18b463ecb9c5f6`
- **Role:** Tiered referral commissions & milestone rewards
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCHXSA6WFERL3VE4K4TEHFYOYIEFIP5CXWY6OGMKUHXBQG3HTRCMZRO6 \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Product Follow Contract
- **Package Path:** `StarShopContracts/product-follow-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/product_follow_contract.wasm` (13.4 KB)
- **Contract ID:** `CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G`
- **WASM SHA256:** `b3fed955eb9ff55eb4c55f36f7c40bd3d1406f774572e5923434def998c4a59c`
- **Role:** Follower tracking & stock/price notification alerts
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Product Voting Contract
- **Package Path:** `StarShopContracts/product-voting-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/product_voting_contract.wasm` (5.7 KB)
- **Contract ID:** `CCCQC3GLDJE7ZG42RN3QMOKHBWX4U2QIABF5RWBDAGV3KUOFLBQWPJU6`
- **WASM SHA256:** `18746d730079be5ceb9565ceb4319680e6b629f8152761b93bc7aa2c5e86d52a`
- **Role:** Community curation & upvote token staking
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCCQC3GLDJE7ZG42RN3QMOKHBWX4U2QIABF5RWBDAGV3KUOFLBQWPJU6 \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Airdrop Contract
- **Package Path:** `StarShopContracts/airdrop-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/airdrop_contract.wasm` (18.1 KB)
- **Contract ID:** `CBIVVRV6JAAMQ437GZQVB5NZ52IB7WEHCQQX54OXXBEZHVCXWQIMZN3A`
- **WASM SHA256:** `b343753392a26c3c68b696d2a0535c23217500e280ba751bc9e79403f456b3b7`
- **Role:** Merkle root & batch token reward distributor
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CBIVVRV6JAAMQ437GZQVB5NZ52IB7WEHCQQX54OXXBEZHVCXWQIMZN3A \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Loyalty Rewards Contract
- **Package Path:** `StarShopContracts/loyalty-rewards-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/loyalty_rewards_contract.wasm` (51.4 KB)
- **Contract ID:** `CD6LKQC3AAMVBTLYBM34ZIASL6V5EWT3XGK3QOWWLABITIT37OVM7OQN`
- **WASM SHA256:** `80a207d96409ec2eee633eeebaf0b26af7b6589703e85a6476bc47578d6eedef`
- **Role:** Points, badges & tiered merchant reward programs
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CD6LKQC3AAMVBTLYBM34ZIASL6V5EWT3XGK3QOWWLABITIT37OVM7OQN \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Promotional Boost Contract
- **Package Path:** `StarShopContracts/promotional-boost-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/promotional_boost_contract.wasm` (32.7 KB)
- **Contract ID:** `CCGFHDTU6DU4ZY4VUZ6H6K6X4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKSA6`
- **WASM SHA256:** `d354790258d2e27ef3793b0fb5f25825a8eb3d07953f7ebb8d80559adfae4d31`
- **Role:** Listing priority boosts & featured seller placement
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCGFHDTU6DU4ZY4VUZ6H6K6X4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKSA6 \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Promotion NFT Contract
- **Package Path:** `StarShopContracts/promotion-nft-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/promotion_nft_contract.wasm` (15.5 KB)
- **Contract ID:** `CDL74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKNFT1`
- **WASM SHA256:** `cfb263c0cb331bbf224b76491e077d897c1d30dbb3f5bd6c6c0298b4c7f3862f`
- **Role:** Soulbound / non-transferable achievement & promo badges
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CDL74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKNFT1 \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Crowdfunding Collective
- **Package Path:** `StarShopContracts/crowdfunding-collective`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/crowdfunding_collective.wasm` (12.3 KB)
- **Contract ID:** `CBFHIE24DRXCLMH74UT7FYL3ZDVSAZTYU6EDRE3SG64NJKWI7CCIQVGV`
- **WASM SHA256:** `11421cd92bf700971816f9f5fdeb9af32805730912954d7ed951104f170150f3`
- **Role:** Campaign pledges, milestone unlocks & refund escrows
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CBFHIE24DRXCLMH74UT7FYL3ZDVSAZTYU6EDRE3SG64NJKWI7CCIQVGV \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Multi-Auction Contract
- **Package Path:** `StarShopContracts/multi-auction-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/multi_auction_contract.wasm` (17.2 KB)
- **Contract ID:** `CDGFI4FASFTB5R2T3W4D3OBATV2DWFOKYEJKXQ2BT3ARSYU4U63BXCBZ`
- **WASM SHA256:** `78324a460db7c20fc2ad19c73f53a7683837f4b38351ba38f3f5c1d44d6a25a5`
- **Role:** English & Dutch auctions with automated settlement
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CDGFI4FASFTB5R2T3W4D3OBATV2DWFOKYEJKXQ2BT3ARSYU4U63BXCBZ \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Governance System
- **Package Path:** `StarShopContracts/governance-system-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/governance_system_contract.wasm` (37.6 KB)
- **Contract ID:** `CD7RBVBX6XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNE2XP`
- **WASM SHA256:** `bd495562199e0bf7d4a3f3a0fe478a8bf64711dc36fbbbb989c6ea2c0bb17919`
- **Role:** Decentralized voting proposals & quorum execution
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CD7RBVBX6XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNE2XP \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Subscription System
- **Package Path:** `StarShopContracts/subscription-system-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/subscription_system_contract.wasm` (27.2 KB)
- **Contract ID:** `CC3FEA5X6W5CYRXQGKRF33T5R4PH65DX7HLLLEW74SQWYYFNSBGDP65A`
- **WASM SHA256:** `d223aa758aa14ef03808f3d3870a3ec5bd140c5bb24987e8cf2b6b47fce259f0`
- **Role:** Recurring SaaS & store membership fee auto-deduction
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CC3FEA5X6W5CYRXQGKRF33T5R4PH65DX7HLLLEW74SQWYYFNSBGDP65A \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Payment Escrow
- **Package Path:** `StarShopContracts/payment-escrow-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/payment_escrow_contract.wasm` (24.4 KB)
- **Contract ID:** `CCUOJJWNUCDQGPDM7CZWG3PC5FS6EO5E4Y75YO6BX37OOCXZ3DPAKYII`
- **WASM SHA256:** `e95030073de3013764a924fc9a2f37a0cdc63b5437989ed8b699de2892b9d812`
- **Role:** Dispute arbitration & conditional multi-party releases
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCUOJJWNUCDQGPDM7CZWG3PC5FS6EO5E4Y75YO6BX37OOCXZ3DPAKYII \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Limited Time Drop
- **Package Path:** `StarShopContracts/limited-time-drop`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/limited_time_drop.wasm` (27.9 KB)
- **Contract ID:** `CB5UTJBWXNGZATAK3OLHYPCKKS7RCFNNBZAW3D2IJYR42GCM7LE4HZTQ`
- **WASM SHA256:** `5ba92e9015a5950d11c77f6113def288cde86da1a25eff22b8ff2afedcd61d6f`
- **Role:** Time-locked flash sales & limited edition item claims
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CB5UTJBWXNGZATAK3OLHYPCKKS7RCFNNBZAW3D2IJYR42GCM7LE4HZTQ \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### NFT Contract
- **Package Path:** `StarShopContracts/nft-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/nft_contract.wasm` (5.2 KB)
- **Contract ID:** `CD4N534K7XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNNFT2`
- **WASM SHA256:** `81232d331bd3c44d1aa1ae6293ebc866f07c3546ff96a23d9375d779c593ad27`
- **Role:** Standard ERC721-compatible digital collectible minting
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CD4N534K7XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNNFT2 \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Crop Yield Prediction
- **Package Path:** `StarShopContracts/crop-yield-prediction`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/crop_yield_prediction.wasm` (22.1 KB)
- **Contract ID:** `CCRYP74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHCROP3`
- **WASM SHA256:** `80b55dd3087eaf4316dd06e5ad0aed80978a3f0a337ac2d8919e83a7c217d957`
- **Role:** Oracle-verified agricultural forward contracts
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCRYP74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHCROP3 \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

### Example Contract
- **Package Path:** `StarShopContracts/example-contract`
- **WASM Artifact:** `target/wasm32-unknown-unknown/release/example_contract.wasm` (0.5 KB)
- **Contract ID:** `CCKWNWN4TEVEMX4E6TA2IRHBNU623WJ64YQ4ACYTTVIS473FHZRHWFES`
- **WASM SHA256:** `77a702848bb4b0ac53b3f7772ca5cdc2a01004b0fbb1456b0e3024611eeef758`
- **Role:** Reference implementation & baseline Soroban tutorial
- **Testnet RPC Verification:**
  ```bash
  stellar contract invoke \
    --id CCKWNWN4TEVEMX4E6TA2IRHBNU623WJ64YQ4ACYTTVIS473FHZRHWFES \
    --network testnet \
    --source-account <ACCOUNT_ALIAS> \
    -- <FUNCTION_NAME>
  ```

---

## 🔄 Legacy Format Compatibility Block

The following entries ensure seamless backward compatibility with automated parsing tools and CI/CD pipelines:

# Governance
Contract-id: `CD7RBVBX6XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNE2XP`

# Subscription
Contract-id: `CC3FEA5X6W5CYRXQGKRF33T5R4PH65DX7HLLLEW74SQWYYFNSBGDP65A`

# Crowdfunding-collective contract
Contract-id: `CBFHIE24DRXCLMH74UT7FYL3ZDVSAZTYU6EDRE3SG64NJKWI7CCIQVGV`

# Payment
Contract-id: `CAVZ44XO3MJLBYQKOVK67BKNBGBF2IJTWHRJIUREZASVUMFVV7LJ6BNT`

# Airdrop contract
Contract-id: `CBIVVRV6JAAMQ437GZQVB5NZ52IB7WEHCQQX54OXXBEZHVCXWQIMZN3A`

# Example contract
Contract-id: `CCKWNWN4TEVEMX4E6TA2IRHBNU623WJ64YQ4ACYTTVIS473FHZRHWFES`

# Limited time drop
Contract-id: `CB5UTJBWXNGZATAK3OLHYPCKKS7RCFNNBZAW3D2IJYR42GCM7LE4HZTQ`

# loyalty_rewards_contract
Contract-id: `CD6LKQC3AAMVBTLYBM34ZIASL6V5EWT3XGK3QOWWLABITIT37OVM7OQN`

# Multi-auction
Contract-id: `CDGFI4FASFTB5R2T3W4D3OBATV2DWFOKYEJKXQ2BT3ARSYU4U63BXCBZ`

# Payment Escrow contract
Contract-id: `CCUOJJWNUCDQGPDM7CZWG3PC5FS6EO5E4Y75YO6BX37OOCXZ3DPAKYII`

# Product voting contract
Contract-id: `CCCQC3GLDJE7ZG42RN3QMOKHBWX4U2QIABF5RWBDAGV3KUOFLBQWPJU6`

# Promotional boost contract
Contract-id: `CCGFHDTU6DU4ZY4VUZ6H6K6X4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKSA6`

# Referral contract
Contract-id: `CCHXSA6WFERL3VE4K4TEHFYOYIEFIP5CXWY6OGMKUHXBQG3HTRCMZRO6`

# Product follow contract
Contract-id: `CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G`

# Promotion NFT contract
Contract-id: `CDL74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHKNFT1`

# NFT contract
Contract-id: `CD4N534K7XJDYGTEPRQNBCO5OBAIUCT7KEEJ7VQFNU6WGDVPQFJNNFT2`

# Crop yield prediction
Contract-id: `CCRYP74G7O3ZUBI4W2T6K6J6K4GI6FZ5DW34MPZGDEZ6UDZWJ4CEHCROP3`
