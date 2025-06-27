uniswab=$(eth contract:deploy -n hardhat --pk hat2 artifacts/UniSwab.bin | jq -r .address)
echo uniswab ${uniswab}
eth address:add uniswab ${uniswab} > /dev/null
echo approve uniswab for 30000
eth contract:send --pk hat2 erc20@usdonc 'approve("'${uniswab}'", 30000)'
