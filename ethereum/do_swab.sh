HAT2=0x70997970C51812dc3A010C7d01b50e0d17dc79C8


balances() {
COIN=`eth contract:call erc20@0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 'balanceOf("'${HAT2}'")'`
echo USDONA for HAT2 = $COIN
COIN=`eth contract:call erc20@0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0 'balanceOf("'${HAT2}'")'`
echo USDONC for HAT2 = $COIN

echo pool0 reserves
eth contract:call uniswap-v2-pair@0xE901fc34EC601a1447C41924BAAE04c00C514859 'getReserves()' | grep reserve
echo pool1 reserves
eth contract:call uniswap-v2-pair@0x288c08A30f12777D78C531c9E51b1BB45d00f792 'getReserves()' | grep reserve

COIN=`eth contract:call erc20@0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 'balanceOf("0xE901fc34EC601a1447C41924BAAE04c00C514859")'`
echo USDONA for pool0 = $COIN
COIN=`eth contract:call erc20@0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0 'balanceOf("0xE901fc34EC601a1447C41924BAAE04c00C514859")'`
echo USDONA for pool1 = $COIN
}

balances
# SWAB!
echo SWAB 55485
eth contract:send --pk hat2 uniswab@0x8464135c8F25Da09e49BC8782676a84730C318bC 'swab(55485, "0xE901fc34EC601a1447C41924BAAE04c00C514859", "0x288c08A30f12777D78C531c9E51b1BB45d00f792")'
balances
