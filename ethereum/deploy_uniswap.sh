artifacts_dir=artifacts
hat1=$(eth address:show hat1 | jq -r '.["*"].address')
echo hat1: ${hat1}
hat2=$(eth address:show hat2 | jq -r '.["*"].address')
echo hat2: ${hat2}

echo #################################################
# ERC20 deploy
USDONA=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonA.bin | jq -r .address)
echo USDONA: ${USDONA}
eth address:add usdona ${USDONA}
USDONC=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonC.bin | jq -r .address)
echo USDONC: ${USDONC} 
eth address:add usdonc ${USDONC}
echo updating abi in eth-cli
eth abi:update dpcoin ./${artifacts_dir}/UsDonA.abi  


mint() {
echo minting $2 USDONA@${USDONA} to $1
eth contract:send --pk hat1 dpcoin@${USDONA} 'mint("'$1'", '$2' )'
echo minting $2 USDONC@${USDONC} to $1
eth contract:send --pk hat1 dpcoin@${USDONC} 'mint("'$1'", '$2' )'
}

mint $hat1 1000000
mint $hat2 1000000

pool() {
echo #################################################
# UNISWAP FACTORY deploy
FACTORY=$(eth contract:deploy -n hardhat --pk hat1 --abi uniswap-v2-factory --args [\"${hat1}\"]  ./artifacts/UniswapV2Factory.bin | jq -r .address )
echo uniswap v2 FACTORY ${FACTORY}

# UNISWAP A/C POOL #1 deploy
#POOL=$(eth contract:deploy -n hardhat --pk hat1 ./artifacts/UniswapV2Pair.bin | jq -r .address )
eth contract:send --pk hat1 uniswap-v2-factory@${FACTORY} 'createPair("'${USDONA}'","'${USDONC}'")' 
POOL=$(eth contract:call uniswap-v2-factory@${FACTORY} 'getPair("'${USDONA}'","'${USDONC}'")' )
echo uniswap v2 DEPLOYED A/C \#1 ${POOL}
#echo uniswap v2 initialitze\(${USDONA}, ${USDONC}\)
#eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'initialize("'${USDONA}'","'${USDONC}'")'

# Add USDONA
echo hat1 USDA balance `eth contract:call dpcoin@${USDONA} 'balanceOf("'${hat1}'")'`
echo transfer $1 USDONA to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONA} 'transfer("'${POOL}'", '$1')'
echo USDA balance for pool `eth contract:call dpcoin@${USDONA} 'balanceOf("'${POOL}'")'`
echo transfer $2 USDONC to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONC} 'transfer("'${POOL}'", '$2')'
echo USDC balance for pool `eth contract:call dpcoin@${USDONC} 'balanceOf("'${POOL}'")'`
echo pool mint to ${hat1}
eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'mint("'${hat1}'")'
eth contract:call uniswap-v2-pair@${POOL} 'getReserves()'

}

pool 320000 220000
pool 210000 310000

