//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "openzeppelin-contracts-5.3.0/contracts/token/ERC20/ERC20.sol";

contract UsdCoin is ERC20 {
    address public owner;
    constructor() ERC20("Eggplant", "EGGP") {
        owner = msg.sender;
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner can call this");
        _;
    }
}
