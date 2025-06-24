artifacts_dir=artifacts
owner=$(eth address:show hat1 | jq -r '.["*"].address')
echo owner: ${owner}

USDONA=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonA.bin | jq -r .address)
echo USDONA: ${USDONA}
USDONC=$(eth contract:deploy -n hardhat --pk hat1  ./${artifacts_dir}/UsDonC.bin | jq -r .address)
echo USDONC: ${USDONC} 
echo updating abi in eth-cli
eth abi:update dpcoin ./${artifacts_dir}/UsDonA.abi  
echo minting 100 USDONA@${USDONA} to owner
eth contract:send --pk hat1 dpcoin@${USDONA} 'mint("'${owner}'", 100 )'
echo minting 1000 USDONC@${USDONC} to owner
eth contract:send --pk hat1 dpcoin@${USDONC} 'mint("'${owner}'", 1000 )'
