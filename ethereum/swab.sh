HAT2=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
POOL0=$(cat logs/AC1)
POOL1=$(cat logs/AC2)
SWAB=$(cat logs/SWAB)


balances() {
COIN=`eth contract:call erc20@usdona 'balanceOf("'${HAT2}'")'`
echo USDONA for HAT2 = $COIN
COIN=`eth contract:call erc20@usdonc 'balanceOf("'${HAT2}'")'`
echo USDONC for HAT2 = $COIN

echo pool0 ${POOL0} reserves
eth contract:call uniswap-v2-pair@${POOL0} 'getReserves()' | grep reserve
echo pool1 ${POOL1} reserves
eth contract:call uniswap-v2-pair@${POOL1} 'getReserves()' | grep reserve

COIN=`eth contract:call erc20@usdona 'balanceOf("'${POOL0}'")'`
echo USDONA for pool0 = $COIN
COIN=`eth contract:call erc20@usdonc 'balanceOf("'${POOL1}'")'`
echo USDONA for pool1 = $COIN
}

balances
# SWAB!
echo SWAB 34485 #55485
eth contract:send --pk hat2 uniswab@${SWAB} 'swab(55485, "'${POOL0}'", "'${POOL1}'")'
balances
