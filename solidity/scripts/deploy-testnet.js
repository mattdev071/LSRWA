// deploy-testnet.js - Script to deploy the LSRWA Express contract to testnet
const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying LSRWA Express contract to testnet...");
  
  // Get the network we're deploying to
  const network = await ethers.provider.getNetwork();
  console.log(`Deploying to network: ${network.name} (chainId: ${network.chainId})`);

  // Get the contract factory
  const LsrwaExpress = await ethers.getContractFactory("LsrwaExpress");
  
  // Deploy the contract
  console.log("Deploying contract...");
  const lsrwaExpress = await LsrwaExpress.deploy();
  
  // Wait for deployment to finish
  console.log("Waiting for deployment transaction to be mined...");
  await lsrwaExpress.deployed();
  
  console.log("LSRWA Express contract deployed to:", lsrwaExpress.address);
  
  // Wait for block confirmations for better Etherscan verification
  console.log("Waiting for block confirmations...");
  await lsrwaExpress.deployTransaction.wait(5);
  
  // Verify the contract on Etherscan
  if (process.env.VERIFY_CONTRACT === 'true') {
    console.log("Verifying contract on block explorer...");
    
    try {
      await hre.run("verify:verify", {
        address: lsrwaExpress.address,
        constructorArguments: [],
      });
      console.log("Contract verified successfully");
    } catch (error) {
      console.log("Error verifying contract:", error.message);
    }
  }
  
  // Output deployment information
  console.log("\nDeployment Information:");
  console.log("------------------------");
  console.log("Network:", network.name);
  console.log("Contract Address:", lsrwaExpress.address);
  console.log("Transaction Hash:", lsrwaExpress.deployTransaction.hash);
  console.log("Deployer Address:", lsrwaExpress.deployTransaction.from);
  console.log("Block Number:", lsrwaExpress.deployTransaction.blockNumber);
  console.log("Gas Used:", lsrwaExpress.deployTransaction.gasLimit.toString());
  
  // Save deployment info to a file
  const fs = require("fs");
  const deploymentInfo = {
    network: network.name,
    chainId: network.chainId,
    contractAddress: lsrwaExpress.address,
    transactionHash: lsrwaExpress.deployTransaction.hash,
    deployerAddress: lsrwaExpress.deployTransaction.from,
    blockNumber: lsrwaExpress.deployTransaction.blockNumber,
    timestamp: new Date().toISOString(),
  };
  
  const deploymentFile = `deployment-${network.name}.json`;
  fs.writeFileSync(
    deploymentFile,
    JSON.stringify(deploymentInfo, null, 2)
  );
  
  console.log(`\nDeployment information saved to ${deploymentFile}`);
}

// Execute the deployment
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("Deployment failed:", error);
    process.exit(1);
  }); 