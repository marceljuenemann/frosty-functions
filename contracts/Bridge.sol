// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title FrostyBridge — Bridges EVM with Frosty Functions running on the Internet Computer
contract FrostyBridge {

    /// @notice Emitted when a new job is submitted for processing.
    /// @param jobId Unique monotonically increasing identifier for the job.
    /// @param caller The EOA or contract that submitted the job.
    /// @param functionId SHA-256 of the wasm binary to execute.
    /// @param data Arbitrary binary payload to pass to the function.
    /// @param gasPayment The ETH amount sent with the transaction, forwarded to `owner`.
    event FunctionInvoked(
        uint256 indexed jobId,
        address indexed caller,
        bytes32 functionId,
        bytes data,
        uint256 gasPayment
    );

    /// @notice Owner of the bridge (the ICP canister).
    address public immutable owner;

    /// @notice Minimum ETH required per job submission, in wei.
    uint256 public immutable minPaymentWei;

    /// @notice Monotonically increasing job counter used to assign new job IDs.
    uint256 public nextJobId;

    /// @param _owner Address that will receive ETH payments for submitted jobs.
    /// @param _minPaymentWei Minimum payment per job, specified in wei.
    constructor(address _owner, uint256 _minPaymentWei) {
        require(_owner != address(0), "Owner cannot be zero address");
        owner = _owner;
        minPaymentWei = _minPaymentWei;
        nextJobId = 1;
    }

    /// @notice Submit a new job to be executed on ICP.
    /// @param functionId SHA-256 hash of the wasm binary to execute.
    /// @param data Arbitrary binary payload to be passed to the function.
    /// @return jobId The unique identifier assigned to this job submission.
    function invokeFunction(
        bytes32 functionId,
        bytes calldata data
    ) external payable returns (uint256 jobId) {
        require(msg.value >= minPaymentWei, "Insufficient ETH: below minimum");
        require(functionId != bytes32(0), "Function ID cannot be zero");

        jobId = nextJobId++;
        
        emit FunctionInvoked(
            jobId,
            msg.sender,
            functionId,
            data,
            msg.value
        );
        
        payable(owner).transfer(msg.value);
        
        return jobId;
    }
}
