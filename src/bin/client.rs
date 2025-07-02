// src/bin/client.rs
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde::{Serialize, Deserialize};
use serde_json;
use std::io::{self, BufRead, Write};
use std::collections::HashMap;

// 라이브러리 크레이트에서 모듈들을 가져옵니다.
// `bingry_blockchain_lib`는 Cargo.toml에 정의된 라이브러리 이름입니다.
use bingry_blockchain_lib::{blockchain, transaction, wallet}; // server는 client에서 직접 사용하지 않으므로 제거

use blockchain::Blockchain;
use transaction::Transaction;
use wallet::Wallet;

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

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("서버에 연결되었습니다.");

    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    let mut wallets: HashMap<String, Wallet> = HashMap::new();

    loop {
        println!("\n--- 메뉴 ---");
        println!("1. 지갑 생성");
        println!("2. 트랜잭션 추가 (코인 전송)");
        println!("3. 블록 채굴");
        println!("4. 잔액 조회");
        println!("5. 블록체인 전체 조회");
        println!("6. 종료");
        print!("선택: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        reader.read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => {
                let request = Request::GenerateWallet;
                let response = send_request(&mut stream, request).await?;
                if let Response::Wallet(wallet) = response {
                    println!("새 지갑이 생성되었습니다: {}", wallet.get_address());
                    wallets.insert(wallet.get_address().to_string(), wallet);
                } else {
                    println!("오류: {:?}", response);
                }
            }
            "2" => {
                if wallets.is_empty() {
                    println!("지갑이 없습니다. 먼저 지갑을 생성해주세요.");
                    continue;
                }
                println!("--- 트랜잭션 생성 ---");
                println!("사용 가능한 지갑:");
                for (addr, _) in &wallets {
                    println!("- {}", addr);
                }

                print!("송신자 주소 입력: ");
                io::stdout().flush()?;
                let mut sender_addr = String::new();
                reader.read_line(&mut sender_addr)?;
                let sender_addr = sender_addr.trim().to_string();

                let sender_wallet = match wallets.get(&sender_addr) {
                    Some(w) => w,
                    None => {
                        println!("오류: 유효하지 않은 송신자 주소입니다.");
                        continue;
                    }
                };

                print!("수신자 주소 입력: ");
                io::stdout().flush()?;
                let mut recipient_addr = String::new();
                reader.read_line(&mut recipient_addr)?;
                let recipient_addr = recipient_addr.trim().to_string();

                print!("금액 입력: ");
                io::stdout().flush()?;
                let mut amount_str = String::new();
                reader.read_line(&mut amount_str)?;
                let amount: u64 = match amount_str.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("오류: 유효하지 않은 금액입니다.");
                        continue;
                    }
                };

                let mut tx = Transaction::new(sender_addr.clone(), recipient_addr, amount);
                let signing_key = sender_wallet.to_signing_key().expect("Failed to get signing key");
                tx.sign(&signing_key, sender_wallet.get_address().to_string());

                let request = Request::AddTransaction(tx);
                let response = send_request(&mut stream, request).await?;
                println!("응답: {:?}", response);
            }
            "3" => {
                if wallets.is_empty() {
                    println!("지갑이 없습니다. 먼저 지갑을 생성해주세요.");
                    continue;
                }
                println!("--- 블록 채굴 ---");
                println!("채굴 보상을 받을 채굴자 주소를 선택하세요:");
                for (addr, _) in &wallets {
                    println!("- {}", addr);
                }
                print!("채굴자 주소 입력: ");
                io::stdout().flush()?;
                let mut miner_addr = String::new();
                reader.read_line(&mut miner_addr)?;
                let miner_addr = miner_addr.trim().to_string();

                if !wallets.contains_key(&miner_addr) {
                    println!("오류: 유효하지 않은 채굴자 주소입니다.");
                    continue;
                }

                let request = Request::MineBlock(miner_addr);
                let response = send_request(&mut stream, request).await?;
                println!("응답: {:?}", response);
            }
            "4" => {
                if wallets.is_empty() {
                    println!("지갑이 없습니다. 먼저 지갑을 생성해주세요.");
                    continue;
                }
                println!("--- 잔액 조회 ---");
                println!("잔액을 조회할 주소를 선택하세요:");
                for (addr, _) in &wallets {
                    println!("- {}", addr);
                }
                print!("주소 입력: ");
                io::stdout().flush()?;
                let mut address = String::new();
                reader.read_line(&mut address)?;
                let address = address.trim().to_string();

                if !wallets.contains_key(&address) {
                    println!("오류: 유효하지 않은 주소입니다.");
                    continue;
                }

                let request = Request::GetBalance(address);
                let response = send_request(&mut stream, request).await?;
                println!("응답: {:?}", response);
            }
            "5" => {
                println!("--- 블록체인 전체 조회 ---");
                let request = Request::GetChain;
                let response = send_request(&mut stream, request).await?;
                if let Response::Blockchain(bc) = response {
                    println!("{:#?}", bc);
                } else {
                    println!("오류: {:?}", response);
                }
            }
            "6" => {
                println!("클라이언트 종료.");
                break;
            }
            _ => {
                println!("잘못된 선택입니다. 다시 시도해주세요.");
            }
        }
    }

    Ok(())
}

async fn send_request(stream: &mut TcpStream, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
    let req_json = serde_json::to_vec(&request)?;
    stream.write_all(&(req_json.len() as u32).to_be_bytes()).await?;
    stream.write_all(&req_json).await?;

    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let resp_len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; resp_len];
    stream.read_exact(&mut buffer).await?;

    let response: Response = serde_json::from_slice(&buffer)?;
    Ok(response)
}
