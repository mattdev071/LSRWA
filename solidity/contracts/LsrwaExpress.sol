// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

/**
 * @title LSRWA Express
 * @dev Smart contract for handling deposit, withdrawal, and borrow requests with gas-optimized processing
 */
contract LsrwaExpress {
    // ======== Type Definitions ========

    enum RequestType { Deposit, Withdrawal, Borrow }
    
    struct Request {
        uint128 id;
        RequestType requestType;
        address walletAddress;
        uint256 amount;
        uint256 timestamp;
        bool isProcessed;
        uint256 processedAmount; // For tracking partial fulfillment
        uint128 linkedRequestId; // For linking to follow-up requests
    }
    
    // ======== State Variables ========
    
    address public owner;
    uint128 public nextRequestId = 1;
    
    // Minimum amounts and ratios
    uint256 public minDepositAmount = 10;
    uint256 public minWithdrawalAmount = 10;
    uint256 public minCollateralRatio = 150; // 150%
    
    // Mappings
    mapping(uint128 => Request) public requests;
    mapping(address => bool) public registeredUsers;
    mapping(address => uint256) public userBalances;
    mapping(address => uint256) public pendingDeposits;
    mapping(address => uint128[]) public userDepositRequests;
    mapping(address => uint128[]) public userWithdrawalRequests;
    mapping(address => uint128[]) public userBorrowRequests;
    
    // ======== Events ========
    
    event DepositRequested(
        uint128 indexed requestId,
        address indexed walletAddress,
        uint256 amount
    );
    
    event WithdrawalRequested(
        uint128 indexed requestId,
        address indexed walletAddress,
        uint256 amount
    );
    
    event RequestProcessed(
        uint128 indexed requestId,
        address indexed walletAddress,
        uint256 amount,
        bool fullyProcessed
    );
    
    event UserRegistered(
        address indexed walletAddress
    );
    
    event BorrowRequested(
        uint128 indexed requestId,
        address indexed walletAddress,
        uint256 amount,
        uint256 collateral
    );
    
    event BatchProcessed(
        RequestType indexed requestType,
        uint32 processedCount,
        uint32 failedCount
    );
    
    event WithdrawalExecuted(
        uint128 indexed requestId,
        address indexed walletAddress,
        uint256 amount
    );
    
    event EmergencyWithdrawal(
        address indexed walletAddress,
        uint256 amount
    );
    
    event PartialRequestFulfillment(
        uint128 indexed originalRequestId,
        uint128 indexed newRequestId,
        uint256 remainingAmount
    );
    
    // ======== Modifiers ========
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    modifier userRegistered(address walletAddress) {
        require(registeredUsers[walletAddress], "User not registered");
        _;
    }
    
    // ======== Constructor ========
    
    constructor() {
        owner = msg.sender;
    }
    
    // ======== User Management Functions ========
    
    /**
     * @dev Register a new user
     */
    function registerUser() external {
        require(!registeredUsers[msg.sender], "User already registered");
        registeredUsers[msg.sender] = true;
        emit UserRegistered(msg.sender);
    }
    
    /**
     * @dev Get user information
     */
    function getUserBalance(address walletAddress) external view returns (uint256 balance, uint256 pending) {
        return (userBalances[walletAddress], pendingDeposits[walletAddress]);
    }
    
    // ======== Request Management Functions ========
    
    /**
     * @dev Create a deposit request
     * @param amount Amount to deposit
     */
    function createDepositRequest(uint256 amount) external userRegistered(msg.sender) returns (uint128) {
        require(amount >= minDepositAmount, "Amount too low");
        require(amount > 0, "Amount zero");
        
        uint128 requestId = nextRequestId++;
        
        // Create the request
        requests[requestId] = Request({
            id: requestId,
            requestType: RequestType.Deposit,
            walletAddress: msg.sender,
            amount: amount,
            timestamp: block.timestamp,
            isProcessed: false,
            processedAmount: 0,
            linkedRequestId: 0
        });
        
        // Add request to user's deposit requests
        userDepositRequests[msg.sender].push(requestId);
        
        // Update user's pending deposits
        pendingDeposits[msg.sender] += amount;
        
        emit DepositRequested(requestId, msg.sender, amount);
        
        return requestId;
    }
    
    /**
     * @dev Create a withdrawal request
     * @param amount Amount to withdraw
     */
    function createWithdrawalRequest(uint256 amount) external userRegistered(msg.sender) returns (uint128) {
        require(amount >= minWithdrawalAmount, "Amount too low");
        require(amount > 0, "Amount zero");
        require(userBalances[msg.sender] >= amount, "Insufficient balance");
        
        uint128 requestId = nextRequestId++;
        
        // Create the request
        requests[requestId] = Request({
            id: requestId,
            requestType: RequestType.Withdrawal,
            walletAddress: msg.sender,
            amount: amount,
            timestamp: block.timestamp,
            isProcessed: false,
            processedAmount: 0,
            linkedRequestId: 0
        });
        
        // Add request to user's withdrawal requests
        userWithdrawalRequests[msg.sender].push(requestId);
        
        // Update user's balances
        userBalances[msg.sender] -= amount;
        
        emit WithdrawalRequested(requestId, msg.sender, amount);
        
        return requestId;
    }
    
    /**
     * @dev Create a borrow request with collateral
     * @param amount Amount to borrow
     * @param collateral Collateral amount
     */
    function createBorrowRequest(uint256 amount, uint256 collateral) external userRegistered(msg.sender) returns (uint128) {
        require(amount > 0, "Amount zero");
        
        // Calculate minimum required collateral
        uint256 minRequiredCollateral = amount * minCollateralRatio / 100;
        require(collateral >= minRequiredCollateral, "Insufficient collateral");
        
        uint128 requestId = nextRequestId++;
        
        // Create the request
        requests[requestId] = Request({
            id: requestId,
            requestType: RequestType.Borrow,
            walletAddress: msg.sender,
            amount: amount,
            timestamp: block.timestamp,
            isProcessed: false,
            processedAmount: 0,
            linkedRequestId: 0
        });
        
        // Add request to user's borrow requests
        userBorrowRequests[msg.sender].push(requestId);
        
        emit BorrowRequested(requestId, msg.sender, amount, collateral);
        
        return requestId;
    }
    
    /**
     * @dev Get request information
     * @param requestId ID of the request
     */
    function getRequest(uint128 requestId) external view returns (Request memory) {
        Request memory request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        return request;
    }
    
    // ======== Request Processing Functions ========
    
    /**
     * @dev Process a deposit request
     * @param requestId ID of the request to process
     */
    function processDepositRequest(uint128 requestId) external onlyOwner returns (bool) {
        Request storage request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        require(request.requestType == RequestType.Deposit, "Not deposit request");
        require(!request.isProcessed, "Already processed");
        
        // Process the deposit
        userBalances[request.walletAddress] += request.amount;
        pendingDeposits[request.walletAddress] -= request.amount;
        
        request.isProcessed = true;
        request.processedAmount = request.amount;
        
        emit RequestProcessed(requestId, request.walletAddress, request.amount, true);
        
        return true;
    }
    
    /**
     * @dev Process a withdrawal request, with support for partial fulfillment
     * @param requestId ID of the request to process
     * @param amountToProcess Amount to process (can be less than the request amount)
     */
    function processWithdrawalRequest(uint128 requestId, uint256 amountToProcess) external onlyOwner returns (bool) {
        Request storage request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        require(request.requestType == RequestType.Withdrawal, "Not withdrawal request");
        require(!request.isProcessed, "Already processed");
        require(amountToProcess > 0 && amountToProcess <= request.amount, "Invalid amount");
        
        // Update processed amount
        request.processedAmount += amountToProcess;
        
        // Check if fully processed
        bool fullyProcessed = (request.processedAmount == request.amount);
        request.isProcessed = fullyProcessed;
        
        // If partially processed, create a new request for the remainder
        if (!fullyProcessed) {
            uint256 remainingAmount = request.amount - request.processedAmount;
            uint128 newRequestId = nextRequestId++;
            
            // Create the new request with the same timestamp as the original
            requests[newRequestId] = Request({
                id: newRequestId,
                requestType: RequestType.Withdrawal,
                walletAddress: request.walletAddress,
                amount: remainingAmount,
                timestamp: request.timestamp, // Keep the original timestamp
                isProcessed: false,
                processedAmount: 0,
                linkedRequestId: requestId // Link to the original request
            });
            
            // Add to user's withdrawal requests
            userWithdrawalRequests[request.walletAddress].push(newRequestId);
            
            // Set link to the new request
            request.linkedRequestId = newRequestId;
            
            emit PartialRequestFulfillment(requestId, newRequestId, remainingAmount);
        }
        
        emit RequestProcessed(requestId, request.walletAddress, amountToProcess, fullyProcessed);
        
        return true;
    }
    
    /**
     * @dev Process a borrow request
     * @param requestId ID of the request to process
     */
    function processBorrowRequest(uint128 requestId) external onlyOwner returns (bool) {
        Request storage request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        require(request.requestType == RequestType.Borrow, "Not borrow request");
        require(!request.isProcessed, "Already processed");
        
        // Process the borrow
        userBalances[request.walletAddress] += request.amount;
        
        request.isProcessed = true;
        request.processedAmount = request.amount;
        
        emit RequestProcessed(requestId, request.walletAddress, request.amount, true);
        
        return true;
    }
    
    // ======== Batch Processing Functions ========
    
    /**
     * @dev Process multiple deposit requests in a batch
     * @param requestIds Array of request IDs to process
     */
    function batchProcessDepositRequests(uint128[] calldata requestIds) external onlyOwner returns (bool) {
        require(requestIds.length > 0, "Empty batch");
        
        uint32 processedCount = 0;
        uint32 failedCount = 0;
        
        for (uint i = 0; i < requestIds.length; i++) {
            try this.processDepositRequest(requestIds[i]) returns (bool success) {
                if (success) {
                    processedCount++;
                } else {
                    failedCount++;
                }
            } catch {
                failedCount++;
            }
        }
        
        emit BatchProcessed(RequestType.Deposit, processedCount, failedCount);
        
        return true;
    }
    
    /**
     * @dev Process multiple withdrawal requests in a batch
     * @param requestIds Array of request IDs to process
     * @param amounts Array of amounts to process for each request
     */
    function batchProcessWithdrawalRequests(uint128[] calldata requestIds, uint256[] calldata amounts) external onlyOwner returns (bool) {
        require(requestIds.length > 0, "Empty batch");
        require(requestIds.length == amounts.length, "Array length mismatch");
        
        uint32 processedCount = 0;
        uint32 failedCount = 0;
        
        for (uint i = 0; i < requestIds.length; i++) {
            try this.processWithdrawalRequest(requestIds[i], amounts[i]) returns (bool success) {
                if (success) {
                    processedCount++;
                } else {
                    failedCount++;
                }
            } catch {
                failedCount++;
            }
        }
        
        emit BatchProcessed(RequestType.Withdrawal, processedCount, failedCount);
        
        return true;
    }
    
    /**
     * @dev Process multiple borrow requests in a batch
     * @param requestIds Array of request IDs to process
     */
    function batchProcessBorrowRequests(uint128[] calldata requestIds) external onlyOwner returns (bool) {
        require(requestIds.length > 0, "Empty batch");
        
        uint32 processedCount = 0;
        uint32 failedCount = 0;
        
        for (uint i = 0; i < requestIds.length; i++) {
            try this.processBorrowRequest(requestIds[i]) returns (bool success) {
                if (success) {
                    processedCount++;
                } else {
                    failedCount++;
                }
            } catch {
                failedCount++;
            }
        }
        
        emit BatchProcessed(RequestType.Borrow, processedCount, failedCount);
        
        return true;
    }
    
    // ======== Withdrawal Execution Functions ========
    
    /**
     * @dev Execute a processed withdrawal
     * @param requestId ID of the withdrawal request to execute
     */
    function executeWithdrawal(uint128 requestId) external returns (bool) {
        Request storage request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        require(request.requestType == RequestType.Withdrawal, "Not withdrawal request");
        require(request.isProcessed, "Withdrawal not processed");
        require(request.walletAddress == msg.sender, "Not request owner");
        
        uint256 amountToWithdraw = request.processedAmount;
        require(amountToWithdraw > 0, "Nothing to withdraw");
        
        // Transfer the funds to the user
        (bool success, ) = payable(msg.sender).call{value: amountToWithdraw}("");
        require(success, "Transfer failed");
        
        emit WithdrawalExecuted(requestId, msg.sender, amountToWithdraw);
        
        return true;
    }
    
    /**
     * @dev Emergency withdrawal function for the owner
     * @param amount Amount to withdraw
     */
    function emergencyWithdraw(uint256 amount) external onlyOwner returns (bool) {
        require(amount > 0, "Amount zero");
        require(address(this).balance >= amount, "Insufficient balance");
        
        // Transfer the funds to the owner
        (bool success, ) = payable(owner).call{value: amount}("");
        require(success, "Transfer failed");
        
        emit EmergencyWithdrawal(owner, amount);
        
        return true;
    }
    
    // ======== Query Functions ========
    
    /**
     * @dev Get all deposit request IDs for a user
     * @param walletAddress Address of the user
     */
    function getUserDepositRequests(address walletAddress) external view returns (uint128[] memory) {
        return userDepositRequests[walletAddress];
    }
    
    /**
     * @dev Get all withdrawal request IDs for a user
     * @param walletAddress Address of the user
     */
    function getUserWithdrawalRequests(address walletAddress) external view returns (uint128[] memory) {
        return userWithdrawalRequests[walletAddress];
    }
    
    /**
     * @dev Get all borrow request IDs for a user
     * @param walletAddress Address of the user
     */
    function getUserBorrowRequests(address walletAddress) external view returns (uint128[] memory) {
        return userBorrowRequests[walletAddress];
    }
    
    /**
     * @dev Get the contract balance
     */
    function getContractBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    /**
     * @dev Get total pending deposits
     */
    function getTotalPendingDeposits() external view returns (uint256) {
        // In a production environment, you would maintain a running total
        // For simplicity, we're just checking the owner
        return pendingDeposits[owner];
    }
    
    // ======== Receive Function ========
    
    /**
     * @dev Allow the contract to receive ETH
     */
    receive() external payable {}
} 