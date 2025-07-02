// src/wallet.rs
use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use rand_core::OsRng; // 운영체제 기반 난수 생성기
use hex::{encode, decode}; // 16진수 인코딩/디코딩
use serde::{Serialize, Deserialize}; // Serialize, Deserialize 트레이트 추가
use k256::elliptic_curve::SecretKey; // SecretKey를 사용하여 개인 키 바이트를 다룰 수 있습니다.

#[derive(Debug, Clone, Serialize, Deserialize)] // Serialize, Deserialize 추가
pub struct Wallet {
    // 개인 키를 직접 저장하는 대신, 16진수 문자열로 저장하여 직렬화 가능하게 합니다.
    pub private_key_hex: String,
    // 공개 키는 이미 address 필드에 16진수 문자열로 저장되어 있으므로 별도 필드는 필요 없습니다.
    pub address: String, // 공개 키의 16진수 표현 (블록체인 주소로 사용)
}

impl Wallet {
    /// 새로운 지갑을 생성합니다 (새로운 개인 키/공개 키 쌍 생성).
    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng); // 안전한 난수로 개인 키 생성
        let public_key = VerifyingKey::from(&private_key); // 개인 키로부터 공개 키 파생

        // 개인 키를 16진수 문자열로 변환하여 저장
        let private_key_hex = encode(private_key.to_bytes());

        // 공개 키를 압축된 SECP256K1 형식의 16진수 문자열로 변환하여 주소로 사용
        let address = encode(public_key.to_encoded_point(true).as_bytes());

        Wallet {
            private_key_hex,
            address,
        }
    }

    /// 지갑의 개인 키 16진수 문자열을 반환합니다.
    pub fn get_private_key_hex(&self) -> &str {
        &self.private_key_hex
    }

    /// 저장된 개인 키 16진수 문자열로부터 SigningKey 객체를 생성하여 반환합니다.
    /// 이 함수는 서명 작업 시에 호출됩니다.
    pub fn to_signing_key(&self) -> Result<SigningKey, hex::FromHexError> {
        let private_key_bytes = decode(&self.private_key_hex)?;
        // SecretKey::from_slice를 사용하여 바이트로부터 SecretKey를 생성합니다.
        // SigningKey는 SecretKey로부터 파생될 수 있습니다.
        Ok(SigningKey::from(SecretKey::from_slice(&private_key_bytes).expect("Invalid private key bytes")))
    }

    /// 저장된 공개 키(주소) 16진수 문자열로부터 VerifyingKey 객체를 생성하여 반환합니다.
    /// 이 함수는 서명 검증 작업 시에 호출됩니다.
    pub fn to_verifying_key(&self) -> Result<VerifyingKey, hex::FromHexError> {
        let public_key_bytes = decode(&self.address)?; // 주소는 공개 키의 16진수 표현입니다.
        Ok(VerifyingKey::from_sec1_bytes(&public_key_bytes).expect("Invalid public key bytes"))
    }

    /// 지갑의 주소(공개 키의 16진수 표현)를 반환합니다.
    pub fn get_address(&self) -> &str {
        &self.address
    }
}
