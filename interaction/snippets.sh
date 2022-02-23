WALLET="./wallets/wallet.pem" # PEM path
ADDRESS=$(erdpy data load --key=address-devnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D
# PROXY=https://testnet-gateway.elrond.com
# CHAIN_ID=T

CONTRACT="output/elrond-nftmanager.wasm"
PAYMENT_TOKEN_ID="SVEN-4b35b0"
NFT_TOKEN_PRICE=1000000000000000
ROYALTIES=300
BASE_URI="https://ipfs.io/ipfs/QmXSFnUfdot3SgLsuZFdpefXii31YuyvtAD23NKdz9toar"

PAYMENT_TOKEN_ID_HEX="0x$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
BASE_URI_HEX="0x$(echo -n ${BASE_URI} | xxd -p -u | tr -d '\n')"

deploy() {
    erdpy --verbose contract deploy \
    --bytecode="$CONTRACT"  \
    --recall-nonce \
    --pem=${WALLET} \
    --gas-limit=50000000 \
    --send --outfile="deploy-devnet.interaction.json" \
    --proxy="${PROXY}" \
    --arguments ${PAYMENT_TOKEN_ID_HEX} ${NFT_TOKEN_PRICE} ${TOKEN_PRICE} ${ROYALTIES} ${BASE_URI_HEX} \
    --metadata-payable  \
    --metadata-payable-by-sc \
    --chain=${CHAIN_ID} || return

    # TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
    # ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")

    # erdpy data store --key=address-devnet --value=${ADDRESS}
    # erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    # echo ""
    # echo "Smart contract address: ${ADDRESS}"
}

# issueNft() {
#     local TOKEN_DISPLAY_NAME=0x4d79546573744e667464  # "MyTestNft"
#     local TOKEN_TICKER=0x544553544e4654  # "TESTNFT"

#     erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
#     --gas-limit=100000000 --value=50000000000000000 --function="issueToken" \
#     --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} \
#     --send --proxy=${PROXY} --chain=${CHAIN_ID}
# }

# setLocalRoles() {
#     erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
#     --gas-limit=100000000 --function="setLocalRoles" \
#     --send --proxy=${PROXY} --chain=${CHAIN_ID}
# }

# createNft() {
#     local TOKEN_NAME=0x4e616d65 # "Name"
#     local ROYALTIES=1000 # 10%
#     local URI=0x72616e647572692e636f6d # randuri.com
#     local SELLING_PRICE=0

#     erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
#     --gas-limit=50000000 --function="createNft" \
#     --arguments ${TOKEN_NAME} ${ROYALTIES} ${URI} ${SELLING_PRICE} \
#     --send --proxy=${PROXY} --chain=${CHAIN_ID}
# }

# buyNft() {
#     local NFT_NONCE=1

#     erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
#     --gas-limit=10000000 --function="buyNft" \
#     --arguments ${NFT_NONCE} \
#     --send --proxy=${PROXY} --chain=${CHAIN_ID}
# }
