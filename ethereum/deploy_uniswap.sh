artifacts_dir=artifacts
hat1=$(eth address:show hat1 | jq -r '.["*"].address')
echo hat1: ${hat1}
hat2=$(eth address:show hat2 | jq -r '.["*"].address')
echo hat2: ${hat2}

echo #################################################
# WETH deploy
WETH=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/WETH9.bin | jq -r .address)
echo WETH: ${WETH}
echo ${WETH} > logs/WETH
# ERC20 deploy
USDONA1=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonA.bin | jq -r .address)
echo USDONA1: ${USDONA1} '(ignored)'
USDONA=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonA.bin | jq -r .address)
echo USDONA: ${USDONA}
echo ${USDONA} > logs/USDONA
eth address:add usdona ${USDONA} > /dev/null
USDONC=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonC.bin | jq -r .address)
echo USDONC: ${USDONC} 
echo ${USDONC} > logs/USDONC
eth address:add usdonc ${USDONC} > /dev/null
echo updating abi in eth-cli
eth abi:update dpcoin ./${artifacts_dir}/UsDonA.abi  


mint() {
echo minting $2 USDONA@${USDONA} to $1
eth contract:send --pk hat1 dpcoin@${USDONA} 'mint("'$1'", "'$2'" )'
echo minting $2 USDONC@${USDONC} to $1
eth contract:send --pk hat1 dpcoin@${USDONC} 'mint("'$1'", "'$2'" )'
}

mint $hat1 1105340744425943157500
mint $hat2 1105340744425943157500

pool() {
echo #################################################
# UNISWAP FACTORY deploy
FACTORY=$(eth contract:deploy -n hardhat --pk hat1 --abi uniswap-v2-factory --args [\"${hat1}\"]  ./artifacts/UniswapV2Factory.bin | jq -r .address )
echo hat1 deployed uniswap v2 FACTORY ${FACTORY}

# UNISWAP A/C POOL #1 deploy
#POOL=$(eth contract:deploy -n hardhat --pk hat1 ./artifacts/UniswapV2Pair.bin | jq -r .address )
eth contract:send --pk hat1 uniswap-v2-factory@${FACTORY} 'createPair("'${USDONA}'","'${USDONC}'")' 
POOL=$(eth contract:call uniswap-v2-factory@${FACTORY} 'getPair("'${USDONA}'","'${USDONC}'")' )
echo hat1 deployed uniswap v2 POOL $3 ${POOL}
echo ${POOL} > logs/"$3"

# Add USDONA
echo hat1 USDA balance `eth contract:call dpcoin@${USDONA} 'balanceOf("'${hat1}'")'`
echo transfer $1 USDONA to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONA} 'transfer("'${POOL}'", "'$1'")'
echo USDA balance for pool `eth contract:call dpcoin@${USDONA} 'balanceOf("'${POOL}'")'`
echo transfer $2 USDONC to ${POOL}
eth contract:send --pk hat1 dpcoin@${USDONC} 'transfer("'${POOL}'", "'$2'")'
echo USDC balance for pool `eth contract:call dpcoin@${USDONC} 'balanceOf("'${POOL}'")'`
echo pool mint to ${hat1}
eth contract:send --pk hat1 uniswap-v2-pair@${POOL} 'mint("'${hat1}'")'
eth contract:call uniswap-v2-pair@${POOL} 'getReserves()' | grep reserve
}

poolparty() {
pool $1 $2 AC1
pool $3 $4 AC2 
}

#poolparty 310000 210000 220000 320000 # prices 0.667, 1.454
poolparty 11053407444259431575 39896272204 96794408650081838290 354112185748
