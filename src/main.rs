// src/main.rs

// 다른 모듈들을 가져옵니다.
mod blockchain;
mod block;
mod transaction;
mod wallet;
mod server; // 서버 모듈 추가

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::run_server().await // 서버 실행
}