// src/transaction.rs
use chrono::Utc;
use k256::ecdsa::{SigningKey, VerifyingKey, Signature};
// `ToEncodedPoint`는 `wallet.rs`에서 주소 생성에 사용되며, `transaction.rs`에서는 직접 사용되지 않습니다.
// 다른 곳에서 올바르게 사용된다고 가정하고 여기서는 사용되지 않는 임포트 경고를 피하기 위해 주석 처리합니다.
// use k256::elliptic_curve::sec1::ToEncodedPoint;
use hex::{encode, decode};

// 암호화 트레이트의 올바른 임포트
// PrehashSigner와 PrehashVerifier는 `hazmat` (위험 물질) 모듈에 있습니다.
use ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
// `DigestSigner`와 `DigestVerifier`는 미리 해시된 서명/검증을 사용할 때는 사용되지 않습니다.
// `sha2::Sha256` 및 `digest::Digest`는 미리 해시된 데이터를 서명/검증하는 데 직접 사용되지 않습니다.
// 메시지 해싱에는 `sha256` 크레이트의 `sha256::digest`가 사용됩니다.

use generic_array::{GenericArray, typenum::U64, typenum::Unsigned}; // Unsigned 트레이트 추가
use core::convert::TryFrom; // TryFrom 트레이트 추가

#[derive(Debug, Clone)]
pub struct Transaction {
    pub sender: String,    // 송신자 주소 (공개 키의 16진수 표현)
    pub recipient: String, // 수신자 주소 (공개 키의 16진수 표현)
    pub amount: u64,       // 금액
    pub timestamp: i64,
    pub public_key: String, // 송신자의 공개 키 (16진수 표현)
    pub signature: String,  // 트랜잭션 서명 (16진수 표현)
}

impl Transaction {
    /// 새로운 트랜잭션을 생성합니다. (서명 전)
    pub fn new(sender: String, recipient: String, amount: u64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            timestamp: Utc::now().timestamp(),
            public_key: String::new(), // 서명 시 설정
            signature: String::new(),   // 서명 시 설정
        }
    }

    /// 트랜잭션 서명에 사용될 해시 데이터를 계산합니다.
    /// 이 해시는 트랜잭션의 고유한 식별자이자 서명의 대상이 됩니다.
    pub fn calculate_hash_for_signing(&self) -> String {
        let data = format!(
            "{}{}{}{}",
            self.sender, self.recipient, self.amount, self.timestamp
        );
        sha256::digest(data) // sha256 크레이트의 digest 함수 사용
    }

    /// 트랜잭션에 서명합니다.
    /// `signing_key`: 송신자의 개인 키
    /// `public_key_hex`: 송신자의 공개 키 (16진수 문자열)
    pub fn sign(&mut self, signing_key: &SigningKey, public_key_hex: String) {
        // 코인베이스 보상 트랜잭션은 서명하지 않습니다.
        if self.sender == "coinbase_reward" {
            self.public_key = public_key_hex; // 채굴자 주소를 public_key로 사용 (실제 키는 아님)
            self.signature = "UNSIGNED_COINBASE_TX".to_string(); // 서명 없음 표시
            return;
        }

        // 서명할 데이터의 해시를 바이트 배열로 변환
        let message_hash_str = self.calculate_hash_for_signing();
        let message_bytes = decode(&message_hash_str).expect("Failed to decode message hash hex");

        // `sign_prehash`는 이미 해시된 메시지 바이트를 받습니다.
        let signature: Signature = signing_key.sign_prehash(&message_bytes).expect("Failed to sign prehash");
        
        // 서명을 16진수 문자열로 인코딩하여 저장
        self.signature = encode(signature.to_bytes());
        self.public_key = public_key_hex; // 송신자의 공개 키 저장
    }

    /// 트랜잭션의 유효성을 검사합니다.
    /// - 해시가 올바른지 (이 함수에서는 서명 검증이 주 목적이므로 해시는 별도로 계산하지 않음)
    /// - 송신자, 수신자, 금액이 유효한지
    /// - 서명이 올바른지 (코인베이스 보상 트랜잭션 제외)
    pub fn is_valid(&self) -> bool {
        // 코인베이스 보상 트랜잭션은 특별히 처리 (서명 검증 없음)
        if self.sender == "coinbase_reward" {
            return self.signature == "UNSIGNED_COINBASE_TX" && !self.recipient.is_empty() && self.amount > 0;
        }

        // 일반 트랜잭션 유효성 검사
        if self.public_key.is_empty() || self.signature.is_empty() {
            println!("Error: Transaction is not signed or public key is missing.");
            return false;
        }
        if self.amount <= 0 {
            println!("Error: Transaction amount must be positive.");
            return false;
        }
        if self.sender.is_empty() || self.recipient.is_empty() {
            println!("Error: Sender or recipient address is empty.");
            return false;
        }
        // 송신자 주소가 공개 키와 일치하는지 확인
        if self.sender != self.public_key {
            println!("Error: Sender address does not match public key.");
            return false;
        }

        // 공개 키와 서명을 바이트 배열로 디코딩
        let public_key_bytes = match decode(&self.public_key) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("Error: Invalid public key hex format.");
                return false;
            }
        };
        let signature_bytes_vec = match decode(&self.signature) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("Error: Invalid signature hex format.");
                return false;
            }
        };

        // VerifyingKey 생성
        let verifying_key = match VerifyingKey::from_sec1_bytes(&public_key_bytes) {
            Ok(key) => key,
            Err(e) => {
                println!("Error: Could not create verifying key from public key bytes: {:?}", e);
                return false;
            }
        };

        // Signature 생성: `Signature::try_from`은 `&[u8]` 슬라이스를 기대합니다.
        // `signature_bytes_vec`의 길이가 올바른지 먼저 확인하고, 안전하게 `Signature`로 변환합니다.
        if signature_bytes_vec.len() != U64::USIZE { // U64::USIZE는 Unsigned 트레이트에서 가져옴
            println!("Error: Signature bytes have incorrect length (expected {} bytes). Actual: {}", U64::USIZE, signature_bytes_vec.len());
            return false;
        }

        let signature = match Signature::try_from(signature_bytes_vec.as_slice()) {
            Ok(sig) => sig,
            Err(e) => {
                println!("Error: Could not create signature from bytes: {:?}", e);
                return false;
            }
        };

        // 서명할 원본 데이터의 해시를 다시 계산합니다.
        let original_signing_data = format!(
            "{}{}{}{}",
            self.sender, self.recipient, self.amount, self.timestamp
        );
        
        let message_hash_for_verification = decode(&sha256::digest(original_signing_data))
            .expect("Failed to decode message hash hex for verification");

        // 서명 검증: `verify_prehash` 사용
        if verifying_key.verify_prehash(&message_hash_for_verification, &signature).is_ok() {
            true
        } else {
            println!("Error: Invalid signature for transaction.");
            false
        }
    }
}
