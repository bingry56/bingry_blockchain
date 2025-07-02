// src/block.rs
use super::transaction::Transaction; // transaction 모듈의 Transaction 구조체 사용
use sha256::digest;
use chrono::Utc; // Utc는 타임스탬프에 필요하므로 유지

#[derive(Debug, Clone)] // Debug는 디버그 출력을 위해, Clone은 복사를 위해 필요
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64, // 채굴을 위한 Nonce 값
}

impl Block {
    /// 새로운 블록을 생성합니다.
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        Block {
            index,
            timestamp: Utc::now().timestamp(), // 현재 UTC 타임스탬프
            transactions,
            previous_hash,
            hash: String::new(), // 초기 해시는 비워둡니다. 채굴 시 계산됩니다.
            nonce: 0, // 초기 Nonce 값
        }
    }

    /// 블록의 해시를 계산합니다.
    pub fn calculate_hash(&self) -> String {
        let mut data = String::new();
        data.push_str(&self.index.to_string());
        data.push_str(&self.timestamp.to_string());
        data.push_str(&self.previous_hash);
        data.push_str(&self.nonce.to_string());

        // 트랜잭션 해시를 모두 연결하여 데이터에 포함
        // 트랜잭션의 서명용 해시를 사용하여 블록 해시에 포함시킵니다.
        for tx in self.transactions.iter() {
            data.push_str(&tx.calculate_hash_for_signing()); // tx.calculate_hash() 대신 calculate_hash_for_signing() 사용
        }

        sha256::digest(data) // sha256 해시 계산
    }

    /// 블록을 채굴합니다 (Proof of Work).
    /// 주어진 난이도에 맞는 해시를 찾을 때까지 Nonce 값을 증가시킵니다.
    pub fn mine_block(&mut self, difficulty: usize) {
        let target_prefix = "0".repeat(difficulty);
        println!("Mining block {} with difficulty {}...", self.index, difficulty);

        while !self.hash.starts_with(&target_prefix) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        println!("Block mined: {} with nonce {}", self.hash, self.nonce);
    }
}
