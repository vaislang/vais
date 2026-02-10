# File I/O

## 개요

File I/O 모듈은 파일 읽기/쓰기/추가를 위한 기본 인터페이스를 제공합니다. C 표준 라이브러리의 `fopen`/`fread`/`fwrite`/`fseek`/`fclose`를 래핑하며, 바이너리와 텍스트 모드를 모두 지원합니다.

## Quick Start

```vais
U std/file

F main() -> i64 {
    f := File::open_write("output.txt")
    f.write_str("Hello, Vais!")
    f.close()
    R 0
}
```

## API 요약

| 함수 | 설명 |
|------|------|
| `File::open_read(path)` | 읽기 모드로 파일 열기 |
| `File::open_write(path)` | 쓰기 모드 (생성/덮어쓰기) |
| `File::open_append(path)` | 추가 모드 (끝에 쓰기) |
| `read_bytes(buf, count)` | 바이너리 읽기 |
| `write_bytes(buf, count)` | 바이너리 쓰기 |
| `read_str(max_len)` | 텍스트 라인 읽기 |
| `write_str(s)` | 텍스트 쓰기 |
| `seek(offset, origin)` | 파일 포인터 이동 |
| `tell()` | 현재 위치 반환 |
| `close()` | 파일 닫기 |
| `is_valid()` | 파일 핸들 유효 여부 |

## 실용 예제

### 예제 1: 텍스트 파일 읽기

```vais
U std/file

F read_config(path: i64) -> i64 {
    f := File::open_read(path)
    I !f.is_valid() {
        print_str("파일 열기 실패")
        R 1
    }

    # 최대 1024바이트 읽기
    content := f.read_str(1024)
    print_str(content)

    f.close()
    R 0
}

F main() -> i64 {
    R read_config("config.txt")
}
```

### 예제 2: 라인별 처리

```vais
U std/file
U std/vec

F read_lines(path: i64) -> Vec<i64> {
    lines := Vec::new()
    f := File::open_read(path)

    I !f.is_valid() {
        R lines
    }

    L f.is_valid() {
        line := f.read_str(256)
        I line == 0 { B }  # EOF
        lines.push(line)
    }

    f.close()
    R lines
}
```

### 예제 3: 바이너리 파일 쓰기

```vais
U std/file

F save_data(path: i64, data: i64, size: i64) -> i64 {
    f := File::open_write(path)
    I !f.is_valid() {
        R 1
    }

    bytes_written := f.write_bytes(data, size)
    f.close()

    I bytes_written == size {
        R 0  # 성공
    } E {
        R 1  # 부분 쓰기 실패
    }
}

F main() -> i64 {
    buffer := malloc(100)
    # buffer에 데이터 채우기...
    R save_data("output.bin", buffer, 100)
}
```

### 예제 4: 파일 포인터 조작 (Seek)

```vais
U std/file

F read_header(path: i64) -> i64 {
    f := File::open_read(path)
    I !f.is_valid() { R 0 }

    # 처음 4바이트 읽기 (magic number)
    magic := malloc(4)
    f.read_bytes(magic, 4)

    # 100바이트 건너뛰기
    f.seek(100, SEEK_CUR)

    # 현재 위치 확인
    pos := f.tell()
    print_i64(pos)  # 104 (4 + 100)

    # 파일 끝으로 이동
    f.seek(0, SEEK_END)
    file_size := f.tell()
    print_i64(file_size)

    f.close()
    R 1
}
```

### 예제 5: Append 모드로 로그 추가

```vais
U std/file
U std/datetime

F append_log(path: i64, message: i64) -> i64 {
    f := File::open_append(path)
    I !f.is_valid() { R 1 }

    # 타임스탬프 + 메시지 추가
    timestamp := get_time()
    f.write_str("[")
    f.write_str(timestamp)
    f.write_str("] ")
    f.write_str(message)
    f.write_str("\n")

    f.close()
    R 0
}

F main() -> i64 {
    append_log("server.log", "Server started")
    append_log("server.log", "Request received")
    R 0
}
```

## 주의사항

### 1. 파일 핸들 검증
`open_*` 함수는 실패 시 `handle=0`인 File을 반환합니다. 항상 `is_valid()`로 검증하세요.

```vais
f := File::open_read(path)
I !f.is_valid() {
    # 에러 처리 (파일 없음, 권한 없음 등)
    R 1
}
```

### 2. 파일 닫기 필수
파일을 열면 반드시 `close()`를 호출하세요. GC는 FILE* 핸들을 자동으로 닫지 않으므로, 파일 디스크립터 누수가 발생합니다.

```vais
# 나쁜 예
F bad() {
    f := File::open_write("temp.txt")
    f.write_str("data")
    # close() 호출 안 함!
}

# 좋은 예
F good() {
    f := File::open_write("temp.txt")
    D f.close()  # defer로 자동 정리
    f.write_str("data")
}
```

### 3. 바이너리 vs 텍스트
- `read_bytes`/`write_bytes`: 바이너리 데이터 (구조체, 배열 등)
- `read_str`/`write_str`: 텍스트 데이터 (null-terminated 문자열)

Windows에서는 텍스트 모드(`"r"`)와 바이너리 모드(`"rb"`)가 줄바꿈 처리에서 다릅니다. 바이너리 데이터는 항상 `"rb"`/`"wb"` 모드를 사용하세요.

### 4. 버퍼 크기 제한
`read_str(max_len)`은 최대 `max_len` 바이트까지만 읽습니다. 큰 파일은 청크 단위로 읽거나, 메모리 매핑을 고려하세요.

```vais
# 대용량 파일 처리
F process_large_file(path: i64) -> i64 {
    f := File::open_read(path)
    L f.is_valid() {
        chunk := f.read_bytes(buffer, 4096)
        I chunk == 0 { B }  # EOF
        process_chunk(buffer, chunk)
    }
    f.close()
    R 0
}
```

### 5. Seek 원점 상수
- `SEEK_SET` (0): 파일 시작
- `SEEK_CUR` (1): 현재 위치
- `SEEK_END` (2): 파일 끝

음수 offset은 `SEEK_END`와 함께 사용하여 끝에서부터 역방향 이동할 수 있습니다.

```vais
# 파일 끝에서 100바이트 앞으로 이동
f.seek(-100, SEEK_END)
```

### 6. 에러 처리
현재 구현은 실패 시 0 또는 invalid handle을 반환합니다. 프로덕션 코드에서는 `Result<T,E>`로 래핑하여 상세 에러 정보를 전달하세요.

```vais
U std/result

E FileError {
    NotFound,
    PermissionDenied,
    IOError
}

F open_checked(path: i64) -> Result<File, FileError> {
    f := File::open_read(path)
    I !f.is_valid() {
        # errno 확인하여 적절한 에러 반환
        R Err(FileError::NotFound)
    }
    R Ok(f)
}
```

### 7. 플랫폼 경로 구분자
Windows는 `\`, Unix는 `/`를 사용합니다. 크로스 플랫폼 코드에서는 `std/path` 모듈을 사용하세요.

```vais
U std/path

path := Path::join("data", "config.txt")  # 플랫폼 자동 감지
f := File::open_read(path)
```

## See Also

- [File API Reference](../api/file.md)
- [Filesystem API Reference](../api/filesystem.md)
- [Path API Reference](../api/path.md)
- [IO API Reference](../api/io.md)
- [ByteBuffer API Reference](../api/bytebuffer.md)
