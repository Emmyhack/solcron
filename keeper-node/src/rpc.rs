use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use log::{warn, info, debug};
use crate::config::KeeperConfig;
use crate::error::{KeeperError, KeeperResult};

#[derive(Clone)]
pub struct RpcManager {
    clients: Arc<RwLock<Vec<RpcClientWrapper>>>,
    current_index: Arc<RwLock<usize>>,
    config: KeeperConfig,
}

#[derive(Clone)]
struct RpcClientWrapper {
    client: Arc<RpcClient>,
    url: String,
    is_healthy: bool,
    last_error: Option<Instant>,
    request_count: u64,
    error_count: u64,
}

impl RpcClientWrapper {
    fn new(url: String) -> Self {
        let client = Arc::new(RpcClient::new_with_commitment(
            url.clone(),
            CommitmentConfig::confirmed(),
        ));
        
        Self {
            client,
            url,
            is_healthy: true,
            last_error: None,
            request_count: 0,
            error_count: 0,
        }
    }

    fn mark_error(&mut self) {
        self.error_count += 1;
        self.last_error = Some(Instant::now());
        
        // Mark unhealthy if error rate is too high
        if self.error_count > 5 && 
           self.error_count as f64 / self.request_count as f64 > 0.1 {
            self.is_healthy = false;
        }
    }

    fn mark_success(&mut self) {
        self.request_count += 1;
        
        // Reset health if it's been a while since last error
        if let Some(last_error) = self.last_error {
            if last_error.elapsed() > Duration::from_secs(300) { // 5 minutes
                self.is_healthy = true;
                self.error_count = 0;
            }
        }
    }

    fn should_retry(&self) -> bool {
        if self.is_healthy {
            return true;
        }
        
        // Allow retry if it's been enough time since last error
        if let Some(last_error) = self.last_error {
            last_error.elapsed() > Duration::from_secs(60) // 1 minute
        } else {
            true
        }
    }
}

impl RpcManager {
    pub fn new(config: KeeperConfig) -> Self {
        let urls = config.get_rpc_urls();
        let clients: Vec<RpcClientWrapper> = urls
            .into_iter()
            .map(RpcClientWrapper::new)
            .collect();
        
        info!("Initialized RPC manager with {} endpoints", clients.len());
        
        Self {
            clients: Arc::new(RwLock::new(clients)),
            current_index: Arc::new(RwLock::new(0)),
            config,
        }
    }

    pub async fn get_client(&self) -> KeeperResult<Arc<RpcClient>> {
        let clients = self.clients.read().await;
        let mut current_index = self.current_index.write().await;
        
        // Find a healthy client
        for _ in 0..clients.len() {
            let client = &clients[*current_index];
            
            if client.should_retry() {
                debug!("Using RPC client: {}", client.url);
                let result = client.client.clone();
                *current_index = (*current_index + 1) % clients.len();
                return Ok(result);
            }
            
            *current_index = (*current_index + 1) % clients.len();
        }
        
        // If no healthy clients, use the first one anyway
        warn!("No healthy RPC clients available, using first client");
        Ok(clients[0].client.clone())
    }

    pub async fn execute_with_retry<F, T>(&self, operation: F) -> KeeperResult<T>
    where
        F: Fn(Arc<RpcClient>) -> KeeperResult<T> + Clone,
    {
        let max_retries = self.config.get_max_rpc_retries();
        let retry_delay = Duration::from_millis(1000); // 1 second base delay
        
        for attempt in 0..=max_retries {
            let client = self.get_client().await?;
            
            match operation(client.clone()) {
                Ok(result) => {
                    self.mark_client_success(&client).await;
                    return Ok(result);
                }
                Err(e) => {
                    self.mark_client_error(&client).await;
                    
                    if attempt == max_retries {
                        return Err(e);
                    }
                    
                    let delay = retry_delay * (2_u32.pow(attempt));
                    warn!("RPC request failed (attempt {}/{}), retrying in {:?}: {:?}", 
                          attempt + 1, max_retries + 1, delay, e);
                    
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        unreachable!()
    }

    async fn mark_client_success(&self, client: &Arc<RpcClient>) {
        let mut clients = self.clients.write().await;
        for wrapper in clients.iter_mut() {
            if Arc::ptr_eq(&wrapper.client, client) {
                wrapper.mark_success();
                break;
            }
        }
    }

    async fn mark_client_error(&self, client: &Arc<RpcClient>) {
        let mut clients = self.clients.write().await;
        for wrapper in clients.iter_mut() {
            if Arc::ptr_eq(&wrapper.client, client) {
                wrapper.mark_error();
                warn!("RPC client {} marked as unhealthy", wrapper.url);
                break;
            }
        }
    }

    pub async fn get_health_status(&self) -> Vec<(String, bool, u64, u64)> {
        let clients = self.clients.read().await;
        clients
            .iter()
            .map(|c| (c.url.clone(), c.is_healthy, c.request_count, c.error_count))
            .collect()
    }

    pub async fn reset_health(&self) {
        let mut clients = self.clients.write().await;
        for wrapper in clients.iter_mut() {
            wrapper.is_healthy = true;
            wrapper.error_count = 0;
            wrapper.last_error = None;
        }
        info!("Reset health status for all RPC clients");
    }
}

// Convenience methods for common RPC operations
impl RpcManager {
    pub async fn get_latest_blockhash(&self) -> KeeperResult<solana_sdk::hash::Hash> {
        self.execute_with_retry(|client| {
            client.get_latest_blockhash()
                .map_err(|e| KeeperError::RpcError(format!("Failed to get latest blockhash: {}", e)))
        }).await
    }

    pub async fn send_and_confirm_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> KeeperResult<solana_sdk::signature::Signature> {
        let transaction = transaction.clone();
        self.execute_with_retry(move |client| {
            client.send_and_confirm_transaction(&transaction)
                .map_err(|e| KeeperError::RpcError(format!("Failed to send transaction: {}", e)))
        }).await
    }

    pub async fn simulate_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> KeeperResult<solana_client::rpc_response::RpcSimulateTransactionResult> {
        let transaction = transaction.clone();
        self.execute_with_retry(move |client| {
            client.simulate_transaction(&transaction)
                .map_err(|e| KeeperError::RpcError(format!("Failed to simulate transaction: {}", e)))
        }).await
    }

    pub async fn get_account_data(
        &self,
        pubkey: &solana_sdk::pubkey::Pubkey,
    ) -> KeeperResult<Option<solana_sdk::account::Account>> {
        let pubkey = *pubkey;
        self.execute_with_retry(move |client| {
            client.get_account(&pubkey)
                .map_err(|e| KeeperError::RpcError(format!("Failed to get account: {}", e)))
                .map(Some)
        }).await.or_else(|e| {
            // Handle account not found as None instead of error
            if e.to_string().contains("AccountNotFound") {
                Ok(None)
            } else {
                Err(e)
            }
        })
    }

    pub async fn get_multiple_accounts(
        &self,
        pubkeys: &[solana_sdk::pubkey::Pubkey],
    ) -> KeeperResult<Vec<Option<solana_sdk::account::Account>>> {
        let pubkeys = pubkeys.to_vec();
        self.execute_with_retry(move |client| {
            client.get_multiple_accounts(&pubkeys)
                .map_err(|e| KeeperError::RpcError(format!("Failed to get multiple accounts: {}", e)))
        }).await
    }

    pub async fn get_transaction_count(&self) -> KeeperResult<u64> {
        self.execute_with_retry(|client| {
            client.get_transaction_count()
                .map_err(|e| KeeperError::RpcError(format!("Failed to get transaction count: {}", e)))
        }).await
    }

    pub async fn get_balance(
        &self,
        pubkey: &solana_sdk::pubkey::Pubkey,
    ) -> KeeperResult<u64> {
        let pubkey = *pubkey;
        self.execute_with_retry(move |client| {
            client.get_balance(&pubkey)
                .map_err(|e| KeeperError::RpcError(format!("Failed to get balance: {}", e)))
        }).await
    }
}