use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, FixedBytes, Keccak256, U256},
    signers::local::PrivateKeySigner,
    signers::Signer,
    sol,
    sol_types::{Eip712Domain, SolStruct},
};
use anyhow::Result;

sol! {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Order {
        address maker;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 side; // 0 for BUY, 1 for SELL
        uint256 feeRateBps;
        uint256 nonce;
        uint256 signer;
        uint256 expiration;
        uint256 salt;
    }
}

pub struct PolymarketSigner {
    signer: PrivateKeySigner,
    domain: Eip712Domain,
}

impl PolymarketSigner {
    pub fn new(private_key: &str) -> Result<Self> {
        let signer = private_key.parse::<PrivateKeySigner>()?;
        
        let domain = Eip712Domain {
            name: Some("Polymarket CTF Exchange".into()),
            version: Some("1".into()),
            chain_id: Some(U256::from(137)),
            verifying_contract: Some(address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E")),
            salt: None,
        };

        Ok(Self { signer, domain })
    }

    pub async fn sign_order(&self, order: &Order) -> Result<FixedBytes<65>> {
        // EIP-712 Signing logic
        let hash = order.eip712_signing_hash(&self.domain);
        let signature = self.signer.sign_hash(&hash).await?;
        Ok(signature.as_bytes().into())
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_sign_dummy_order() -> Result<()> {
        // Dummy private key for testing
        let pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let pm_signer = PolymarketSigner::new(pk)?;
        
        let order = Order {
            maker: pm_signer.address(),
            taker: Address::ZERO,
            tokenId: U256::from(1),
            makerAmount: U256::from(100),
            takerAmount: U256::from(50),
            side: U256::from(0),
            feeRateBps: U256::from(0),
            nonce: U256::from(0),
            signer: U256::from(0),
            expiration: U256::from(1734471600),
            salt: U256::from(123456789),
        };

        let signature = pm_signer.sign_order(&order).await?;
        println!("Derived Address: {:?}", pm_signer.address());
        println!("Order Signature: {:?}", signature);
        
        assert_eq!(pm_signer.address(), address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));
        Ok(())
    }
}
