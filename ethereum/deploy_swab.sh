eth abi:update uniswab artifacts/UniSwab.abi
uniswab=$(eth contract:deploy -n hardhat --pk hat2 artifacts/UniSwab.bin | jq -r .address)
echo uniswab ${uniswab}
echo ${uniswab} > logs/SWAB
eth address:add uniswab ${uniswab} > /dev/null
amount=340282366920938463463374607431768211455 # 2^32-1
echo hat2 approves uniswab for ${amount} usdona
eth contract:send --pk hat2 erc20@usdona 'approve("'${uniswab}'", "'${amount}'")'
echo hat2 approves uniswab for ${amount} usdonc
eth contract:send --pk hat2 erc20@usdonc 'approve("'${uniswab}'", "'${amount}'")'
