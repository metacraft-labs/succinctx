use anyhow::Result;
use ethers::types::U256;
use log::debug;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

/// A client used for connecting and querying the beaconcha.in API.
#[derive(Debug, Clone)]
pub struct BeaconchainAPIClient {
    pub api_url: String,
    pub client: Client,
}

#[derive(Debug, Deserialize)]
pub struct APIResponse<T> {
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionBlock {
    // TODO: may need to be bigint
    pub pos_consensus: ExecutionBlockConsensusData,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionBlockConsensusData {
    pub slot: u64,
}

impl BeaconchainAPIClient {
    /// Creates a new BeaconChainAPIClient based on a api url and key.
    pub fn new(api_url: String, api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("apikey", HeaderValue::from_str(api_key.as_str()).unwrap());
        let client = Client::builder().default_headers(headers).build().unwrap();
        Self { api_url, client }
    }

    /// GET /api/v1/execution/block/{blockNumbers}
    /// Returns the execution + consensus block data for the given eth1 block numbers, in descending block number order.
    pub fn get_execution_blocks(&self, eth1_block_numbers: &[U256]) -> Result<Vec<ExecutionBlock>> {
        let query_str = eth1_block_numbers
            .iter()
            .map(|block_number| block_number.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let endpoint = format!("{}/api/v1/execution/block/{}", self.api_url, query_str);
        debug!("{}", endpoint);
        let response = self.client.get(endpoint).send()?;

        let parsed: APIResponse<Vec<ExecutionBlock>> = response.json()?;

        Ok(parsed.data.unwrap())
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_get_execution_blocks() {
        dotenv::dotenv().ok();
        let api_key = env::var("BEACONCHAIN_API_KEY_1").unwrap();
        let client = BeaconchainAPIClient::new("https://beaconcha.in".to_string(), api_key);
        let withdrawals = client
            .get_execution_blocks(&[U256::from(18173221)])
            .unwrap();
        assert_eq!(withdrawals.len(), 1);
        println!("withdrawals: {:?}", withdrawals);
    }
}
