// src/server.rs
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use serde_json;
use std::sync::{Arc, Mutex};

// 라이브러리 크레이트에서 모듈들을 가져옵니다.
use crate::{blockchain, transaction, wallet}; 
use blockchain::Blockchain;
use transaction::Transaction;
use wallet::Wallet;

// 클라이언트와 서버 간 메시지 유형 정의
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    AddTransaction(Transaction),
    MineBlock(String), // miner_address
    GetBalance(String), // address
    GetChain,
    GenerateWallet,
    // 필요에 따라 다른 요청 추가
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Success(String),
    Blockchain(Blockchain),
    Balance(u64),
    Wallet(Wallet),
    Error(String),
    // 필요에 따라 다른 응답 추가
}

// #[tokio::main] // 이 매크로를 제거합니다.
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("블록체인 서버가 127.0.0.1:8080에서 실행 중입니다.");

    let blockchain = Arc::new(Mutex::new(Blockchain::new(2)));

    loop {
        let (socket, addr) = listener.accept().await?;
        let blockchain_clone = Arc::clone(&blockchain);

        println!("새 클라이언트 연결: {}", addr);
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, blockchain_clone).await {
                eprintln!("클라이언트 처리 오류: {:?}", e);
            }
        });
    }
}

async fn handle_client(
    mut socket: TcpStream,
    blockchain: Arc<Mutex<Blockchain>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut total_bytes_read = 0;

    loop {
        let mut len_bytes = [0u8; 4];
        if socket.read_exact(&mut len_bytes).await.is_err() {
            break;
        }
        let msg_len = u32::from_be_bytes(len_bytes) as usize;

        if msg_len == 0 {
            break;
        }

        buffer.resize(msg_len, 0);
        if socket.read_exact(&mut buffer).await.is_err() {
            break;
        }

        let request: Request = match serde_json::from_slice(&buffer) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("요청 역직렬화 오류: {:?}", e);
                let err_resp = Response::Error(format!("Invalid request format: {}", e));
                let resp_json = serde_json::to_vec(&err_resp)?;
                socket.write_all(&(resp_json.len() as u32).to_be_bytes()).await?;
                socket.write_all(&resp_json).await?;
                continue;
            }
        };

        println!("수신된 요청: {:?}", request);

        let response = match request {
            Request::AddTransaction(tx) => {
                let mut bc = blockchain.lock().unwrap();
                if bc.get_balance_of_address(&tx.sender) < tx.amount {
                    Response::Error(format!("잔액 부족: 송신자 {}의 잔액은 {} 코인입니다. 요청된 금액: {}", 
                                            tx.sender, bc.get_balance_of_address(&tx.sender), tx.amount))
                } else {
                    bc.add_transaction(tx);
                    Response::Success("트랜잭션이 대기열에 추가되었습니다.".to_string())
                }
            }
            Request::MineBlock(miner_address) => {
                let mut bc = blockchain.lock().unwrap();
                let prev_chain_len = bc.chain.len();
                bc.mine_pending_transactions(miner_address.clone());
                if bc.chain.len() > prev_chain_len {
                    Response::Success(format!("새 블록이 채굴되어 블록체인에 추가되었습니다. 채굴자: {}", miner_address))
                } else {
                    Response::Success("채굴할 대기 중인 트랜잭션이 없어 새 블록이 생성되지 않았습니다.".to_string())
                }
            }
            Request::GetBalance(address) => {
                let bc = blockchain.lock().unwrap();
                let balance = bc.get_balance_of_address(&address);
                Response::Balance(balance)
            }
            Request::GetChain => {
                let bc = blockchain.lock().unwrap();
                Response::Blockchain(bc.clone())
            }
            Request::GenerateWallet => {
                let wallet = Wallet::new();
                Response::Wallet(wallet)
            }
        };

        let resp_json = serde_json::to_vec(&response)?;
        socket.write_all(&(resp_json.len() as u32).to_be_bytes()).await?;
        socket.write_all(&resp_json).await?;
    }

    println!("클라이언트 연결 종료.");
    Ok(())
}
