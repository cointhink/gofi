//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "v2-core-1.0.1/contracts/interfaces/IUniswapV2Pair.sol";
import "openzeppelin-contracts-5.3.0/contracts/token/ERC20/ERC20.sol";

contract UniSwab {
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner can call this");
        _;
    }

    function swab(
        uint256 amount1In,
        address pool0,
        address pool1
    ) public onlyOwner {
        (uint112 _reserve00, uint112 _reserve01, ) = IUniswapV2Pair(pool0)
            .getReserves();
        // function transfer(address to, uint value) external returns (bool);
        ERC20(IUniswapV2Pair(pool0).token0()).transfer(pool0, amount1In);
        // uint256 amount0Out = getAmountOut(amount1In, _reserve01, _reserve00);

        // function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
        // IUniswapV2Pair(pool0).swap(amount0Out, 0, owner, new bytes(0));
        // IUniswapV2Pair(pool1).swap(0, amount1Out, owner, new bytes(0));
    }

    // given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    function getAmountOut(
        uint256 amountIn,
        uint256 reserveIn,
        uint256 reserveOut
    ) internal pure returns (uint256 amountOut) {
        require(amountIn > 0, "INSUFFICIENT_INPUT_AMOUNT");
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
        require(amountOut > 0, "INSUFFICIENT_OUTPUT_AMOUNT");
        require(reserveIn > 0 && reserveOut > 0);
        uint256 numerator = reserveIn * amountOut * 1000;
        uint256 denominator = (reserveOut - amountOut) * 997;
        amountIn = (numerator / denominator) + 1;
    }
}
