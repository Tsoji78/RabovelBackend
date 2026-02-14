// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title SecurityToken
 * @dev ERC-1400-style security token for tokenized stocks.
 * Placeholder: full ERC-1400 (transfer restrictions, document management) can be added.
 */
contract SecurityToken is ERC20, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    string public symbolName; // e.g. "Sample Stock 1"

    constructor(
        string memory _name,
        string memory _symbol,
        string memory _symbolName
    ) ERC20(_name, _symbol) {
        symbolName = _symbolName;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
    }

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyRole(MINTER_ROLE) {
        _burn(from, amount);
    }
}
