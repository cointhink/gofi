//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "v2-core-1.0.1/contracts/interfaces/IUniswapV2Pair.sol";
import "openzeppelin-contracts-5.3.0/contracts/token/ERC20/ERC20.sol";
import "console.sol";

contract UniSwab {
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner can call this");
        _;
    }

    struct PoolState {
        address addr;
        uint112 reserve0;
        uint112 reserve1;
    }

    function swab(
        uint256 amount1In,
        PoolState calldata pool0,
        PoolState calldata pool1
    ) public onlyOwner {
        console.log("amount1In", amount1In);
        IUniswapV2Pair cpool0 = IUniswapV2Pair(pool0.addr);
        IUniswapV2Pair cpool1 = IUniswapV2Pair(pool1.addr);
        (uint112 _reserve00, uint112 _reserve01, ) = cpool0.getReserves();
        require(pool0.reserve0 == _reserve00, "pool0_token0 OLD");
        require(pool0.reserve1 == _reserve01, "pool0_token1 OLD");
        (uint112 _reserve10, uint112 _reserve11, ) = cpool1.getReserves();
        require(pool1.reserve0 == _reserve10, "pool1_token0 OLD");
        require(pool1.reserve1 == _reserve11, "pool1_token1 OLD");


        // Step 1
        //  transferFrom(sender, recipient, amount)
        ERC20(cpool0.token1()).transferFrom(msg.sender, pool0.addr, amount1In);
        uint256 amount0Out = getAmountOut(amount1In, _reserve01, _reserve00);
        // function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
        cpool0.swap(amount0Out, 0, owner, new bytes(0));

        // Step 2
        ERC20(cpool0.token0()).transferFrom(msg.sender, pool1.addr, amount0Out);
        uint256 amount1Out = getAmountOut(amount0Out, _reserve10, _reserve11);
        cpool1.swap(0, amount1Out, owner, new bytes(0));
        require(amount1Out > amount0Out, "UniSwab: no profit");
    }

    // given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    function getAmountOut(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut
    ) internal pure returns (uint256 amountOut) {
        require(amountIn > 0, "GAO: INSUFFICIENT_INPUT_AMOUNT");
        require(reserveIn > 0 && reserveOut > 0);
        uint256 amountInWithFee = amountIn * 997;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = (reserveIn * 1000) + amountInWithFee;
        amountOut = numerator / denominator;
    }

    // given an output amount of an asset and pair reserves, returns a required input amount of the other asset
    function getAmountIn(
        uint256 amountOut,
        uint256 reserveIn,
        uint256 reserveOut
    ) internal pure returns (uint256 amountIn) {
        require(amountOut > 0, "GAI: INSUFFICIENT_OUTPUT_AMOUNT");
        require(reserveIn > 0 && reserveOut > 0);
        uint256 numerator = reserveIn * amountOut * 1000;
        uint256 denominator = (reserveOut - amountOut) * 997;
        amountIn = (numerator / denominator) + 1;
    }
}
