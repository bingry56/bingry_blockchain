// src/lib.rs

// 다른 모듈들을 pub으로 선언하여 외부에서 접근 가능하게 합니다.
pub mod blockchain;
pub mod block;
pub mod transaction;
pub mod wallet;
pub mod server; // server 모듈도 lib.rs에 포함시킵니다.

// 이제 각 모듈의 내용은 해당 파일(src/blockchain.rs, src/block.rs 등)에 그대로 유지됩니다.
// 이 파일은 단순히 모듈들을 묶어주는 역할을 합니다.
