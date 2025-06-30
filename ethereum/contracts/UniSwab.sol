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

    function _swap(
        uint256 amount1In,
        ERC20 token1,
        IUniswapV2Pair pool,
        uint112 reserve0,
        uint112 reserve1
    ) internal onlyOwner returns (uint256) {
        token1.transferFrom(msg.sender, address(pool), amount1In);
        uint256 amount0Out = getAmountOut(amount1In, reserve1, reserve0);
        pool.swap(amount0Out, 0, owner, new bytes(0));
        return amount0Out;
    }

    function swab(
        uint256 amount1In,
        address pool0_addr,
        address pool1_addr
    ) public onlyOwner {
        // Step 1
        IUniswapV2Pair pool0 = IUniswapV2Pair(pool0_addr);
        (uint112 _reserve00, uint112 _reserve01, ) = pool0.getReserves();
        uint256 amount0Out = _swap(
            amount1In,
            ERC20(pool0.token1()),
            pool0,
            _reserve01,
            _reserve00
        );
        console.log("amount0Out", amount0Out);

        // Step 2
        IUniswapV2Pair pool1 = IUniswapV2Pair(pool1_addr);
        (uint112 _reserve10, uint112 _reserve11, ) = pool1.getReserves();
        uint256 amount1Out = _swap(
            amount0Out,
            ERC20(pool0.token1()),
            pool1,
            _reserve10,
            _reserve11
        );
        console.log("amount1Out", amount1Out);
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
}
