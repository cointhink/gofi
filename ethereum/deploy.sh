artifacts_dir=artifacts
owner=$(eth address:show hat1 | jq -r '.["*"].address')
echo owner: ${owner}

# ERC20 deploy
USDONA=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonA.bin | jq -r .address)
echo USDONA: ${USDONA}
USDONC=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonC.bin | jq -r .address)
echo USDONC: ${USDONC} 
echo updating abi in eth-cli
eth abi:update dpcoin ./${artifacts_dir}/UsDonA.abi  
echo minting  100 USDONA@${USDONA} to owner
eth contract:send --pk hat1 dpcoin@${USDONA} 'mint("'${owner}'", 100 )'
echo minting 1000 USDONC@${USDONC} to owner
eth contract:send --pk hat1 dpcoin@${USDONC} 'mint("'${owner}'", 1000 )'

# UNISWAP POOL deploy
POOL=$( eth contract:deploy -n hardhat --pk hat1 ./artifacts/UniswapV2Pair.bin | jq -r .address )
echo uniswap v2 DEPLOYED ${POOL}
echo uniswap v2 initialitze\(${USDONA}, ${USDONC}\)
eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'initialize("'${USDONA}'","'${USDONC}'")'

# Add USDONA
echo owner USDA balance `eth contract:call dpcoin@${USDONA} 'balanceOf("'${owner}'")'`
echo transfer 2 USDONA to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONA} 'transfer("'${POOL}'", 2 )'
echo pool USDA balance `eth contract:call uniswap-v2-pair@${POOL} 'balanceOf("'${owner}'")'`
echo transfer 3 USDONC to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONC} 'transfer("'${POOL}'", 3 )'
echo pool mint to ${owner}
eth contract:call uniswap-v2-pair@${POOL} 'getReserves()'
eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'mint("'${owner}'")'
