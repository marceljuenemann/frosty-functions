// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract HelloWorld {
    // Event to log the hello message
    event HelloMessage(string message);
    
    // Function to emit the hello event with a custom message
    function sayHello(string memory message) public {
        emit HelloMessage(message);
    }
}
