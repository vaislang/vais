//! Runtime helper functions and WASM runtime support

use crate::CodeGenerator;

impl CodeGenerator {
    /// Generate helper functions for low-level memory operations
    pub(crate) fn generate_helper_functions(&self) -> String {
        let mut ir = String::new();

        // Declare C library functions needed by runtime helpers
        // Note: exit and strlen are already declared by builtins
        ir.push_str("\n; C library function declarations\n");
        ir.push_str("declare i64 @write(i32, i8*, i64)\n");

        // Global constant for newline (used by panic functions)
        ir.push_str("\n; Global constants for runtime functions\n");
        ir.push_str("@.panic_newline = private unnamed_addr constant [2 x i8] c\"\\0A\\00\"\n");

        // __panic: runtime panic function (used by assert)
        // Prints message to stderr (fd=2) and exits with code 1
        ir.push_str("\n; Runtime panic function (used by assert)\n");
        ir.push_str("define i64 @__panic(i8* %msg) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  ; Calculate message length\n");
        ir.push_str("  %len = call i64 @strlen(i8* %msg)\n");
        ir.push_str("  ; Write message to stderr (fd=2)\n");
        ir.push_str("  %0 = call i64 @write(i32 2, i8* %msg, i64 %len)\n");
        ir.push_str("  ; Write newline\n");
        ir.push_str("  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)\n");
        ir.push_str("  call void @exit(i32 1)\n");
        ir.push_str("  unreachable\n");
        ir.push_str("}\n");

        // __contract_fail: runtime contract failure function
        // Prints contract failure message to stderr and exits with code 1
        ir.push_str("\n; Runtime contract failure function\n");
        ir.push_str("define i64 @__contract_fail(i64 %kind, i8* %condition, i8* %file, i64 %line, i8* %func) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  ; Calculate message length\n");
        ir.push_str("  %len = call i64 @strlen(i8* %condition)\n");
        ir.push_str("  ; Write contract failure message to stderr (fd=2)\n");
        ir.push_str("  %0 = call i64 @write(i32 2, i8* %condition, i64 %len)\n");
        ir.push_str("  ; Write newline\n");
        ir.push_str("  %1 = call i64 @write(i32 2, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.panic_newline, i64 0, i64 0), i64 1)\n");
        ir.push_str("  call void @exit(i32 1)\n");
        ir.push_str("  unreachable\n");
        ir.push_str("}\n");

        // __load_byte: load a byte from memory address
        ir.push_str("\n; Helper function: load byte from memory\n");
        ir.push_str("define i64 @__load_byte(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = load i8, i8* %0\n");
        ir.push_str("  %2 = zext i8 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_byte: store a byte to memory address
        ir.push_str("\n; Helper function: store byte to memory\n");
        ir.push_str("define void @__store_byte(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = trunc i64 %val to i8\n");
        ir.push_str("  store i8 %1, i8* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i64: load a 64-bit integer from memory address
        ir.push_str("\n; Helper function: load i64 from memory\n");
        ir.push_str("define i64 @__load_i64(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i64*\n");
        ir.push_str("  %1 = load i64, i64* %0\n");
        ir.push_str("  ret i64 %1\n");
        ir.push_str("}\n");

        // __store_i64: store a 64-bit integer to memory address
        ir.push_str("\n; Helper function: store i64 to memory\n");
        ir.push_str("define void @__store_i64(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i64*\n");
        ir.push_str("  store i64 %val, i64* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __swap: swap two i64 elements in array by index
        ir.push_str("\n; Helper function: swap two i64 elements in array\n");
        ir.push_str("define void @__swap(i64 %ptr, i64 %idx1, i64 %idx2) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %off1 = mul i64 %idx1, 8\n");
        ir.push_str("  %addr1 = add i64 %ptr, %off1\n");
        ir.push_str("  %off2 = mul i64 %idx2, 8\n");
        ir.push_str("  %addr2 = add i64 %ptr, %off2\n");
        ir.push_str("  %p1 = inttoptr i64 %addr1 to i64*\n");
        ir.push_str("  %p2 = inttoptr i64 %addr2 to i64*\n");
        ir.push_str("  %v1 = load i64, i64* %p1\n");
        ir.push_str("  %v2 = load i64, i64* %p2\n");
        ir.push_str("  store i64 %v2, i64* %p1\n");
        ir.push_str("  store i64 %v1, i64* %p2\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_f64: load a 64-bit float from memory address
        ir.push_str("\n; Helper function: load f64 from memory\n");
        ir.push_str("define double @__load_f64(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to double*\n");
        ir.push_str("  %1 = load double, double* %0\n");
        ir.push_str("  ret double %1\n");
        ir.push_str("}\n");

        // __store_f64: store a 64-bit float to memory address
        ir.push_str("\n; Helper function: store f64 to memory\n");
        ir.push_str("define void @__store_f64(i64 %ptr, double %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to double*\n");
        ir.push_str("  store double %val, double* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i8: load an 8-bit integer from memory address
        ir.push_str("\n; Helper function: load i8 from memory\n");
        ir.push_str("define i64 @__load_i8(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = load i8, i8* %0\n");
        ir.push_str("  %2 = zext i8 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_i8: store an 8-bit integer to memory address
        ir.push_str("\n; Helper function: store i8 to memory\n");
        ir.push_str("define void @__store_i8(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i8*\n");
        ir.push_str("  %1 = trunc i64 %val to i8\n");
        ir.push_str("  store i8 %1, i8* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i16: load a 16-bit integer from memory address
        ir.push_str("\n; Helper function: load i16 from memory\n");
        ir.push_str("define i64 @__load_i16(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i16*\n");
        ir.push_str("  %1 = load i16, i16* %0\n");
        ir.push_str("  %2 = zext i16 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_i16: store a 16-bit integer to memory address
        ir.push_str("\n; Helper function: store i16 to memory\n");
        ir.push_str("define void @__store_i16(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i16*\n");
        ir.push_str("  %1 = trunc i64 %val to i16\n");
        ir.push_str("  store i16 %1, i16* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_i32: load a 32-bit integer from memory address
        ir.push_str("\n; Helper function: load i32 from memory\n");
        ir.push_str("define i64 @__load_i32(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i32*\n");
        ir.push_str("  %1 = load i32, i32* %0\n");
        ir.push_str("  %2 = zext i32 %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __store_i32: store a 32-bit integer to memory address
        ir.push_str("\n; Helper function: store i32 to memory\n");
        ir.push_str("define void @__store_i32(i64 %ptr, i64 %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to i32*\n");
        ir.push_str("  %1 = trunc i64 %val to i32\n");
        ir.push_str("  store i32 %1, i32* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // __load_f32: load a 32-bit float from memory address
        ir.push_str("\n; Helper function: load f32 from memory\n");
        ir.push_str("define double @__load_f32(i64 %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to float*\n");
        ir.push_str("  %1 = load float, float* %0\n");
        ir.push_str("  %2 = fpext float %1 to double\n");
        ir.push_str("  ret double %2\n");
        ir.push_str("}\n");

        // __store_f32: store a 32-bit float to memory address
        ir.push_str("\n; Helper function: store f32 to memory\n");
        ir.push_str("define void @__store_f32(i64 %ptr, double %val) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %ptr to float*\n");
        ir.push_str("  %1 = fptrunc double %val to float\n");
        ir.push_str("  store float %1, float* %0\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n");

        // === Async runtime helper functions ===

        // __call_poll: call an indirect function pointer (poll_fn) with future_ptr
        // poll_fn is a function pointer: i64 (i64) -> i64
        // Returns packed i64 with status in high 32 bits, value in low 32 bits
        ir.push_str("\n; Async helper: call indirect poll function\n");
        ir.push_str("define i64 @__call_poll(i64 %poll_fn, i64 %future_ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %poll_fn to i64 (i64)*\n");
        ir.push_str("  %1 = call i64 %0(i64 %future_ptr)\n");
        ir.push_str("  ret i64 %1\n");
        ir.push_str("}\n");

        // __extract_poll_status: extract status from packed poll result
        // status = result >> 32 (high 32 bits: 0=Pending, 1=Ready)
        ir.push_str("\n; Async helper: extract poll status from packed result\n");
        ir.push_str("define i64 @__extract_poll_status(i64 %poll_result) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = lshr i64 %poll_result, 32\n");
        ir.push_str("  %1 = and i64 %0, 4294967295\n");
        ir.push_str("  ret i64 %1\n");
        ir.push_str("}\n");

        // __extract_poll_value: extract value from packed poll result
        // value = result & 0xFFFFFFFF (low 32 bits)
        ir.push_str("\n; Async helper: extract poll value from packed result\n");
        ir.push_str("define i64 @__extract_poll_value(i64 %poll_result) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = and i64 %poll_result, 4294967295\n");
        ir.push_str("  ret i64 %0\n");
        ir.push_str("}\n");

        // __time_now_ms: get current time in milliseconds using gettimeofday
        ir.push_str("\n; Async helper: current time in milliseconds\n");
        ir.push_str("declare i32 @gettimeofday(i8*, i8*)\n");
        ir.push_str("define i64 @__time_now_ms() {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %tv = alloca [16 x i8], align 8\n");
        ir.push_str("  %tvptr = bitcast [16 x i8]* %tv to i8*\n");
        ir.push_str("  %0 = call i32 @gettimeofday(i8* %tvptr, i8* null)\n");
        ir.push_str("  %secptr = bitcast [16 x i8]* %tv to i64*\n");
        ir.push_str("  %sec = load i64, i64* %secptr\n");
        ir.push_str(
            "  %usecptr = getelementptr inbounds [16 x i8], [16 x i8]* %tv, i64 0, i64 8\n",
        );
        ir.push_str("  %usecptr64 = bitcast i8* %usecptr to i64*\n");
        ir.push_str("  %usec = load i64, i64* %usecptr64\n");
        ir.push_str("  %ms_sec = mul i64 %sec, 1000\n");
        ir.push_str("  %ms_usec = sdiv i64 %usec, 1000\n");
        ir.push_str("  %ms = add i64 %ms_sec, %ms_usec\n");
        ir.push_str("  ret i64 %ms\n");
        ir.push_str("}\n");

        // === macOS-only: kqueue helpers ===
        // Only include kqueue-related functions on macOS (they use the kevent syscall)
        #[cfg(target_os = "macos")]
        {
            // __kevent_register: wrapper around kevent syscall for registration
            ir.push_str("\n; Async helper: kqueue event registration\n");
            ir.push_str("declare i32 @kevent(i32, i8*, i32, i8*, i32, i8*)\n");
            ir.push_str(
                "define i64 @__kevent_register(i64 %kq, i64 %fd, i64 %filter, i64 %flags) {\n",
            );
            ir.push_str("entry:\n");
            // Allocate kevent struct (sizeof(struct kevent) = 64 bytes on macOS)
            ir.push_str("  %ev = alloca [64 x i8], align 8\n");
            ir.push_str("  %evptr = bitcast [64 x i8]* %ev to i8*\n");
            // Set ident (fd) at offset 0
            ir.push_str("  %identptr = bitcast [64 x i8]* %ev to i64*\n");
            ir.push_str("  store i64 %fd, i64* %identptr\n");
            // Set filter at offset 8 (i16)
            ir.push_str(
                "  %filterptr = getelementptr inbounds [64 x i8], [64 x i8]* %ev, i64 0, i64 8\n",
            );
            ir.push_str("  %filterptr16 = bitcast i8* %filterptr to i16*\n");
            ir.push_str("  %filter16 = trunc i64 %filter to i16\n");
            ir.push_str("  store i16 %filter16, i16* %filterptr16\n");
            // Set flags at offset 10 (u16)
            ir.push_str(
                "  %flagsptr = getelementptr inbounds [64 x i8], [64 x i8]* %ev, i64 0, i64 10\n",
            );
            ir.push_str("  %flagsptr16 = bitcast i8* %flagsptr to i16*\n");
            ir.push_str("  %flags16 = trunc i64 %flags to i16\n");
            ir.push_str("  store i16 %flags16, i16* %flagsptr16\n");
            // Call kevent
            ir.push_str("  %kq32 = trunc i64 %kq to i32\n");
            ir.push_str(
                "  %ret = call i32 @kevent(i32 %kq32, i8* %evptr, i32 1, i8* null, i32 0, i8* null)\n",
            );
            ir.push_str("  %retval = sext i32 %ret to i64\n");
            ir.push_str("  ret i64 %retval\n");
            ir.push_str("}\n");

            // __kevent_wait: wait for events with timeout
            ir.push_str("\n; Async helper: kqueue event wait\n");
            ir.push_str("define i64 @__kevent_wait(i64 %kq, i64 %events_buf, i64 %max_events, i64 %timeout_ms) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %bufptr = inttoptr i64 %events_buf to i8*\n");
            ir.push_str("  %kq32 = trunc i64 %kq to i32\n");
            ir.push_str("  %max32 = trunc i64 %max_events to i32\n");
            // Allocate timespec for timeout
            ir.push_str("  %ts = alloca [16 x i8], align 8\n");
            ir.push_str("  %tsptr = bitcast [16 x i8]* %ts to i8*\n");
            ir.push_str("  %secval = sdiv i64 %timeout_ms, 1000\n");
            ir.push_str("  %nsval = mul i64 %timeout_ms, 1000000\n");
            ir.push_str("  %nsrem = srem i64 %nsval, 1000000000\n");
            ir.push_str("  %secptr = bitcast [16 x i8]* %ts to i64*\n");
            ir.push_str("  store i64 %secval, i64* %secptr\n");
            ir.push_str(
                "  %nsptr = getelementptr inbounds [16 x i8], [16 x i8]* %ts, i64 0, i64 8\n",
            );
            ir.push_str("  %nsptr64 = bitcast i8* %nsptr to i64*\n");
            ir.push_str("  store i64 %nsrem, i64* %nsptr64\n");
            ir.push_str("  %ret = call i32 @kevent(i32 %kq32, i8* null, i32 0, i8* %bufptr, i32 %max32, i8* %tsptr)\n");
            ir.push_str("  %retval = sext i32 %ret to i64\n");
            ir.push_str("  ret i64 %retval\n");
            ir.push_str("}\n");

            // __kevent_get_fd: get fd from kevent result at index
            ir.push_str("\n; Async helper: get fd from kevent result\n");
            ir.push_str("define i64 @__kevent_get_fd(i64 %events_buf, i64 %index) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %base = inttoptr i64 %events_buf to i8*\n");
            // sizeof(struct kevent) = 64 on macOS
            ir.push_str("  %offset = mul i64 %index, 64\n");
            ir.push_str("  %evptr = getelementptr inbounds i8, i8* %base, i64 %offset\n");
            ir.push_str("  %identptr = bitcast i8* %evptr to i64*\n");
            ir.push_str("  %ident = load i64, i64* %identptr\n");
            ir.push_str("  ret i64 %ident\n");
            ir.push_str("}\n");

            // __kevent_get_filter: get filter from kevent result at index
            ir.push_str("\n; Async helper: get filter from kevent result\n");
            ir.push_str("define i64 @__kevent_get_filter(i64 %events_buf, i64 %index) {\n");
            ir.push_str("entry:\n");
            ir.push_str("  %base = inttoptr i64 %events_buf to i8*\n");
            ir.push_str("  %offset = mul i64 %index, 64\n");
            ir.push_str("  %evptr = getelementptr inbounds i8, i8* %base, i64 %offset\n");
            // filter is at offset 8 (i16)
            ir.push_str("  %filterptr = getelementptr inbounds i8, i8* %evptr, i64 8\n");
            ir.push_str("  %filterptr16 = bitcast i8* %filterptr to i16*\n");
            ir.push_str("  %filter16 = load i16, i16* %filterptr16\n");
            ir.push_str("  %filter = sext i16 %filter16 to i64\n");
            ir.push_str("  ret i64 %filter\n");
            ir.push_str("}\n");
        }

        // __write_byte: write a single byte to file descriptor
        ir.push_str("\n; Async helper: write byte to fd\n");
        ir.push_str("define i64 @__write_byte(i64 %fd, i64 %value) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %buf = alloca i8\n");
        ir.push_str("  %byte = trunc i64 %value to i8\n");
        ir.push_str("  store i8 %byte, i8* %buf\n");
        ir.push_str("  %fd32 = trunc i64 %fd to i32\n");
        ir.push_str("  %ret = call i64 @write(i32 %fd32, i8* %buf, i64 1)\n");
        ir.push_str("  ret i64 %ret\n");
        ir.push_str("}\n");

        // __read_byte: read a single byte from file descriptor
        ir.push_str("\n; Async helper: read byte from fd\n");
        ir.push_str("declare i64 @read(i32, i8*, i64)\n");
        ir.push_str("define i64 @__read_byte(i64 %fd) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %buf = alloca i8\n");
        ir.push_str("  %fd32 = trunc i64 %fd to i32\n");
        ir.push_str("  %ret = call i64 @read(i32 %fd32, i8* %buf, i64 1)\n");
        ir.push_str("  %byte = load i8, i8* %buf\n");
        ir.push_str("  %val = zext i8 %byte to i64\n");
        ir.push_str("  ret i64 %val\n");
        ir.push_str("}\n");

        // __readdir_wrapper: readdir wrapper that returns pointer to d_name
        ir.push_str("\n; Filesystem helper: readdir wrapper\n");
        ir.push_str("%struct.dirent = type opaque\n");
        ir.push_str("declare %struct.dirent* @readdir(i8*)\n");
        ir.push_str("define i64 @__readdir_wrapper(i64 %dirp) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %dirp to i8*\n");
        ir.push_str("  %1 = call %struct.dirent* @readdir(i8* %0)\n");
        ir.push_str("  %2 = icmp eq %struct.dirent* %1, null\n");
        ir.push_str("  br i1 %2, label %ret_null, label %ret_name\n");
        ir.push_str("ret_null:\n");
        ir.push_str("  ret i64 0\n");
        ir.push_str("ret_name:\n");
        ir.push_str("  %3 = bitcast %struct.dirent* %1 to i8*\n");
        // On macOS (Darwin), d_name is at offset 21
        // On Linux, d_name is at offset 19
        let d_name_offset = if cfg!(target_os = "linux") { 19 } else { 21 };
        ir.push_str(&format!(
            "  %4 = getelementptr inbounds i8, i8* %3, i64 {}\n",
            d_name_offset
        ));
        ir.push_str("  %5 = ptrtoint i8* %4 to i64\n");
        ir.push_str("  ret i64 %5\n");
        ir.push_str("}\n");

        // __getcwd_wrapper: getcwd wrapper that converts ptr result to i64
        ir.push_str("\n; Filesystem helper: getcwd wrapper\n");
        ir.push_str("declare i8* @getcwd(i8*, i64)\n");
        ir.push_str("define i64 @__getcwd_wrapper(i64 %buf, i64 %size) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %0 = inttoptr i64 %buf to i8*\n");
        ir.push_str("  %1 = call i8* @getcwd(i8* %0, i64 %size)\n");
        ir.push_str("  %2 = ptrtoint i8* %1 to i64\n");
        ir.push_str("  ret i64 %2\n");
        ir.push_str("}\n");

        // __stat_size: get file size using stat
        ir.push_str("\n; Filesystem helper: stat file size\n");
        ir.push_str("%struct.stat = type opaque\n");
        ir.push_str("declare i32 @stat(i8*, %struct.stat*)\n");
        ir.push_str("define i64 @__stat_size(i8* %path) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %statbuf = alloca [144 x i8], align 8\n");
        ir.push_str("  %0 = bitcast [144 x i8]* %statbuf to %struct.stat*\n");
        ir.push_str("  %1 = call i32 @stat(i8* %path, %struct.stat* %0)\n");
        ir.push_str("  %2 = icmp ne i32 %1, 0\n");
        ir.push_str("  br i1 %2, label %error, label %success\n");
        ir.push_str("error:\n");
        ir.push_str("  ret i64 -1\n");
        ir.push_str("success:\n");
        // st_size is at offset 96 on macOS (after dev:4, mode:2, nlink:2, ino:8, uid:4, gid:4, rdev:4, atim:16, mtim:16, ctim:16, birthtim:16, size:8)
        // Actually on macOS x86_64, st_size is at offset 96
        ir.push_str(
            "  %3 = getelementptr inbounds [144 x i8], [144 x i8]* %statbuf, i64 0, i64 96\n",
        );
        ir.push_str("  %4 = bitcast i8* %3 to i64*\n");
        ir.push_str("  %5 = load i64, i64* %4\n");
        ir.push_str("  ret i64 %5\n");
        ir.push_str("}\n");

        // __stat_mtime: get file modification time using stat
        ir.push_str("\n; Filesystem helper: stat modification time\n");
        ir.push_str("define i64 @__stat_mtime(i8* %path) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %statbuf = alloca [144 x i8], align 8\n");
        ir.push_str("  %0 = bitcast [144 x i8]* %statbuf to %struct.stat*\n");
        ir.push_str("  %1 = call i32 @stat(i8* %path, %struct.stat* %0)\n");
        ir.push_str("  %2 = icmp ne i32 %1, 0\n");
        ir.push_str("  br i1 %2, label %error, label %success\n");
        ir.push_str("error:\n");
        ir.push_str("  ret i64 -1\n");
        ir.push_str("success:\n");
        // st_mtimespec is at offset 48 on macOS (after dev:4, mode:2, nlink:2, ino:8, uid:4, gid:4, rdev:4, atim:16, mtim starts here)
        // The tv_sec field is the first 8 bytes of the timespec
        ir.push_str(
            "  %3 = getelementptr inbounds [144 x i8], [144 x i8]* %statbuf, i64 0, i64 48\n",
        );
        ir.push_str("  %4 = bitcast i8* %3 to i64*\n");
        ir.push_str("  %5 = load i64, i64* %4\n");
        ir.push_str("  ret i64 %5\n");
        ir.push_str("}\n");

        ir
    }

    /// Generate WASM-specific runtime functions and declarations.
    ///
    /// For `wasm32-unknown-unknown` targets, this generates:
    /// - Linear memory export
    /// - `_start` entry point that calls main
    /// - `fd_write`-based `puts` implementation (no libc)
    /// - Simple bump allocator using `memory.grow`
    ///
    /// For WASI targets, this generates:
    /// - WASI-compatible `_start` entry point
    /// - Memory export for WASI runtime
    pub(crate) fn generate_wasm_runtime(&self) -> String {
        use crate::TargetTriple;

        if !self.target.is_wasm() {
            return String::new();
        }

        let mut ir = String::new();

        ir.push_str("\n; ========================================\n");
        ir.push_str("; WASM Runtime Support\n");
        ir.push_str("; ========================================\n\n");

        match &self.target {
            TargetTriple::Wasm32Unknown => {
                self.generate_wasm32_unknown_runtime(&mut ir);
            }
            TargetTriple::WasiPreview1 | TargetTriple::WasiPreview2 => {
                self.generate_wasi_runtime(&mut ir);
            }
            _ => {}
        }

        ir
    }

    /// Generate runtime for wasm32-unknown-unknown (no WASI, browser environment)
    fn generate_wasm32_unknown_runtime(&self, ir: &mut String) {
        // Memory export (1 page = 64KB initial)
        ir.push_str("; Linear memory (exported)\n");
        ir.push_str("@__wasm_memory = external global i8\n\n");

        // Bump allocator state: heap pointer starts at 1MB (leaves stack space)
        ir.push_str("; Bump allocator heap pointer (starts at 1MB offset)\n");
        ir.push_str("@__heap_ptr = global i32 1048576\n\n");

        // malloc replacement using bump allocator
        ir.push_str("; WASM malloc: bump allocator with memory.grow\n");
        ir.push_str("define i8* @malloc(i64 %size) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %size32 = trunc i64 %size to i32\n");
        ir.push_str("  ; Align to 8 bytes\n");
        ir.push_str("  %aligned = add i32 %size32, 7\n");
        ir.push_str("  %aligned_size = and i32 %aligned, -8\n");
        ir.push_str("  ; Load current heap pointer\n");
        ir.push_str("  %cur = load i32, i32* @__heap_ptr\n");
        ir.push_str("  %new = add i32 %cur, %aligned_size\n");
        ir.push_str("  ; Check if we need to grow memory\n");
        ir.push_str("  %cur_pages = call i32 @llvm.wasm.memory.size.i32(i32 0)\n");
        ir.push_str("  %cur_bytes = mul i32 %cur_pages, 65536\n");
        ir.push_str("  %needs_grow = icmp ugt i32 %new, %cur_bytes\n");
        ir.push_str("  br i1 %needs_grow, label %grow, label %done\n");
        ir.push_str("grow:\n");
        ir.push_str("  %needed = sub i32 %new, %cur_bytes\n");
        ir.push_str("  %pages_needed_raw = add i32 %needed, 65535\n");
        ir.push_str("  %pages_needed = udiv i32 %pages_needed_raw, 65536\n");
        ir.push_str(
            "  %grow_result = call i32 @llvm.wasm.memory.grow.i32(i32 0, i32 %pages_needed)\n",
        );
        ir.push_str("  %grow_failed = icmp eq i32 %grow_result, -1\n");
        ir.push_str("  br i1 %grow_failed, label %oom, label %done\n");
        ir.push_str("oom:\n");
        ir.push_str("  call void @__wasm_trap()\n");
        ir.push_str("  unreachable\n");
        ir.push_str("done:\n");
        ir.push_str("  ; Update heap pointer\n");
        ir.push_str("  store i32 %new, i32* @__heap_ptr\n");
        ir.push_str("  %ptr = inttoptr i32 %cur to i8*\n");
        ir.push_str("  ret i8* %ptr\n");
        ir.push_str("}\n\n");

        // free is a no-op for bump allocator
        ir.push_str("; WASM free: no-op for bump allocator\n");
        ir.push_str("define void @free(i8* %ptr) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  ret void\n");
        ir.push_str("}\n\n");

        // realloc: allocate new block and copy
        ir.push_str("; WASM realloc: allocate new + copy (conservative)\n");
        ir.push_str("define i8* @realloc(i8* %old, i64 %new_size) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %new_ptr = call i8* @malloc(i64 %new_size)\n");
        ir.push_str("  %old_is_null = icmp eq i8* %old, null\n");
        ir.push_str("  br i1 %old_is_null, label %done, label %copy\n");
        ir.push_str("copy:\n");
        ir.push_str("  ; Copy old data (conservative: copy new_size bytes)\n");
        ir.push_str("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* %new_ptr, i8* %old, i64 %new_size, i1 false)\n");
        ir.push_str("  br label %done\n");
        ir.push_str("done:\n");
        ir.push_str("  ret i8* %new_ptr\n");
        ir.push_str("}\n\n");

        // puts replacement: write string to fd 1 (stdout) via imported function
        ir.push_str("; WASM puts: calls imported __wasm_write for output\n");
        ir.push_str("define i64 @puts(i8* %str) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = call i64 @strlen(i8* %str)\n");
        ir.push_str("  %len32 = trunc i64 %len to i32\n");
        ir.push_str("  %ptr32 = ptrtoint i8* %str to i32\n");
        ir.push_str("  call void @__wasm_write(i32 1, i32 %ptr32, i32 %len32)\n");
        ir.push_str("  ; Write newline\n");
        ir.push_str("  call void @__wasm_write_byte(i32 1, i32 10)\n");
        ir.push_str("  ret i64 0\n");
        ir.push_str("}\n\n");

        // printf replacement (simplified: just write the format string)
        ir.push_str("; WASM printf: simplified output\n");
        ir.push_str("define i64 @printf(i8* %fmt, ...) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = call i64 @strlen(i8* %fmt)\n");
        ir.push_str("  %len32 = trunc i64 %len to i32\n");
        ir.push_str("  %ptr32 = ptrtoint i8* %fmt to i32\n");
        ir.push_str("  call void @__wasm_write(i32 1, i32 %ptr32, i32 %len32)\n");
        ir.push_str("  ret i64 %len\n");
        ir.push_str("}\n\n");

        // strlen implementation (no libc)
        ir.push_str("; WASM strlen: pure LLVM implementation\n");
        ir.push_str("define i64 @__wasm_strlen(i8* %str) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  br label %loop\n");
        ir.push_str("loop:\n");
        ir.push_str("  %i = phi i64 [0, %entry], [%next, %loop]\n");
        ir.push_str("  %ptr = getelementptr i8, i8* %str, i64 %i\n");
        ir.push_str("  %ch = load i8, i8* %ptr\n");
        ir.push_str("  %is_zero = icmp eq i8 %ch, 0\n");
        ir.push_str("  %next = add i64 %i, 1\n");
        ir.push_str("  br i1 %is_zero, label %done, label %loop\n");
        ir.push_str("done:\n");
        ir.push_str("  ret i64 %i\n");
        ir.push_str("}\n\n");

        // exit implementation via trap
        ir.push_str("; WASM exit: unreachable trap\n");
        ir.push_str("define void @exit(i32 %code) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  call void @__wasm_trap()\n");
        ir.push_str("  unreachable\n");
        ir.push_str("}\n\n");

        // Imported functions from JS host
        ir.push_str("; Host-imported functions (provided by JS runtime)\n");
        ir.push_str("declare void @__wasm_write(i32 %fd, i32 %ptr, i32 %len)\n");
        ir.push_str("declare void @__wasm_write_byte(i32 %fd, i32 %byte)\n");
        ir.push_str("declare void @__wasm_trap()\n\n");

        // LLVM intrinsics for WASM
        ir.push_str("; LLVM WASM intrinsics\n");
        ir.push_str("declare i32 @llvm.wasm.memory.size.i32(i32)\n");
        ir.push_str("declare i32 @llvm.wasm.memory.grow.i32(i32, i32)\n");
        ir.push_str("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)\n\n");

        // _start entry point that calls main
        ir.push_str("; _start entry point (calls main)\n");
        ir.push_str("define void @_start() {\n");
        ir.push_str("entry:\n");
        if self.types.functions.contains_key("main") {
            ir.push_str("  %ret = call i64 @main()\n");
        }
        ir.push_str("  ret void\n");
        ir.push_str("}\n\n");
    }

    /// Generate runtime for WASI targets
    fn generate_wasi_runtime(&self, ir: &mut String) {
        // WASI _start entry point
        ir.push_str("; WASI _start entry point\n");
        ir.push_str("define void @_start() {\n");
        ir.push_str("entry:\n");
        if self.types.functions.contains_key("main") {
            ir.push_str("  %ret = call i64 @main()\n");
            ir.push_str("  ; Exit with main's return code\n");
            ir.push_str("  %code = trunc i64 %ret to i32\n");
            ir.push_str("  call void @__wasi_proc_exit(i32 %code)\n");
        } else {
            ir.push_str("  call void @__wasi_proc_exit(i32 0)\n");
        }
        ir.push_str("  unreachable\n");
        ir.push_str("}\n\n");

        // WASI fd_write-based puts
        ir.push_str("; WASI puts: fd_write based\n");
        ir.push_str("; Uses WASI fd_write(fd, iovs, iovs_len, nwritten) -> errno\n");
        ir.push_str("@__wasi_iov = global [2 x i32] zeroinitializer\n");
        ir.push_str("@__wasi_nwritten = global i32 0\n\n");
        ir.push_str("define i64 @__wasi_puts(i8* %str) {\n");
        ir.push_str("entry:\n");
        ir.push_str("  %len = call i64 @strlen(i8* %str)\n");
        ir.push_str("  %len32 = trunc i64 %len to i32\n");
        ir.push_str("  %ptr32 = ptrtoint i8* %str to i32\n");
        ir.push_str("  ; Set up iov: [ptr, len]\n");
        ir.push_str("  %iov_ptr = getelementptr [2 x i32], [2 x i32]* @__wasi_iov, i32 0, i32 0\n");
        ir.push_str("  store i32 %ptr32, i32* %iov_ptr\n");
        ir.push_str("  %iov_len = getelementptr [2 x i32], [2 x i32]* @__wasi_iov, i32 0, i32 1\n");
        ir.push_str("  store i32 %len32, i32* %iov_len\n");
        ir.push_str("  ; Call fd_write(stdout=1, iovs, iovs_len=1, nwritten)\n");
        ir.push_str("  %errno = call i32 @__wasi_fd_write(i32 1, i32* %iov_ptr, i32 1, i32* @__wasi_nwritten)\n");
        ir.push_str("  %result = sext i32 %errno to i64\n");
        ir.push_str("  ret i64 %result\n");
        ir.push_str("}\n\n");

        // WASI syscall declarations
        ir.push_str("; WASI syscall declarations\n");
        ir.push_str("declare void @__wasi_proc_exit(i32) noreturn\n");
        ir.push_str("declare i32 @__wasi_fd_write(i32, i32*, i32, i32*)\n");
        ir.push_str("declare i32 @__wasi_fd_read(i32, i32*, i32, i32*)\n\n");
    }
}
