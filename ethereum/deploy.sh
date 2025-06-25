artifacts_dir=artifacts
owner=$(eth address:show hat1 | jq -r '.["*"].address')
echo owner: ${owner}

echo #################################################
UNISWAB=$(eth contract:deploy -n hardhat --pk hat2  ./${artifacts_dir}/UniSwab.bin | jq -r .address)
echo UNISWAB: ${UNISWAB}
exit

echo #################################################
# ERC20 deploy
USDONA=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonA.bin | jq -r .address)
echo USDONA: ${USDONA}
USDONC=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonC.bin | jq -r .address)
echo USDONC: ${USDONC} 
echo updating abi in eth-cli
eth abi:update dpcoin ./${artifacts_dir}/UsDonA.abi  
echo minting 1000000 USDONA@${USDONA} to owner
eth contract:send --pk hat1 dpcoin@${USDONA} 'mint("'${owner}'", 1000000 )'
echo minting 1000000 USDONC@${USDONC} to owner
eth contract:send --pk hat1 dpcoin@${USDONC} 'mint("'${owner}'", 1000000 )'


pool() {
echo #################################################
# UNISWAP FACTORY deploy
FACTORY=$(eth contract:deploy -n hardhat --pk hat1 --abi uniswap-v2-factory --args [\"${owner}\"]  ./artifacts/UniswapV2Factory.bin | jq -r .address )
echo uniswap v2 FACTORY ${FACTORY}

# UNISWAP A/C POOL #1 deploy
#POOL=$(eth contract:deploy -n hardhat --pk hat1 ./artifacts/UniswapV2Pair.bin | jq -r .address )
eth contract:send --pk hat1 uniswap-v2-factory@${FACTORY} 'createPair("'${USDONA}'","'${USDONC}'")' 
POOL=$(eth contract:call uniswap-v2-factory@${FACTORY} 'getPair("'${USDONA}'","'${USDONC}'")' )
echo uniswap v2 DEPLOYED A/C \#1 ${POOL}
#echo uniswap v2 initialitze\(${USDONA}, ${USDONC}\)
#eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'initialize("'${USDONA}'","'${USDONC}'")'

# Add USDONA
echo owner USDA balance `eth contract:call dpcoin@${USDONA} 'balanceOf("'${owner}'")'`
echo transfer $1 USDONA to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONA} 'transfer("'${POOL}'", '$1')'
echo USDA balance for pool `eth contract:call dpcoin@${USDONA} 'balanceOf("'${POOL}'")'`
echo transfer $2 USDONC to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONC} 'transfer("'${POOL}'", '$2')'
echo USDC balance for pool `eth contract:call dpcoin@${USDONC} 'balanceOf("'${POOL}'")'`
echo pool mint to ${owner}
eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'mint("'${owner}'")'
eth contract:call uniswap-v2-pair@${POOL} 'getReserves()'

}

pool 210000 310000
pool 320000 220000

