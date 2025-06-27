uniswab=$(eth contract:deploy -n hardhat --pk hat2 artifacts/UniSwab.bin | jq -r .address)
echo uniswab ${uniswab}
eth address:add uniswab ${uniswab} > /dev/null
amount=50000
echo hat2 approves uniswab for ${amount} usdona
eth contract:send --pk hat2 erc20@usdona 'approve("'${uniswab}'", '${amount}')'
echo hat2 approves uniswab for ${amount} usdonc
eth contract:send --pk hat2 erc20@usdonc 'approve("'${uniswab}'", '${amount}')'
