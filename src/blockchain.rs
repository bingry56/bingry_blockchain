// src/blockchain.rs
use super::block::Block;
use super::transaction::Transaction;

// 사용되지 않는 임포트 제거
// use sha256::digest;
// use chrono::Utc;


#[derive(Debug)]
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
        genesis_block.mine_block(self.difficulty);
        genesis_block.hash = genesis_block.calculate_hash();
        self.chain.push(genesis_block);
        println!("Genesis block created: {:?}", self.chain[0]);
    }

    fn add_block(&mut self, mut new_block: Block) { // `new_block`은 내부에서 수정되므로 `mut` 유지
        let latest_block = self.get_latest_block().unwrap();
        new_block.previous_hash = latest_block.hash.clone();
        new_block.mine_block(self.difficulty);
        new_block.hash = new_block.calculate_hash();
        self.chain.push(new_block);
        println!("New block added: {:?}", self.get_latest_block());
    }

    /// 트랜잭션을 대기 중인 풀에 추가합니다.
    /// 트랜잭션은 반드시 서명되어 있어야 합니다.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        // 코인베이스 보상 트랜잭션이 아니면서 서명이 없는 경우 거부
        if transaction.sender != "coinbase_reward" && transaction.signature.is_empty() {
            println!("Error: Cannot add unsigned transaction to pending pool.");
            return;
        }

        if !transaction.is_valid() {
            println!("Error: Invalid transaction added to pending pool.");
            return;
        }
        self.pending_transactions.push(transaction);
        println!("Transaction added to pending pool: {:?}", self.pending_transactions.last());
    }

    /// 대기 중인 트랜잭션을 채굴하여 새 블록에 추가합니다.
    /// 채굴자에게 보상 트랜잭션을 발행합니다.
    pub fn mine_pending_transactions(&mut self, miner_address: String) {
        // NOTE: pending_transactions가 비어있어도 블록을 채굴하고 싶다면
        // (예: 코인베이스 보상 블록 발행), 다음 `if` 문을 제거하거나 수정해야 합니다.
        if self.pending_transactions.is_empty() {
            println!("No pending transactions to mine.");
            return;
        }

        // 채굴 보상 트랜잭션 생성 (서명 필요 없음)
        let reward_tx = Transaction::new("coinbase_reward".to_string(), miner_address.clone(), self.mining_reward);
        
        let mut transactions_to_mine = vec![reward_tx];
        // 대기 중인 모든 트랜잭션을 새 블록에 포함
        transactions_to_mine.extend(self.pending_transactions.drain(..));

        let new_block_index = self.chain.len() as u64;
        let latest_block_hash = self.get_latest_block().unwrap().hash.clone();

        let new_block = Block::new(new_block_index, latest_block_hash, transactions_to_mine); // `mut` 제거
        self.add_block(new_block);
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// 주어진 주소의 잔액을 계산합니다.
    /// 트랜잭션 금액은 u64 타입으로 가정하며, saturating_sub/saturating_add를 사용합니다.
    pub fn get_balance_of_address(&self, address: &str) -> u64 {
        let mut balance: u64 = 0;

        for block in self.chain.iter() {
            for tx in block.transactions.iter() {
                let amount_u64 = tx.amount as u64; // u64로 명시적 캐스팅

                // 송신자가 주소와 일치하고, 코인베이스 보상 트랜잭션이 아닌 경우 잔액에서 뺍니다.
                if tx.sender == address && tx.sender != "coinbase_reward" {
                    balance = balance.saturating_sub(amount_u64);
                }
                // 수신자가 주소와 일치하는 경우 잔액에 더합니다.
                if tx.recipient == address {
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
                    let tx_index = current_block.transactions.iter().position(|t| t.calculate_hash_for_signing() == tx.calculate_hash_for_signing()).unwrap_or(0);
                    println!("Invalid transaction in block {} at index {}. Transaction: {:?}", i, tx_index, tx);
                    return false;
                }
            }
        }
        true
    }
}
