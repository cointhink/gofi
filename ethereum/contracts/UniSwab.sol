//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "v2-core-1.0.1/contracts/interfaces/IUniswapV2Pair.sol";
import "openzeppelin-contracts-5.3.0/contracts/token/ERC20/utils/SafeERC20.sol";

contract UniSwab {
    using SafeERC20 for IERC20;
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner can call this");
        _;
    }

    function _swap1to0(
        uint256 amount1In,
        IUniswapV2Pair pool
    ) internal onlyOwner returns (uint256) {
        IERC20(pool.token1()).safeTransferFrom(msg.sender, address(pool), amount1In);
        (uint112 reserve0, uint112 reserve1, ) = pool.getReserves();
        uint256 amount0Out = getAmountOut(amount1In, reserve1, reserve0);
        pool.swap(amount0Out, 0, owner, new bytes(0));
        return amount0Out;
    }

    function _swap0to1(
        uint256 amount0In,
        IUniswapV2Pair pool
    ) internal onlyOwner returns (uint256) {
        IERC20(pool.token0()).safeTransferFrom(msg.sender, address(pool), amount0In);
        (uint112 reserve0, uint112 reserve1, ) = pool.getReserves();
        uint256 amount1Out = getAmountOut(amount0In, reserve0, reserve1);
        pool.swap(0, amount1Out, owner, new bytes(0));
        return amount1Out;
    }

    function swab(
        uint256 amount1In,
        address pool0_addr,
        address pool1_addr
    ) public onlyOwner {
        // Step 1
        uint256 amount0Out = _swap1to0(amount1In, IUniswapV2Pair(pool0_addr));

        // Step 2
        uint256 amount1Out = _swap0to1(amount0Out, IUniswapV2Pair(pool1_addr));
        require(amount1Out > amount1In, "UniSwab: no profit");
    }

    // given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    function getAmountOut(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut
    ) internal pure returns (uint256 amountOut) {
        require(amountIn > 0, "GAO: INSUFFICIENT_INPUT_AMOUNT");
        require(reserveIn > 0 && reserveOut > 0, "GAO: INSUFFICIENT_LIQUIDITY");
        uint256 amountInWithFee = amountIn * 997;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = (reserveIn * 1000) + amountInWithFee;
        amountOut = numerator / denominator;
    }
}
