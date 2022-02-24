WALLET="./wallets/wallet.pem" # PEM path
ADDRESS=$(erdpy data load --key=address-devnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-devnet)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D
# PROXY=https://testnet-gateway.elrond.com
# CHAIN_ID=T

CONTRACT="output/elrond-nftmanager.wasm"
PAYMENT_TOKEN_ID="EGLD"
NFT_TOKEN_PRICE=1000000000000000
ROYALTIES=300
IMAGE_BASE_URI="https://ipfs.io/ipfs/QmXSFnUfdot3SgLsuZFdpefXii31YuyvtAD23NKdz9toar"
METADATA_BASE_URI="https://ipfs.io/ipfs/QmS1Zn9ytigCjkNtQPAduFuzptadNw2kC9asptoYq9ZBwS"

PAYMENT_TOKEN_ID_HEX="0x$(echo -n ${PAYMENT_TOKEN_ID} | xxd -p -u | tr -d '\n')"
IMAGE_BASE_URI_HEX="0x$(echo -n ${IMAGE_BASE_URI} | xxd -p -u | tr -d '\n')"
METADATA_BASE_URI_HEX="0x$(echo -n ${METADATA_BASE_URI} | xxd -p -u | tr -d '\n')"

deploy() {
    erdpy --verbose contract deploy \
    --bytecode="$CONTRACT"  \
    --recall-nonce \
    --pem=${WALLET} \
    --gas-limit=100000000 \
    --send --outfile="deploy-devnet.interaction.json" \
    --proxy="${PROXY}" \
    --arguments ${PAYMENT_TOKEN_ID_HEX} ${NFT_TOKEN_PRICE} ${TOKEN_PRICE} ${ROYALTIES} ${IMAGE_BASE_URI_HEX} ${METADATA_BASE_URI_HEX} \
    --metadata-payable  \
    --metadata-payable-by-sc \
    --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-devnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-devnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-devnet --value=${TRANSACTION}

    # echo ""
    echo "Smart contract address: ${ADDRESS}"
}

issueNft() {
    local TOKEN_DISPLAY_NAME=0x5376656e4e4654  # "SvenNFT"
    local TOKEN_TICKER=0x5356454e4e4654  # "SVENNFT"

    erdpy --verbose contract call ${ADDRESS} \
    --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 --value=50000000000000000 \
    --function="issueNft" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRoles() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${WALLET} \
    --gas-limit=100000000 --function="setLocalRoles" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

startMinting() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${WALLET} \
    --gas-limit=10000000 --function="startMinting" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

mintNft() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${WALLET} \
    --gas-limit=10000000 --function="mint" \
    --value 1000000000000000 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
