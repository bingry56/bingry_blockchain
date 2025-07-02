// src/blockchain.rs
use super::block::Block;           // block 모듈의 Block 구조체 사용
use super::transaction::Transaction; // transaction 모듈의 Transaction 구조체 사용

// Blockchain 내부에서 필요한 sha256, chrono도 여기서 use 해줍니다.
use sha256::digest;
use chrono::Utc;
use serde::{Serialize, Deserialize}; // Serialize, Deserialize 트레이트 추가


#[derive(Debug, Clone, Serialize, Deserialize)] // Clone, Serialize, Deserialize 추가
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: u64,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty,
            pending_transactions: Vec::new(),
            mining_reward: 100,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let mut genesis_block = Block::new(0, "0".to_string(), vec![]);
        // mine_block 내부에서 해시를 계산하고 설정해야 하지만,
        // 사용자 코드의 흐름을 유지하기 위해 calculate_hash 호출을 남겨둡니다.
        genesis_block.mine_block(self.difficulty);
        genesis_block.hash = genesis_block.calculate_hash();
        self.chain.push(genesis_block);
        println!("Genesis block created: {:?}", self.chain[0]);
    }

    fn add_block(&mut self, mut new_block: Block) {
        let latest_block = self.get_latest_block().unwrap();
        new_block.previous_hash = latest_block.hash.clone();
        new_block.mine_block(self.difficulty);
        new_block.hash = new_block.calculate_hash(); // calculate_hash 호출 유지
        self.chain.push(new_block);
        println!("New block added: {:?}", self.get_latest_block());
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        // 트랜잭션 유효성 검사 (is_valid()는 transaction.rs에 정의되어야 함)
        if !transaction.is_valid() {
            println!("Error: Invalid transaction added to pending pool.");
            return;
        }
        self.pending_transactions.push(transaction);
        println!("Transaction added to pending pool: {:?}", self.pending_transactions.last());
    }

    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        if self.pending_transactions.is_empty() {
            println!("No pending transactions to mine.");
            return;
        }

        // 채굴 보상 트랜잭션 생성
        let reward_tx = Transaction::new("coinbase_reward".to_string(), miner_address, self.mining_reward);
        
        // 보상 트랜잭션을 포함하여 모든 대기 중인 트랜잭션을 블록에 추가
        let mut transactions_to_mine = vec![reward_tx];
        transactions_to_mine.extend(self.pending_transactions.drain(..)); // pending_transactions 비움

        let new_block_index = self.chain.len() as u64;
        let latest_block_hash = self.get_latest_block().unwrap().hash.clone();

        let mut new_block = Block::new(new_block_index, latest_block_hash, transactions_to_mine);
        self.add_block(new_block);
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// 주어진 주소의 잔액을 계산합니다.
    /// 트랜잭션 금액은 u64 타입으로 가정하며, saturating_sub/saturating_add를 사용합니다.
    pub fn get_balance_of_address(&self, address: &str) -> u64 {
        let mut balance: u64 = 0; // 잔액을 u64로 명시적으로 선언

        for block in self.chain.iter() {
            for tx in block.transactions.iter() {
                // tx.amount를 u64로 명시적 캐스팅하여 타입 불일치 오류 방지
                // 만약 tx.amount가 이미 u64라면 이 캐스팅은 최적화됩니다.
                let amount_u64 = tx.amount as u64;

                // 코인베이스 보상 트랜잭션은 송신자가 없으므로 잔액에서 빼지 않습니다.
                // 이 부분은 transaction.rs의 is_valid 로직과 일관성을 유지해야 합니다.
                if tx.sender == address && tx.sender != "coinbase_reward" {
                    // 잔액에서 송금액을 안전하게 뺍니다 (언더플로우 방지)
                    balance = balance.saturating_sub(amount_u64);
                }
                if tx.recipient == address {
                    // 잔액에 수금액을 안전하게 더합니다 (오버플로우 방지)
                    balance = balance.saturating_add(amount_u64);
                }
            }
        }
        balance
    }

    /// 블록체인의 유효성을 검사합니다.
    /// 각 블록의 해시와 이전 블록 해시가 올바른지, 난이도 조건을 충족하는지,
    /// 그리고 블록 내 모든 트랜잭션이 유효한지 확인합니다.
    pub fn is_chain_valid(&self) -> bool {
        // 제네시스 블록은 이전 블록이 없으므로 인덱스 1부터 시작
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // 1. 현재 블록의 해시가 올바르게 계산되었는지 확인
            if current_block.hash != current_block.calculate_hash() {
                println!("Invalid block hash at index {}. Expected: {}, Got: {}",
                         i, current_block.calculate_hash(), current_block.hash);
                return false;
            }

            // 2. 현재 블록의 previous_hash가 이전 블록의 실제 해시와 일치하는지 확인
            if current_block.previous_hash != previous_block.hash {
                println!("Invalid previous hash at index {}. Expected: {}, Got: {}",
                         i, previous_block.hash, current_block.previous_hash);
                return false;
            }

            // 3. 현재 블록의 해시가 난이도 조건을 충족하는지 확인
            let target_prefix = "0".repeat(self.difficulty);
            if !current_block.hash.starts_with(&target_prefix) {
                println!("Block {} does not meet difficulty requirement. Hash: {}", i, current_block.hash);
                return false;
            }

            // 4. 블록 내 모든 트랜잭션이 유효한지 확인
            for tx in current_block.transactions.iter() {
                if !tx.is_valid() {
                    // 유효하지 않은 트랜잭션이 발견되면 해당 블록과 트랜잭션의 위치를 출력
                    // calculate_hash_for_signing()을 사용하여 트랜잭션의 고유 식별자를 얻습니다.
                    let tx_index = current_block.transactions.iter().position(|t| t.calculate_hash_for_signing() == tx.calculate_hash_for_signing()).unwrap_or(0);
                    println!("Invalid transaction in block {} at index {}. Transaction: {:?}", i, tx_index, tx);
                    return false;
                }
            }
        }
        true // 모든 검사를 통과하면 유효
    }
}
