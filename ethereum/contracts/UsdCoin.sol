//SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import 'openzeppelin-contracts-5.3.0/contracts/token/ERC20/ERC20.sol';

    
contract UsdCoin is ERC20 {
  constructor() ERC20("Eggplant", "EGGP") {}

  function mint(address to, uint256 amount) public {
    _mint(to, amount);
  }
}
