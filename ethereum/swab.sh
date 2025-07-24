#!/usr/bin/zsh
HAT2=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
POOL0=$(cat logs/AC1)
POOL1=$(cat logs/AC2)
SWAB=$(cat logs/SWAB)

if [[ -z "$1" ]];
then
AY_IN=40371
else
AY_IN=$1
fi

balances() {
COIN=`eth contract:call erc20@usdona 'balanceOf("'${HAT2}'")'`
echo USDONA for HAT2 = $COIN
COIN=`eth contract:call erc20@usdonc 'balanceOf("'${HAT2}'")'`
echo USDONC for HAT2 = $COIN
}

reserves() {
echo pool0 ${POOL0} reserves
eth contract:call uniswap-v2-pair@${POOL0} 'getReserves()' | grep reserve
echo token0 `eth contract:call uniswap-v2-pair@${POOL0} 'token0()'`
echo token1 `eth contract:call uniswap-v2-pair@${POOL0} 'token1()'`
echo pool1 ${POOL1} reserves
eth contract:call uniswap-v2-pair@${POOL1} 'getReserves()' | grep reserve
echo token0 `eth contract:call uniswap-v2-pair@${POOL1} 'token0()'`
echo token1 `eth contract:call uniswap-v2-pair@${POOL1} 'token1()'`

COIN=`eth contract:call erc20@usdona 'balanceOf("'${POOL0}'")'`
echo USDONA for pool0 = $COIN
COIN=`eth contract:call erc20@usdonc 'balanceOf("'${POOL0}'")'`
echo USDONC for pool0 = $COIN
COIN=`eth contract:call erc20@usdona 'balanceOf("'${POOL1}'")'`
echo USDONA for pool1 = $COIN
COIN=`eth contract:call erc20@usdonc 'balanceOf("'${POOL1}'")'`
echo USDONC for pool1 = $COIN
}

STARTC=`eth contract:call erc20@usdonc 'balanceOf("'${HAT2}'")'`
balances
# SWAB!
echo swab\(${AY_IN}, ${POOL0}, ${POOL1}\)
eth contract:send --pk hat2 uniswab@${SWAB} 'swab('${AY_IN}', "'${POOL0}'", "'${POOL1}'")'
balances
reserves
ENDC=`eth contract:call erc20@usdonc 'balanceOf("'${HAT2}'")'`
echo ay_in ${AY_IN} profit `calc "${ENDC} - ${STARTC}"`

