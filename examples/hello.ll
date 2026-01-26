; ModuleID = 'hello'
source_filename = "<vais>"

declare i32 @putchar(i32)
declare i64 @vais_gc_bytes_allocated()
declare i32 @sched_yield()
declare i32 @fclose(i64)
declare i64 @fgetc(i64)
declare i64 @vais_gc_init()
declare i64 @vais_gc_print_stats()
declare i64 @vais_gc_collections()
declare i64 @malloc(i64)
declare i64 @fputs(i8*, i64)
declare i64 @fgets(i64, i64, i64)
declare i64 @memcpy(i64, i64, i64)
declare i32 @strncmp(i8*, i8*, i64)
declare i64 @fputc(i64, i64)
declare i64 @vais_gc_remove_root(i64)
declare i64 @fflush(i64)
declare i64 @vais_gc_add_root(i64)
declare i32 @usleep(i64)
declare i64 @fseek(i64, i64, i64)
declare i32 @strcmp(i8*, i8*)
declare i64 @ftell(i64)
declare void @free(i64)
declare i64 @feof(i64)
declare void @exit(i32)
declare i64 @fwrite(i64, i64, i64, i64)
declare i64 @strlen(i8*)
declare i32 @puts(i64)
declare i64 @vais_gc_alloc(i64, i32)
declare i32 @printf(i8*)
declare i64 @vais_gc_collect()
declare i64 @vais_gc_objects_count()
declare i64 @vais_gc_set_threshold(i64)
declare i64 @fread(i64, i64, i64, i64)
declare i64 @fopen(i8*, i8*)
define i64 @main() {
entry:
  ret i64 42
}


; Helper function: load byte from memory
define i64 @__load_byte(i64 %ptr) {
entry:
  %0 = inttoptr i64 %ptr to i8*
  %1 = load i8, i8* %0
  %2 = zext i8 %1 to i64
  ret i64 %2
}

; Helper function: store byte to memory
define void @__store_byte(i64 %ptr, i64 %val) {
entry:
  %0 = inttoptr i64 %ptr to i8*
  %1 = trunc i64 %val to i8
  store i8 %1, i8* %0
  ret void
}

; Helper function: load i64 from memory
define i64 @__load_i64(i64 %ptr) {
entry:
  %0 = inttoptr i64 %ptr to i64*
  %1 = load i64, i64* %0
  ret i64 %1
}

; Helper function: store i64 to memory
define void @__store_i64(i64 %ptr, i64 %val) {
entry:
  %0 = inttoptr i64 %ptr to i64*
  store i64 %val, i64* %0
  ret void
}
