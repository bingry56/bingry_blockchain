// src/wallet.rs
use k256::ecdsa::{SigningKey, VerifyingKey};
#[allow(unused_imports)]
use k256::elliptic_curve::sec1::ToEncodedPoint; // 이 임포트는 여기서 사용됩니다!
use rand_core::OsRng; // 운영체제 기반 난수 생성기
use hex::encode; // 16진수 인코딩


#[derive(Debug, Clone)]
pub struct Wallet {
    pub private_key: SigningKey, // 개인 키
    pub public_key: VerifyingKey, // 공개 키
    pub address: String, // 공개 키의 16진수 표현 (블록체인 주소로 사용)
}

impl Wallet {
    /// 새로운 지갑을 생성합니다 (새로운 개인 키/공개 키 쌍 생성).
    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng); // 안전한 난수로 개인 키 생성
        let public_key = VerifyingKey::from(&private_key); // 개인 키로부터 공개 키 파생

        // 공개 키를 압축된 SECP256K1 형식의 16진수 문자열로 변환하여 주소로 사용
        let address = encode(public_key.to_encoded_point(true).as_bytes());

        Wallet {
            private_key,
            public_key,
            address,
        }
    }

    /// 지갑의 개인 키를 반환합니다.
    pub fn get_private_key(&self) -> &SigningKey {
        &self.private_key
    }

    /// 지갑의 공개 키를 반환합니다.
    pub fn get_public_key(&self) -> &VerifyingKey {
        &self.public_key
    }

    /// 지갑의 주소(공개 키의 16진수 표현)를 반환합니다.
    pub fn get_address(&self) -> &str {
        &self.address
    }
}
