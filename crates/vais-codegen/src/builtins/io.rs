use super::*;

impl CodeGenerator {
    pub(super) fn register_io_functions(&mut self) {
        // printf for printing (variadic, extern C)
        register_vararg!(
            self,
            "printf",
            vec![("format".to_string(), ResolvedType::Str)],
            ResolvedType::I32,
            extern
        );

        // putchar for single character output
        register_extern!(
            self,
            "putchar",
            vec![("c".to_string(), ResolvedType::I32)],
            ResolvedType::I32
        );

        // puts for simple string output
        register_extern!(
            self,
            "puts",
            vec![("s".to_string(), ResolvedType::Str)],
            ResolvedType::I32
        );

        // puts_ptr: print string from pointer (maps to C puts)
        register_extern!(self, "puts_ptr" => "puts",
            vec![("s".to_string(), ResolvedType::Str)], ResolvedType::I32);

        // print: format string output (no newline, vararg)
        register_vararg!(
            self,
            "print",
            vec![("format".to_string(), ResolvedType::Str)],
            ResolvedType::Unit
        );

        // println: format string output (with newline, vararg)
        register_vararg!(
            self,
            "println",
            vec![("format".to_string(), ResolvedType::Str)],
            ResolvedType::Unit
        );

        // format: format string, returns allocated string (vararg)
        register_vararg!(
            self,
            "format",
            vec![("format".to_string(), ResolvedType::Str)],
            ResolvedType::Str
        );

        // exit: (i32) -> void (noreturn)
        register_extern!(
            self,
            "exit",
            vec![("code".to_string(), ResolvedType::I32)],
            ResolvedType::Unit
        );
    }
}
