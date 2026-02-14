// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title Trading
 * @dev Order-matching contract placeholder.
 * In production: match buy/sell orders, escrow, settle via security tokens.
 */
contract Trading is ReentrancyGuard, Ownable {
    enum OrderStatus { Pending, Filled, Cancelled }

    struct Order {
        bytes32 orderId;
        address user;
        string symbol;
        bool isBuy;
        uint256 quantity;
        uint256 price;
        OrderStatus status;
    }

    mapping(bytes32 => Order) public orders;

    event OrderPlaced(bytes32 indexed orderId, address indexed user, string symbol, bool isBuy, uint256 quantity, uint256 price);
    event OrderFilled(bytes32 indexed orderId);
    event OrderCancelled(bytes32 indexed orderId);

    function placeOrder(
        bytes32 orderId,
        string calldata symbol,
        bool isBuy,
        uint256 quantity,
        uint256 price
    ) external nonReentrant {
        require(orders[orderId].user == address(0), "Order exists");
        orders[orderId] = Order({
            orderId,
            user: msg.sender,
            symbol,
            isBuy,
            quantity,
            price,
            status: OrderStatus.Pending
        });
        emit OrderPlaced(orderId, msg.sender, symbol, isBuy, quantity, price);
    }

    function cancelOrder(bytes32 orderId) external nonReentrant {
        Order storage o = orders[orderId];
        require(o.user == msg.sender, "Not owner");
        require(o.status == OrderStatus.Pending, "Not pending");
        o.status = OrderStatus.Cancelled;
        emit OrderCancelled(orderId);
    }

    function getOrder(bytes32 orderId) external view returns (
        string memory symbol,
        bool isBuy,
        uint256 quantity,
        uint256 price,
        OrderStatus status
    ) {
        Order storage o = orders[orderId];
        return (o.symbol, o.isBuy, o.quantity, o.price, o.status);
    }

    function fillOrder(bytes32 orderId) external onlyOwner {
        Order storage o = orders[orderId];
        require(o.status == OrderStatus.Pending, "Not pending");
        o.status = OrderStatus.Filled;
        emit OrderFilled(orderId);
    }
}
