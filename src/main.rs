mod block;
mod transaction;
mod blockchain;
mod wallet; // wallet 모듈 추가

use blockchain::Blockchain;
use transaction::Transaction;
use wallet::Wallet; // Wallet 구조체 사용

fn main() {
    println!("Creating a new blockchain with difficulty 2...");
    let mut my_blockchain = Blockchain::new(2); // 난이도 2로 블록체인 생성

    // --- 지갑 생성 ---
    println!("\n--- Creating Wallets ---");
    let wallet_alice = Wallet::new();
    let wallet_bob = Wallet::new();
    let wallet_charlie = Wallet::new();
    let wallet_miner = Wallet::new(); // 채굴자 지갑

    println!("Alice's address: {}", wallet_alice.get_address());
    println!("Bob's address: {}", wallet_bob.get_address());
    println!("Charlie's address: {}", wallet_charlie.get_address());
    println!("Miner's address: {}", wallet_miner.get_address());

    // --- 첫 번째 채굴 (채굴자에게 보상 지급) ---
    println!("\n--- Mining Block 1 (Initial Reward) ---");
    // mine_pending_transactions 함수의 시작 부분에 있는 `if self.pending_transactions.is_empty() { return; }`
    // 이 부분을 주석 처리하거나 제거해야 첫 채굴 시 보상 트랜잭션만으로 블록이 생성됩니다.
    // 현재 제공된 blockchain.rs 코드에는 이 부분이 있으니 유의해주세요.
    // 여기서는 일단 이 부분을 제거했다고 가정하고 진행합니다.

    // 첫 블록 채굴 (채굴자에게 초기 보상 지급)
    my_blockchain.mine_pending_transactions(wallet_miner.get_address().to_string());
    println!("Miner's balance after first block: {}", my_blockchain.get_balance_of_address(wallet_miner.get_address()));
    println!("Is blockchain valid? {}", my_blockchain.is_chain_valid());


    // --- 트랜잭션 생성 및 서명 ---
    println!("\n--- Creating and Signing Transactions ---");

    // 앨리스 -> 밥 10 코인 전송
    let mut tx1 = Transaction::new(
        wallet_alice.get_address().to_string(),
        wallet_bob.get_address().to_string(),
        10,
    );
    tx1.sign(wallet_alice.get_private_key(), wallet_alice.get_address().to_string());
    println!("Transaction 1 (Alice -> Bob) signed: {:?}", tx1);
    my_blockchain.add_transaction(tx1);

    // 밥 -> 찰리 5 코인 전송 (아직 밥은 코인이 없음, 잔액 부족으로 is_valid()에서 실패할 수 있음)
    let mut tx2 = Transaction::new(
        wallet_bob.get_address().to_string(),
        wallet_charlie.get_address().to_string(),
        5,
    );
    tx2.sign(wallet_bob.get_private_key(), wallet_bob.get_address().to_string());
    println!("Transaction 2 (Bob -> Charlie) signed: {:?}", tx2);
    my_blockchain.add_transaction(tx2); // 이 트랜잭션은 is_valid()에서 실패할 수 있습니다. (잔액 부족)

    // --- 두 번째 채굴 (트랜잭션 처리 및 채굴자 보상) ---
    println!("\n--- Mining Block 2 ---");
    my_blockchain.mine_pending_transactions(wallet_miner.get_address().to_string());

    println!("\n--- Balances after Block 2 ---");
    println!("Miner's balance: {}", my_blockchain.get_balance_of_address(wallet_miner.get_address()));
    println!("Alice's balance: {}", my_blockchain.get_balance_of_address(wallet_alice.get_address()));
    println!("Bob's balance: {}", my_blockchain.get_balance_of_address(wallet_bob.get_address()));
    println!("Charlie's balance: {}", my_blockchain.get_balance_of_address(wallet_charlie.get_address()));

    println!("\nIs blockchain valid? {}", my_blockchain.is_chain_valid());

    // --- 잔액 부족 트랜잭션 시도 및 유효성 검사 ---
    println!("\n--- Attempting transaction with insufficient balance ---");
    let mut tx3 = Transaction::new(
        wallet_miner.get_address().to_string(),
        wallet_alice.get_address().to_string(),
        500, // 채굴자가 가진 것보다 많은 금액
    );
    tx3.sign(wallet_miner.get_private_key(), wallet_miner.get_address().to_string());
    println!("Transaction 3 (Miner -> Alice) signed: {:?}", tx3);
    // 이 트랜잭션은 is_valid() 자체는 통과하지만, 실제 잔액 검사는 get_balance_of_address에서만 이루어짐.
    // add_transaction에서 잔액 검사를 추가해야 함 (다음 단계 실습으로 고려).
    my_blockchain.add_transaction(tx3);

    println!("\n--- Mining Block 3 ---");
    my_blockchain.mine_pending_transactions(wallet_bob.get_address().to_string()); // 밥이 채굴하여 보상 받음

    println!("\n--- Balances after Block 3 ---");
    println!("Miner's balance: {}", my_blockchain.get_balance_of_address(wallet_miner.get_address()));
    println!("Alice's balance: {}", my_blockchain.get_balance_of_address(wallet_alice.get_address()));
    println!("Bob's balance: {}", my_blockchain.get_balance_of_address(wallet_bob.get_address()));
    println!("Charlie's balance: {}", my_blockchain.get_balance_of_address(wallet_charlie.get_address()));

    println!("\nIs blockchain valid? {}", my_blockchain.is_chain_valid());

    // --- 체인 조작 시도 (유효성 검사 실패 예시) ---
    println!("\n--- Attempting to tamper with the blockchain ---");
    if let Some(block_to_tamper) = my_blockchain.chain.get_mut(1) { // 1번 블록 (제네시스 다음)
        // 트랜잭션 금액을 변경하고 해시를 다시 계산
        // 하지만 nonce는 변경되지 않았으므로 난이도 조건을 만족하지 못해 is_chain_valid()가 false를 반환할 것
        block_to_tamper.transactions[1].amount = 9999; // Alice -> Bob 트랜잭션 금액 조작
        block_to_tamper.hash = block_to_tamper.calculate_hash();
        println!("Block 1 tampered! New hash: {}", block_to_tamper.hash);
    }
    println!("\nIs blockchain valid after tampering? {}", my_blockchain.is_chain_valid());

    // --- 조작된 트랜잭션의 서명 변경 시도 (is_valid() 실패 예시) ---
    println!("\n--- Attempting to tamper with a transaction's signature ---");
    if let Some(block_to_tamper) = my_blockchain.chain.get_mut(2) { // 2번 블록 (두 번째 채굴 블록)
        if block_to_tamper.transactions.len() > 1 {
            // 두 번째 트랜잭션 (Alice -> Bob)의 서명을 무효화
            block_to_tamper.transactions[1].signature = "INVALID_SIGNATURE".to_string();
            // 블록 해시도 다시 계산해야 함 (실제로는 Nonce도 바뀌어야 함)
            block_to_tamper.hash = block_to_tamper.calculate_hash();
            println!("Transaction in Block 2 tampered! Signature invalidated.");
        }
    }
    println!("\nIs blockchain valid after transaction tampering? {}", my_blockchain.is_chain_valid());
}