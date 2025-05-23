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
    }
    
    // ======== State Variables ========
    
    address public owner;
    uint128 public nextRequestId = 1;
    
    // Minimum amounts and ratios
    uint256 public minDepositAmount = 10;
    uint256 public minWithdrawalAmount = 10;
    uint256 public minCollateralRatio = 150; // 150%
    
    // APR and time-based variables
    uint256 public annualPercentageRate = 500; // 5.00% APR (stored as basis points, 1% = 100)
    uint256 public lastEpochTimestamp;
    uint256 public constant BASIS_POINTS = 10000; // 100.00%
    uint256 public constant SECONDS_PER_YEAR = 31536000; // 365 days
    
    // Reward variables
    uint256 public rewardRate = 300; // 3.00% annual reward rate (stored as basis points)
    uint256 public lastRewardCalculationTimestamp;
    uint256 public totalRewardsDistributed;
    
    // Mappings
    mapping(uint128 => Request) public requests;
    mapping(address => bool) public registeredUsers;
    mapping(address => uint256) public userBalances;
    mapping(address => uint256) public pendingDeposits;
    mapping(address => uint256) public userRewards;
    mapping(address => uint256) public lastUserRewardTimestamp;
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
    
    event AprUpdated(
        uint256 oldApr,
        uint256 newApr
    );
    
    event EpochUpdated(
        uint256 timestamp
    );
    
    event RewardRateUpdated(
        uint256 oldRate,
        uint256 newRate
    );
    
    event RewardsCalculated(
        address indexed walletAddress,
        uint256 rewardAmount
    );
    
    event RewardsClaimed(
        address indexed walletAddress,
        uint256 amount
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
    
    modifier updateRewards(address walletAddress) {
        if (registeredUsers[walletAddress]) {
            calculateRewards(walletAddress);
        }
        _;
    }
    
    // ======== Constructor ========
    
    constructor() {
        owner = msg.sender;
        lastEpochTimestamp = block.timestamp;
        lastRewardCalculationTimestamp = block.timestamp;
    }
    
    // ======== Admin Functions ========
    
    /**
     * @dev Update the APR used for partial fulfillment calculations
     * @param newApr New APR value in basis points (e.g., 500 = 5.00%)
     */
    function updateApr(uint256 newApr) external onlyOwner {
        require(newApr <= BASIS_POINTS, "APR cannot exceed 100%");
        
        uint256 oldApr = annualPercentageRate;
        annualPercentageRate = newApr;
        
        emit AprUpdated(oldApr, newApr);
    }
    
    /**
     * @dev Update the reward rate
     * @param newRate New reward rate in basis points (e.g., 300 = 3.00%)
     */
    function updateRewardRate(uint256 newRate) external onlyOwner {
        require(newRate <= BASIS_POINTS, "Reward rate cannot exceed 100%");
        
        uint256 oldRate = rewardRate;
        rewardRate = newRate;
        
        emit RewardRateUpdated(oldRate, newRate);
    }
    
    /**
     * @dev Update the lastEpochTimestamp to the current time
     */
    function updateEpoch() external onlyOwner {
        lastEpochTimestamp = block.timestamp;
        
        emit EpochUpdated(lastEpochTimestamp);
    }
    
    // ======== User Management Functions ========
    
    /**
     * @dev Register a new user
     */
    function registerUser() external {
        require(!registeredUsers[msg.sender], "User already registered");
        registeredUsers[msg.sender] = true;
        lastUserRewardTimestamp[msg.sender] = block.timestamp;
        emit UserRegistered(msg.sender);
    }
    
    /**
     * @dev Get user information
     */
    function getUserBalance(address walletAddress) external view returns (uint256 balance, uint256 pending, uint256 rewards) {
        return (userBalances[walletAddress], pendingDeposits[walletAddress], userRewards[walletAddress]);
    }
    
    // ======== Request Management Functions ========
    
    /**
     * @dev Create a deposit request
     * @param amount Amount to deposit
     */
    function createDepositRequest(uint256 amount) external userRegistered(msg.sender) updateRewards(msg.sender) returns (uint128) {
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
            processedAmount: 0
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
    function createWithdrawalRequest(uint256 amount) external userRegistered(msg.sender) updateRewards(msg.sender) returns (uint128) {
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
            processedAmount: 0
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
    function createBorrowRequest(uint256 amount, uint256 collateral) external userRegistered(msg.sender) updateRewards(msg.sender) returns (uint128) {
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
            processedAmount: 0
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
    
    // ======== Reward Functions ========
    
    /**
     * @dev Calculate rewards for a user based on their balance and time elapsed
     * @param walletAddress Address of the user
     */
    function calculateRewards(address walletAddress) public {
        // Skip if user is not registered or has no balance
        if (!registeredUsers[walletAddress] || userBalances[walletAddress] == 0) {
            return;
        }
        
        // Calculate time elapsed since last reward calculation
        uint256 lastCalculation = lastUserRewardTimestamp[walletAddress];
        if (block.timestamp <= lastCalculation) {
            return;
        }
        
        uint256 timeElapsed = block.timestamp - lastCalculation;
        
        // Calculate rewards based on user balance, time elapsed, and reward rate
        uint256 userBalance = userBalances[walletAddress];
        uint256 rewardAmount = (userBalance * rewardRate * timeElapsed) / (BASIS_POINTS * SECONDS_PER_YEAR);
        
        // Update user rewards
        userRewards[walletAddress] += rewardAmount;
        lastUserRewardTimestamp[walletAddress] = block.timestamp;
        totalRewardsDistributed += rewardAmount;
        
        emit RewardsCalculated(walletAddress, rewardAmount);
    }
    
    /**
     * @dev Get pending rewards for a user
     * @param walletAddress Address of the user
     * @return Current rewards plus pending rewards that would be calculated now
     */
    function getPendingRewards(address walletAddress) external view returns (uint256) {
        if (!registeredUsers[walletAddress] || userBalances[walletAddress] == 0) {
            return userRewards[walletAddress];
        }
        
        // Calculate time elapsed since last reward calculation
        uint256 lastCalculation = lastUserRewardTimestamp[walletAddress];
        if (block.timestamp <= lastCalculation) {
            return userRewards[walletAddress];
        }
        
        uint256 timeElapsed = block.timestamp - lastCalculation;
        
        // Calculate pending rewards
        uint256 userBalance = userBalances[walletAddress];
        uint256 pendingReward = (userBalance * rewardRate * timeElapsed) / (BASIS_POINTS * SECONDS_PER_YEAR);
        
        return userRewards[walletAddress] + pendingReward;
    }
    
    /**
     * @dev Claim rewards
     * @param amount Amount of rewards to claim (0 for all)
     */
    function claimRewards(uint256 amount) external userRegistered(msg.sender) updateRewards(msg.sender) returns (bool) {
        uint256 rewardsToClaim = amount > 0 ? amount : userRewards[msg.sender];
        require(rewardsToClaim > 0, "No rewards to claim");
        require(rewardsToClaim <= userRewards[msg.sender], "Insufficient rewards");
        
        // Update user rewards
        userRewards[msg.sender] -= rewardsToClaim;
        
        // Add rewards to user balance
        userBalances[msg.sender] += rewardsToClaim;
        
        emit RewardsClaimed(msg.sender, rewardsToClaim);
        
        return true;
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
        
        // Calculate rewards before updating balance
        calculateRewards(request.walletAddress);
        
        // Process the deposit
        userBalances[request.walletAddress] += request.amount;
        pendingDeposits[request.walletAddress] -= request.amount;
        
        request.isProcessed = true;
        request.processedAmount = request.amount;
        
        emit RequestProcessed(requestId, request.walletAddress, request.amount, true);
        
        return true;
    }
    
    /**
     * @dev Calculate partial fulfillment amount based on time and APR
     * @param amount Total requested amount
     * @return The amount that should be processed in the current period
     */
    function calculatePartialFulfillment(uint256 amount) public view returns (uint256) {
        // If we're in a new epoch, return the full amount
        if (block.timestamp <= lastEpochTimestamp) {
            return amount;
        }
        
        // Calculate time passed since last epoch
        uint256 secondsPassed = block.timestamp - lastEpochTimestamp;
        
        // Calculate time-based factor (percentage of a year)
        uint256 timeFactor = (secondsPassed * BASIS_POINTS) / SECONDS_PER_YEAR;
        
        // Calculate APR-adjusted amount for the time period
        // amount * (APR * timeFactor / BASIS_POINTS) / BASIS_POINTS
        uint256 timeBasedAmount = (amount * annualPercentageRate * timeFactor) / (BASIS_POINTS * BASIS_POINTS);
        
        // Return the minimum of the calculated amount and the total amount
        return timeBasedAmount < amount ? timeBasedAmount : amount;
    }
    
    /**
     * @dev Process a withdrawal request with time-based partial fulfillment
     * @param requestId ID of the request to process
     */
    function processWithdrawalRequest(uint128 requestId) external onlyOwner returns (bool) {
        Request storage request = requests[requestId];
        require(request.walletAddress != address(0), "Request not found");
        require(request.requestType == RequestType.Withdrawal, "Not withdrawal request");
        require(!request.isProcessed, "Already processed");
        
        // Calculate rewards before processing withdrawal
        calculateRewards(request.walletAddress);
        
        // Calculate how much to process based on time and APR
        uint256 remainingAmount = request.amount - request.processedAmount;
        uint256 amountToProcess = calculatePartialFulfillment(remainingAmount);
        
        // Update processed amount
        request.processedAmount += amountToProcess;
        
        // Check if fully processed
        bool fullyProcessed = (request.processedAmount == request.amount);
        request.isProcessed = fullyProcessed;
        
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
        
        // Calculate rewards before updating balance
        calculateRewards(request.walletAddress);
        
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
    
    // ======== Withdrawal Execution Functions ========
    
    /**
     * @dev Execute a processed withdrawal
     * @param requestId ID of the withdrawal request to execute
     */
    function executeWithdrawal(uint128 requestId) external updateRewards(msg.sender) returns (bool) {
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
        // For simplicity, we're just checking the owner
        return pendingDeposits[owner];
    }
    
    /**
     * @dev Get total rewards distributed
     */
    function getTotalRewardsDistributed() external view returns (uint256) {
        return totalRewardsDistributed;
    }
    
    // ======== Receive Function ========
    
    /**
     * @dev Allow the contract to receive ETH
     */
    receive() external payable {}
} 