// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

/**
 * @title LSRWA Express
 * @dev Smart contract for handling deposit, withdrawal, and borrow requests with epoch-based batch processing
 */
contract LsrwaExpress {
    // ======== Type Definitions ========

    enum RequestType { Deposit, Withdrawal, Borrow }
    
    enum EpochStatus { Active, Processing, Completed }
    
    struct Request {
        uint128 id;
        RequestType requestType;
        address walletAddress;
        uint256 amount;
        uint256 timestamp;
        bool isProcessed;
    }
    
    struct User {
        address walletAddress;
        bool isRegistered;
        uint256 activeBalance;
        uint256 pendingDeposits;
        uint256 pendingWithdrawals;
    }
    
    struct Epoch {
        uint32 id;
        uint256 startTimestamp;
        uint256 endTimestamp;
        EpochStatus status;
        uint32 processedDepositCount;
        uint32 processedWithdrawalCount;
        uint32 processedBorrowCount;
    }
    
    // ======== State Variables ========
    
    address public owner;
    uint128 public nextRequestId = 1;
    uint32 public nextEpochId = 2;
    
    // Minimum amounts and ratios
    uint256 public minDepositAmount = 10;
    uint256 public minWithdrawalAmount = 10;
    uint256 public minCollateralRatio = 150; // 150%
    
    // Mappings
    mapping(uint128 => Request) public requests;
    mapping(address => User) public users;
    mapping(address => uint128[]) public userDepositRequests;
    mapping(address => uint128[]) public userWithdrawalRequests;
    mapping(address => uint128[]) public userBorrowRequests;
    mapping(uint32 => Epoch) public epochs;
    
    // Current epoch
    Epoch public currentEpoch;
    
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
        uint256 amount
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
    
    event EpochClosed(
        uint32 indexed epochId,
        uint256 startTimestamp,
        uint256 endTimestamp,
        uint32 processedDepositCount,
        uint32 processedWithdrawalCount,
        uint32 processedBorrowCount
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
    
    // ======== Modifiers ========
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    modifier userExists(address walletAddress) {
        require(users[walletAddress].walletAddress != address(0), "User not found");
        _;
    }
    
    modifier userRegistered(address walletAddress) {
        require(users[walletAddress].isRegistered, "User not registered");
        _;
    }
    
    // ======== Constructor ========
    
    constructor() {
        owner = msg.sender;
        
        // Create the initial epoch
        currentEpoch = Epoch({
            id: 1,
            startTimestamp: block.timestamp,
            endTimestamp: 0,
            status: EpochStatus.Active,
            processedDepositCount: 0,
            processedWithdrawalCount: 0,
            processedBorrowCount: 0
        });
    }
    
    // ======== User Management Functions ========
    
    /**
     * @dev Register a new user
     */
    function registerUser() external {
        address walletAddress = msg.sender;
        
        // Check if user already exists
        if (users[walletAddress].walletAddress != address(0)) {
            require(!users[walletAddress].isRegistered, "User already registered");
            users[walletAddress].isRegistered = true;
        } else {
            users[walletAddress] = User({
                walletAddress: walletAddress,
                isRegistered: true,
                activeBalance: 0,
                pendingDeposits: 0,
                pendingWithdrawals: 0
            });
        }
        
        emit UserRegistered(walletAddress);
    }
    
    /**
     * @dev Get user information
     */
    function getUser(address walletAddress) external view returns (User memory) {
        return users[walletAddress];
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
            isProcessed: false
        });
        
        // Add request to user's deposit requests
        userDepositRequests[msg.sender].push(requestId);
        
        // Update user's pending deposits
        User storage user = users[msg.sender];
        user.pendingDeposits += amount;
        
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
        
        User storage user = users[msg.sender];
        require(user.activeBalance >= amount, "Insufficient balance");
        
        uint128 requestId = nextRequestId++;
        
        // Create the request
        requests[requestId] = Request({
            id: requestId,
            requestType: RequestType.Withdrawal,
            walletAddress: msg.sender,
            amount: amount,
            timestamp: block.timestamp,
            isProcessed: false
        });
        
        // Add request to user's withdrawal requests
        userWithdrawalRequests[msg.sender].push(requestId);
        
        // Update user's balances
        user.activeBalance -= amount;
        user.pendingWithdrawals += amount;
        
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
            isProcessed: false
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
        
        User storage user = users[request.walletAddress];
        
        // Process the deposit
        user.activeBalance += request.amount;
        user.pendingDeposits -= request.amount;
        request.isProcessed = true;
        
        // Update epoch stats if there is an active epoch
        if (currentEpoch.status == EpochStatus.Active) {
            currentEpoch.processedDepositCount++;
        }
        
        emit RequestProcessed(requestId, request.walletAddress, request.amount);
        
        return true;
    }
    
    /**
     * @dev Process a withdrawal request
     * @param requestId ID of the request to process
     */
    function processWithdrawalRequest(uint128 requestId) external onlyOwner returns (bool) {
        Request storage request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        require(request.requestType == RequestType.Withdrawal, "Not withdrawal request");
        require(!request.isProcessed, "Already processed");
        
        User storage user = users[request.walletAddress];
        
        // Process the withdrawal
        user.pendingWithdrawals -= request.amount;
        request.isProcessed = true;
        
        // Update epoch stats if there is an active epoch
        if (currentEpoch.status == EpochStatus.Active) {
            currentEpoch.processedWithdrawalCount++;
        }
        
        emit RequestProcessed(requestId, request.walletAddress, request.amount);
        
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
        
        User storage user = users[request.walletAddress];
        
        // Process the borrow
        user.activeBalance += request.amount;
        request.isProcessed = true;
        
        // Update epoch stats if there is an active epoch
        if (currentEpoch.status == EpochStatus.Active) {
            currentEpoch.processedBorrowCount++;
        }
        
        emit RequestProcessed(requestId, request.walletAddress, request.amount);
        
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
     */
    function batchProcessWithdrawalRequests(uint128[] calldata requestIds) external onlyOwner returns (bool) {
        require(requestIds.length > 0, "Empty batch");
        
        uint32 processedCount = 0;
        uint32 failedCount = 0;
        
        for (uint i = 0; i < requestIds.length; i++) {
            try this.processWithdrawalRequest(requestIds[i]) returns (bool success) {
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
    
    // ======== Epoch Management Functions ========
    
    /**
     * @dev Close the current epoch and start a new one
     */
    function closeCurrentEpoch() external onlyOwner returns (uint32) {
        require(currentEpoch.status == EpochStatus.Active, "No active epoch");
        
        // Update the current epoch
        currentEpoch.status = EpochStatus.Completed;
        currentEpoch.endTimestamp = block.timestamp;
        
        // Store the completed epoch
        epochs[currentEpoch.id] = currentEpoch;
        
        // Emit event for the closed epoch
        emit EpochClosed(
            currentEpoch.id,
            currentEpoch.startTimestamp,
            currentEpoch.endTimestamp,
            currentEpoch.processedDepositCount,
            currentEpoch.processedWithdrawalCount,
            currentEpoch.processedBorrowCount
        );
        
        // Create a new epoch
        uint32 newEpochId = nextEpochId++;
        currentEpoch = Epoch({
            id: newEpochId,
            startTimestamp: block.timestamp,
            endTimestamp: 0,
            status: EpochStatus.Active,
            processedDepositCount: 0,
            processedWithdrawalCount: 0,
            processedBorrowCount: 0
        });
        
        return newEpochId;
    }
    
    /**
     * @dev Get the current epoch information
     */
    function getCurrentEpoch() external view returns (Epoch memory) {
        return currentEpoch;
    }
    
    /**
     * @dev Get epoch information by ID
     * @param epochId ID of the epoch
     */
    function getEpoch(uint32 epochId) external view returns (Epoch memory) {
        return epochs[epochId];
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
        
        // Transfer the funds to the user
        (bool success, ) = payable(msg.sender).call{value: request.amount}("");
        require(success, "Transfer failed");
        
        emit WithdrawalExecuted(requestId, msg.sender, request.amount);
        
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
        uint256 total = 0;
        
        // In a production environment, you would iterate through all users
        // or maintain a running total. For simplicity, we're just checking the owner.
        User memory ownerUser = users[owner];
        if (ownerUser.walletAddress != address(0)) {
            total += ownerUser.pendingDeposits;
        }
        
        return total;
    }
    
    /**
     * @dev Get total pending withdrawals
     */
    function getTotalPendingWithdrawals() external view returns (uint256) {
        uint256 total = 0;
        
        // In a production environment, you would iterate through all users
        // or maintain a running total. For simplicity, we're just checking the owner.
        User memory ownerUser = users[owner];
        if (ownerUser.walletAddress != address(0)) {
            total += ownerUser.pendingWithdrawals;
        }
        
        return total;
    }
    
    // ======== Receive Function ========
    
    /**
     * @dev Allow the contract to receive ETH
     */
    receive() external payable {}
} 