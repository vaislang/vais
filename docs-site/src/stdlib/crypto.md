# Crypto

## 개요

Crypto 모듈은 암호화 프리미티브를 제공합니다. SHA-256 해시, AES-256 암호화, HMAC 메시지 인증 코드를 포함하며, 교육 목적 구현입니다. **프로덕션에서는 감사된 라이브러리(OpenSSL, libsodium)를 사용하세요.**

## Quick Start

```vais
U std/crypto

F main() -> i64 {
    # SHA-256 해시
    hasher := Sha256::new()
    hasher.update("Hello, World!")
    digest := hasher.finalize()

    print_str("Hash: ~{digest}")
    R 0
}
```

## API 요약

### SHA-256

| 함수 | 설명 |
|------|------|
| `Sha256::new()` | 해시 컨텍스트 생성 |
| `update(data)` | 데이터 추가 (누적) |
| `finalize()` | 최종 해시 값 반환 (32바이트) |

### AES-256

| 함수 | 설명 |
|------|------|
| `Aes256::new(key)` | 256비트 키로 컨텍스트 생성 |
| `encrypt_block(plaintext)` | 16바이트 블록 암호화 |
| `decrypt_block(ciphertext)` | 16바이트 블록 복호화 |
| `encrypt_cbc(plaintext, iv)` | CBC 모드 암호화 |
| `decrypt_cbc(ciphertext, iv)` | CBC 모드 복호화 |

### HMAC

| 함수 | 설명 |
|------|------|
| `hmac_sha256(key, message)` | HMAC-SHA256 계산 |

## 실용 예제

### 예제 1: 파일 무결성 검증 (SHA-256)

```vais
U std/crypto
U std/file

F hash_file(path: i64) -> i64 {
    f := File::open_read(path)
    I !f.is_valid() { R 0 }

    hasher := Sha256::new()
    buffer := malloc(4096)

    L 1 {
        bytes := f.read_bytes(buffer, 4096)
        I bytes <= 0 { B }
        hasher.update_bytes(buffer, bytes)
    }

    f.close()
    R hasher.finalize()
}

F main() -> i64 {
    hash1 := hash_file("document.pdf")
    hash2 := hash_file("document_copy.pdf")

    I memcmp(hash1, hash2, 32) == 0 {
        print_str("파일 동일")
    } E {
        print_str("파일 다름")
    }
    R 0
}
```

### 예제 2: 비밀번호 해싱

```vais
U std/crypto
U std/random

F hash_password(password: i64, salt: i64) -> i64 {
    # Salt + Password 조합
    combined := malloc(strlen(salt) + strlen(password))
    memcpy(combined, salt, strlen(salt))
    memcpy(combined + strlen(salt), password, strlen(password))

    hasher := Sha256::new()
    hasher.update(combined)
    R hasher.finalize()
}

F verify_password(password: i64, salt: i64, stored_hash: i64) -> i64 {
    computed_hash := hash_password(password, salt)
    R memcmp(computed_hash, stored_hash, 32) == 0
}

F main() -> i64 {
    # Salt 생성 (16바이트 랜덤)
    salt := random_bytes(16)

    # 비밀번호 해싱
    password := "my_secure_password"
    hash := hash_password(password, salt)

    # 검증
    I verify_password(password, salt, hash) {
        print_str("비밀번호 일치")
    } E {
        print_str("비밀번호 불일치")
    }
    R 0
}
```

### 예제 3: AES-256 암호화/복호화

```vais
U std/crypto

F main() -> i64 {
    # 32바이트 키 (256비트)
    key := "01234567890123456789012345678901"

    # AES 컨텍스트 생성
    aes := Aes256::new(key)

    # 16바이트 평문
    plaintext := "Hello, World!!!!"
    ciphertext := aes.encrypt_block(plaintext)

    print_str("암호문: ~{hex_encode(ciphertext, 16)}")

    # 복호화
    decrypted := aes.decrypt_block(ciphertext)
    print_str("복호문: ~{decrypted}")  # "Hello, World!!!!"

    R 0
}
```

### 예제 4: CBC 모드 암호화 (긴 메시지)

```vais
U std/crypto

F encrypt_message(message: i64, key: i64) -> i64 {
    aes := Aes256::new(key)

    # 16바이트 IV (랜덤)
    iv := random_bytes(16)

    # PKCS7 패딩 추가
    padded := pkcs7_pad(message, 16)

    # CBC 암호화
    ciphertext := aes.encrypt_cbc(padded, iv)

    # IV + 암호문 결합
    result := malloc(16 + strlen(ciphertext))
    memcpy(result, iv, 16)
    memcpy(result + 16, ciphertext, strlen(ciphertext))

    R result
}

F decrypt_message(encrypted: i64, key: i64) -> i64 {
    aes := Aes256::new(key)

    # IV 추출
    iv := encrypted
    ciphertext := encrypted + 16

    # CBC 복호화
    padded := aes.decrypt_cbc(ciphertext, iv)

    # PKCS7 패딩 제거
    R pkcs7_unpad(padded)
}

F main() -> i64 {
    key := "01234567890123456789012345678901"
    message := "This is a long message that needs CBC mode."

    encrypted := encrypt_message(message, key)
    decrypted := decrypt_message(encrypted, key)

    print_str(decrypted)
    R 0
}
```

### 예제 5: HMAC 메시지 인증

```vais
U std/crypto
U std/net

F send_authenticated_message(socket: TcpStream, message: i64, secret: i64) -> i64 {
    # HMAC 계산
    mac := hmac_sha256(secret, message)

    # 메시지 + HMAC 전송
    socket.send(message, strlen(message))
    socket.send(mac, 32)

    R 0
}

F recv_authenticated_message(socket: TcpStream, secret: i64) -> i64 {
    # 메시지 수신
    message := malloc(1024)
    msg_len := socket.recv(message, 1024)

    # HMAC 수신
    received_mac := malloc(32)
    socket.recv(received_mac, 32)

    # HMAC 검증
    computed_mac := hmac_sha256(secret, message)

    I memcmp(received_mac, computed_mac, 32) != 0 {
        print_str("메시지 위조 감지!")
        R 0
    }

    print_str("메시지 검증 성공: ~{message}")
    R 1
}
```

## 주의사항

### 1. 교육 목적 구현
이 모듈은 학습용입니다. **프로덕션 환경에서는 검증된 라이브러리를 사용하세요:**
- OpenSSL (C/C++)
- libsodium (단순화된 암호화)
- RustCrypto (Rust)

### 2. 키 관리
암호화 키를 하드코딩하지 마세요. 환경 변수, 키 관리 시스템(KMS), 하드웨어 보안 모듈(HSM)을 사용하세요.

```vais
# 나쁜 예: 하드코딩
key := "my_secret_key_12345"

# 좋은 예: 환경 변수
key := getenv("ENCRYPTION_KEY")
I key == 0 {
    print_str("키가 설정되지 않음")
    exit(1)
}
```

### 3. IV (Initialization Vector) 재사용 금지
CBC/CTR 모드에서 같은 키와 IV를 재사용하면 패턴이 노출됩니다. 항상 랜덤 IV를 생성하세요.

```vais
# 나쁜 예: 고정 IV
iv := "0000000000000000"

# 좋은 예: 랜덤 IV
iv := random_bytes(16)
```

### 4. Salt 사용 (비밀번호 해싱)
비밀번호를 해싱할 때는 반드시 Salt를 추가하세요. Rainbow table 공격을 방지합니다.

```vais
# 나쁜 예: Salt 없음
hash := sha256(password)

# 좋은 예: Salt 추가
salt := random_bytes(16)
hash := sha256(salt + password)
```

### 5. 타이밍 공격 방지
HMAC 검증 시 상수 시간 비교를 사용하세요.

```vais
# 나쁜 예: 타이밍 공격 가능
I memcmp(mac1, mac2, 32) == 0 { ... }

# 좋은 예: 상수 시간 비교
F secure_compare(a: i64, b: i64, len: i64) -> i64 {
    result := 0
    i := 0
    L i < len {
        result = result | (load_byte(a + i) ^ load_byte(b + i))
        i = i + 1
    }
    R result == 0
}
```

### 6. 패딩 오라클 공격
CBC 모드에서 패딩 에러를 노출하지 마세요. 모든 에러를 동일하게 처리하세요.

```vais
# 나쁜 예
decrypted := aes.decrypt_cbc(ciphertext, iv)
I !is_valid_padding(decrypted) {
    print_str("패딩 에러")  # 공격자에게 정보 제공!
    R 0
}

# 좋은 예
decrypted := aes.decrypt_cbc(ciphertext, iv)
I !is_valid_padding(decrypted) {
    print_str("복호화 실패")  # 일반적인 에러 메시지
    R 0
}
```

### 7. 해시 충돌
SHA-256은 충돌 저항성이 강하지만, MD5/SHA-1은 사용하지 마세요.

```vais
# 안전한 해시 알고리즘
- SHA-256
- SHA-3
- BLAKE2

# 사용 금지 (충돌 취약)
- MD5
- SHA-1
```

### 8. AES 블록 크기
AES는 16바이트 블록 단위로 동작합니다. 입력 길이가 16의 배수가 아니면 패딩이 필요합니다.

```vais
# PKCS7 패딩
F pkcs7_pad(data: i64, block_size: i64) -> i64 {
    len := strlen(data)
    pad_len := block_size - (len % block_size)

    padded := malloc(len + pad_len)
    memcpy(padded, data, len)

    # 패딩 바이트 (값 = 패딩 길이)
    i := 0
    L i < pad_len {
        store_byte(padded + len + i, pad_len)
        i = i + 1
    }
    R padded
}
```

### 9. HMAC 키 길이
HMAC 키는 해시 출력 크기(SHA-256의 경우 32바이트) 이상이어야 합니다.

```vais
# 나쁜 예: 짧은 키
key := "secret"  # 6바이트

# 좋은 예: 충분한 키 길이
key := random_bytes(32)  # 256비트
```

### 10. 난수 생성
암호화 키/IV/Salt는 암호학적으로 안전한 난수 생성기(CSPRNG)를 사용하세요.

```vais
# 나쁜 예: 예측 가능
key := pseudo_random(seed)

# 좋은 예: CSPRNG
key := crypto_random_bytes(32)  # /dev/urandom 또는 CryptGenRandom
```

## See Also

- [Crypto API Reference](../api/crypto.md)
- [Hash Functions](../api/hash.md)
- [Random Number Generation](../api/random.md)
- [Security Best Practices](../security/best-practices.md)
- [Base64 Encoding](../api/base64.md)
