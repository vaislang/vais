#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

#define VAIS_VERSION "0.3.1"

extern int64_t compile(char *src);

static const char *HOST_INTRINSIC_IR =
    "declare i64 @fs_exists(i8*)\n"
    "declare i8* @fs_read_text(i8*)\n"
    "declare i8* @fs_cwd()\n"
    "declare i8* @fs_temp_dir()\n"
    "declare i8* @path_join(i8*, i8*)\n"
    "declare i8* @path_basename(i8*)\n"
    "declare i8* @path_dirname(i8*)\n"
    "declare i8* @str_concat(i8*, i8*)\n"
    "declare i8* @str_slice(i8*, i64, i64)\n"
    "declare i8* @str_byte(i64)\n"
    "declare i64 @str_builder_new()\n"
    "declare i64 @str_builder_push(i64, i64)\n"
    "declare i64 @str_builder_append(i64, i8*)\n"
    "declare i8* @str_builder_finish(i64)\n"
    "declare i64 @fs_write_text(i8*, i8*)\n"
    "declare i64 @fs_mkdirs(i8*)\n"
    "declare i64 @fs_remove(i8*)\n"
    "declare i64 @proc_argc()\n"
    "declare i8* @proc_arg(i64)\n"
    "declare i8* @proc_capture_stdout(i64*)\n"
    "declare i8* @proc_capture_stderr(i64*)\n"
    "declare i64 @proc_capture_to(i64*, i8*, i8*)\n"
    "declare i64 @proc_run(i64*)\n"
    "declare i64 @proc_run_env(i64*, i64*)\n";

typedef struct {
    char *data;
    size_t len;
    size_t cap;
} StrBuf;

typedef struct {
    char **items;
    size_t len;
    size_t cap;
} LineVec;

typedef struct {
    char *name;
    char *path;
    int line_no;
} ModuleSymbol;

typedef struct {
    char *path;
    char *module;
} ModuleStackEntry;

typedef struct {
    char *alias;
    char *source_root;
} PackageRootInfo;

typedef struct {
    char *root;
    PackageRootInfo packages[64];
    int package_count;
    PackageRootInfo dependencies[64];
    int dependency_count;
    LineVec visited;
    ModuleStackEntry stack[128];
    int stack_count;
    ModuleSymbol symbols[512];
    int symbol_count;
} ModuleResolver;

typedef struct {
    char *name;
    int tag;
    int field_count;
    int field_is_struct[8];
} VariantInfo;

typedef struct {
    char *name;
    int builtin_kind;
    int is_payload;
    int count;
    char *struct_names[16];
    int struct_count;
    VariantInfo variants[16];
} EnumInfo;

typedef struct {
    char *maker;
    char *apply;
} ClosureMaker;

typedef struct {
    char *name;
    char *params[16];
    char *param_types[16];
    int param_count;
    char *return_type;
    int line_no;
} DirectFnInfo;

typedef struct {
    char *name;
    char *type;
    int is_ref;
} DirectLocalInfo;

typedef struct {
    DirectLocalInfo items[128];
    int count;
    char *current_return_type;
    int temp_count;
} DirectNameSet;

typedef struct {
    char *name;
    char *fields[16];
    int field_count;
    int line_no;
} DirectStructInfo;

static int run_program_wait(char *const argv[]);
static int make_tmp_path(char *buf, size_t buflen, const char *suffix);

static StrBuf *direct_current_prelude = NULL;

static void die_oom(void) {
    fprintf(stderr, "error: out of memory\n");
    exit(1);
}

static void sb_init(StrBuf *sb) {
    sb->cap = 4096;
    sb->len = 0;
    sb->data = (char *)malloc(sb->cap);
    if (sb->data == NULL) die_oom();
    sb->data[0] = '\0';
}

static void sb_reserve(StrBuf *sb, size_t need) {
    if (need <= sb->cap) return;
    while (sb->cap < need) sb->cap *= 2;
    char *next = (char *)realloc(sb->data, sb->cap);
    if (next == NULL) die_oom();
    sb->data = next;
}

static void sb_append_n(StrBuf *sb, const char *text, size_t n) {
    sb_reserve(sb, sb->len + n + 1);
    memcpy(sb->data + sb->len, text, n);
    sb->len += n;
    sb->data[sb->len] = '\0';
}

static void sb_append(StrBuf *sb, const char *text) {
    sb_append_n(sb, text, strlen(text));
}

static char *sb_take(StrBuf *sb) {
    char *out = sb->data;
    sb->data = NULL;
    sb->len = 0;
    sb->cap = 0;
    return out;
}

static void lines_init(LineVec *lv) {
    lv->len = 0;
    lv->cap = 32;
    lv->items = (char **)calloc(lv->cap, sizeof(char *));
    if (lv->items == NULL) die_oom();
}

static void lines_push(LineVec *lv, char *line) {
    if (lv->len + 1 > lv->cap) {
        lv->cap *= 2;
        char **next = (char **)realloc(lv->items, lv->cap * sizeof(char *));
        if (next == NULL) die_oom();
        lv->items = next;
    }
    lv->items[lv->len++] = line;
}

static void lines_free(LineVec *lv) {
    for (size_t i = 0; i < lv->len; i++) free(lv->items[i]);
    free(lv->items);
    lv->items = NULL;
    lv->len = 0;
    lv->cap = 0;
}

static LineVec split_lines(const char *text) {
    LineVec lv;
    lines_init(&lv);
    const char *line = text;
    while (*line != '\0') {
        const char *end = strchr(line, '\n');
        size_t n = end ? (size_t)(end - line) : strlen(line);
        char *copy = (char *)malloc(n + 1);
        if (copy == NULL) die_oom();
        memcpy(copy, line, n);
        copy[n] = '\0';
        lines_push(&lv, copy);
        if (end == NULL) break;
        line = end + 1;
    }
    return lv;
}

static char *join_lines(LineVec *lv, int trailing_newline) {
    StrBuf out;
    sb_init(&out);
    for (size_t i = 0; i < lv->len; i++) {
        sb_append(&out, lv->items[i]);
        if (i + 1 < lv->len || trailing_newline) sb_append(&out, "\n");
    }
    return sb_take(&out);
}

static int starts_with(const char *s, const char *prefix) {
    return strncmp(s, prefix, strlen(prefix)) == 0;
}

static int is_ident_start(char ch) {
    return (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z') || ch == '_';
}

static int is_ident_continue(char ch) {
    return is_ident_start(ch) || (ch >= '0' && ch <= '9');
}

static char *read_file(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (fp == NULL) {
        fprintf(stderr, "error: cannot open %s: %s\n", path, strerror(errno));
        return NULL;
    }
    if (fseek(fp, 0, SEEK_END) != 0) {
        fprintf(stderr, "error: cannot seek %s\n", path);
        fclose(fp);
        return NULL;
    }
    long size = ftell(fp);
    if (size < 0) {
        fprintf(stderr, "error: cannot size %s\n", path);
        fclose(fp);
        return NULL;
    }
    if (fseek(fp, 0, SEEK_SET) != 0) {
        fprintf(stderr, "error: cannot rewind %s\n", path);
        fclose(fp);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (buf == NULL) die_oom();
    size_t nread = fread(buf, 1, (size_t)size, fp);
    if (nread != (size_t)size) {
        fprintf(stderr, "error: short read for %s\n", path);
        free(buf);
        fclose(fp);
        return NULL;
    }
    buf[size] = '\0';
    fclose(fp);
    return buf;
}

static int write_file_text(const char *path, const char *text) {
    FILE *fp = fopen(path, "wb");
    if (fp == NULL) {
        fprintf(stderr, "error: cannot write %s: %s\n", path, strerror(errno));
        return 1;
    }
    if (fputs(text, fp) < 0) {
        fprintf(stderr, "error: cannot write %s: %s\n", path, strerror(errno));
        fclose(fp);
        return 1;
    }
    if (fclose(fp) != 0) {
        fprintf(stderr, "error: cannot close %s: %s\n", path, strerror(errno));
        return 1;
    }
    return 0;
}

static int has_vais_suffix(const char *path) {
    size_t n = strlen(path);
    return n >= 5 && strcmp(path + n - 5, ".vais") == 0;
}

static char *strip_line_comment(const char *line, size_t n) {
    StrBuf out;
    sb_init(&out);
    char delim = '\0';
    int escaped = 0;
    for (size_t i = 0; i < n; i++) {
        char ch = line[i];
        if (escaped) {
            sb_append_n(&out, &ch, 1);
            escaped = 0;
            continue;
        }
        if ((delim == '"' || delim == '\'') && ch == '\\') {
            sb_append_n(&out, &ch, 1);
            escaped = 1;
            continue;
        }
        if (ch == '"' || ch == '`') {
            if (delim == '\0') delim = ch;
            else if (delim == ch) delim = '\0';
            sb_append_n(&out, &ch, 1);
            continue;
        }
        if (delim == '\0' && ch == '#') break;
        sb_append_n(&out, &ch, 1);
    }
    while (out.len > 0 && (out.data[out.len - 1] == ' ' || out.data[out.len - 1] == '\t' || out.data[out.len - 1] == '\r')) {
        out.len--;
        out.data[out.len] = '\0';
    }
    return out.data;
}

static const char *skip_ws(const char *p) {
    while (*p == ' ' || *p == '\t' || *p == '\r') p++;
    return p;
}

static int line_starts_stmt(const char *s, const char *word) {
    s = skip_ws(s);
    size_t n = strlen(word);
    if (strncmp(s, word, n) != 0) return 0;
    char next = s[n];
    return next == '\0' || next == ' ' || next == '\t' || next == '(';
}

static int statement_needs_semicolon(const char *line) {
    const char *s = skip_ws(line);
    size_t n = strlen(s);
    if (n == 0) return 0;
    if (s[n - 1] == ';' || s[n - 1] == '{' || s[n - 1] == ',' || strcmp(s, "{") == 0 || strcmp(s, "}") == 0) return 0;
    if (starts_with(s, "fn ") || starts_with(s, "struct ") || starts_with(s, "enum ") ||
        starts_with(s, "if ") || starts_with(s, "else") || starts_with(s, "while ") || starts_with(s, "for ")) {
        return 0;
    }
    if (line_starts_stmt(s, "let") || line_starts_stmt(s, "return") ||
        starts_with(s, "print(") || starts_with(s, "putchar(") || starts_with(s, "puts(")) {
        return 1;
    }
    if (is_ident_start(*s)) {
        const char *p = s;
        while (is_ident_continue(*p) || *p == '.') p++;
        p = skip_ws(p);
        if (*p == '=' || *p == '(') return 1;
    }
    return 0;
}

static char *lower_struct_field_line(const char *line) {
    const char *s = skip_ws(line);
    if (!is_ident_start(*s)) return strdup(line);
    const char *name_start = s;
    const char *p = s + 1;
    while (is_ident_continue(*p)) p++;
    const char *after_name = skip_ws(p);
    if (*after_name != ':') return strdup(line);
    const char *q = after_name + 1;
    int saw_type = 0;
    while (*q != '\0' && *q != ',') {
        if (*q != ' ' && *q != '\t' && *q != '\r') saw_type = 1;
        q++;
    }
    if (!saw_type) return strdup(line);
    if (*q == ',') {
        const char *rest = skip_ws(q + 1);
        if (*rest != '\0') return strdup(line);
    }
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(name_start - line));
    sb_append_n(&out, name_start, (size_t)(p - name_start));
    if (*q == ',') sb_append(&out, ",");
    return out.data;
}

static int append_lowered_struct_part(StrBuf *out, const char *start, const char *end) {
    while (start < end && (*start == ' ' || *start == '\t' || *start == '\r')) start++;
    while (end > start && (end[-1] == ' ' || end[-1] == '\t' || end[-1] == '\r')) end--;
    if (start >= end) return 0;
    const char *p = start;
    if (!is_ident_start(*p)) {
        sb_append_n(out, start, (size_t)(end - start));
        return 0;
    }
    p++;
    while (p < end && is_ident_continue(*p)) p++;
    const char *after_name = skip_ws(p);
    if (after_name >= end || *after_name != ':') {
        sb_append_n(out, start, (size_t)(end - start));
        return 0;
    }
    const char *type_start = skip_ws(after_name + 1);
    if (type_start >= end) {
        sb_append_n(out, start, (size_t)(end - start));
        return 0;
    }
    sb_append_n(out, start, (size_t)(p - start));
    return 1;
}

static char *lower_struct_one_line_fields(const char *line) {
    const char *trim = skip_ws(line);
    if (!starts_with(trim, "struct ")) return strdup(line);
    const char *open = strchr(trim, '{');
    const char *close = open == NULL ? NULL : strchr(open + 1, '}');
    if (open == NULL || close == NULL) return strdup(line);

    StrBuf body;
    sb_init(&body);
    int changed = 0;
    int first = 1;
    const char *part_start = open + 1;
    int depth = 0;
    char delim = '\0';
    int escaped = 0;
    for (const char *p = open + 1; p <= close; p++) {
        char ch = *p;
        if (p < close) {
            if (escaped) {
                escaped = 0;
                continue;
            }
            if (delim == '"' && ch == '\\') {
                escaped = 1;
                continue;
            }
            if (ch == '"' || ch == '`') {
                if (delim == '\0') delim = ch;
                else if (delim == ch) delim = '\0';
                continue;
            }
            if (delim == '\0') {
                if (ch == '(' || ch == '[' || ch == '{' || ch == '<') depth++;
                else if (ch == ')' || ch == ']' || ch == '}' || ch == '>') depth--;
            }
        }
        if ((p == close || (ch == ',' && depth == 0 && delim == '\0'))) {
            if (!first) sb_append(&body, ", ");
            changed |= append_lowered_struct_part(&body, part_start, p);
            first = 0;
            part_start = p + 1;
        }
    }
    if (!changed) {
        free(body.data);
        return strdup(line);
    }

    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(open + 1 - line));
    sb_append(&out, body.data);
    sb_append(&out, close);
    free(body.data);
    return out.data;
}

static char *replace_print_token(const char *line) {
    StrBuf out;
    sb_init(&out);
    int in_dquote = 0;
    int in_backtick = 0;
    int escaped = 0;
    for (size_t i = 0; line[i] != '\0';) {
        if (!in_dquote && !in_backtick && strncmp(line + i, "print", 5) == 0) {
            int before_ok = i == 0 || !is_ident_continue(line[i - 1]);
            const char *after_name = line + i + 5;
            const char *after_ws = skip_ws(after_name);
            if (before_ok && *after_ws == '(') {
                sb_append(&out, "puts(");
                i = (size_t)(after_ws - line) + 1;
                continue;
            }
        }

        char c = line[i];
        sb_append_n(&out, line + i, 1);
        if (escaped) {
            escaped = 0;
        } else if (in_dquote && c == '\\') {
            escaped = 1;
        } else if (!in_backtick && c == '"') {
            in_dquote = !in_dquote;
        } else if (!in_dquote && c == '`') {
            in_backtick = !in_backtick;
        }
        i++;
    }
    return out.data;
}

static char *lower_fn_int_annotations(const char *line) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "fn ")) return strdup(line);
    StrBuf out;
    sb_init(&out);
    int in_params = 0;
    for (size_t i = 0; line[i] != '\0';) {
        if (line[i] == '(') {
            in_params = 1;
            sb_append_n(&out, line + i, 1);
            i++;
            continue;
        }
        if (line[i] == ')') {
            in_params = 0;
            sb_append_n(&out, line + i, 1);
            i++;
            const char *rest = line + i;
            const char *r = skip_ws(rest);
            if (starts_with(r, "->")) {
                const char *t = skip_ws(r + 2);
                if (starts_with(t, "Int")) {
                    const char *after = t + 3;
                    if (!is_ident_continue(*after)) {
                        i = (size_t)(after - line);
                    }
                }
            }
            continue;
        }
        if (in_params && line[i] == ':') {
            const char *t = skip_ws(line + i + 1);
            if (starts_with(t, "Int") && !is_ident_continue(t[3])) {
                i = (size_t)(t + 3 - line);
                continue;
            }
        }
        sb_append_n(&out, line + i, 1);
        i++;
    }
    return out.data;
}

static char *lower_let_int_annotation(const char *line) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "let ")) return strdup(line);
    const char *colon = strchr(s, ':');
    if (colon == NULL) return strdup(line);
    const char *eq = strchr(s, '=');
    if (eq == NULL || colon > eq) return strdup(line);
    const char *t = skip_ws(colon + 1);
    if (!starts_with(t, "Int") || is_ident_continue(t[3])) return strdup(line);
    const char *after = skip_ws(t + 3);
    if (after != eq) return strdup(line);
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(colon - line));
    sb_append(&out, " ");
    sb_append(&out, eq);
    return out.data;
}

static char *trim_copy(const char *text) {
    const char *start = skip_ws(text);
    const char *end = start + strlen(start);
    while (end > start && (end[-1] == ' ' || end[-1] == '\t' || end[-1] == '\r' || end[-1] == ',' || end[-1] == ';')) end--;
    char *out = (char *)malloc((size_t)(end - start) + 1);
    if (out == NULL) die_oom();
    memcpy(out, start, (size_t)(end - start));
    out[end - start] = '\0';
    return out;
}

static char *substr_copy(const char *start, size_t n) {
    char *out = (char *)malloc(n + 1);
    if (out == NULL) die_oom();
    memcpy(out, start, n);
    out[n] = '\0';
    return out;
}

static int is_string_delim_c(char ch) {
    return ch == '"' || ch == '`';
}

static int skip_string_literal_c(const char *text, int start) {
    char delim = text[start];
    if (!is_string_delim_c(delim)) return start + 1;
    for (int i = start + 1; text[i] != '\0'; i++) {
        if (delim == '"' && text[i] == '\\' && text[i + 1] != '\0') {
            i++;
            continue;
        }
        if (text[i] == delim) return i + 1;
    }
    return -1;
}

static int skip_char_literal_c(const char *text, int start) {
    if (text[start] != '\'') return start + 1;
    if (text[start + 1] == '\0') return -1;
    if (text[start + 1] == '\\') {
        if (text[start + 2] == '\0' || text[start + 3] != '\'') return -1;
        return start + 4;
    }
    if (text[start + 2] == '\'') return start + 3;
    return -1;
}

static void sb_append_c_escaped_byte(StrBuf *out, char ch) {
    unsigned char c = (unsigned char)ch;
    if (c == '\\') {
        sb_append(out, "\\\\");
    } else if (c == '"') {
        sb_append(out, "\\\"");
    } else if (c == '\n') {
        sb_append(out, "\\n");
    } else if (c == '\r') {
        sb_append(out, "\\r");
    } else if (c == '\t') {
        sb_append(out, "\\t");
    } else if (c < 32 || c >= 127) {
        char buf[8];
        snprintf(buf, sizeof(buf), "\\x%02x", c);
        sb_append(out, buf);
    } else {
        sb_append_n(out, &ch, 1);
    }
}

static int split_top_level_commas_c(const char *text, char **parts, int max_parts) {
    int count = 0;
    int depth = 0;
    const char *start = text;
    for (const char *p = text; ; p++) {
        char ch = *p;
        if (is_string_delim_c(ch)) {
            int end = skip_string_literal_c(text, (int)(p - text));
            if (end < 0) {
                p = text + strlen(text);
                ch = *p;
            } else {
                p = text + end - 1;
                continue;
            }
        }
        if (ch == '\'') {
            int end = skip_char_literal_c(text, (int)(p - text));
            if (end < 0) {
                p = text + strlen(text);
                ch = *p;
            } else {
                p = text + end - 1;
                continue;
            }
        }
        if (ch == '(' || ch == '[' || ch == '{') depth++;
        if (ch == ')' || ch == ']' || ch == '}') depth--;
        if ((ch == ',' && depth == 0) || ch == '\0') {
            if (count >= max_parts) return -1;
            char *raw = substr_copy(start, (size_t)(p - start));
            parts[count++] = trim_copy(raw);
            free(raw);
            if (ch == '\0') break;
            start = p + 1;
        }
        if (ch == '\0') break;
    }
    return count;
}

static int split_top_level_type_commas_c(const char *text, char **parts, int max_parts) {
    int count = 0;
    int depth = 0;
    const char *start = text;
    for (const char *p = text; ; p++) {
        char ch = *p;
        if (is_string_delim_c(ch)) {
            int end = skip_string_literal_c(text, (int)(p - text));
            if (end < 0) {
                p = text + strlen(text);
                ch = *p;
            } else {
                p = text + end - 1;
                continue;
            }
        }
        if (ch == '\'') {
            int end = skip_char_literal_c(text, (int)(p - text));
            if (end < 0) {
                p = text + strlen(text);
                ch = *p;
            } else {
                p = text + end - 1;
                continue;
            }
        }
        if (ch == '(' || ch == '[' || ch == '{' || ch == '<') depth++;
        if (ch == ')' || ch == ']' || ch == '}' || ch == '>') depth--;
        if ((ch == ',' && depth == 0) || ch == '\0') {
            if (count >= max_parts) return -1;
            char *raw = substr_copy(start, (size_t)(p - start));
            parts[count++] = trim_copy(raw);
            free(raw);
            if (ch == '\0') break;
            start = p + 1;
        }
        if (ch == '\0') break;
    }
    return count;
}

static VariantInfo *find_variant(EnumInfo *info, const char *name) {
    for (int i = 0; i < info->count; i++) {
        if (strcmp(info->variants[i].name, name) == 0) return &info->variants[i];
    }
    return NULL;
}

static int parse_single_field_struct_line(const char *line, char **name_out) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "struct ")) return 0;
    s += 7;
    const char *name_start = skip_ws(s);
    if (!is_ident_start(*name_start)) return 0;
    const char *name_end = name_start + 1;
    while (is_ident_continue(*name_end)) name_end++;
    const char *open = strchr(name_end, '{');
    const char *close = open == NULL ? NULL : strrchr(open + 1, '}');
    if (open == NULL || close == NULL || close <= open) return 0;
    char *body = substr_copy(open + 1, (size_t)(close - open - 1));
    char *parts[2] = {0};
    int n = split_top_level_commas_c(body, parts, 2);
    free(body);
    for (int i = 0; i < n && i < 2; i++) free(parts[i]);
    if (n != 1) return 0;
    *name_out = substr_copy(name_start, (size_t)(name_end - name_start));
    return 1;
}

static int enum_known_single_field_struct(EnumInfo *info, const char *name) {
    for (int i = 0; i < info->struct_count; i++) {
        if (strcmp(info->struct_names[i], name) == 0) return 1;
    }
    return 0;
}

static int parse_enum_line(const char *line, EnumInfo *info) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "enum ")) return 0;
    s += 5;
    const char *name_start = skip_ws(s);
    if (!is_ident_start(*name_start)) return 0;
    const char *name_end = name_start + 1;
    while (is_ident_continue(*name_end)) name_end++;
    const char *open = strchr(name_end, '{');
    const char *close = strrchr(name_end, '}');
    if (open == NULL || close == NULL || close <= open) return 0;
    info->name = substr_copy(name_start, (size_t)(name_end - name_start));
    info->count = 0;
    info->is_payload = strchr(open, '(') != NULL;
    char *body = substr_copy(open + 1, (size_t)(close - open - 1));
    char *parts[16] = {0};
    int n = split_top_level_commas_c(body, parts, 16);
    free(body);
    if (n <= 0) return 0;
    for (int i = 0; i < n; i++) {
        char *part = parts[i];
        char *paren = strchr(part, '(');
        char *vname = NULL;
        int fields = 0;
        if (paren != NULL) {
            vname = trim_copy(substr_copy(part, (size_t)(paren - part)));
            char *close_paren = strrchr(paren, ')');
            if (close_paren == NULL) fields = 0;
            else {
                char *inside = substr_copy(paren + 1, (size_t)(close_paren - paren - 1));
                char *field_parts[8] = {0};
                fields = split_top_level_commas_c(inside, field_parts, 8);
                for (int k = 0; k < fields; k++) {
                    int is_struct = enum_known_single_field_struct(info, field_parts[k]);
                    if (strcmp(field_parts[k], "Int") != 0 && strcmp(field_parts[k], info->name) != 0 && is_struct == 0) {
                        fields = -1;
                    } else {
                        info->variants[i].field_is_struct[k] = is_struct;
                    }
                    free(field_parts[k]);
                }
                free(inside);
            }
        } else {
            vname = trim_copy(part);
        }
        if (fields < 0) {
            free(vname);
            free(part);
            for (int k = i + 1; k < n; k++) free(parts[k]);
            return 0;
        }
        info->variants[i].name = vname;
        info->variants[i].tag = i;
        info->variants[i].field_count = fields;
        free(part);
    }
    info->count = n;
    return 1;
}

static void free_enum_info(EnumInfo *info) {
    free(info->name);
    for (int i = 0; i < info->count; i++) free(info->variants[i].name);
    for (int i = 0; i < info->struct_count; i++) free(info->struct_names[i]);
    memset(info, 0, sizeof(*info));
}

static int generic_type_at(
    const char *line,
    size_t i,
    const char *name,
    const char *arg1,
    const char *arg2,
    size_t *end_out
) {
    size_t name_len = strlen(name);
    if (i > 0 && is_ident_continue(line[i - 1])) return 0;
    if (strncmp(line + i, name, name_len) != 0) return 0;
    if (is_ident_continue(line[i + name_len])) return 0;
    size_t k = i + name_len;
    while (line[k] == ' ' || line[k] == '\t') k++;
    if (line[k] != '<') return 0;
    k++;
    while (line[k] == ' ' || line[k] == '\t') k++;
    size_t arg1_len = strlen(arg1);
    if (strncmp(line + k, arg1, arg1_len) != 0) return 0;
    if (is_ident_continue(line[k + arg1_len])) return 0;
    k += arg1_len;
    while (line[k] == ' ' || line[k] == '\t') k++;
    if (arg2 != NULL) {
        if (line[k] != ',') return 0;
        k++;
        while (line[k] == ' ' || line[k] == '\t') k++;
        size_t arg2_len = strlen(arg2);
        if (strncmp(line + k, arg2, arg2_len) != 0) return 0;
        if (is_ident_continue(line[k + arg2_len])) return 0;
        k += arg2_len;
        while (line[k] == ' ' || line[k] == '\t') k++;
    }
    if (line[k] != '>') return 0;
    k++;
    if (is_ident_continue(line[k])) return 0;
    *end_out = k;
    return 1;
}

static int contains_generic_type(const char *text, const char *name, const char *arg1, const char *arg2) {
    for (size_t i = 0; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, (int)i);
            if (end < 0) return 0;
            i = (size_t)end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, (int)i);
            if (end < 0) return 0;
            i = (size_t)end - 1;
            continue;
        }
        size_t end = 0;
        if (generic_type_at(text, i, name, arg1, arg2, &end)) return 1;
    }
    return 0;
}

static int option_constructor_marker_at(const char *text, size_t i) {
    if (strncmp(text + i, "Some", 4) == 0) {
        if (i > 0 && is_ident_continue(text[i - 1])) return 0;
        if (is_ident_continue(text[i + 4])) return 0;
        size_t k = i + 4;
        while (text[k] == ' ' || text[k] == '\t') k++;
        return text[k] == '(';
    }
    if (strncmp(text + i, "None", 4) == 0) {
        if (i > 0 && is_ident_continue(text[i - 1])) return 0;
        if (is_ident_continue(text[i + 4])) return 0;
        return 1;
    }
    return 0;
}

static int contains_option_constructor_marker(const char *text) {
    for (size_t i = 0; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, (int)i);
            if (end < 0) return 0;
            i = (size_t)end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, (int)i);
            if (end < 0) return 0;
            i = (size_t)end - 1;
            continue;
        }
        if (option_constructor_marker_at(text, i)) return 1;
    }
    return 0;
}

static char *replace_generic_type(
    const char *line,
    const char *name,
    const char *arg1,
    const char *arg2,
    const char *replacement
) {
    StrBuf out;
    sb_init(&out);
    int changed = 0;
    for (size_t i = 0; line[i] != '\0';) {
        if (is_string_delim_c(line[i])) {
            int end = skip_string_literal_c(line, (int)i);
            if (end < 0) {
                sb_append(&out, line + i);
                break;
            }
            sb_append_n(&out, line + i, (size_t)end - i);
            i = (size_t)end;
            continue;
        }
        if (line[i] == '\'') {
            int end = skip_char_literal_c(line, (int)i);
            if (end < 0) {
                sb_append(&out, line + i);
                break;
            }
            sb_append_n(&out, line + i, (size_t)end - i);
            i = (size_t)end;
            continue;
        }
        size_t end = 0;
        if (generic_type_at(line, i, name, arg1, arg2, &end)) {
            sb_append(&out, replacement);
            i = end;
            changed = 1;
            continue;
        }
        sb_append_n(&out, line + i, 1);
        i++;
    }
    if (!changed) {
        free(out.data);
        return strdup(line);
    }
    return sb_take(&out);
}

static void init_builtin_option_int(EnumInfo *info) {
    info->name = strdup("Option");
    if (info->name == NULL) die_oom();
    info->builtin_kind = 1;
    info->is_payload = 1;
    info->count = 2;
    info->variants[0].name = strdup("Some");
    info->variants[0].tag = 0;
    info->variants[0].field_count = 1;
    info->variants[1].name = strdup("None");
    info->variants[1].tag = 1;
    info->variants[1].field_count = 0;
    if (info->variants[0].name == NULL || info->variants[1].name == NULL) die_oom();
}

static void init_builtin_result_int_int(EnumInfo *info) {
    info->name = strdup("Result");
    if (info->name == NULL) die_oom();
    info->builtin_kind = 2;
    info->is_payload = 1;
    info->count = 2;
    info->variants[0].name = strdup("Ok");
    info->variants[0].tag = 0;
    info->variants[0].field_count = 1;
    info->variants[1].name = strdup("Err");
    info->variants[1].tag = 1;
    info->variants[1].field_count = 1;
    if (info->variants[0].name == NULL || info->variants[1].name == NULL) die_oom();
}

static char *replace_exact(const char *line, const char *needle, const char *replacement) {
    const char *p = strstr(line, needle);
    if (p == NULL) return strdup(line);
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(p - line));
    sb_append(&out, replacement);
    sb_append(&out, p + strlen(needle));
    return sb_take(&out);
}

static char *replace_enum_type_token(const char *line, const char *needle, const char *replacement) {
    size_t needle_len = strlen(needle);
    StrBuf out;
    sb_init(&out);
    int changed = 0;
    for (size_t i = 0; line[i] != '\0';) {
        if (strncmp(line + i, needle, needle_len) == 0) {
            char after = line[i + needle_len];
            if (!is_ident_continue(after) && after != '.') {
                sb_append(&out, replacement);
                i += needle_len;
                changed = 1;
                continue;
            }
        }
        sb_append_n(&out, line + i, 1);
        i++;
    }
    if (!changed) {
        free(out.data);
        return strdup(line);
    }
    return sb_take(&out);
}

static char *replace_enum_types(const char *line, EnumInfo *info) {
    if (info->builtin_kind == 1) {
        return replace_generic_type(line, "Option", "Int", NULL, "Int");
    }
    if (info->builtin_kind == 2) {
        return replace_generic_type(line, "Result", "Int", "Int", "Int");
    }
    char needle[160];
    snprintf(needle, sizeof(needle), ": %s", info->name);
    char *step = replace_enum_type_token(line, needle, ": Int");
    snprintf(needle, sizeof(needle), "-> %s", info->name);
    char *out = replace_enum_type_token(step, needle, "-> Int");
    free(step);
    return out;
}

static int find_matching_paren_c(const char *text, int open_index) {
    int depth = 0;
    for (int i = open_index; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '(') depth++;
        if (text[i] == ')') {
            depth--;
            if (depth == 0) return i;
        }
    }
    return -1;
}

static int find_matching_bracket_c(const char *text, int open_index) {
    int paren_depth = 0;
    int bracket_depth = 0;
    int brace_depth = 0;
    for (int i = open_index; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '(') paren_depth++;
        else if (text[i] == ')') paren_depth--;
        else if (text[i] == '{') brace_depth++;
        else if (text[i] == '}') brace_depth--;
        else if (text[i] == '[') bracket_depth++;
        else if (text[i] == ']') {
            bracket_depth--;
            if (bracket_depth == 0 && paren_depth == 0 && brace_depth == 0) return i;
        }
    }
    return -1;
}

static char *rewrite_constructors(const char *line, EnumInfo *info);

static void free_split_parts(char **parts, int count, int max_parts) {
    int n = count;
    if (n < 0 || n > max_parts) n = max_parts;
    for (int i = 0; i < n; i++) {
        free(parts[i]);
        parts[i] = NULL;
    }
}

static char *extract_single_field_struct_payload_value(const char *arg) {
    char *trimmed = trim_copy(arg);
    const char *s = skip_ws(trimmed);
    if (!is_ident_start(*s)) return trimmed;
    const char *p = s + 1;
    while (is_ident_continue(*p)) p++;
    p = skip_ws(p);
    if (*p != '{') return trimmed;
    const char *close = strrchr(p + 1, '}');
    if (close == NULL || close <= p) return trimmed;
    char *body = substr_copy(p + 1, (size_t)(close - p - 1));
    char *parts[2] = {0};
    int n = split_top_level_commas_c(body, parts, 2);
    free(body);
    if (n != 1) {
        free_split_parts(parts, n, 2);
        return trimmed;
    }
    char *colon = strchr(parts[0], ':');
    if (colon == NULL) {
        free_split_parts(parts, n, 2);
        return trimmed;
    }
    char *value = trim_copy(colon + 1);
    free_split_parts(parts, n, 2);
    free(trimmed);
    return value;
}

static char *encode_payload_call(VariantInfo *variant, char **args, int argc, EnumInfo *info) {
    if (argc != variant->field_count) return strdup("0");
    StrBuf payload;
    sb_init(&payload);
    for (int i = 0; i < argc; i++) {
        char *payload_arg = variant->field_is_struct[i] ? extract_single_field_struct_payload_value(args[i]) : strdup(args[i]);
        char *rewritten = rewrite_constructors(payload_arg, info);
        if (i > 0) sb_append(&payload, " + ");
        long long factor = 1;
        for (int k = 0; k < i; k++) factor *= 1000000LL;
        if (factor != 1) {
            char factor_text[64];
            snprintf(factor_text, sizeof(factor_text), "%lld * ", factor);
            sb_append(&payload, factor_text);
        }
        sb_append(&payload, "((");
        sb_append(&payload, rewritten);
        sb_append(&payload, "))");
        free(payload_arg);
        free(rewritten);
    }
    char tag_text[64];
    snprintf(tag_text, sizeof(tag_text), "(%d + %d * (", variant->tag, info->count);
    StrBuf out;
    sb_init(&out);
    sb_append(&out, tag_text);
    sb_append(&out, payload.len ? payload.data : "0");
    sb_append(&out, "))");
    free(payload.data);
    return sb_take(&out);
}

static char *rewrite_constructors(const char *line, EnumInfo *info) {
    StrBuf out;
    sb_init(&out);
    for (int i = 0; line[i] != '\0';) {
        if (is_string_delim_c(line[i])) {
            int end = skip_string_literal_c(line, i);
            if (end < 0) {
                sb_append(&out, line + i);
                break;
            }
            sb_append_n(&out, line + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (line[i] == '\'') {
            int end = skip_char_literal_c(line, i);
            if (end < 0) {
                sb_append(&out, line + i);
                break;
            }
            sb_append_n(&out, line + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (!is_ident_start(line[i])) {
            sb_append_n(&out, line + i, 1);
            i++;
            continue;
        }
        int start = i;
        i++;
        while (is_ident_continue(line[i])) i++;
        char *first = substr_copy(line + start, (size_t)(i - start));
        int cursor = i;
        while (line[cursor] == ' ' || line[cursor] == '\t') cursor++;
        VariantInfo *variant = NULL;
        int replace_end = i;
        if (strcmp(first, info->name) == 0 && line[cursor] == '.') {
            int vstart = cursor + 1;
            while (line[vstart] == ' ' || line[vstart] == '\t') vstart++;
            int vend = vstart;
            while (is_ident_continue(line[vend])) vend++;
            char *vname = substr_copy(line + vstart, (size_t)(vend - vstart));
            variant = find_variant(info, vname);
            free(vname);
            replace_end = vend;
            cursor = vend;
            while (line[cursor] == ' ' || line[cursor] == '\t') cursor++;
        } else {
            variant = find_variant(info, first);
        }
        if (variant == NULL) {
            sb_append_n(&out, line + start, (size_t)(i - start));
            free(first);
            continue;
        }
        if (info->is_payload && variant->field_count > 0 && line[cursor] == '(') {
            int close = find_matching_paren_c(line, cursor);
            if (close < 0) {
                sb_append_n(&out, line + start, (size_t)(i - start));
                free(first);
                continue;
            }
            char *inside = substr_copy(line + cursor + 1, (size_t)(close - cursor - 1));
            char *args[8] = {0};
            int argc = split_top_level_commas_c(inside, args, 8);
            char *encoded = encode_payload_call(variant, args, argc, info);
            sb_append(&out, encoded);
            for (int k = 0; k < argc; k++) free(args[k]);
            free(encoded);
            free(inside);
            i = close + 1;
        } else {
            char tag_text[32];
            snprintf(tag_text, sizeof(tag_text), "%d", variant->tag);
            sb_append(&out, tag_text);
            i = replace_end;
        }
        free(first);
    }
    return sb_take(&out);
}

static char *parse_match_expr(const char *line) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "match ")) return NULL;
    s += 6;
    const char *open = strrchr(s, '{');
    if (open == NULL) return NULL;
    return trim_copy(substr_copy(s, (size_t)(open - s)));
}

static int parse_inline_match_let(const char *line, char **name, char **match_expr, char **arms_text) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "let ") || is_ident_continue(s[3])) return 0;
    s = skip_ws(s + 4);
    if (starts_with(s, "mut") && !is_ident_continue(s[3])) {
        s = skip_ws(s + 3);
    }
    if (!is_ident_start(*s)) return 0;
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *name_end = s;
    s = skip_ws(s);
    if (*s != '=') return 0;
    s = skip_ws(s + 1);
    if (!starts_with(s, "match ") || is_ident_continue(s[5])) return 0;
    s = skip_ws(s + 6);
    const char *open = strrchr(s, '{');
    const char *close = open == NULL ? NULL : strrchr(open + 1, '}');
    if (open == NULL || close == NULL || close <= open) return 0;
    *name = substr_copy(name_start, (size_t)(name_end - name_start));
    *match_expr = trim_copy(substr_copy(s, (size_t)(open - s)));
    *arms_text = substr_copy(open + 1, (size_t)(close - open - 1));
    return 1;
}

static int parse_try_let(const char *line, char **name, char **expr) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "let ") || is_ident_continue(s[3])) return 0;
    s = skip_ws(s + 4);
    if (starts_with(s, "mut") && !is_ident_continue(s[3])) {
        s = skip_ws(s + 3);
    }
    if (!is_ident_start(*s)) return 0;
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *name_end = s;
    s = skip_ws(s);
    if (*s != '=') return 0;
    s = skip_ws(s + 1);
    const char *q = strrchr(s, '?');
    if (q == NULL) return 0;
    const char *tail = skip_ws(q + 1);
    if (*tail == ';') tail = skip_ws(tail + 1);
    if (*tail != '\0') return 0;
    *name = substr_copy(name_start, (size_t)(name_end - name_start));
    *expr = trim_copy(substr_copy(s, (size_t)(q - s)));
    return 1;
}

static int parse_arm(const char *line, char **pattern, char **expr) {
    const char *arrow = strstr(line, "=>");
    if (arrow == NULL) return 0;
    char *pat_raw = substr_copy(line, (size_t)(arrow - line));
    *pattern = trim_copy(pat_raw);
    free(pat_raw);
    const char *body = skip_ws(arrow + 2);
    if (starts_with(body, "return ")) body += 7;
    *expr = trim_copy(body);
    return 1;
}

static char *rewrite_struct_payload_binder_expr(const char *expr, VariantInfo *variant, char **binders, int binder_count) {
    StrBuf out;
    sb_init(&out);
    int changed = 0;
    for (int i = 0; expr[i] != '\0';) {
        if (is_string_delim_c(expr[i])) {
            int end = skip_string_literal_c(expr, i);
            if (end < 0) {
                sb_append(&out, expr + i);
                break;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (expr[i] == '\'') {
            int end = skip_char_literal_c(expr, i);
            if (end < 0) {
                sb_append(&out, expr + i);
                break;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (!is_ident_start(expr[i])) {
            sb_append_n(&out, expr + i, 1);
            i++;
            continue;
        }
        int start = i;
        i++;
        while (is_ident_continue(expr[i])) i++;
        char *name = substr_copy(expr + start, (size_t)(i - start));
        int matched = 0;
        for (int k = 0; k < binder_count && k < variant->field_count; k++) {
            if (!variant->field_is_struct[k] || strcmp(name, binders[k]) != 0) continue;
            int cursor = i;
            while (expr[cursor] == ' ' || expr[cursor] == '\t') cursor++;
            if (expr[cursor] != '.') continue;
            int fstart = cursor + 1;
            while (expr[fstart] == ' ' || expr[fstart] == '\t') fstart++;
            if (!is_ident_start(expr[fstart])) continue;
            int fend = fstart + 1;
            while (is_ident_continue(expr[fend])) fend++;
            sb_append(&out, name);
            i = fend;
            changed = 1;
            matched = 1;
            break;
        }
        if (!matched) {
            sb_append_n(&out, expr + start, (size_t)(i - start));
        }
        free(name);
    }
    if (!changed) {
        free(out.data);
        return strdup(expr);
    }
    return sb_take(&out);
}

static int is_int_match_pattern(const char *pattern) {
    const char *p = skip_ws(pattern);
    if (*p == '-' || *p == '+') p++;
    if (*p < '0' || *p > '9') return 0;
    while (*p >= '0' && *p <= '9') p++;
    p = skip_ws(p);
    return *p == '\0';
}

static int is_wildcard_match_pattern(const char *pattern) {
    const char *p = skip_ws(pattern);
    if (*p != '_') return 0;
    p++;
    p = skip_ws(p);
    return *p == '\0';
}

static char *lower_enum_text(const char *text) {
    LineVec lines = split_lines(text);
    EnumInfo info;
    memset(&info, 0, sizeof(info));
    int enum_line = -1;
    for (size_t i = 0; i < lines.len; i++) {
        char *struct_name = NULL;
        if (parse_single_field_struct_line(lines.items[i], &struct_name)) {
            if (info.struct_count < 16) {
                info.struct_names[info.struct_count++] = struct_name;
            } else {
                free(struct_name);
            }
        }
    }
    for (size_t i = 0; i < lines.len; i++) {
        if (parse_enum_line(lines.items[i], &info)) {
            enum_line = (int)i;
            break;
        }
    }
    if (enum_line < 0 && (contains_generic_type(text, "Option", "Int", NULL) || contains_option_constructor_marker(text))) {
        init_builtin_option_int(&info);
    } else if (enum_line < 0 && contains_generic_type(text, "Result", "Int", "Int")) {
        init_builtin_result_int_int(&info);
    }
    int has_enum = enum_line >= 0;
    if (info.builtin_kind != 0) has_enum = 1;

    LineVec out;
    lines_init(&out);
    int try_count = 0;
    for (size_t i = 0; i < lines.len; i++) {
        if (has_enum && (int)i == enum_line) continue;
        char *typed = has_enum ? replace_enum_types(lines.items[i], &info) : strdup(lines.items[i]);
        char *try_name = NULL;
        char *try_expr = NULL;
        if ((info.builtin_kind == 1 || info.builtin_kind == 2) && parse_try_let(typed, &try_name, &try_expr)) {
            char tmp_name[64];
            snprintf(tmp_name, sizeof(tmp_name), "__vais_try%d", try_count++);
            char *rewritten_try_expr = rewrite_constructors(try_expr, &info);
            StrBuf bind_tmp;
            sb_init(&bind_tmp);
            sb_append(&bind_tmp, "    let ");
            sb_append(&bind_tmp, tmp_name);
            sb_append(&bind_tmp, " = ");
            sb_append(&bind_tmp, rewritten_try_expr);
            lines_push(&out, sb_take(&bind_tmp));
            StrBuf propagate;
            sb_init(&propagate);
            sb_append(&propagate, "    if ");
            sb_append(&propagate, tmp_name);
            if (info.builtin_kind == 1) {
                sb_append(&propagate, " % 2 != 0 { return 1 }");
            } else {
                sb_append(&propagate, " % 2 != 0 { return ");
                sb_append(&propagate, tmp_name);
                sb_append(&propagate, " }");
            }
            lines_push(&out, sb_take(&propagate));
            StrBuf bind_value;
            sb_init(&bind_value);
            sb_append(&bind_value, "    let ");
            sb_append(&bind_value, try_name);
            sb_append(&bind_value, " = (");
            sb_append(&bind_value, tmp_name);
            sb_append(&bind_value, " / 2) % 1000000");
            lines_push(&out, sb_take(&bind_value));
            free(rewritten_try_expr);
            free(try_name);
            free(try_expr);
            free(typed);
            continue;
        }
        free(try_name);
        free(try_expr);
        char *inline_name = NULL;
        char *inline_match_expr = NULL;
        char *inline_arms = NULL;
        if (has_enum && parse_inline_match_let(typed, &inline_name, &inline_match_expr, &inline_arms)) {
            char *arm_parts[16] = {0};
            char *patterns[16] = {0};
            char *exprs[16] = {0};
            int arm_count = split_top_level_commas_c(inline_arms, arm_parts, 16);
            int inline_ok = arm_count > 0;
            for (int a = 0; a < arm_count && a < 16; a++) {
                if (!parse_arm(arm_parts[a], &patterns[a], &exprs[a])) inline_ok = 0;
            }
            if (inline_ok) {
                StrBuf decl;
                sb_init(&decl);
                sb_append(&decl, "    let mut ");
                sb_append(&decl, inline_name);
                sb_append(&decl, " = 0");
                lines_push(&out, sb_take(&decl));
                for (int a = 0; a < arm_count && a < 16; a++) {
                    char *pattern = patterns[a];
                    char *expr = exprs[a];
                    char *rewritten_pattern = rewrite_constructors(pattern, &info);
                    VariantInfo *variant = NULL;
                    char *binders[8] = {0};
                    int binder_count = 0;
                    int wildcard_pattern = is_wildcard_match_pattern(pattern);
                    if (info.is_payload && !wildcard_pattern) {
                        char *paren = strchr(pattern, '(');
                        char *vname = NULL;
                        if (paren != NULL) {
                            vname = trim_copy(substr_copy(pattern, (size_t)(paren - pattern)));
                            char *close = strrchr(paren, ')');
                            if (close != NULL) {
                                char *inside = substr_copy(paren + 1, (size_t)(close - paren - 1));
                                binder_count = split_top_level_commas_c(inside, binders, 8);
                                free(inside);
                            }
                        } else {
                            vname = trim_copy(pattern);
                        }
                        variant = find_variant(&info, vname);
                        free(vname);
                    }
                    char *struct_expr = (info.is_payload && variant != NULL)
                        ? rewrite_struct_payload_binder_expr(expr, variant, binders, binder_count)
                        : strdup(expr);
                    char *rewritten_expr = rewrite_constructors(struct_expr, &info);
                    free(struct_expr);
                    StrBuf b;
                    sb_init(&b);
                    if (wildcard_pattern) {
                        sb_append(&b, a == 0 ? "    if 1 == 1 {" : "    else {");
                    } else {
                        sb_append(&b, a == 0 ? "    if " : "    else if ");
                    }
                    if (!wildcard_pattern && info.is_payload && variant != NULL) {
                        char tag_text[64];
                        snprintf(tag_text, sizeof(tag_text), "%% %d == %d {", info.count, variant->tag);
                        sb_append(&b, inline_match_expr);
                        sb_append(&b, " ");
                        sb_append(&b, tag_text);
                    } else if (!wildcard_pattern) {
                        sb_append(&b, inline_match_expr);
                        sb_append(&b, " == ");
                        sb_append(&b, rewritten_pattern);
                        sb_append(&b, " {");
                    }
                    lines_push(&out, sb_take(&b));
                    if (info.is_payload && variant != NULL) {
                        for (int k = 0; k < binder_count; k++) {
                            long long denom = info.count;
                            for (int p = 0; p < k; p++) denom *= 1000000LL;
                            StrBuf bind;
                            sb_init(&bind);
                            char denom_text[80];
                            snprintf(denom_text, sizeof(denom_text), "        let %s = (%s / %lld) %% 1000000", binders[k], inline_match_expr, denom);
                            sb_append(&bind, denom_text);
                            lines_push(&out, sb_take(&bind));
                            free(binders[k]);
                        }
                    }
                    StrBuf assign;
                    sb_init(&assign);
                    sb_append(&assign, "        ");
                    sb_append(&assign, inline_name);
                    sb_append(&assign, " = ");
                    sb_append(&assign, rewritten_expr);
                    lines_push(&out, sb_take(&assign));
                    lines_push(&out, strdup("    }"));
                    free(rewritten_expr);
                    free(rewritten_pattern);
                }
            } else {
                lines_push(&out, strdup(typed));
            }
            for (int a = 0; a < 16; a++) {
                free(arm_parts[a]);
                free(patterns[a]);
                free(exprs[a]);
            }
            free(inline_name);
            free(inline_match_expr);
            free(inline_arms);
            free(typed);
            continue;
        }
        free(inline_name);
        free(inline_match_expr);
        free(inline_arms);
        char *match_expr = parse_match_expr(typed);
        if (match_expr == NULL) {
            char *rewritten = has_enum && strstr(typed, "match ") == NULL ? rewrite_constructors(typed, &info) : strdup(typed);
            lines_push(&out, rewritten);
            free(typed);
            continue;
        }

        i++;
        int arm_index = 0;
        while (i < lines.len && strcmp(skip_ws(lines.items[i]), "}") != 0) {
            char *pattern = NULL;
            char *expr = NULL;
            if (!parse_arm(lines.items[i], &pattern, &expr)) {
                free(match_expr);
                free(typed);
                lines_free(&lines);
                lines_free(&out);
                free_enum_info(&info);
                return strdup(text);
            }
            char *rewritten_pattern = has_enum ? rewrite_constructors(pattern, &info) : strdup(pattern);
            VariantInfo *variant = NULL;
            char *binders[8] = {0};
            int binder_count = 0;
            int wildcard_pattern = is_wildcard_match_pattern(pattern);
            if (!has_enum && !wildcard_pattern && !is_int_match_pattern(pattern)) {
                free(match_expr);
                free(typed);
                free(pattern);
                free(expr);
                free(rewritten_pattern);
                lines_free(&lines);
                lines_free(&out);
                free_enum_info(&info);
                return strdup(text);
            }
            if (has_enum && info.is_payload && !wildcard_pattern) {
                char *paren = strchr(pattern, '(');
                char *vname = NULL;
                if (paren != NULL) {
                    vname = trim_copy(substr_copy(pattern, (size_t)(paren - pattern)));
                    char *close = strrchr(paren, ')');
                    if (close != NULL) {
                        char *inside = substr_copy(paren + 1, (size_t)(close - paren - 1));
                        binder_count = split_top_level_commas_c(inside, binders, 8);
                        free(inside);
                    }
                } else {
                    vname = trim_copy(pattern);
                }
                variant = find_variant(&info, vname);
                free(vname);
            }
            char *struct_expr = (has_enum && info.is_payload && variant != NULL)
                ? rewrite_struct_payload_binder_expr(expr, variant, binders, binder_count)
                : strdup(expr);
            char *rewritten_expr = has_enum ? rewrite_constructors(struct_expr, &info) : strdup(struct_expr);
            free(struct_expr);
            StrBuf b;
            sb_init(&b);
            if (wildcard_pattern) {
                sb_append(&b, arm_index == 0 ? "    if 1 == 1 {" : "    else {");
            } else {
                sb_append(&b, arm_index == 0 ? "    if " : "    else if ");
            }
            if (!wildcard_pattern && has_enum && info.is_payload && variant != NULL) {
                char tag_text[64];
                snprintf(tag_text, sizeof(tag_text), "%% %d == %d {", info.count, variant->tag);
                sb_append(&b, match_expr);
                sb_append(&b, " ");
                sb_append(&b, tag_text);
            } else if (!wildcard_pattern) {
                sb_append(&b, match_expr);
                sb_append(&b, " == ");
                sb_append(&b, rewritten_pattern);
                sb_append(&b, " {");
            }
            lines_push(&out, sb_take(&b));
            if (has_enum && info.is_payload && variant != NULL) {
                for (int k = 0; k < binder_count; k++) {
                    long long denom = info.count;
                    for (int p = 0; p < k; p++) denom *= 1000000LL;
                    StrBuf bind;
                    sb_init(&bind);
                    char denom_text[80];
                    snprintf(denom_text, sizeof(denom_text), "        let %s = (%s / %lld) %% 1000000", binders[k], match_expr, denom);
                    sb_append(&bind, denom_text);
                    lines_push(&out, sb_take(&bind));
                    free(binders[k]);
                }
            }
            StrBuf ret;
            sb_init(&ret);
            sb_append(&ret, "        return ");
            sb_append(&ret, rewritten_expr);
            lines_push(&out, sb_take(&ret));
            lines_push(&out, strdup("    }"));
            free(pattern);
            free(expr);
            free(rewritten_expr);
            free(rewritten_pattern);
            arm_index++;
            i++;
        }
        lines_push(&out, strdup("    return 0"));
        free(match_expr);
        free(typed);
    }

    size_t text_len = strlen(text);
    char *joined = join_lines(&out, text_len > 0 && text[text_len - 1] == '\n');
    lines_free(&lines);
    lines_free(&out);
    free_enum_info(&info);
    return joined;
}

static char *replace_word_all(const char *text, const char *name, const char *replacement) {
    StrBuf out;
    sb_init(&out);
    size_t name_len = strlen(name);
    for (size_t i = 0; text[i] != '\0';) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, (int)i);
            if (end < 0) {
                sb_append(&out, text + i);
                break;
            }
            sb_append_n(&out, text + i, (size_t)end - i);
            i = (size_t)end;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, (int)i);
            if (end < 0) {
                sb_append(&out, text + i);
                break;
            }
            sb_append_n(&out, text + i, (size_t)end - i);
            i = (size_t)end;
            continue;
        }
        int before_ok = i == 0 || !is_ident_continue(text[i - 1]);
        int after_ok = !is_ident_continue(text[i + name_len]);
        if (before_ok && after_ok && strncmp(text + i, name, name_len) == 0) {
            sb_append(&out, replacement);
            i += name_len;
        } else {
            sb_append_n(&out, text + i, 1);
            i++;
        }
    }
    return sb_take(&out);
}

static int parse_closure_fn(const char *line, char **maker, char **capture) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "fn ")) return 0;
    s += 3;
    const char *name_start = s;
    while (is_ident_continue(*s)) s++;
    const char *open = strchr(s, '(');
    const char *colon = strchr(s, ':');
    const char *close = strchr(s, ')');
    if (open == NULL || colon == NULL || close == NULL || !(open < colon && colon < close)) return 0;
    if (strstr(close, "-> fn") == NULL) return 0;
    *maker = substr_copy(name_start, (size_t)(s - name_start));
    char *cap_raw = substr_copy(open + 1, (size_t)(colon - open - 1));
    *capture = trim_copy(cap_raw);
    free(cap_raw);
    return 1;
}

static int parse_closure_return(const char *line, char **arg, char **expr) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "return |")) return 0;
    s += 8;
    const char *bar = strchr(s, '|');
    if (bar == NULL) return 0;
    char *arg_raw = substr_copy(s, (size_t)(bar - s));
    *arg = trim_copy(arg_raw);
    free(arg_raw);
    *expr = trim_copy(bar + 1);
    return 1;
}

static char *rewrite_closure_call_line(const char *line, const char *var, const char *apply) {
    char needle[128];
    snprintf(needle, sizeof(needle), "%s(", var);
    const char *p = strstr(line, needle);
    if (p == NULL) return strdup(line);
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(p - line));
    sb_append(&out, apply);
    sb_append(&out, "(");
    sb_append(&out, var);
    sb_append(&out, ", ");
    sb_append(&out, p + strlen(needle));
    return sb_take(&out);
}

static char *lower_closure_text(const char *text) {
    LineVec lines = split_lines(text);
    LineVec out;
    lines_init(&out);
    ClosureMaker makers[8];
    int maker_count = 0;
    memset(makers, 0, sizeof(makers));

    for (size_t i = 0; i < lines.len; i++) {
        char *maker = NULL;
        char *capture = NULL;
        if (i + 2 < lines.len && parse_closure_fn(lines.items[i], &maker, &capture)) {
            char *arg = NULL;
            char *expr = NULL;
            if (parse_closure_return(lines.items[i + 1], &arg, &expr) && strcmp(skip_ws(lines.items[i + 2]), "}") == 0) {
                char apply_name[160];
                snprintf(apply_name, sizeof(apply_name), "%s__apply", maker);
                StrBuf a;
                sb_init(&a);
                sb_append(&a, "fn ");
                sb_append(&a, maker);
                sb_append(&a, "(");
                sb_append(&a, capture);
                sb_append(&a, ": Int) -> Int {");
                lines_push(&out, sb_take(&a));
                StrBuf r;
                sb_init(&r);
                sb_append(&r, "    return ");
                sb_append(&r, capture);
                lines_push(&out, sb_take(&r));
                lines_push(&out, strdup("}"));
                lines_push(&out, strdup(""));
                StrBuf fn;
                sb_init(&fn);
                sb_append(&fn, "fn ");
                sb_append(&fn, apply_name);
                sb_append(&fn, "(env: Int, ");
                sb_append(&fn, arg);
                sb_append(&fn, ": Int) -> Int {");
                lines_push(&out, sb_take(&fn));
                char *body = replace_word_all(expr, capture, "env");
                StrBuf br;
                sb_init(&br);
                sb_append(&br, "    return ");
                sb_append(&br, body);
                lines_push(&out, sb_take(&br));
                lines_push(&out, strdup("}"));
                makers[maker_count].maker = strdup(maker);
                makers[maker_count].apply = strdup(apply_name);
                maker_count++;
                free(body);
                free(arg);
                free(expr);
                free(maker);
                free(capture);
                i += 2;
                continue;
            }
            free(arg);
            free(expr);
        }
        free(maker);
        free(capture);
        lines_push(&out, strdup(lines.items[i]));
    }

    LineVec rewritten;
    lines_init(&rewritten);
    char *closure_var = NULL;
    char *closure_apply = NULL;
    for (size_t i = 0; i < out.len; i++) {
        char *line = out.items[i];
        const char *s = skip_ws(line);
        if (starts_with(s, "fn ")) {
            free(closure_var);
            free(closure_apply);
            closure_var = NULL;
            closure_apply = NULL;
        }
        if (starts_with(s, "let ")) {
            const char *name_start = s + 4;
            const char *name_end = name_start;
            while (is_ident_continue(*name_end)) name_end++;
            const char *eq = strchr(name_end, '=');
            if (eq != NULL) {
                const char *callee = skip_ws(eq + 1);
                for (int m = 0; m < maker_count; m++) {
                    size_t maker_len = strlen(makers[m].maker);
                    if (strncmp(callee, makers[m].maker, maker_len) == 0 && callee[maker_len] == '(') {
                        free(closure_var);
                        free(closure_apply);
                        closure_var = substr_copy(name_start, (size_t)(name_end - name_start));
                        closure_apply = strdup(makers[m].apply);
                    }
                }
            }
        }
        if (closure_var != NULL && closure_apply != NULL) {
            lines_push(&rewritten, rewrite_closure_call_line(line, closure_var, closure_apply));
        } else {
            lines_push(&rewritten, strdup(line));
        }
    }
    char *joined = join_lines(&rewritten, text[strlen(text) - 1] == '\n');
    for (int i = 0; i < maker_count; i++) {
        free(makers[i].maker);
        free(makers[i].apply);
    }
    free(closure_var);
    free(closure_apply);
    lines_free(&lines);
    lines_free(&out);
    lines_free(&rewritten);
    return joined;
}

static char *normalize_source_text(const char *raw, int core_lower) {
    StrBuf out;
    sb_init(&out);
    int in_struct = 0;
    int struct_depth = 0;
    const char *line = raw;
    while (*line != '\0') {
        const char *end = strchr(line, '\n');
        size_t n = end ? (size_t)(end - line) : strlen(line);
        char *stripped = strip_line_comment(line, n);
        char *step1 = NULL;
        char *step2 = NULL;
        char *step3 = NULL;
        char *step4 = NULL;
        const char *trim = skip_ws(stripped);

        if (!in_struct && starts_with(trim, "struct ") && strchr(trim, '{') != NULL) {
            if (strchr(trim, '}') != NULL) {
                step1 = lower_struct_one_line_fields(stripped);
            } else {
                in_struct = 1;
                struct_depth = 1;
            }
        } else if (in_struct) {
            if (strcmp(trim, "}") != 0) {
                step1 = lower_struct_field_line(stripped);
            }
        }

        if (step1 == NULL) step1 = strdup(stripped);
        if (core_lower) {
            step2 = lower_fn_int_annotations(step1);
            step3 = lower_let_int_annotation(step2);
            step4 = replace_print_token(step3);
        } else {
            step2 = strdup(step1);
            step3 = strdup(step2);
            step4 = strdup(step3);
        }

        if (statement_needs_semicolon(step4)) {
            sb_append(&out, step4);
            sb_append(&out, ";");
        } else {
            sb_append(&out, step4);
        }
        if (end != NULL) sb_append(&out, "\n");

        if (in_struct) {
            for (const char *p = trim; *p != '\0'; p++) {
                if (*p == '{') struct_depth++;
                if (*p == '}') struct_depth--;
            }
            if (strcmp(trim, "}") == 0 || struct_depth <= 0) {
                in_struct = 0;
                struct_depth = 0;
            }
        }

        free(stripped);
        free(step1);
        free(step2);
        free(step3);
        free(step4);
        if (end == NULL) break;
        line = end + 1;
    }
    return out.data;
}

static char *prepare_source_text(const char *raw) {
    char *normalized = normalize_source_text(raw, 0);
    char *enum_lowered = lower_enum_text(normalized);
    char *closure_lowered = lower_closure_text(enum_lowered);
    char *prepared = normalize_source_text(closure_lowered, 1);
    free(normalized);
    free(enum_lowered);
    free(closure_lowered);
    return prepared;
}

static int find_col(const char *line, const char *needle) {
    const char *p = strstr(line, needle);
    return p == NULL ? 1 : (int)(p - line) + 1;
}

static void print_caret(int col) {
    for (int i = 1; i < col; i++) fputc(' ', stderr);
    fputs("^\n", stderr);
}

static void report_issue(
    const char *path,
    int line_no,
    int col,
    const char *line,
    const char *message,
    const char *help,
    const char *fix
) {
    fprintf(stderr, "error: %s\n", message);
    fprintf(stderr, "  --> %s:%d:%d\n", path, line_no, col);
    fprintf(stderr, "  %s\n", line);
    fputs("  ", stderr);
    print_caret(col);
    fprintf(stderr, "  help: %s\n", help);
    if (fix != NULL) fprintf(stderr, "  fix: %s\n", fix);
    fputc('\n', stderr);
}

static char *replace_once_for_fix(const char *line, const char *old_text, const char *new_text) {
    return replace_exact(line, old_text, new_text);
}

static char *fix_as_cast(const char *line) {
    const char *p = strstr(line, " as Int");
    if (p == NULL) return strdup("Int(expr)");
    const char *end = p;
    const char *start = end;
    while (start > line && (start[-1] == ' ' || start[-1] == '\t')) start--;
    while (start > line && (is_ident_continue(start[-1]) || start[-1] == ')' || start[-1] == '(')) start--;
    if (start == end) return strdup("Int(expr)");
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(start - line));
    sb_append(&out, "Int(");
    sb_append_n(&out, start, (size_t)(end - start));
    sb_append(&out, ")");
    sb_append(&out, p + 7);
    return sb_take(&out);
}

static int front_type_is_map_int_int(const char *type);
static int front_type_is_map_int_bool(const char *type);
static int front_type_is_map_int_char(const char *type);
static int front_type_is_map_str_int(const char *type);
static int front_type_is_map_str_bool(const char *type);
static int front_type_is_map_str_char(const char *type);
static int front_type_is_list_int(const char *type);
static int front_type_is_supported_map_param(const char *type);
static int front_type_is_supported_map_return(const char *type);
static const char *front_map_type_for(char **names, char **types, int count, const char *name);
static int front_type_is_known_struct(const char *type, char **struct_names, int struct_count);

static int is_valid_int_params(const char *params, char **struct_names, int struct_count) {
    char *copy = strdup(params);
    if (copy == NULL) die_oom();
    char *parts[16] = {0};
    int n = split_top_level_type_commas_c(copy, parts, 16);
    free(copy);
    if (n < 0) return 0;
    for (int i = 0; i < n; i++) {
        char *colon = strchr(parts[i], ':');
        if (colon == NULL) {
            for (int k = 0; k < n; k++) free(parts[k]);
            return 0;
        }
        char *ty = trim_copy(colon + 1);
        int borrowed = 0;
        if (ty[0] == '&') {
            char *inner = trim_copy(ty + 1);
            free(ty);
            ty = inner;
            borrowed = 1;
        }
        int ok = borrowed
            ? front_type_is_list_int(ty)
            : (strcmp(ty, "Int") == 0 ||
                strcmp(ty, "Str") == 0 ||
                strcmp(ty, "Bool") == 0 ||
                strcmp(ty, "Char") == 0 ||
                front_type_is_list_int(ty) ||
                front_type_is_supported_map_param(ty) ||
                front_type_is_known_struct(ty, struct_names, struct_count));
        free(ty);
        if (!ok) {
            for (int k = 0; k < n; k++) free(parts[k]);
            return 0;
        }
    }
    for (int i = 0; i < n; i++) free(parts[i]);
    return 1;
}

static int front_has_map_type(const char *text) {
    return strstr(text, "Map<") != NULL || strstr(text, "Map <") != NULL;
}

static int front_type_is_map_int_int(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "Map") || is_ident_continue(p[3])) return 0;
    const char *q = skip_ws(p + 3);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Int") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != ',') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Int") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_map_int_bool(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "Map") || is_ident_continue(p[3])) return 0;
    const char *q = skip_ws(p + 3);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Int") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != ',') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Bool") || is_ident_continue(q[4])) return 0;
    q = skip_ws(q + 4);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_map_int_char(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "Map") || is_ident_continue(p[3])) return 0;
    const char *q = skip_ws(p + 3);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Int") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != ',') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Char") || is_ident_continue(q[4])) return 0;
    q = skip_ws(q + 4);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_map_str_int(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "Map") || is_ident_continue(p[3])) return 0;
    const char *q = skip_ws(p + 3);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Str") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != ',') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Int") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_map_str_bool(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "Map") || is_ident_continue(p[3])) return 0;
    const char *q = skip_ws(p + 3);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Str") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != ',') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Bool") || is_ident_continue(q[4])) return 0;
    q = skip_ws(q + 4);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_map_str_char(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "Map") || is_ident_continue(p[3])) return 0;
    const char *q = skip_ws(p + 3);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Str") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != ',') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Char") || is_ident_continue(q[4])) return 0;
    q = skip_ws(q + 4);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_list_int(const char *type) {
    const char *p = skip_ws(type);
    if (!starts_with(p, "List") || is_ident_continue(p[4])) return 0;
    const char *q = skip_ws(p + 4);
    if (*q != '<') return 0;
    q = skip_ws(q + 1);
    if (!starts_with(q, "Int") || is_ident_continue(q[3])) return 0;
    q = skip_ws(q + 3);
    if (*q != '>') return 0;
    q = skip_ws(q + 1);
    return *q == '\0';
}

static int front_type_is_supported_map_param(const char *type) {
    return front_type_is_map_int_int(type) || front_type_is_map_int_bool(type) || front_type_is_map_int_char(type) || front_type_is_map_str_int(type) || front_type_is_map_str_bool(type) || front_type_is_map_str_char(type);
}

static int front_type_is_supported_map_return(const char *type) {
    return front_type_is_map_int_int(type) || front_type_is_map_int_bool(type) || front_type_is_map_int_char(type) || front_type_is_map_str_int(type) || front_type_is_map_str_bool(type) || front_type_is_map_str_char(type);
}

static int front_type_is_known_struct(const char *type, char **struct_names, int struct_count) {
    char *ty = trim_copy(type);
    int ok = 0;
    for (int i = 0; i < struct_count; i++) {
        if (strcmp(ty, struct_names[i]) == 0) {
            ok = 1;
            break;
        }
    }
    free(ty);
    return ok;
}

static char *front_canonical_supported_map_type(const char *type) {
    const char *canonical = NULL;
    if (front_type_is_map_int_int(type)) canonical = "Map<Int,Int>";
    else if (front_type_is_map_int_bool(type)) canonical = "Map<Int,Bool>";
    else if (front_type_is_map_int_char(type)) canonical = "Map<Int,Char>";
    else if (front_type_is_map_str_int(type)) canonical = "Map<Str,Int>";
    else if (front_type_is_map_str_bool(type)) canonical = "Map<Str,Bool>";
    else if (front_type_is_map_str_char(type)) canonical = "Map<Str,Char>";
    if (canonical == NULL) return NULL;
    char *out = strdup(canonical);
    if (out == NULL) die_oom();
    return out;
}

static char *front_return_type_text(const char *arrow) {
    if (arrow == NULL) return NULL;
    const char *r = skip_ws(arrow + 2);
    const char *rend = r;
    int depth = 0;
    while (*rend != '\0') {
        if (*rend == '<') depth++;
        else if (*rend == '>') {
            if (depth > 0) depth--;
        } else if (depth == 0 && (*rend == '{' || *rend == '\n' || *rend == '\r')) {
            break;
        }
        rend++;
    }
    while (rend > r && (rend[-1] == ' ' || rend[-1] == '\t')) rend--;
    return substr_copy(r, (size_t)(rend - r));
}

static int front_params_have_unsupported_map_type(const char *params) {
    char *copy = strdup(params);
    if (copy == NULL) die_oom();
    char *parts[16] = {0};
    int n = split_top_level_type_commas_c(copy, parts, 16);
    free(copy);
    if (n < 0) return 1;
    for (int i = 0; i < n; i++) {
        char *colon = strchr(parts[i], ':');
        if (colon != NULL) {
            char *ty = trim_copy(colon + 1);
            int unsupported = front_has_map_type(ty) && !front_type_is_supported_map_param(ty);
            free(ty);
            if (unsupported) {
                for (int k = 0; k < n; k++) free(parts[k]);
                return 1;
            }
        }
    }
    for (int i = 0; i < n; i++) free(parts[i]);
    return 0;
}

static int check_fn_contract_line(
    const char *path,
    int line_no,
    const char *line,
    int *has_main,
    int *has_bad_main,
    char **struct_names,
    int struct_count
) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "fn ")) return 0;
    const char *name_start = s + 3;
    const char *name_end = name_start;
    while (is_ident_continue(*name_end)) name_end++;
    if (*name_end == '<') return 0;
    char *name = substr_copy(name_start, (size_t)(name_end - name_start));
    const char *open = strchr(name_end, '(');
    const char *close = open == NULL ? NULL : strchr(open, ')');
    const char *arrow = close == NULL ? NULL : strstr(close, "->");
    char *ret = front_return_type_text(arrow);
    char *params = NULL;
    if (open != NULL && close != NULL) params = substr_copy(open + 1, (size_t)(close - open - 1));
    int issue = 0;
    if (strcmp(name, "main") == 0) {
        if (params != NULL && strlen(skip_ws(params)) == 0 && ret != NULL && strcmp(ret, "Int") == 0) {
            *has_main = 1;
        } else {
            *has_bad_main = 1;
        }
    } else {
        if (arrow != NULL && front_has_map_type(arrow + 2) && !front_type_is_supported_map_return(ret)) {
            report_issue(path, line_no, find_col(line, "Map"), line,
                "only Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, and Map<Str,Char> return values are verified yet",
                "return Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, or Map<Str,Char> in this slice; keep generic Map returns local until their ABI slices are promoted.",
                NULL);
            issue = 1;
        } else if (ret == NULL || (strcmp(ret, "Int") != 0 && strcmp(ret, "Str") != 0 && strcmp(ret, "Bool") != 0 && strcmp(ret, "Char") != 0 && !front_type_is_supported_map_return(ret) && !front_type_is_known_struct(ret, struct_names, struct_count))) {
            report_issue(path, line_no, find_col(line, "fn "), line,
                "Vais native helper functions must return a verified scalar type",
                "write helpers as `fn name(a: Int, ...) -> Int`, `-> Bool`, `-> Char`, `-> Str`, a declared struct type, `-> Map<Int,Int>`, `-> Map<Int,Bool>`, `-> Map<Int,Char>`, `-> Map<Str,Int>`, `-> Map<Str,Bool>`, or `-> Map<Str,Char>`.",
                NULL);
            issue = 1;
        }
        if (params != NULL && strlen(skip_ws(params)) > 0 && front_params_have_unsupported_map_type(params)) {
            report_issue(path, line_no, find_col(line, "Map"), line,
                "only Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, and Map<Str,Char> parameters are verified yet",
                "keep generic Map parameters local until their ABI slices are promoted.",
                NULL);
            issue = 1;
        } else if (params != NULL && strlen(skip_ws(params)) > 0 && !is_valid_int_params(params, struct_names, struct_count)) {
            report_issue(path, line_no, find_col(line, params), line,
                "Vais native helper parameters must use verified scalar types",
                "use `Int`, `Str`, `Bool`, `Char`, `List<Int>`, `&List<Int>`, declared struct types, `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>` parameters in this slice.",
                NULL);
            issue = 1;
        }
    }
    free(name);
    free(ret);
    free(params);
    return issue;
}

static void front_add_map_binding(char **map_names, char **map_types, int *map_count, char *name, char *type) {
    if (*map_count < 128) {
        map_names[*map_count] = name;
        map_types[*map_count] = type;
        *map_count = *map_count + 1;
    } else {
        free(name);
        free(type);
    }
}

static void front_register_map_params(const char *line, char **map_names, char **map_types, int *map_count) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "fn ") || is_ident_continue(s[2])) return;
    const char *open = strchr(s, '(');
    const char *close = open == NULL ? NULL : strchr(open, ')');
    if (open == NULL || close == NULL || close < open) return;
    char *params = substr_copy(open + 1, (size_t)(close - open - 1));
    char *parts[16] = {0};
    int n = split_top_level_type_commas_c(params, parts, 16);
    free(params);
    if (n < 0) return;
    for (int i = 0; i < n; i++) {
        char *colon = strchr(parts[i], ':');
        if (colon != NULL) {
            char *name = trim_copy(substr_copy(parts[i], (size_t)(colon - parts[i])));
            char *raw_type = trim_copy(colon + 1);
            char *type = front_canonical_supported_map_type(raw_type);
            if (type != NULL && is_ident_start(name[0])) {
                front_add_map_binding(map_names, map_types, map_count, name, type);
                name = NULL;
                type = NULL;
            }
            free(name);
            free(type);
            free(raw_type);
        }
        free(parts[i]);
    }
}

static void front_register_map_return_fn(const char *line, char **map_fn_names, char **map_fn_types, int *map_fn_count) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "fn ") || is_ident_continue(s[2])) return;
    const char *name_start = s + 3;
    const char *name_end = name_start;
    while (is_ident_continue(*name_end)) name_end++;
    const char *open = strchr(name_end, '(');
    const char *close = open == NULL ? NULL : strchr(open, ')');
    const char *arrow = close == NULL ? NULL : strstr(close, "->");
    char *ret = front_return_type_text(arrow);
    char *type = front_canonical_supported_map_type(ret == NULL ? "" : ret);
    free(ret);
    if (type == NULL) return;
    char *name = substr_copy(name_start, (size_t)(name_end - name_start));
    front_add_map_binding(map_fn_names, map_fn_types, map_fn_count, name, type);
}

static void front_register_struct_name(const char *line, char **struct_names, int *struct_count) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "struct ") || is_ident_continue(s[6])) return;
    s += 7;
    s = skip_ws(s);
    if (!is_ident_start(*s)) return;
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    char *name = substr_copy(name_start, (size_t)(s - name_start));
    for (int i = 0; i < *struct_count; i++) {
        if (strcmp(name, struct_names[i]) == 0) {
            free(name);
            return;
        }
    }
    if (*struct_count < 128) {
        struct_names[*struct_count] = name;
        (*struct_count)++;
    } else {
        free(name);
    }
}

static const char *front_map_return_call_type(const char *expr, char **map_fn_names, char **map_fn_types, int map_fn_count) {
    const char *s = skip_ws(expr);
    if (!is_ident_start(*s)) return NULL;
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    char *name = substr_copy(name_start, (size_t)(s - name_start));
    s = skip_ws(s);
    if (*s != '(') {
        free(name);
        return NULL;
    }
    int close = find_matching_paren_c(s, 0);
    if (close < 0) {
        free(name);
        return NULL;
    }
    const char *tail = skip_ws(s + close + 1);
    if (*tail != '\0' && *tail != ';') {
        free(name);
        return NULL;
    }
    const char *type = front_map_type_for(map_fn_names, map_fn_types, map_fn_count, name);
    free(name);
    return type;
}

static char *front_supported_map_local_name(const char *line, char **type_out) {
    if (type_out != NULL) *type_out = NULL;
    const char *s = skip_ws(line);
    if (!starts_with(s, "let") || is_ident_continue(s[3])) return NULL;
    s = skip_ws(s + 3);
    if (starts_with(s, "mut") && !is_ident_continue(s[3])) {
        s = skip_ws(s + 3);
    }
    if (!is_ident_start(*s)) return NULL;
    const char *name_start = s;
    while (is_ident_continue(*s)) s++;
    const char *name_end = s;
    s = skip_ws(s);
    if (*s != ':') return NULL;
    const char *q = skip_ws(s + 1);
    if (!starts_with(q, "Map") || is_ident_continue(q[3])) return NULL;
    q = skip_ws(q + 3);
    if (*q != '<') return NULL;
    q = skip_ws(q + 1);
    const char *key_type = NULL;
    if (strncmp(q, "Int", 3) == 0 && !is_ident_continue(q[3])) {
        key_type = "Int";
        q = skip_ws(q + 3);
    } else if (strncmp(q, "Str", 3) == 0 && !is_ident_continue(q[3])) {
        key_type = "Str";
        q = skip_ws(q + 3);
    } else {
        return NULL;
    }
    if (*q != ',') return NULL;
    q = skip_ws(q + 1);
    const char *value_type = NULL;
    if (strncmp(q, "Int", 3) == 0 && !is_ident_continue(q[3])) {
        value_type = "Int";
        q = skip_ws(q + 3);
    } else if (strncmp(q, "Bool", 4) == 0 && !is_ident_continue(q[4])) {
        value_type = "Bool";
        q = skip_ws(q + 4);
    } else if (strncmp(q, "Char", 4) == 0 && !is_ident_continue(q[4])) {
        value_type = "Char";
        q = skip_ws(q + 4);
    } else {
        return NULL;
    }
    if (strcmp(key_type, "Str") == 0 && strcmp(value_type, "Int") != 0 && strcmp(value_type, "Bool") != 0 && strcmp(value_type, "Char") != 0) return NULL;
    if (strcmp(key_type, "Int") != 0 && strcmp(key_type, "Str") != 0) return NULL;
    if (*q != '>') return NULL;
    int value_is_return_supported =
        (strcmp(key_type, "Int") == 0 &&
            (strcmp(value_type, "Int") == 0 || strcmp(value_type, "Bool") == 0 || strcmp(value_type, "Char") == 0)) ||
        (strcmp(key_type, "Str") == 0 &&
            (strcmp(value_type, "Int") == 0 || strcmp(value_type, "Bool") == 0 || strcmp(value_type, "Char") == 0));
    const char *eq = strchr(q, '=');
    if (eq == NULL) return NULL;
    const char *rhs = skip_ws(eq + 1);
    int ok_initializer = 0;
    if (*rhs == '{') {
        const char *r = skip_ws(rhs + 1);
        if (*r == '}') {
            r = skip_ws(r + 1);
            ok_initializer = *r == '\0' || *r == ';';
        }
    } else if (value_is_return_supported && is_ident_start(*rhs)) {
        const char *r = rhs + 1;
        while (is_ident_continue(*r)) r++;
        r = skip_ws(r);
        if (*r == '(') {
            int close = find_matching_paren_c(rhs, (int)(r - rhs));
            if (close >= 0) {
                r = skip_ws(rhs + close + 1);
                ok_initializer = *r == '\0' || *r == ';';
            }
        }
    }
    if (!ok_initializer) return NULL;
    if (type_out != NULL) {
        StrBuf type;
        sb_init(&type);
        sb_append(&type, "Map<");
        sb_append(&type, key_type);
        sb_append(&type, ",");
        sb_append(&type, value_type);
        sb_append(&type, ">");
        *type_out = sb_take(&type);
    }
    return substr_copy(name_start, (size_t)(name_end - name_start));
}

static int front_supported_map_local_line(const char *line) {
    char *type = NULL;
    char *name = front_supported_map_local_name(line, &type);
    int ok = name != NULL;
    free(name);
    free(type);
    return ok;
}

static int front_name_in_list(char **names, int count, const char *name) {
    for (int i = 0; i < count; i++) {
        if (strcmp(names[i], name) == 0) return 1;
    }
    return 0;
}

static const char *front_map_type_for(char **names, char **types, int count, const char *name) {
    if (name == NULL) return NULL;
    for (int i = count - 1; i >= 0; i--) {
        if (strcmp(names[i], name) == 0) return types[i];
    }
    return NULL;
}

static int front_supported_map_clear_line(const char *line, char **map_locals, int map_local_count) {
    const char *s = skip_ws(line);
    if (!is_ident_start(*s)) return 0;
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    char *name = substr_copy(name_start, (size_t)(s - name_start));
    int is_map = front_name_in_list(map_locals, map_local_count, name);
    free(name);
    if (!is_map) return 0;
    s = skip_ws(s);
    if (*s != '.') return 0;
    s = skip_ws(s + 1);
    if (!starts_with(s, "clear") || is_ident_continue(s[5])) return 0;
    s = skip_ws(s + 5);
    if (*s != '(') return 0;
    int close = find_matching_paren_c(s, 0);
    if (close < 0) return 0;
    const char *args = s + 1;
    while (args < s + close && (*args == ' ' || *args == '\t' || *args == '\n' || *args == '\r')) args++;
    if (args < s + close) return 0;
    const char *tail = skip_ws(s + close + 1);
    return *tail == '\0' || *tail == ';';
}

static char *front_map_assignment_name(const char *line, char **map_locals, int map_local_count, char **rhs_out) {
    const char *s = skip_ws(line);
    if (starts_with(s, "let") && !is_ident_continue(s[3])) return NULL;
    if (!is_ident_start(*s)) return NULL;
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    char *name = substr_copy(name_start, (size_t)(s - name_start));
    const char *op = skip_ws(s);
    if (*op != '=' || op[1] == '=') {
        free(name);
        return NULL;
    }
    if (front_name_in_list(map_locals, map_local_count, name)) {
        const char *rhs = skip_ws(op + 1);
        const char *rhs_end = rhs + strlen(rhs);
        while (rhs_end > rhs && (rhs_end[-1] == ' ' || rhs_end[-1] == '\t' || rhs_end[-1] == '\n' || rhs_end[-1] == '\r')) rhs_end--;
        if (rhs_end > rhs && rhs_end[-1] == ';') {
            rhs_end--;
            while (rhs_end > rhs && (rhs_end[-1] == ' ' || rhs_end[-1] == '\t' || rhs_end[-1] == '\n' || rhs_end[-1] == '\r')) rhs_end--;
        }
        if (rhs_end > rhs) {
            *rhs_out = substr_copy(rhs, (size_t)(rhs_end - rhs));
        }
        return name;
    }
    free(name);
    return NULL;
}

static char *front_probe_line(const char *line) {
    StrBuf out;
    sb_init(&out);
    char delim = '\0';
    int escaped = 0;
    for (size_t i = 0; line[i] != '\0'; i++) {
        char ch = line[i];
        if (escaped) {
            sb_append(&out, " ");
            escaped = 0;
            continue;
        }
        if (delim == '"' && ch == '\\') {
            sb_append(&out, " ");
            escaped = 1;
            continue;
        }
        if (delim != '\0') {
            sb_append(&out, " ");
            if (ch == delim) delim = '\0';
            continue;
        }
        if (ch == '#' ) {
            while (line[i] != '\0') {
                sb_append(&out, " ");
                i++;
            }
            break;
        }
        if (ch == '"' || ch == '`' || ch == '\'') {
            delim = ch;
            sb_append(&out, " ");
            continue;
        }
        sb_append_n(&out, &ch, 1);
    }
    return sb_take(&out);
}

static int generic_named_type_at(const char *line, size_t i, const char *name, size_t *end_out) {
    size_t name_len = strlen(name);
    if (i > 0 && is_ident_continue(line[i - 1])) return 0;
    if (strncmp(line + i, name, name_len) != 0) return 0;
    if (is_ident_continue(line[i + name_len])) return 0;
    size_t k = i + name_len;
    while (line[k] == ' ' || line[k] == '\t') k++;
    if (line[k] != '<') return 0;
    int depth = 0;
    for (; line[k] != '\0'; k++) {
        if (line[k] == '<') {
            depth++;
        } else if (line[k] == '>') {
            depth--;
            if (depth == 0) {
                *end_out = k + 1;
                return 1;
            }
        }
    }
    return 0;
}

static int check_option_result_generic_surface_text(const char *text, const char *path) {
    LineVec lines = split_lines(text);
    int issues = 0;
    for (size_t row = 0; row < lines.len; row++) {
        const char *line = lines.items[row];
        char *probe = front_probe_line(line);
        for (size_t i = 0; probe[i] != '\0'; i++) {
            size_t generic_end = 0;
            if (generic_named_type_at(probe, i, "Option", &generic_end)) {
                size_t supported_end = 0;
                if (!generic_type_at(probe, i, "Option", "Int", NULL, &supported_end)) {
                    report_issue(path, (int)row + 1, (int)i + 1, line,
                        "only Option<Int> is verified for now",
                        "use Option<Int> in this slice; generic Option<T> and nested Option/Result payloads are not verified yet.",
                        NULL);
                    issues++;
                }
                i = generic_end > i ? generic_end - 1 : i;
            } else if (generic_named_type_at(probe, i, "Result", &generic_end)) {
                size_t supported_end = 0;
                if (!generic_type_at(probe, i, "Result", "Int", "Int", &supported_end)) {
                    report_issue(path, (int)row + 1, (int)i + 1, line,
                        "only Result<Int,Int> is verified for now",
                        "use Result<Int,Int> in this slice; generic Result<T,E> and non-Int payloads are not verified yet.",
                        NULL);
                    issues++;
                }
                i = generic_end > i ? generic_end - 1 : i;
            }
        }
        free(probe);
    }
    lines_free(&lines);
    return issues == 0 ? 0 : 1;
}

static int check_front_contract_text(const char *text, const char *path) {
    LineVec lines = split_lines(text);
    int issues = 0;
    int has_main = 0;
    int has_bad_main = 0;
    char *map_locals[128] = {0};
    char *map_types[128] = {0};
    int map_local_count = 0;
    char *map_fns[128] = {0};
    char *map_fn_types[128] = {0};
    int map_fn_count = 0;
    char *struct_names[128] = {0};
    int struct_count = 0;
    for (size_t i = 0; i < lines.len; i++) {
        char *probe = front_probe_line(lines.items[i]);
        front_register_map_return_fn(probe, map_fns, map_fn_types, &map_fn_count);
        front_register_struct_name(probe, struct_names, &struct_count);
        free(probe);
    }
    for (size_t i = 0; i < lines.len; i++) {
        const char *line = lines.items[i];
        char *probe = front_probe_line(line);
        int line_no = (int)i + 1;
        issues += check_fn_contract_line(path, line_no, line, &has_main, &has_bad_main, struct_names, struct_count);
        front_register_map_params(probe, map_locals, map_types, &map_local_count);

        char *fix = NULL;
        const char *trim = skip_ws(probe);
        const char *module_kw = NULL;
        if (starts_with(trim, "module") && !is_ident_continue(trim[6])) module_kw = "module";
        else if (starts_with(trim, "package") && !is_ident_continue(trim[7])) module_kw = "package";
        char *map_assignment_rhs = NULL;
        char *map_assignment = front_map_assignment_name(probe, map_locals, map_local_count, &map_assignment_rhs);
        if (map_assignment != NULL) {
            const char *target_type = front_map_type_for(map_locals, map_types, map_local_count, map_assignment);
            const char *source_type = front_map_type_for(map_locals, map_types, map_local_count, map_assignment_rhs);
            if (source_type == NULL && map_assignment_rhs != NULL) source_type = front_map_return_call_type(map_assignment_rhs, map_fns, map_fn_types, map_fn_count);
            if (map_assignment_rhs == NULL || source_type == NULL) {
                report_issue(path, line_no, find_col(line, map_assignment), line,
                    "Map assignment requires another local or parameter Map value",
                    "assign from a local, same-type parameter, or same-type Map-returning call for `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>`; broader generic key/value forms are not verified yet.",
                    NULL);
                issues++;
            } else if (target_type == NULL || strcmp(target_type, source_type) != 0) {
                report_issue(path, line_no, find_col(line, map_assignment), line,
                    "Map assignment source type must match the target Map type",
                    "copy between locals with the same concrete Map type, such as `Map<Int,Int>` to `Map<Int,Int>` or `Map<Str,Char>` to `Map<Str,Char>`.",
                    NULL);
                issues++;
            }
            free(map_assignment_rhs);
            free(map_assignment);
        } else if (module_kw != NULL) {
            report_issue(path, line_no, find_col(probe, module_kw), line,
                "module and package declarations are not implemented yet",
                "omit the declaration; module names are derived from file paths in the first import slice.",
                NULL);
            issues++;
        } else if (strstr(probe, "&&") != NULL) {
            fix = replace_once_for_fix(line, "&&", "and");
            report_issue(path, line_no, find_col(probe, "&&"), line,
                "logical AND uses the word `and`, not `&&`",
                "replace `&&` with `and`.", fix);
            free(fix);
            issues++;
        } else if (strstr(probe, "||") != NULL) {
            fix = replace_once_for_fix(line, "||", "or");
            report_issue(path, line_no, find_col(probe, "||"), line,
                "logical OR uses the word `or`, not `||`",
                "replace `||` with `or`.", fix);
            free(fix);
            issues++;
        } else if (strstr(probe, " as Int") != NULL) {
            fix = fix_as_cast(line);
            report_issue(path, line_no, find_col(probe, " as Int"), line,
                "type conversion is explicit `Type(x)`, not `x as Type`",
                "write `Type(expr)` instead of `expr as Type`.", fix);
            free(fix);
            issues++;
        } else if (strstr(probe, "Vec<") != NULL && strstr(probe, "::new") != NULL) {
            fix = replace_once_for_fix(line, "Vec<Int>::new()", "[]");
            report_issue(path, line_no, find_col(probe, "Vec<"), line,
                "no turbofish constructor; use a literal instead of `Type<...>::new()`",
                "use a list/map literal such as `[]`, `[1, 2]`, or `{}`.", fix);
            free(fix);
            issues++;
        } else if (strstr(probe, "::") != NULL) {
            fix = replace_once_for_fix(line, "::", ".");
            report_issue(path, line_no, find_col(probe, "::"), line,
                "enum/path access uses `.`, not `::`",
                "replace `::` with `.`.", fix);
            free(fix);
            issues++;
        } else if (strstr(probe, "i32") != NULL) {
            fix = replace_once_for_fix(line, "i32", "Int");
            report_issue(path, line_no, find_col(probe, "i32"), line,
                "Vais scalar types are capitalized, not Rust scalar names",
                "use `Int` for the verified release scalar type.", fix);
            free(fix);
            issues++;
        } else if (strstr(probe, "enum ") != NULL) {
            report_issue(path, line_no, find_col(probe, "enum"), line,
                "enum declarations beyond payload-free tags or small Int-coded payload enums are not in the Vais native front subset yet",
                "use payload-free enum tags or Int/self-recursive payload enums with simple return-arm match; keep broader payload enums on the full compiler path.",
                NULL);
            issues++;
        } else if (strstr(probe, "match ") != NULL) {
            report_issue(path, line_no, find_col(probe, "match"), line,
                "`match` beyond simple enum return arms is not in the Vais native front subset yet",
                "use if/else for native sources, or keep payload match code on the full compiler path.",
                NULL);
            issues++;
        } else if (strstr(probe, "String") != NULL) {
            report_issue(path, line_no, 1, line,
                "this scalar type is not in the verified native front subset",
                "use `Int`, `Str`, `Bool`, or `Char` in this slice.",
                NULL);
            issues++;
        } else if ((strstr(probe, "Map<") != NULL || strstr(probe, "Map <") != NULL) && !starts_with(trim, "fn ")) {
            if (!front_supported_map_local_line(probe)) {
                int col = strstr(probe, "Map<") != NULL ? find_col(probe, "Map<") : find_col(probe, "Map <");
                report_issue(path, line_no, col, line,
                "only local Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, and Map<Str,Char> values are verified for now",
                "write `let name: Map<Int,Int> = {}`, `let name: Map<Int,Bool> = {}`, `let name: Map<Int,Char> = {}`, `let name: Map<Str,Int> = {}`, `let name: Map<Str,Bool> = {}`, or `let name: Map<Str,Char> = {}`; return initialization is limited to the verified return Map types.",
                NULL);
            issues++;
        }
        } else if (strstr(probe, "|") != NULL) {
            report_issue(path, line_no, find_col(probe, "|"), line,
                "closures beyond the single-Int closure-return slice are not in the Vais native front subset yet",
                "use a single Int capture returning `fn(Int) -> Int`, or write a named function for broader closure cases.",
                NULL);
            issues++;
        } else if (strstr(probe, ".clear(") != NULL && !front_supported_map_clear_line(probe, map_locals, map_local_count)) {
            report_issue(path, line_no, find_col(probe, ".clear("), line,
                "method calls beyond push/len/is_empty/last/pop/sum are not in the Vais native front subset yet",
                "use a plain function call, or keep this source on the full compiler path until that method is promoted.",
                NULL);
            issues++;
        }
        char *map_local_type = NULL;
        char *map_local = front_supported_map_local_name(probe, &map_local_type);
        if (map_local != NULL) {
            front_add_map_binding(map_locals, map_types, &map_local_count, map_local, map_local_type);
        }
        free(probe);
    }
    if (has_bad_main && !has_main) {
        report_issue(path, 1, 1, lines.len ? lines.items[0] : "",
            "Vais native day-1 front requires `fn main() -> Int` exactly",
            "write the entrypoint as `fn main() -> Int { ... }`.",
            NULL);
        issues++;
    } else if (!has_main) {
        report_issue(path, 1, 1, lines.len ? lines.items[0] : "",
            "Vais native day-1 front requires `fn main() -> Int`",
            "add `fn main() -> Int { return <int> }` as the program entrypoint.",
            NULL);
        issues++;
    }
    for (int m = 0; m < map_local_count; m++) {
        free(map_locals[m]);
        free(map_types[m]);
    }
    for (int m = 0; m < map_fn_count; m++) {
        free(map_fns[m]);
        free(map_fn_types[m]);
    }
    for (int s = 0; s < struct_count; s++) free(struct_names[s]);
    lines_free(&lines);
    return issues == 0 ? 0 : 1;
}

typedef struct {
    char *name;
    int line_no;
    char *line;
} ImportInfo;

typedef struct {
    char *alias;
    char *path;
    int line_no;
    char *line;
} PackageDependencyInfo;

typedef struct {
    char *name;
    char *version;
    char *source;
    int name_line;
    int version_line;
    int source_line;
    char *name_text;
    char *version_text;
    char *source_text;
    PackageDependencyInfo dependencies[32];
    int dependency_count;
} PackageManifestInfo;

static char *dirname_copy(const char *path) {
    const char *slash = strrchr(path, '/');
    if (slash == NULL) return strdup(".");
    if (slash == path) return strdup("/");
    return substr_copy(path, (size_t)(slash - path));
}

static char *canonical_existing_path(const char *path) {
    char resolved[4096];
    if (realpath(path, resolved) == NULL) return NULL;
    return strdup(resolved);
}

static char *path_join2(const char *base, const char *part) {
    StrBuf out;
    sb_init(&out);
    sb_append(&out, base);
    if (base[0] != '\0' && base[strlen(base) - 1] != '/') sb_append(&out, "/");
    sb_append(&out, part);
    return sb_take(&out);
}

static const char *path_basename_ptr(const char *path) {
    const char *slash = strrchr(path, '/');
    return slash == NULL ? path : slash + 1;
}

static int str_ends_with(const char *s, const char *suffix) {
    size_t slen = strlen(s);
    size_t suffix_len = strlen(suffix);
    return slen >= suffix_len && strcmp(s + slen - suffix_len, suffix) == 0;
}

static int is_repo_self_host_tier_path(const char *resolved) {
    return str_ends_with(resolved, "/compiler/self/fixpoint.vais") ||
        str_ends_with(resolved, "/compiler/self/fixpoint2.vais") ||
        str_ends_with(resolved, "/compiler/self/fixpoint3.vais") ||
        str_ends_with(resolved, "/compiler/self/fixpoint_full.vais");
}

static int trust_root_matches_source(const char *resolved, const char *source_name, const char *raw_root) {
    if (raw_root == NULL || raw_root[0] == '\0') return 0;
    char *compiler_dir = path_join2(raw_root, "compiler/self");
    char *trusted_path = path_join2(compiler_dir, source_name);
    char *trusted_real = canonical_existing_path(trusted_path);
    int ok = trusted_real != NULL && strcmp(resolved, trusted_real) == 0;
    free(compiler_dir);
    free(trusted_path);
    free(trusted_real);
    return ok;
}

static int is_trusted_self_host_source(const char *path) {
    char *resolved = canonical_existing_path(path);
    if (resolved == NULL) return 0;
    if (is_repo_self_host_tier_path(resolved)) {
        free(resolved);
        return 1;
    }

    const char *env = getenv("VAISC_SELF_HOST_TRUST_ROOTS");
    if (env == NULL || env[0] == '\0') {
        free(resolved);
        return 0;
    }

    const char *source_name = path_basename_ptr(resolved);
    char *roots = strdup(env);
    if (roots == NULL) die_oom();
    int ok = 0;
    char *cursor = roots;
    while (cursor != NULL && ok == 0) {
        char *next = strchr(cursor, ':');
        if (next != NULL) {
            *next = '\0';
            next++;
        }
        if (trust_root_matches_source(resolved, source_name, cursor)) ok = 1;
        cursor = next;
    }
    free(roots);
    free(resolved);
    return ok;
}

static char *strip_manifest_comment_c(const char *line) {
    StrBuf out;
    sb_init(&out);
    int in_string = 0;
    int escaped = 0;
    for (const char *p = line; *p != '\0'; p++) {
        char ch = *p;
        if (escaped) {
            sb_append_n(&out, &ch, 1);
            escaped = 0;
            continue;
        }
        if (ch == '\\' && in_string) {
            sb_append_n(&out, &ch, 1);
            escaped = 1;
            continue;
        }
        if (ch == '"') {
            in_string = !in_string;
            sb_append_n(&out, &ch, 1);
            continue;
        }
        if (ch == '#' && !in_string) break;
        sb_append_n(&out, &ch, 1);
    }
    return sb_take(&out);
}

static int parse_manifest_assignment_c(const char *code, char **key_out, char **value_out) {
    const char *s = skip_ws(code);
    if (!is_ident_start(*s)) return 0;
    const char *key_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *key_end = s;
    s = skip_ws(s);
    if (*s != '=') return 0;
    s = skip_ws(s + 1);
    if (*s != '"') return 0;
    s++;
    const char *value_start = s;
    while (*s != '\0' && *s != '"') s++;
    if (*s != '"') return 0;
    const char *value_end = s;
    s = skip_ws(s + 1);
    if (*s != '\0') return 0;
    *key_out = substr_copy(key_start, (size_t)(key_end - key_start));
    *value_out = substr_copy(value_start, (size_t)(value_end - value_start));
    return 1;
}

static char *parse_manifest_section_c(const char *code) {
    const char *s = skip_ws(code);
    if (*s != '[') return NULL;
    s++;
    if (!is_ident_start(*s)) return NULL;
    const char *start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *end = s;
    s = skip_ws(s);
    if (*s != ']') return NULL;
    s = skip_ws(s + 1);
    if (*s != '\0') return NULL;
    return substr_copy(start, (size_t)(end - start));
}

static int package_source_path_is_safe_c(const char *value) {
    if (value[0] == '\0' || value[0] == '/' || strchr(value, '\\') != NULL) return 0;
    const char *p = value;
    while (*p != '\0') {
        const char *start = p;
        while (*p != '\0' && *p != '/') p++;
        size_t n = (size_t)(p - start);
        if (n == 0) return 0;
        if (n == 2 && start[0] == '.' && start[1] == '.') return 0;
        if (*p == '/') p++;
    }
    return 1;
}

static int package_dependency_path_is_safe_c(const char *value) {
    if (value[0] == '\0' || value[0] == '/' || strchr(value, '\\') != NULL || strchr(value, ':') != NULL) return 0;
    const char *p = value;
    while (*p != '\0') {
        const char *start = p;
        while (*p != '\0' && *p != '/') p++;
        if (p == start) return 0;
        if (*p == '/') p++;
    }
    return 1;
}

static int canonical_path_is_under(const char *path, const char *root) {
    if (strcmp(path, root) == 0) return 1;
    size_t n = strlen(root);
    if (strcmp(root, "/") == 0) return path[0] == '/';
    return strncmp(path, root, n) == 0 && path[n] == '/';
}

static void package_manifest_info_free(PackageManifestInfo *info) {
    free(info->name);
    free(info->version);
    free(info->source);
    free(info->name_text);
    free(info->version_text);
    free(info->source_text);
    for (int i = 0; i < info->dependency_count; i++) {
        free(info->dependencies[i].alias);
        free(info->dependencies[i].path);
        free(info->dependencies[i].line);
    }
}

static int set_manifest_key(
    PackageManifestInfo *info,
    const char *manifest_path,
    int line_no,
    const char *line,
    const char *key,
    char *value
) {
    char **slot = NULL;
    int *line_slot = NULL;
    char **text_slot = NULL;
    if (strcmp(key, "name") == 0) {
        slot = &info->name;
        line_slot = &info->name_line;
        text_slot = &info->name_text;
    } else if (strcmp(key, "version") == 0) {
        slot = &info->version;
        line_slot = &info->version_line;
        text_slot = &info->version_text;
    } else if (strcmp(key, "source") == 0) {
        slot = &info->source;
        line_slot = &info->source_line;
        text_slot = &info->source_text;
    } else {
        report_issue(manifest_path, line_no, 1, line,
            "unsupported package manifest key",
            "use only top-level `name`, `version`, and `source`; put local packages under `[dependencies]`.",
            NULL);
        free(value);
        return 1;
    }
    if (*slot != NULL) {
        StrBuf msg;
        sb_init(&msg);
        sb_append(&msg, "duplicate package manifest key `");
        sb_append(&msg, key);
        sb_append(&msg, "`");
        report_issue(manifest_path, line_no, 1, line, msg.data,
            "keep exactly one `name`, one `version`, and one `source` key.",
            NULL);
        free(msg.data);
        free(value);
        return 1;
    }
    *slot = value;
    *line_slot = line_no;
    *text_slot = strdup(line);
    if (*text_slot == NULL) die_oom();
    return 0;
}

static int add_manifest_dependency(
    PackageManifestInfo *info,
    const char *manifest_path,
    int line_no,
    const char *line,
    char *alias,
    char *path
) {
    for (int i = 0; i < info->dependency_count; i++) {
        if (strcmp(info->dependencies[i].alias, alias) == 0) {
            StrBuf msg;
            sb_init(&msg);
            sb_append(&msg, "duplicate local dependency alias `");
            sb_append(&msg, alias);
            sb_append(&msg, "`");
            report_issue(manifest_path, line_no, 1, line, msg.data,
                "keep exactly one path for each local dependency alias.",
                NULL);
            free(msg.data);
            free(alias);
            free(path);
            return 1;
        }
    }
    if (info->dependency_count >= 32) {
        report_issue(manifest_path, line_no, 1, line,
            "too many local dependencies in package manifest",
            "split the package or reduce dependencies in this first package slice.",
            NULL);
        free(alias);
        free(path);
        return 1;
    }
    PackageDependencyInfo *dep = &info->dependencies[info->dependency_count++];
    dep->alias = alias;
    dep->path = path;
    dep->line_no = line_no;
    dep->line = strdup(line);
    if (dep->line == NULL) die_oom();
    return 0;
}

static int parse_package_manifest_info(const char *manifest_path, PackageManifestInfo *info) {
    memset(info, 0, sizeof(*info));
    char *raw = read_file(manifest_path);
    if (raw == NULL) return 1;
    LineVec lines = split_lines(raw);
    int failed = 0;
    int section = 0;
    for (size_t i = 0; i < lines.len && !failed; i++) {
        const char *line = lines.items[i];
        char *code = strip_manifest_comment_c(line);
        if (strlen(skip_ws(code)) == 0) {
            free(code);
            continue;
        }
        char *section_name = parse_manifest_section_c(code);
        if (section_name != NULL) {
            if (strcmp(section_name, "dependencies") != 0) {
                StrBuf msg;
                sb_init(&msg);
                sb_append(&msg, "unsupported package manifest section `[");
                sb_append(&msg, section_name);
                sb_append(&msg, "]`");
                report_issue(manifest_path, (int)i + 1, 1, line, msg.data,
                    "only the `[dependencies]` section is supported in this package manifest slice.",
                    NULL);
                free(msg.data);
                failed = 1;
            } else {
                section = 1;
            }
            free(section_name);
            free(code);
            continue;
        }
        char *key = NULL;
        char *value = NULL;
        if (!parse_manifest_assignment_c(code, &key, &value)) {
            report_issue(manifest_path, (int)i + 1, 1, line,
                "invalid package manifest entry",
                "write top-level string keys `name`, `version`, and `source`, plus optional `[dependencies]` string entries.",
                NULL);
            failed = 1;
            free(code);
            break;
        }
        if (section == 1) {
            failed = add_manifest_dependency(info, manifest_path, (int)i + 1, line, key, value);
        } else {
            failed = set_manifest_key(info, manifest_path, (int)i + 1, line, key, value);
            free(key);
        }
        free(code);
    }

    const char *first_line = lines.len > 0 ? lines.items[0] : "";
    const char *missing_key = NULL;
    if (!failed && info->name == NULL) missing_key = "name";
    else if (!failed && info->version == NULL) missing_key = "version";
    else if (!failed && info->source == NULL) missing_key = "source";
    if (missing_key != NULL) {
        StrBuf msg;
        sb_init(&msg);
        sb_append(&msg, "package manifest is missing required key `");
        sb_append(&msg, missing_key);
        sb_append(&msg, "`");
        report_issue(manifest_path, 1, 1, first_line, msg.data,
            "write `name`, `version`, and `source` before compiling this package.",
            NULL);
        free(msg.data);
        failed = 1;
    }

    if (!failed && !package_source_path_is_safe_c(info->source)) {
        report_issue(manifest_path, info->source_line, 1, info->source_text,
            "package manifest source must be a local relative path",
            "use a source path such as `src`; absolute paths and `..` are not supported.",
            NULL);
        failed = 1;
    }

    lines_free(&lines);
    free(raw);
    return failed;
}

static char *find_package_manifest_path(const char *start_dir) {
    char *current = canonical_existing_path(start_dir);
    if (current == NULL) current = strdup(start_dir);
    if (current == NULL) die_oom();
    while (1) {
        char *candidate = path_join2(current, "vais.toml");
        if (access(candidate, R_OK) == 0) {
            free(current);
            return candidate;
        }
        free(candidate);
        char *parent = dirname_copy(current);
        if (strcmp(parent, current) == 0) {
            free(parent);
            free(current);
            return NULL;
        }
        free(current);
        current = parent;
    }
}

static void linevec_pop(LineVec *lv) {
    if (lv->len == 0) return;
    lv->len--;
    free(lv->items[lv->len]);
    lv->items[lv->len] = NULL;
}

static int linevec_contains(LineVec *lv, const char *text);

static char *package_manifest_source_root(const char *manifest_path, PackageManifestInfo *info) {
    char *manifest_dir = dirname_copy(manifest_path);
    char *raw_source_root = path_join2(manifest_dir, info->source);
    char *source_root = canonical_existing_path(raw_source_root);
    if (source_root == NULL) {
        StrBuf help;
        sb_init(&help);
        sb_append(&help, "create the source directory at ");
        sb_append(&help, raw_source_root);
        sb_append(&help, " or update `source`.");
        report_issue(manifest_path, info->source_line, 1, info->source_text,
            "package manifest source directory not found", help.data, NULL);
        free(help.data);
    }
    free(raw_source_root);
    free(manifest_dir);
    return source_root;
}

static int module_resolver_add_package_root(ModuleResolver *r, const char *alias, const char *source_root) {
    for (int i = 0; i < r->package_count; i++) {
        if (strcmp(r->packages[i].source_root, source_root) == 0) return 0;
    }
    if (r->package_count >= 64) {
        fprintf(stderr, "error: too many local packages in module graph\n");
        return 1;
    }
    PackageRootInfo *item = &r->packages[r->package_count++];
    item->alias = strdup(alias);
    item->source_root = strdup(source_root);
    if (item->alias == NULL || item->source_root == NULL) die_oom();
    return 0;
}

static int module_resolver_add_dependency_root(
    ModuleResolver *r,
    const char *issue_path,
    int issue_line,
    const char *issue_text,
    const char *alias,
    const char *source_root
) {
    for (int i = 0; i < r->dependency_count; i++) {
        if (strcmp(r->dependencies[i].alias, alias) == 0) {
            if (strcmp(r->dependencies[i].source_root, source_root) == 0) return 0;
            StrBuf msg;
            sb_init(&msg);
            sb_append(&msg, "duplicate local dependency alias `");
            sb_append(&msg, alias);
            sb_append(&msg, "`");
            report_issue(issue_path, issue_line > 0 ? issue_line : 1, 1, issue_text ? issue_text : "", msg.data,
                "use distinct dependency aliases across the loaded package graph.",
                NULL);
            free(msg.data);
            return 1;
        }
    }
    if (r->dependency_count >= 64) {
        report_issue(issue_path, issue_line > 0 ? issue_line : 1, 1, issue_text ? issue_text : "",
            "too many local dependencies in module graph",
            "reduce dependencies in this first package slice.",
            NULL);
        return 1;
    }
    PackageRootInfo *item = &r->dependencies[r->dependency_count++];
    item->alias = strdup(alias);
    item->source_root = strdup(source_root);
    if (item->alias == NULL || item->source_root == NULL) die_oom();
    return 0;
}

static char *package_dependency_cycle_help(LineVec *stack, const char *next_manifest) {
    StrBuf help;
    sb_init(&help);
    sb_append(&help, "remove one dependency from the cycle: ");
    for (size_t i = 0; i < stack->len; i++) {
        if (i > 0) sb_append(&help, " -> ");
        sb_append(&help, stack->items[i]);
    }
    sb_append(&help, " -> ");
    sb_append(&help, next_manifest);
    sb_append(&help, ".");
    return sb_take(&help);
}

static int module_resolver_collect_manifest(
    ModuleResolver *r,
    const char *manifest_path,
    const char *alias,
    const char *entry_path,
    const char *issue_path,
    int issue_line,
    const char *issue_text,
    LineVec *stack
) {
    char *manifest_real = canonical_existing_path(manifest_path);
    if (manifest_real == NULL) manifest_real = strdup(manifest_path);
    if (manifest_real == NULL) die_oom();
    lines_push(stack, strdup(manifest_real));
    if (stack->items[stack->len - 1] == NULL) die_oom();

    PackageManifestInfo info;
    if (parse_package_manifest_info(manifest_real, &info) != 0) {
        package_manifest_info_free(&info);
        linevec_pop(stack);
        free(manifest_real);
        return 1;
    }

    int failed = 0;
    char *source_root = package_manifest_source_root(manifest_real, &info);
    if (source_root == NULL) {
        failed = 1;
    }
    if (!failed && r->root == NULL) {
        r->root = strdup(source_root);
        if (r->root == NULL) die_oom();
    }
    if (!failed && entry_path != NULL) {
        char *entry_real = canonical_existing_path(entry_path);
        if (entry_real == NULL || !canonical_path_is_under(entry_real, source_root)) {
            StrBuf help;
            sb_init(&help);
            sb_append(&help, "compile a `.vais` file under ");
            sb_append(&help, source_root);
            sb_append(&help, " or update `source`.");
            report_issue(manifest_real, info.source_line, 1, info.source_text,
                "package entry is outside manifest source root", help.data, NULL);
            free(help.data);
            failed = 1;
        }
        free(entry_real);
    }
    if (!failed) failed = module_resolver_add_package_root(r, alias, source_root);
    if (!failed && alias[0] != '\0') {
        failed = module_resolver_add_dependency_root(
            r,
            issue_path ? issue_path : manifest_real,
            issue_line,
            issue_text,
            alias,
            source_root);
    }

    char *manifest_dir = dirname_copy(manifest_real);
    for (int i = 0; i < info.dependency_count && !failed; i++) {
        PackageDependencyInfo *dep = &info.dependencies[i];
        if (!package_dependency_path_is_safe_c(dep->path)) {
            report_issue(manifest_real, dep->line_no, 1, dep->line,
                "local dependency path must be a relative local path",
                "use a path such as `../mathlib`; absolute paths, URLs, and empty segments are not supported.",
                NULL);
            failed = 1;
            break;
        }
        char *dep_dir = path_join2(manifest_dir, dep->path);
        char *dep_manifest_raw = path_join2(dep_dir, "vais.toml");
        char *dep_manifest = canonical_existing_path(dep_manifest_raw);
        if (dep_manifest == NULL) {
            StrBuf help;
            sb_init(&help);
            sb_append(&help, "expected local dependency manifest at ");
            sb_append(&help, dep_manifest_raw);
            sb_append(&help, ".");
            report_issue(manifest_real, dep->line_no, 1, dep->line,
                "local dependency manifest not found", help.data, NULL);
            free(help.data);
            failed = 1;
        } else if (linevec_contains(stack, dep_manifest)) {
            char *help = package_dependency_cycle_help(stack, dep_manifest);
            report_issue(manifest_real, dep->line_no, 1, dep->line,
                "local dependency cycle detected", help, NULL);
            free(help);
            failed = 1;
        } else {
            failed = module_resolver_collect_manifest(
                r,
                dep_manifest,
                dep->alias,
                NULL,
                manifest_real,
                dep->line_no,
                dep->line,
                stack);
        }
        free(dep_manifest);
        free(dep_manifest_raw);
        free(dep_dir);
    }
    free(manifest_dir);
    free(source_root);
    package_manifest_info_free(&info);
    linevec_pop(stack);
    free(manifest_real);
    return failed;
}

static int module_resolver_init_roots(ModuleResolver *r, const char *path) {
    char *dir = dirname_copy(path);
    char *manifest = find_package_manifest_path(dir);
    if (manifest == NULL) {
        char *root = canonical_existing_path(dir);
        if (root == NULL) root = strdup(dir);
        if (root == NULL) die_oom();
        r->root = strdup(root);
        if (r->root == NULL) die_oom();
        int failed = module_resolver_add_package_root(r, "", root);
        free(root);
        free(dir);
        return failed;
    }
    LineVec stack;
    lines_init(&stack);
    int failed = module_resolver_collect_manifest(r, manifest, "", path, NULL, 0, NULL, &stack);
    lines_free(&stack);
    free(manifest);
    free(dir);
    return failed;
}

static int linevec_contains(LineVec *lv, const char *text) {
    for (size_t i = 0; i < lv->len; i++) {
        if (strcmp(lv->items[i], text) == 0) return 1;
    }
    return 0;
}

static const char *module_resolver_root_for_path(ModuleResolver *r, const char *path) {
    const char *root = r->root;
    size_t best_len = root ? strlen(root) : 0;
    for (int i = 0; i < r->package_count; i++) {
        const char *candidate = r->packages[i].source_root;
        size_t len = strlen(candidate);
        if (len >= best_len && canonical_path_is_under(path, candidate)) {
            root = candidate;
            best_len = len;
        }
    }
    return root;
}

static const char *module_resolver_alias_for_root(ModuleResolver *r, const char *root) {
    for (int i = 0; i < r->package_count; i++) {
        if (strcmp(r->packages[i].source_root, root) == 0) return r->packages[i].alias;
    }
    return "";
}

static char *module_name_for_path_c(ModuleResolver *r, const char *path) {
    const char *rel = path;
    const char *root = module_resolver_root_for_path(r, path);
    size_t root_len = strlen(root);
    if (strcmp(path, root) == 0) rel = "";
    else if (strncmp(path, root, root_len) == 0 && path[root_len] == '/') rel = path + root_len + 1;
    char *out = strdup(rel);
    if (out == NULL) die_oom();
    size_t n = strlen(out);
    if (n >= 5 && strcmp(out + n - 5, ".vais") == 0) out[n - 5] = '\0';
    for (char *p = out; *p != '\0'; p++) {
        if (*p == '/') *p = '.';
    }
    const char *alias = module_resolver_alias_for_root(r, root);
    if (alias[0] == '\0') return out;
    StrBuf prefixed;
    sb_init(&prefixed);
    sb_append(&prefixed, alias);
    if (out[0] != '\0') {
        sb_append(&prefixed, ".");
        sb_append(&prefixed, out);
    }
    free(out);
    return sb_take(&prefixed);
}

static char *module_path_under_root(const char *root, const char *name) {
    StrBuf out;
    sb_init(&out);
    sb_append(&out, root);
    sb_append(&out, "/");
    for (const char *p = name; *p != '\0'; p++) {
        char ch = *p == '.' ? '/' : *p;
        sb_append_n(&out, &ch, 1);
    }
    sb_append(&out, ".vais");
    return sb_take(&out);
}

static char *module_path_for_import(ModuleResolver *r, const char *current_path, const char *name) {
    const char *current_root = module_resolver_root_for_path(r, current_path);
    char *local = module_path_under_root(current_root, name);
    if (access(local, R_OK) == 0) return local;

    const char *dot = strchr(name, '.');
    if (dot != NULL && dot != name && dot[1] != '\0') {
        char *alias = substr_copy(name, (size_t)(dot - name));
        const char *rest = dot + 1;
        for (int i = 0; i < r->dependency_count; i++) {
            if (strcmp(r->dependencies[i].alias, alias) == 0) {
                free(local);
                char *target = module_path_under_root(r->dependencies[i].source_root, rest);
                free(alias);
                return target;
            }
        }
        free(alias);
    }
    return local;
}

static char *parse_import_name_c(const char *code) {
    const char *s = skip_ws(code);
    if (!starts_with(s, "import") || is_ident_continue(s[6])) return NULL;
    s = skip_ws(s + 6);
    const char *start = s;
    if (!is_ident_start(*s)) return NULL;
    while (1) {
        while (is_ident_continue(*s)) s++;
        if (*s != '.') break;
        s++;
        if (!is_ident_start(*s)) return NULL;
    }
    const char *end = s;
    s = skip_ws(s);
    if (*s == ';') s = skip_ws(s + 1);
    if (*s != '\0') return NULL;
    return substr_copy(start, (size_t)(end - start));
}

static char *parse_top_level_symbol_c(const char *code) {
    const char *s = skip_ws(code);
    const char *name = NULL;
    if (starts_with(s, "fn") && !is_ident_continue(s[2])) name = skip_ws(s + 2);
    else if (starts_with(s, "struct") && !is_ident_continue(s[6])) name = skip_ws(s + 6);
    else if (starts_with(s, "enum") && !is_ident_continue(s[4])) name = skip_ws(s + 4);
    else return NULL;
    if (!is_ident_start(*name)) return NULL;
    const char *end = name + 1;
    while (is_ident_continue(*end)) end++;
    return substr_copy(name, (size_t)(end - name));
}

static int import_info_cmp(const void *a, const void *b) {
    const ImportInfo *ia = (const ImportInfo *)a;
    const ImportInfo *ib = (const ImportInfo *)b;
    return strcmp(ia->name, ib->name);
}

static int module_resolver_stack_index(ModuleResolver *r, const char *path) {
    for (int i = 0; i < r->stack_count; i++) {
        if (strcmp(r->stack[i].path, path) == 0) return i;
    }
    return -1;
}

static int module_resolver_add_symbol(ModuleResolver *r, const char *path, int line_no, const char *line, const char *name) {
    for (int i = 0; i < r->symbol_count; i++) {
        if (strcmp(r->symbols[i].name, name) == 0) {
            StrBuf help;
            sb_init(&help);
            sb_append(&help, "first definition is at ");
            sb_append(&help, r->symbols[i].path);
            sb_append(&help, ":");
            char num[32];
            snprintf(num, sizeof(num), "%d", r->symbols[i].line_no);
            sb_append(&help, num);
            sb_append(&help, "; rename one symbol before importing both files.");
            StrBuf msg;
            sb_init(&msg);
            sb_append(&msg, "duplicate top-level symbol `");
            sb_append(&msg, name);
            sb_append(&msg, "`");
            report_issue(path, line_no, find_col(line, name), line, msg.data, help.data, NULL);
            free(help.data);
            free(msg.data);
            return 1;
        }
    }
    if (r->symbol_count >= 512) {
        fprintf(stderr, "error: too many top-level symbols in module graph\n");
        return 1;
    }
    r->symbols[r->symbol_count].name = strdup(name);
    r->symbols[r->symbol_count].path = strdup(path);
    r->symbols[r->symbol_count].line_no = line_no;
    if (r->symbols[r->symbol_count].name == NULL || r->symbols[r->symbol_count].path == NULL) die_oom();
    r->symbol_count++;
    return 0;
}

static char *module_resolver_load(ModuleResolver *r, const char *path, const char *issue_path, int issue_line, const char *issue_text);

static char *module_resolver_cycle_help(ModuleResolver *r, int start) {
    StrBuf out;
    sb_init(&out);
    sb_append(&out, "remove one import from the cycle: ");
    for (int i = start; i < r->stack_count; i++) {
        if (i > start) sb_append(&out, " -> ");
        sb_append(&out, r->stack[i].module);
    }
    sb_append(&out, " -> ");
    sb_append(&out, r->stack[start].module);
    sb_append(&out, ".");
    return sb_take(&out);
}

static char *module_resolver_load(ModuleResolver *r, const char *path, const char *issue_path, int issue_line, const char *issue_text) {
    char *real = canonical_existing_path(path);
    if (real == NULL) {
        if (issue_path != NULL && issue_text != NULL) {
            StrBuf help;
            sb_init(&help);
            sb_append(&help, "expected local module file at ");
            sb_append(&help, path);
            sb_append(&help, ".");
            report_issue(issue_path, issue_line, find_col(issue_text, "import"), issue_text,
                "import path not found", help.data, NULL);
            free(help.data);
        } else {
            fprintf(stderr, "error: source not found: %s\n", path);
        }
        return NULL;
    }
    int cycle_start = module_resolver_stack_index(r, real);
    if (cycle_start >= 0) {
        char *help = module_resolver_cycle_help(r, cycle_start);
        report_issue(issue_path ? issue_path : path, issue_line > 0 ? issue_line : 1, issue_text ? find_col(issue_text, "import") : 1, issue_text ? issue_text : "",
            "import cycle detected", help, NULL);
        free(help);
        free(real);
        return NULL;
    }
    if (linevec_contains(&r->visited, real)) {
        free(real);
        return strdup("");
    }
    if (r->stack_count >= 128) {
        fprintf(stderr, "error: import graph is too deep\n");
        free(real);
        return NULL;
    }
    char *module_name = module_name_for_path_c(r, real);
    r->stack[r->stack_count].path = strdup(real);
    r->stack[r->stack_count].module = module_name;
    if (r->stack[r->stack_count].path == NULL) die_oom();
    r->stack_count++;

    char *raw = read_file(real);
    if (raw == NULL) {
        r->stack_count--;
        free(r->stack[r->stack_count].path);
        free(r->stack[r->stack_count].module);
        free(real);
        return NULL;
    }
    LineVec lines = split_lines(raw);
    LineVec body;
    lines_init(&body);
    ImportInfo imports[64];
    int import_count = 0;
    int failed = 0;
    for (size_t i = 0; i < lines.len && !failed; i++) {
        const char *line = lines.items[i];
        int line_no = (int)i + 1;
        char *code = strip_line_comment(line, strlen(line));
        const char *trim = skip_ws(code);
        char *import_name = parse_import_name_c(code);
        if (import_name != NULL) {
            if (import_count >= 64) {
                report_issue(real, line_no, find_col(line, "import"), line,
                    "too many imports in one module",
                    "split the module or reduce imports in this first Phase 2 slice.",
                    NULL);
                free(import_name);
                failed = 1;
            } else {
                imports[import_count].name = import_name;
                imports[import_count].line_no = line_no;
                imports[import_count].line = strdup(line);
                if (imports[import_count].line == NULL) die_oom();
                import_count++;
            }
            free(code);
            continue;
        }
        if (starts_with(trim, "import") && !is_ident_continue(trim[6])) {
            report_issue(real, line_no, find_col(line, "import"), line,
                "invalid import path",
                "write a static dotted local import such as `import math.add`.",
                NULL);
            failed = 1;
            free(code);
            break;
        }
        if ((starts_with(trim, "module") && !is_ident_continue(trim[6])) ||
            (starts_with(trim, "package") && !is_ident_continue(trim[7]))) {
            const char *kw = starts_with(trim, "module") ? "module" : "package";
            report_issue(real, line_no, find_col(line, kw), line,
                "module and package declarations are not implemented yet",
                "omit the declaration; module names are derived from file paths in the first import slice.",
                NULL);
            failed = 1;
            free(code);
            break;
        }
        char *symbol = parse_top_level_symbol_c(code);
        if (symbol != NULL) {
            failed = module_resolver_add_symbol(r, real, line_no, line, symbol);
            free(symbol);
            if (failed) {
                free(code);
                break;
            }
        }
        lines_push(&body, strdup(line));
        if (body.items[body.len - 1] == NULL) die_oom();
        free(code);
    }

    StrBuf merged;
    sb_init(&merged);
    if (!failed) {
        qsort(imports, (size_t)import_count, sizeof(ImportInfo), import_info_cmp);
        for (int i = 0; i < import_count; i++) {
            char *target = module_path_for_import(r, real, imports[i].name);
            char *piece = module_resolver_load(r, target, real, imports[i].line_no, imports[i].line);
            free(target);
            if (piece == NULL) {
                failed = 1;
                break;
            }
            if (piece[0] != '\0') {
                sb_append(&merged, piece);
                if (merged.len > 0 && merged.data[merged.len - 1] != '\n') sb_append(&merged, "\n");
            }
            free(piece);
        }
    }
    if (!failed) {
        char *body_text = join_lines(&body, 1);
        sb_append(&merged, body_text);
        free(body_text);
        lines_push(&r->visited, strdup(real));
        if (r->visited.items[r->visited.len - 1] == NULL) die_oom();
    }

    for (int i = 0; i < import_count; i++) {
        free(imports[i].name);
        free(imports[i].line);
    }
    lines_free(&body);
    lines_free(&lines);
    free(raw);
    r->stack_count--;
    free(r->stack[r->stack_count].path);
    free(r->stack[r->stack_count].module);
    free(real);
    if (failed) {
        free(merged.data);
        return NULL;
    }
    return sb_take(&merged);
}

static void module_resolver_free(ModuleResolver *r) {
    free(r->root);
    for (int i = 0; i < r->package_count; i++) {
        free(r->packages[i].alias);
        free(r->packages[i].source_root);
    }
    for (int i = 0; i < r->dependency_count; i++) {
        free(r->dependencies[i].alias);
        free(r->dependencies[i].source_root);
    }
    lines_free(&r->visited);
    for (int i = 0; i < r->symbol_count; i++) {
        free(r->symbols[i].name);
        free(r->symbols[i].path);
    }
}

static char *resolve_module_graph_source(const char *path) {
    ModuleResolver r;
    memset(&r, 0, sizeof(r));
    lines_init(&r.visited);
    if (module_resolver_init_roots(&r, path) != 0) {
        module_resolver_free(&r);
        return NULL;
    }
    char *merged = module_resolver_load(&r, path, NULL, 0, NULL);
    module_resolver_free(&r);
    return merged;
}

static char *prepare_source_file(const char *path) {
    if (!has_vais_suffix(path)) {
        fprintf(stderr, "error: Vais source files must use the .vais extension: %s\n", path);
        return NULL;
    }
    int trusted_self_host = is_trusted_self_host_source(path);
    char *merged = resolve_module_graph_source(path);
    if (merged == NULL) return NULL;
    if (!trusted_self_host && check_option_result_generic_surface_text(merged, path) != 0) {
        free(merged);
        return NULL;
    }
    char *normalized = normalize_source_text(merged, 0);
    char *enum_lowered = lower_enum_text(normalized);
    char *closure_lowered = lower_closure_text(enum_lowered);
    if (!trusted_self_host && check_front_contract_text(closure_lowered, path) != 0) {
        free(merged);
        free(normalized);
        free(enum_lowered);
        free(closure_lowered);
        return NULL;
    }
    char *prepared = normalize_source_text(closure_lowered, 1);
    free(merged);
    free(normalized);
    free(enum_lowered);
    free(closure_lowered);
    return prepared;
}

static void direct_names_free(DirectNameSet *set) {
    for (int i = 0; i < set->count; i++) {
        free(set->items[i].name);
        free(set->items[i].type);
    }
    set->count = 0;
    set->temp_count = 0;
    free(set->current_return_type);
    set->current_return_type = NULL;
}

static DirectLocalInfo *direct_names_find(DirectNameSet *set, const char *name) {
    for (int i = 0; i < set->count; i++) {
        if (strcmp(set->items[i].name, name) == 0) return &set->items[i];
    }
    return NULL;
}

static int direct_names_has(DirectNameSet *set, const char *name) {
    return direct_names_find(set, name) != NULL;
}

static const char *direct_names_type(DirectNameSet *set, const char *name) {
    DirectLocalInfo *info = direct_names_find(set, name);
    return info == NULL ? NULL : info->type;
}

static void direct_names_add_typed(DirectNameSet *set, const char *name, const char *type) {
    DirectLocalInfo *existing = direct_names_find(set, name);
    if (existing != NULL) {
        free(existing->type);
        existing->type = strdup(type);
        existing->is_ref = 0;
        return;
    }
    if (set->count >= 128) return;
    set->items[set->count].name = strdup(name);
    set->items[set->count].type = strdup(type);
    set->items[set->count].is_ref = 0;
    set->count++;
}

static void direct_names_add_typed_ref(DirectNameSet *set, const char *name, const char *type, int is_ref) {
    DirectLocalInfo *existing = direct_names_find(set, name);
    if (existing != NULL) {
        free(existing->type);
        existing->type = strdup(type);
        existing->is_ref = is_ref;
        return;
    }
    if (set->count >= 128) return;
    set->items[set->count].name = strdup(name);
    set->items[set->count].type = strdup(type);
    set->items[set->count].is_ref = is_ref;
    set->count++;
}

static void direct_names_remove(DirectNameSet *set, const char *name) {
    for (int i = 0; i < set->count; i++) {
        if (strcmp(set->items[i].name, name) != 0) continue;
        free(set->items[i].name);
        free(set->items[i].type);
        for (int j = i; j + 1 < set->count; j++) {
            set->items[j] = set->items[j + 1];
        }
        set->count--;
        return;
    }
}

static int direct_names_is_ref(DirectNameSet *set, const char *name) {
    DirectLocalInfo *info = direct_names_find(set, name);
    return info != NULL && info->is_ref;
}

static void direct_fns_free(DirectFnInfo *fns, int count) {
    for (int i = 0; i < count; i++) {
        free(fns[i].name);
        free(fns[i].return_type);
        for (int p = 0; p < fns[i].param_count; p++) {
            free(fns[i].params[p]);
            free(fns[i].param_types[p]);
        }
    }
}

static DirectFnInfo *direct_find_fn(DirectFnInfo *fns, int count, const char *name) {
    for (int i = 0; i < count; i++) {
        if (strcmp(fns[i].name, name) == 0) return &fns[i];
    }
    return NULL;
}

static void direct_struct_free_one(DirectStructInfo *info) {
    free(info->name);
    for (int i = 0; i < info->field_count; i++) free(info->fields[i]);
    memset(info, 0, sizeof(*info));
}

static void direct_structs_free(DirectStructInfo *structs, int count) {
    for (int i = 0; i < count; i++) direct_struct_free_one(&structs[i]);
}

static DirectStructInfo *direct_find_struct(DirectStructInfo *structs, int count, const char *name) {
    for (int i = 0; i < count; i++) {
        if (strcmp(structs[i].name, name) == 0) return &structs[i];
    }
    return NULL;
}

static int direct_struct_has_field(DirectStructInfo *info, const char *field) {
    for (int i = 0; i < info->field_count; i++) {
        if (strcmp(info->fields[i], field) == 0) return 1;
    }
    return 0;
}

static int direct_is_plain_ident(const char *text) {
    if (!is_ident_start(text[0])) return 0;
    int i = 1;
    while (is_ident_continue(text[i])) i++;
    return text[i] == '\0';
}

static int direct_is_list_int_type(const char *type) {
    return type != NULL && strcmp(type, "List<Int>") == 0;
}

static int direct_is_map_int_int_type(const char *type) {
    return type != NULL && strcmp(type, "Map<Int,Int>") == 0;
}

static int direct_is_map_int_bool_type(const char *type) {
    return type != NULL && strcmp(type, "Map<Int,Bool>") == 0;
}

static int direct_is_map_int_char_type(const char *type) {
    return type != NULL && strcmp(type, "Map<Int,Char>") == 0;
}

static int direct_is_map_str_int_type(const char *type) {
    return type != NULL && strcmp(type, "Map<Str,Int>") == 0;
}

static int direct_is_map_str_bool_type(const char *type) {
    return type != NULL && strcmp(type, "Map<Str,Bool>") == 0;
}

static int direct_is_map_str_char_type(const char *type) {
    return type != NULL && strcmp(type, "Map<Str,Char>") == 0;
}

static int direct_is_map_type(const char *type) {
    return direct_is_map_int_int_type(type) || direct_is_map_int_bool_type(type) || direct_is_map_int_char_type(type) || direct_is_map_str_int_type(type) || direct_is_map_str_bool_type(type) || direct_is_map_str_char_type(type);
}

static int direct_is_supported_map_return_type(const char *type) {
    return direct_is_map_int_int_type(type) || direct_is_map_int_bool_type(type) || direct_is_map_int_char_type(type) || direct_is_map_str_int_type(type) || direct_is_map_str_bool_type(type) || direct_is_map_str_char_type(type);
}

static const char *direct_map_key_type(const char *type) {
    if (direct_is_map_str_char_type(type)) return "Str";
    if (direct_is_map_str_bool_type(type)) return "Str";
    if (direct_is_map_str_int_type(type)) return "Str";
    if (direct_is_map_type(type)) return "Int";
    return NULL;
}

static const char *direct_map_value_type(const char *type) {
    if (direct_is_map_str_char_type(type)) return "Char";
    if (direct_is_map_str_bool_type(type)) return "Bool";
    if (direct_is_map_str_int_type(type)) return "Int";
    if (direct_is_map_int_int_type(type)) return "Int";
    if (direct_is_map_int_bool_type(type)) return "Bool";
    if (direct_is_map_int_char_type(type)) return "Char";
    return NULL;
}

static const char *direct_map_helper_name(const char *type, const char *method) {
    if (direct_is_map_str_int_type(type) || direct_is_map_str_bool_type(type) || direct_is_map_str_char_type(type)) {
        if (strcmp(method, "insert") == 0) return "__vais_map_str_int_insert";
        if (strcmp(method, "remove") == 0) return "__vais_map_str_int_remove";
        if (strcmp(method, "clear") == 0) return "__vais_map_str_int_clear";
        if (strcmp(method, "copy") == 0) return "__vais_map_str_int_copy";
        if (strcmp(method, "get") == 0) return "__vais_map_str_int_get";
        if (strcmp(method, "get_opt") == 0) return "__vais_map_str_int_get_opt";
        if (strcmp(method, "contains") == 0) return "__vais_map_str_int_contains";
        if (strcmp(method, "len") == 0) return "__vais_map_str_int_len";
    }
    if (strcmp(method, "insert") == 0) return "__vais_map_int_int_insert";
    if (strcmp(method, "remove") == 0) return "__vais_map_int_int_remove";
    if (strcmp(method, "clear") == 0) return "__vais_map_int_int_clear";
    if (strcmp(method, "copy") == 0) return "__vais_map_int_int_copy";
    if (strcmp(method, "get") == 0) return "__vais_map_int_int_get";
    if (strcmp(method, "get_opt") == 0) return "__vais_map_int_int_get_opt";
    if (strcmp(method, "contains") == 0) return "__vais_map_int_int_contains";
    if (strcmp(method, "len") == 0) return "__vais_map_int_int_len";
    return "__vais_map_int_int_len";
}

static int direct_map_arg_type_compatible(const char *expected, const char *actual) {
    if (expected == NULL || actual == NULL) return 0;
    return strcmp(expected, actual) == 0;
}

static int direct_is_list_type(const char *type) {
    size_t n = type == NULL ? 0 : strlen(type);
    return n > 6 && starts_with(type, "List<") && type[n - 1] == '>';
}

static char *direct_list_element_type(const char *type) {
    if (!direct_is_list_type(type)) return NULL;
    size_t n = strlen(type);
    return substr_copy(type + 5, n - 6);
}

static int direct_is_list_struct_type(DirectStructInfo *structs, int struct_count, const char *type) {
    char *elem = direct_list_element_type(type);
    if (elem == NULL) return 0;
    int ok = strcmp(elem, "Int") != 0 && direct_find_struct(structs, struct_count, elem) != NULL;
    free(elem);
    return ok;
}

static int direct_is_str_type(const char *type) {
    return type != NULL && strcmp(type, "Str") == 0;
}

static int direct_is_bool_type(const char *type) {
    return type != NULL && strcmp(type, "Bool") == 0;
}

static int direct_is_char_type(const char *type) {
    return type != NULL && strcmp(type, "Char") == 0;
}

static int direct_is_intlike_scalar_type(const char *type) {
    return type != NULL && (
        strcmp(type, "Int") == 0 ||
        direct_is_bool_type(type) ||
        direct_is_char_type(type)
    );
}

static int direct_return_type_allowed(DirectStructInfo *structs, int struct_count, const char *type) {
    return strcmp(type, "Int") == 0 ||
        direct_is_str_type(type) ||
        direct_is_bool_type(type) ||
        direct_is_char_type(type) ||
        direct_is_list_int_type(type) ||
        direct_is_list_struct_type(structs, struct_count, type) ||
        direct_is_supported_map_return_type(type) ||
        direct_find_struct(structs, struct_count, type) != NULL;
}

static int direct_param_type_allowed(DirectStructInfo *structs, int struct_count, const char *type) {
    return direct_return_type_allowed(structs, struct_count, type) ||
        direct_is_map_int_int_type(type) ||
        direct_is_map_int_bool_type(type) ||
        direct_is_map_int_char_type(type) ||
        direct_is_map_str_int_type(type) ||
        direct_is_map_str_bool_type(type) ||
        direct_is_map_str_char_type(type);
}

static int direct_local_type_allowed(DirectStructInfo *structs, int struct_count, const char *type) {
    return direct_return_type_allowed(structs, struct_count, type) ||
        direct_is_list_struct_type(structs, struct_count, type) ||
        direct_is_map_type(type);
}

static int direct_type_compatible(const char *expected, const char *actual) {
    if (expected == NULL || actual == NULL) return 1;
    if (direct_is_intlike_scalar_type(expected) && direct_is_intlike_scalar_type(actual)) return 1;
    return strcmp(expected, actual) == 0;
}

static const char *direct_c_type(const char *type) {
    if (direct_is_str_type(type)) return "Str";
    if (direct_is_bool_type(type)) return "Bool";
    if (direct_is_char_type(type)) return "long";
    if (direct_is_list_int_type(type)) return "DirectListInt";
    if (direct_is_map_str_int_type(type) || direct_is_map_str_bool_type(type) || direct_is_map_str_char_type(type)) return "DirectMapStrInt";
    if (direct_is_map_type(type)) return "DirectMapIntInt";
    if (direct_is_list_type(type)) {
        static char bufs[4][128];
        static int slot = 0;
        char *elem = direct_list_element_type(type);
        slot = (slot + 1) % 4;
        snprintf(bufs[slot], sizeof(bufs[slot]), "DirectList_%s", elem == NULL ? "Unknown" : elem);
        free(elem);
        return bufs[slot];
    }
    return strcmp(type, "Int") == 0 ? "long" : type;
}

static const char *direct_c_param_type(const char *type) {
    if (direct_is_map_str_int_type(type) || direct_is_map_str_bool_type(type) || direct_is_map_str_char_type(type)) return "DirectMapStrInt *";
    if (direct_is_map_type(type)) return "DirectMapIntInt *";
    if (direct_is_list_type(type)) {
        static char bufs[4][136];
        static int slot = 0;
        slot = (slot + 1) % 4;
        snprintf(bufs[slot], sizeof(bufs[slot]), "%s *", direct_c_type(type));
        return bufs[slot];
    }
    return direct_c_type(type);
}

static char *direct_parse_local_type(const char **cursor) {
    const char *p = skip_ws(*cursor);
    if (starts_with(p, "Map") && !is_ident_continue(p[3])) {
        const char *q = skip_ws(p + 3);
        if (*q == '<') {
            q = skip_ws(q + 1);
            if (!is_ident_start(*q)) return NULL;
            const char *key_start = q;
            q++;
            while (is_ident_continue(*q)) q++;
            char *key = substr_copy(key_start, (size_t)(q - key_start));
            q = skip_ws(q);
            if (*q == ',') {
                q = skip_ws(q + 1);
                if (!is_ident_start(*q)) {
                    free(key);
                    return NULL;
                }
                const char *value_start = q;
                q++;
                while (is_ident_continue(*q)) q++;
                char *value = substr_copy(value_start, (size_t)(q - value_start));
                q = skip_ws(q);
                if (*q == '>') {
                    *cursor = skip_ws(q + 1);
                    StrBuf out;
                    sb_init(&out);
                    sb_append(&out, "Map<");
                    sb_append(&out, key);
                    sb_append(&out, ",");
                    sb_append(&out, value);
                    sb_append(&out, ">");
                    free(key);
                    free(value);
                    return sb_take(&out);
                }
                free(value);
            }
            free(key);
        }
    }
    if (starts_with(p, "List") && !is_ident_continue(p[4])) {
        const char *q = skip_ws(p + 4);
        if (*q == '<') {
            q = skip_ws(q + 1);
            if (!is_ident_start(*q)) return NULL;
            const char *elem_start = q;
            q++;
            while (is_ident_continue(*q)) q++;
            char *elem = substr_copy(elem_start, (size_t)(q - elem_start));
            q = skip_ws(q);
            if (*q == '>') {
                *cursor = skip_ws(q + 1);
                StrBuf out;
                sb_init(&out);
                sb_append(&out, "List<");
                sb_append(&out, elem);
                sb_append(&out, ">");
                free(elem);
                return sb_take(&out);
            }
            free(elem);
        }
    }
    if (!is_ident_start(*p)) return NULL;
    const char *type_start = p;
    p++;
    while (is_ident_continue(*p)) p++;
    *cursor = p;
    return substr_copy(type_start, (size_t)(p - type_start));
}

static int find_matching_brace_c(const char *text, int open_index) {
    int depth = 0;
    for (int i = open_index; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '{') depth++;
        if (text[i] == '}') {
            depth--;
            if (depth == 0) return i;
        }
    }
    return -1;
}

static int direct_parse_struct_field(
    const char *path,
    int line_no,
    const char *source_line,
    const char *text,
    DirectStructInfo *out
) {
    char *part = trim_copy(text);
    const char *s = skip_ws(part);
    if (*s == '\0') {
        free(part);
        return 0;
    }
    if (!is_ident_start(*s)) {
        report_issue(path, line_no, 1, source_line,
            "direct native emitter expected a struct field name",
            "write struct fields as `name: Int`.",
            NULL);
        free(part);
        return -1;
    }
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    char *field = substr_copy(name_start, (size_t)(s - name_start));
    const char *rest = skip_ws(s);
    if (*rest == ':') {
        char *ty = trim_copy(rest + 1);
        int ok = strcmp(ty, "Int") == 0;
        free(ty);
        if (!ok) {
            report_issue(path, line_no, find_col(source_line, field), source_line,
                "direct native emitter supports Int struct fields only",
                "write this direct-engine field as `name: Int`.",
                NULL);
            free(field);
            free(part);
            return -1;
        }
    } else if (*rest != '\0') {
        report_issue(path, line_no, find_col(source_line, field), source_line,
            "direct native emitter expected an Int struct field",
            "write struct fields as `name: Int`.",
            NULL);
        free(field);
        free(part);
        return -1;
    }
    if (direct_struct_has_field(out, field)) {
        report_issue(path, line_no, find_col(source_line, field), source_line,
            "direct native emitter found a duplicate struct field",
            "use each field name only once in a struct declaration.",
            NULL);
        free(field);
        free(part);
        return -1;
    }
    if (out->field_count >= 16) {
        report_issue(path, line_no, 1, source_line,
            "direct native emitter supports up to 16 struct fields",
            "split this direct-engine struct into a smaller shape for now.",
            NULL);
        free(field);
        free(part);
        return -1;
    }
    out->fields[out->field_count++] = field;
    free(part);
    return 0;
}

static int direct_parse_struct_fields(
    const char *path,
    int line_no,
    const char *source_line,
    const char *body,
    DirectStructInfo *out
) {
    const char *start = body;
    for (const char *p = body; ; p++) {
        char ch = *p;
        if (ch == ',' || ch == '\n' || ch == '\0') {
            char *piece = substr_copy(start, (size_t)(p - start));
            int rc = direct_parse_struct_field(path, line_no, source_line, piece, out);
            free(piece);
            if (rc < 0) return -1;
            if (ch == '\0') break;
            start = p + 1;
        }
        if (ch == '\0') break;
    }
    if (out->field_count == 0) {
        report_issue(path, line_no, 1, source_line,
            "direct native emitter expected at least one Int struct field",
            "write a shape such as `struct Box { value: Int }`.",
            NULL);
        return -1;
    }
    return 0;
}

static int direct_parse_struct_decl(
    LineVec *lines,
    size_t start_index,
    DirectStructInfo *out,
    size_t *end_index,
    const char *path
) {
    char *first = strip_line_comment(lines->items[start_index], strlen(lines->items[start_index]));
    const char *s = skip_ws(first);
    if (!starts_with(s, "struct ")) {
        free(first);
        return 0;
    }
    s += 7;
    if (!is_ident_start(*s)) {
        report_issue(path, (int)start_index + 1, find_col(lines->items[start_index], "struct"), lines->items[start_index],
            "direct native emitter expected a struct name",
            "write `struct Name { field: Int }`.",
            NULL);
        free(first);
        return -1;
    }
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *open = strchr(s, '{');
    if (open == NULL) {
        report_issue(path, (int)start_index + 1, find_col(lines->items[start_index], "struct"), lines->items[start_index],
            "direct native emitter expected a struct body",
            "write `struct Name { field: Int }`.",
            NULL);
        free(first);
        return -1;
    }

    memset(out, 0, sizeof(*out));
    out->name = substr_copy(name_start, (size_t)(s - name_start));
    out->line_no = (int)start_index + 1;

    StrBuf body;
    sb_init(&body);
    const char *close = strchr(open + 1, '}');
    if (close != NULL) {
        sb_append_n(&body, open + 1, (size_t)(close - open - 1));
        *end_index = start_index;
    } else {
        sb_append(&body, open + 1);
        sb_append(&body, "\n");
        int found = 0;
        for (size_t i = start_index + 1; i < lines->len; i++) {
            char *field_line = strip_line_comment(lines->items[i], strlen(lines->items[i]));
            const char *field_close = strchr(field_line, '}');
            if (field_close != NULL) {
                sb_append_n(&body, field_line, (size_t)(field_close - field_line));
                *end_index = i;
                found = 1;
                free(field_line);
                break;
            }
            sb_append(&body, field_line);
            sb_append(&body, "\n");
            free(field_line);
        }
        if (!found) {
            report_issue(path, (int)start_index + 1, find_col(lines->items[start_index], "struct"), lines->items[start_index],
                "direct native emitter expected `}` to close the struct",
                "close the struct declaration before declaring functions.",
                NULL);
            free(body.data);
            direct_struct_free_one(out);
            free(first);
            return -1;
        }
    }

    if (direct_parse_struct_fields(path, (int)start_index + 1, lines->items[start_index], body.data, out) != 0) {
        free(body.data);
        direct_struct_free_one(out);
        free(first);
        return -1;
    }
    free(body.data);
    free(first);
    return 1;
}

static int parse_direct_fn_header(const char *line, DirectFnInfo *out) {
    const char *s = skip_ws(line);
    if (!starts_with(s, "fn ")) return 0;
    s += 3;
    if (!is_ident_start(*s)) return -1;
    const char *name_start = s;
    while (is_ident_continue(*s)) s++;
    char *name = substr_copy(name_start, (size_t)(s - name_start));
    const char *open = strchr(s, '(');
    const char *close = open == NULL ? NULL : strchr(open, ')');
    const char *brace = close == NULL ? NULL : strchr(close, '{');
    const char *arrow = close == NULL ? NULL : strstr(close, "->");
    if (open == NULL || close == NULL || brace == NULL || arrow == NULL) {
        free(name);
        return -1;
    }
    const char *ret = skip_ws(arrow + 2);
    const char *ret_cursor = ret;
    char *return_type = direct_parse_local_type(&ret_cursor);
    if (return_type == NULL) {
        free(name);
        return -1;
    }
    if (*skip_ws(ret_cursor) != '{') {
        free(return_type);
        free(name);
        return -1;
    }
    memset(out, 0, sizeof(*out));
    out->name = name;
    out->return_type = return_type;
    char *params_text = substr_copy(open + 1, (size_t)(close - open - 1));
    char *parts[16] = {0};
    int n = split_top_level_type_commas_c(params_text, parts, 16);
    free(params_text);
    if (n < 0) {
        direct_fns_free(out, 1);
        return -1;
    }
    if (n == 1 && parts[0] != NULL && strlen(skip_ws(parts[0])) == 0) {
        free(parts[0]);
        return 1;
    }
    for (int i = 0; i < n; i++) {
        char *part = parts[i];
        char *colon = strchr(part, ':');
        if (colon == NULL) {
            free(part);
            direct_fns_free(out, 1);
            for (int k = i + 1; k < n; k++) free(parts[k]);
            return -1;
        }
        char *pname_raw = substr_copy(part, (size_t)(colon - part));
        char *pname = trim_copy(pname_raw);
        free(pname_raw);
        const char *pty_cursor = skip_ws(colon + 1);
        char *pty = direct_parse_local_type(&pty_cursor);
        int ok = direct_is_plain_ident(pname) && pty != NULL && *skip_ws(pty_cursor) == '\0';
        free(part);
        if (!ok) {
            free(pname);
            free(pty);
            free(out->name);
            free(out->return_type);
            for (int p = 0; p < out->param_count; p++) {
                free(out->params[p]);
                free(out->param_types[p]);
            }
            for (int k = i + 1; k < n; k++) free(parts[k]);
            return -1;
        }
        out->params[out->param_count] = pname;
        out->param_types[out->param_count] = pty;
        out->param_count++;
    }
    return 1;
}

static int direct_validate_fn_types(
    const char *path,
    const char *line,
    DirectFnInfo *info,
    DirectStructInfo *structs,
    int struct_count
) {
    int issues = 0;
    if (!direct_return_type_allowed(structs, struct_count, info->return_type)) {
        report_issue(path, info->line_no, find_col(line, info->return_type), line,
            "direct native emitter function return type is not available",
            "use `Int`, `Bool`, `Char`, `Str`, `List<Int>`, `List<Struct>`, `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, or a struct declared in this file.",
            "fn f() -> Int");
        issues++;
    }
    for (int p = 0; p < info->param_count; p++) {
        if (!direct_param_type_allowed(structs, struct_count, info->param_types[p])) {
            report_issue(path, info->line_no, find_col(line, info->param_types[p]), line,
                "direct native emitter function parameter type is not available",
                "use `Int`, `Bool`, `Char`, `Str`, `List<Int>`, `List<Struct>`, `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, or a struct declared in this file.",
                "fn f(x: Int) -> Int");
            issues++;
        }
    }
    return issues;
}

static char *direct_translate_expr(const char *expr) {
    char *a = replace_word_all(expr, "and", "&&");
    char *b = replace_word_all(a, "or", "||");
    free(a);
    char *c = replace_word_all(b, "not", "!");
    free(b);
    return c;
}

static int direct_parse_list_literal_items(const char *expr, char **parts, int max_parts, int *count) {
    *count = 0;
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    if (*s != '[') {
        free(trimmed);
        return 0;
    }
    int open = (int)(s - trimmed);
    int close = find_matching_bracket_c(trimmed, open);
    if (close < 0 || *skip_ws(trimmed + close + 1) != '\0') {
        free(trimmed);
        return 0;
    }
    char *body = substr_copy(trimmed + open + 1, (size_t)(close - open - 1));
    char *body_trim = trim_copy(body);
    free(body);
    if (body_trim[0] == '\0') {
        free(body_trim);
        free(trimmed);
        return 1;
    }
    int n = split_top_level_commas_c(body_trim, parts, max_parts);
    free(body_trim);
    free(trimmed);
    if (n < 0) return -1;
    *count = n;
    return 1;
}

static int direct_is_list_constructor_expr(const char *expr) {
    char *trimmed = trim_copy(expr);
    int ok = strcmp(trimmed, "list()") == 0;
    free(trimmed);
    return ok;
}

static int direct_parse_list_initializer_items(const char *expr, char **parts, int max_parts, int *count) {
    *count = 0;
    if (direct_is_list_constructor_expr(expr)) return 1;
    return direct_parse_list_literal_items(expr, parts, max_parts, count);
}

static int direct_list_initializer_state(const char *expr) {
    char *parts[16] = {0};
    int count = 0;
    int parsed = direct_parse_list_initializer_items(expr, parts, 16, &count);
    for (int i = 0; i < 16; i++) free(parts[i]);
    return parsed;
}

static int direct_is_list_initializer_expr(const char *expr) {
    int parsed = direct_list_initializer_state(expr);
    return parsed == 1;
}

static int direct_is_map_empty_initializer_expr(const char *expr) {
    char *trimmed = trim_copy(expr);
    int ok = strcmp(trimmed, "{}") == 0;
    free(trimmed);
    return ok;
}

static char *direct_rewrite_struct_literals(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectStructInfo *structs,
    int struct_count
) {
    StrBuf out;
    sb_init(&out);
    for (int i = 0; expr[i] != '\0';) {
        if (is_string_delim_c(expr[i])) {
            int end = skip_string_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an unterminated string literal",
                    "close the string before the end of the line.",
                    NULL);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (expr[i] == '\'') {
            int end = skip_char_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an invalid char literal",
                    "write a single-byte character literal such as `'A'`.",
                    NULL);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (!is_ident_start(expr[i])) {
            sb_append_n(&out, expr + i, 1);
            i++;
            continue;
        }
        int start = i;
        i++;
        while (is_ident_continue(expr[i])) i++;
        char *name = substr_copy(expr + start, (size_t)(i - start));
        int cursor = i;
        while (expr[cursor] == ' ' || expr[cursor] == '\t') cursor++;
        DirectStructInfo *st = direct_find_struct(structs, struct_count, name);
        if (st == NULL || expr[cursor] != '{') {
            sb_append_n(&out, expr + start, (size_t)(i - start));
            free(name);
            continue;
        }
        int close = find_matching_brace_c(expr, cursor);
        if (close < 0) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter expected `}` to close the struct literal",
                "write a complete literal such as `Box { value: 42 }`.",
                NULL);
            free(name);
            free(out.data);
            return NULL;
        }
        char *body = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
        char *parts[16] = {0};
        int n = split_top_level_commas_c(body, parts, 16);
        free(body);
        if (n < 0) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter supports up to 16 struct literal fields",
                "keep this direct-engine literal within the small struct slice.",
                NULL);
            free(name);
            free(out.data);
            return NULL;
        }
        sb_append(&out, "(");
        sb_append(&out, name);
        sb_append(&out, "){");
        for (int p = 0; p < n; p++) {
            char *colon = strchr(parts[p], ':');
            if (colon == NULL) {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter expected named struct literal fields",
                    "write fields as `name: expr` inside the literal.",
                    NULL);
                for (int k = 0; k < n; k++) free(parts[k]);
                free(name);
                free(out.data);
                return NULL;
            }
            char *field_raw = substr_copy(parts[p], (size_t)(colon - parts[p]));
            char *field = trim_copy(field_raw);
            free(field_raw);
            if (!direct_struct_has_field(st, field)) {
                report_issue(path, line_no, find_col(line, field), line,
                    "direct native emitter found an unknown struct field",
                    "use a field declared on this struct.",
                    NULL);
                for (int k = 0; k < n; k++) free(parts[k]);
                free(field);
                free(name);
                free(out.data);
                return NULL;
            }
            char *value = trim_copy(colon + 1);
            char *rewritten_value = direct_rewrite_struct_literals(path, line_no, line, value, structs, struct_count);
            free(value);
            if (rewritten_value == NULL) {
                for (int k = 0; k < n; k++) free(parts[k]);
                free(field);
                free(name);
                free(out.data);
                return NULL;
            }
            if (p > 0) sb_append(&out, ", ");
            sb_append(&out, ".");
            sb_append(&out, field);
            sb_append(&out, " = ");
            sb_append(&out, rewritten_value);
            free(rewritten_value);
            free(field);
        }
        sb_append(&out, "}");
        for (int p = 0; p < n; p++) free(parts[p]);
        free(name);
        i = close + 1;
    }
    return sb_take(&out);
}

static int direct_expr_bare_list_local(DirectNameSet *locals, const char *expr, char **name_out) {
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    if (!is_ident_start(*s)) {
        free(trimmed);
        return 0;
    }
    const char *start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    if (*skip_ws(s) != '\0') {
        free(trimmed);
        return 0;
    }
    char *name = substr_copy(start, (size_t)(s - start));
    int ok = direct_is_list_type(direct_names_type(locals, name));
    if (ok && name_out != NULL) {
        *name_out = name;
    } else {
        free(name);
    }
    free(trimmed);
    return ok;
}

static int direct_expr_bare_map_local(DirectNameSet *locals, const char *expr, char **name_out) {
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    if (!is_ident_start(*s)) {
        free(trimmed);
        return 0;
    }
    const char *start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    if (*skip_ws(s) != '\0') {
        free(trimmed);
        return 0;
    }
    char *name = substr_copy(start, (size_t)(s - start));
    int ok = direct_is_map_type(direct_names_type(locals, name));
    if (ok && name_out != NULL) {
        *name_out = name;
    } else {
        free(name);
    }
    free(trimmed);
    return ok;
}

static DirectFnInfo *direct_expr_exact_call_fn(const char *expr, DirectFnInfo *fns, int fn_count) {
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    if (!is_ident_start(*s)) {
        free(trimmed);
        return NULL;
    }
    const char *name_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    char *name = substr_copy(name_start, (size_t)(s - name_start));
    const char *rest = skip_ws(s);
    if (*rest != '(') {
        free(name);
        free(trimmed);
        return NULL;
    }
    int close = find_matching_paren_c(trimmed, (int)(rest - trimmed));
    if (close < 0 || *skip_ws(trimmed + close + 1) != '\0') {
        free(name);
        free(trimmed);
        return NULL;
    }
    DirectFnInfo *fn = direct_find_fn(fns, fn_count, name);
    free(name);
    free(trimmed);
    return fn;
}

static int direct_expr_exact_list_return_call(const char *expr, DirectFnInfo *fns, int fn_count) {
    DirectFnInfo *fn = direct_expr_exact_call_fn(expr, fns, fn_count);
    return fn != NULL && direct_is_list_type(fn->return_type);
}

static int direct_expr_exact_map_return_call(const char *expr, DirectFnInfo *fns, int fn_count) {
    DirectFnInfo *fn = direct_expr_exact_call_fn(expr, fns, fn_count);
    return fn != NULL && direct_is_supported_map_return_type(fn->return_type);
}

static void direct_append_map_ptr_ref(StrBuf *out, const char *name, int is_ref) {
    if (!is_ref) sb_append(out, "&");
    sb_append(out, name);
}

static void direct_append_list_len_ref(StrBuf *out, const char *name, int is_ref) {
    sb_append(out, name);
    sb_append(out, is_ref ? "->len" : ".len");
}

static void direct_append_list_data_ref(StrBuf *out, const char *name, int is_ref) {
    sb_append(out, name);
    sb_append(out, is_ref ? "->data" : ".data");
}

static void direct_append_list_sum_ref(StrBuf *out, const char *name, int is_ref) {
    sb_append(out, "__vais_list_int_sum(");
    if (!is_ref) sb_append(out, "&");
    sb_append(out, name);
    sb_append(out, ")");
}

static void direct_append_list_is_empty_ref(StrBuf *out, const char *name, int is_ref) {
    sb_append(out, "(");
    direct_append_list_len_ref(out, name, is_ref);
    sb_append(out, " == 0 ? 1 : 0)");
}

static void direct_append_list_last_ref(StrBuf *out, const char *name, int is_ref) {
    direct_append_list_data_ref(out, name, is_ref);
    sb_append(out, "[__vais_list_checked_last(");
    direct_append_list_len_ref(out, name, is_ref);
    sb_append(out, ")]");
}

static void direct_append_list_pop_ref(StrBuf *out, const char *name, int is_ref, const char *base_type, DirectNameSet *locals) {
    if (direct_current_prelude == NULL) {
        direct_append_list_data_ref(out, name, is_ref);
        sb_append(out, "[__vais_list_checked_pop_index(&");
        direct_append_list_len_ref(out, name, is_ref);
        sb_append(out, ")]");
        return;
    }
    char *elem_type = direct_list_element_type(base_type);
    char tmp_name[64];
    snprintf(tmp_name, sizeof(tmp_name), "__vais_list_pop_%d", locals->temp_count++);
    sb_append(direct_current_prelude, direct_c_type(elem_type == NULL ? "Int" : elem_type));
    sb_append(direct_current_prelude, " ");
    sb_append(direct_current_prelude, tmp_name);
    sb_append(direct_current_prelude, " = ");
    direct_append_list_data_ref(direct_current_prelude, name, is_ref);
    sb_append(direct_current_prelude, "[__vais_list_checked_last(");
    direct_append_list_len_ref(direct_current_prelude, name, is_ref);
    sb_append(direct_current_prelude, ")];\n");
    direct_append_list_len_ref(direct_current_prelude, name, is_ref);
    sb_append(direct_current_prelude, " -= 1;\n");
    sb_append(out, tmp_name);
    free(elem_type);
}

static char *direct_rewrite_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
);

static int direct_check_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
);

static char *direct_infer_expr_type(
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
);

static int direct_is_parse_builtin_name(const char *name) {
    return strcmp(name, "parse_uint") == 0 || strcmp(name, "parse_int") == 0;
}

static int direct_expr_is_string_literal(const char *expr) {
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    int ok = 0;
    if (is_string_delim_c(*s)) {
        int end = skip_string_literal_c(s, 0);
        ok = end >= 0 && *skip_ws(s + end) == '\0';
    }
    free(trimmed);
    return ok;
}

static int direct_expr_is_char_literal(const char *expr) {
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    int ok = 0;
    if (*s == '\'') {
        int end = skip_char_literal_c(s, 0);
        ok = end >= 0 && *skip_ws(s + end) == '\0';
    }
    free(trimmed);
    return ok;
}

static char *direct_rewrite_string_literals(
    const char *path,
    int line_no,
    const char *line,
    const char *expr
) {
    StrBuf out;
    sb_init(&out);
    for (int i = 0; expr[i] != '\0';) {
        if (!is_string_delim_c(expr[i])) {
            sb_append_n(&out, expr + i, 1);
            i++;
            continue;
        }
        char delim = expr[i];
        int end = skip_string_literal_c(expr, i);
        if (end < 0) {
            report_issue(path, line_no, 1, line,
                "direct native emitter found an unterminated string literal",
                "close the string before the end of the line.",
                NULL);
            free(out.data);
            return NULL;
        }
        sb_append(&out, "\"");
        int j = i + 1;
        while (j < end - 1) {
            if (delim == '"' && expr[j] == '\\' && j + 1 < end - 1) {
                j++;
                sb_append_c_escaped_byte(&out, expr[j]);
                j++;
                continue;
            }
            sb_append_c_escaped_byte(&out, expr[j]);
            j++;
        }
        sb_append(&out, "\"");
        i = end;
    }
    return sb_take(&out);
}

static char *direct_rewrite_parse_builtin_calls(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    StrBuf out;
    sb_init(&out);
    for (int i = 0; expr[i] != '\0';) {
        if (is_string_delim_c(expr[i])) {
            int end = skip_string_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an unterminated string literal",
                    "close the string before the end of the line.",
                    NULL);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (expr[i] == '\'') {
            int end = skip_char_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an invalid char literal",
                    "write a single-byte character literal such as `'A'`.",
                    NULL);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (!is_ident_start(expr[i])) {
            sb_append_n(&out, expr + i, 1);
            i++;
            continue;
        }
        int start = i;
        i++;
        while (is_ident_continue(expr[i])) i++;
        char *name = substr_copy(expr + start, (size_t)(i - start));
        int cursor = i;
        while (expr[cursor] == ' ' || expr[cursor] == '\t') cursor++;
        if (!direct_is_parse_builtin_name(name) || expr[cursor] != '(') {
            sb_append_n(&out, expr + start, (size_t)(i - start));
            free(name);
            continue;
        }
        int close = find_matching_paren_c(expr, cursor);
        if (close < 0) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter expected `)` to close the parse call",
                "write `parse_uint(s)` or `parse_int(s)`.",
                NULL);
            free(name);
            free(out.data);
            return NULL;
        }
        char *inside = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
        char *args[16] = {0};
        int argc = split_top_level_commas_c(inside, args, 16);
        free(inside);
        if (argc != 1 || args[0] == NULL || strlen(skip_ws(args[0])) == 0) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter parse helpers take one Str argument",
                "write `parse_uint(s)` or `parse_int(s)`.",
                NULL);
            for (int k = 0; k < 16; k++) free(args[k]);
            free(name);
            free(out.data);
            return NULL;
        }
        char *rewritten_arg = direct_rewrite_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count);
        for (int k = 0; k < 16; k++) free(args[k]);
        if (rewritten_arg == NULL) {
            free(name);
            free(out.data);
            return NULL;
        }
        sb_append(&out, strcmp(name, "parse_uint") == 0 ? "__vais_parse_uint(" : "__vais_parse_int(");
        sb_append(&out, rewritten_arg);
        sb_append(&out, ")");
        free(rewritten_arg);
        free(name);
        i = close + 1;
    }
    return sb_take(&out);
}

static int direct_find_top_level_eq_op(const char *expr, int *op_len) {
    int paren_depth = 0;
    int bracket_depth = 0;
    int brace_depth = 0;
    for (int i = 0; expr[i] != '\0'; i++) {
        if (is_string_delim_c(expr[i])) {
            int end = skip_string_literal_c(expr, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (expr[i] == '\'') {
            int end = skip_char_literal_c(expr, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (expr[i] == '(') paren_depth++;
        else if (expr[i] == ')') paren_depth--;
        else if (expr[i] == '[') bracket_depth++;
        else if (expr[i] == ']') bracket_depth--;
        else if (expr[i] == '{') brace_depth++;
        else if (expr[i] == '}') brace_depth--;
        else if (paren_depth == 0 && bracket_depth == 0 && brace_depth == 0) {
            if (expr[i] == '=' && expr[i + 1] == '=') {
                *op_len = 2;
                return i;
            }
            if (expr[i] == '!' && expr[i + 1] == '=') {
                *op_len = 2;
                return i;
            }
        }
    }
    return -1;
}

static char *direct_rewrite_str_equality_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    int op_len = 0;
    int op = direct_find_top_level_eq_op(expr, &op_len);
    if (op < 0) return NULL;
    char *lhs_raw = substr_copy(expr, (size_t)op);
    char *lhs = trim_copy(lhs_raw);
    free(lhs_raw);
    char *rhs = trim_copy(expr + op + op_len);
    char *lhs_type = direct_infer_expr_type(lhs, locals, fns, fn_count, structs, struct_count);
    char *rhs_type = direct_infer_expr_type(rhs, locals, fns, fn_count, structs, struct_count);
    int is_str_eq = direct_is_str_type(lhs_type) && direct_is_str_type(rhs_type);
    free(lhs_type);
    free(rhs_type);
    if (!is_str_eq) {
        free(lhs);
        free(rhs);
        return NULL;
    }
    char *rewritten_lhs = direct_rewrite_expr(path, line_no, line, lhs, locals, fns, fn_count, structs, struct_count);
    char *rewritten_rhs = direct_rewrite_expr(path, line_no, line, rhs, locals, fns, fn_count, structs, struct_count);
    free(lhs);
    free(rhs);
    if (rewritten_lhs == NULL || rewritten_rhs == NULL) {
        free(rewritten_lhs);
        free(rewritten_rhs);
        return NULL;
    }
    StrBuf out;
    sb_init(&out);
    int not_equal = expr[op] == '!';
    if (not_equal) sb_append(&out, "(!");
    sb_append(&out, "__vais_str_eq(");
    sb_append(&out, rewritten_lhs);
    sb_append(&out, ", ");
    sb_append(&out, rewritten_rhs);
    sb_append(&out, ")");
    if (not_equal) sb_append(&out, ")");
    free(rewritten_lhs);
    free(rewritten_rhs);
    return sb_take(&out);
}

static char *direct_rewrite_list_literal_value_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    const char *list_type,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    if (!direct_is_list_type(list_type)) return NULL;
    char *items[16] = {0};
    int item_count = 0;
    int literal_state = direct_parse_list_initializer_items(expr, items, 16, &item_count);
    if (literal_state < 0) {
        report_issue(path, line_no, find_col(line, "List"), line,
            "direct native emitter supports up to 16 List literal items",
            "bind a local list and call `push` for larger direct-engine lists.",
            NULL);
        for (int k = 0; k < 16; k++) free(items[k]);
        return NULL;
    }
    if (literal_state != 1) {
        for (int k = 0; k < 16; k++) free(items[k]);
        return NULL;
    }

    StrBuf out;
    sb_init(&out);
    sb_append(&out, "((");
    sb_append(&out, direct_c_type(list_type));
    sb_append(&out, "){ .data = {");
    if (item_count == 0) {
        sb_append(&out, "0");
    }
    for (int item = 0; item < item_count; item++) {
        char *rewritten_item = direct_rewrite_expr(path, line_no, line, items[item], locals, fns, fn_count, structs, struct_count);
        if (rewritten_item == NULL) {
            for (int k = item; k < 16; k++) free(items[k]);
            free(out.data);
            return NULL;
        }
        char *c_item = direct_translate_expr(rewritten_item);
        if (item > 0) sb_append(&out, ", ");
        sb_append(&out, c_item);
        free(c_item);
        free(rewritten_item);
        free(items[item]);
        items[item] = NULL;
    }
    sb_append(&out, "}, .len = ");
    char len_buf[32];
    snprintf(len_buf, sizeof(len_buf), "%d", item_count);
    sb_append(&out, len_buf);
    sb_append(&out, " })");
    return sb_take(&out);
}

static char *direct_rewrite_list_arg_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    const char *list_type,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    if (direct_is_list_initializer_expr(expr)) {
        char *value = direct_rewrite_list_literal_value_expr(path, line_no, line, expr, list_type, locals, fns, fn_count, structs, struct_count);
        if (value == NULL) return NULL;
        StrBuf out;
        sb_init(&out);
        sb_append(&out, "&");
        sb_append(&out, value);
        free(value);
        return sb_take(&out);
    }
    char *name = NULL;
    if (direct_expr_bare_list_local(locals, expr, &name)) {
        StrBuf out;
        sb_init(&out);
        if (!direct_names_is_ref(locals, name)) sb_append(&out, "&");
        sb_append(&out, name);
        free(name);
        return sb_take(&out);
    }
    if (direct_expr_exact_list_return_call(expr, fns, fn_count)) {
        char *rewritten_value = direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        if (rewritten_value == NULL) return NULL;
        char *c_value = direct_translate_expr(rewritten_value);
        free(rewritten_value);
        if (direct_current_prelude == NULL) {
            StrBuf out;
            sb_init(&out);
            sb_append(&out, "&((");
            sb_append(&out, direct_c_type(list_type));
            sb_append(&out, "[]){");
            sb_append(&out, c_value);
            sb_append(&out, "})[0]");
            free(c_value);
            return sb_take(&out);
        }
        char tmp_name[64];
        snprintf(tmp_name, sizeof(tmp_name), "__vais_list_arg_%d", locals->temp_count++);
        sb_append(direct_current_prelude, direct_c_type(list_type));
        sb_append(direct_current_prelude, " ");
        sb_append(direct_current_prelude, tmp_name);
        sb_append(direct_current_prelude, " = ");
        sb_append(direct_current_prelude, c_value);
        sb_append(direct_current_prelude, ";\n");
        free(c_value);
        StrBuf out;
        sb_init(&out);
        sb_append(&out, "&");
        sb_append(&out, tmp_name);
        return sb_take(&out);
    }
    report_issue(path, line_no, find_col(line, expr), line,
        "direct native emitter requires a local List argument",
        "bind the list value to a local before passing it.",
        "let xs: List<Int> = []");
    return NULL;
}

static char *direct_rewrite_map_arg_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals
) {
    char *name = NULL;
    if (direct_expr_bare_map_local(locals, expr, &name)) {
        StrBuf out;
        sb_init(&out);
        direct_append_map_ptr_ref(&out, name, direct_names_is_ref(locals, name));
        free(name);
        return sb_take(&out);
    }
    report_issue(path, line_no, find_col(line, expr), line,
        "direct native emitter requires a local Map argument",
        "bind the Map value to a local before passing it.",
        "let m: Map<Int,Int> = {}");
    return NULL;
}

static int direct_check_map_value_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    const char *map_type,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    if (!direct_is_supported_map_return_type(map_type)) {
        report_issue(path, line_no, find_col(line, expr), line,
            "direct native emitter Map return values are verified only for Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, and Map<Str,Char>",
            "use `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>` for returned Map values; keep generic Map values local.",
            "fn make() -> Map<Int,Int>");
        return 1;
    }
    char *name = NULL;
    if (direct_expr_bare_map_local(locals, expr, &name)) {
        const char *actual = direct_names_type(locals, name);
        int ok = strcmp(actual, map_type) == 0;
        free(name);
        if (ok) return 0;
    }
    if (direct_expr_exact_map_return_call(expr, fns, fn_count)) {
        char *expr_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
        int ok_type = strcmp(expr_type, map_type) == 0;
        free(expr_type);
        if (!ok_type) {
            report_issue(path, line_no, find_col(line, expr), line,
                "direct native emitter Map value expression type does not match",
                "return or initialize with the same concrete Map type.",
                NULL);
            return 1;
        }
        return direct_check_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
    }
    report_issue(path, line_no, find_col(line, expr), line,
        "direct native emitter Map value expression must be a local Map or Map-returning call",
        "return a local `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>`, or initialize from a helper returning the same Map type.",
        "let scores: Map<Int,Int> = make()");
    return 1;
}

static char *direct_rewrite_map_value_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    const char *map_type,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    if (!direct_is_supported_map_return_type(map_type)) return NULL;
    char *name = NULL;
    if (direct_expr_bare_map_local(locals, expr, &name)) {
        const char *actual = direct_names_type(locals, name);
        if (actual != NULL && strcmp(actual, map_type) == 0) {
            StrBuf out;
            sb_init(&out);
            if (direct_names_is_ref(locals, name)) sb_append(&out, "*");
            sb_append(&out, name);
            free(name);
            return sb_take(&out);
        }
        free(name);
    }
    if (direct_expr_exact_map_return_call(expr, fns, fn_count)) {
        return direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
    }
    report_issue(path, line_no, find_col(line, expr), line,
        "direct native emitter Map value expression must be a local Map or Map-returning call",
        "return a local `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>`, or initialize from a helper returning the same Map type.",
        "let scores: Map<Int,Int> = make()");
    return NULL;
}

static char *direct_rewrite_list_value_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    const char *list_type,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    if (direct_is_list_initializer_expr(expr)) {
        return direct_rewrite_list_literal_value_expr(path, line_no, line, expr, list_type, locals, fns, fn_count, structs, struct_count);
    }
    char *name = NULL;
    if (direct_expr_bare_list_local(locals, expr, &name)) {
        StrBuf out;
        sb_init(&out);
        if (direct_names_is_ref(locals, name)) sb_append(&out, "*");
        sb_append(&out, name);
        free(name);
        return sb_take(&out);
    }
    return direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
}

static char *direct_rewrite_list_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    StrBuf out;
    sb_init(&out);
    for (int i = 0; expr[i] != '\0';) {
        if (is_string_delim_c(expr[i])) {
            int end = skip_string_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an unterminated string literal",
                    "close the string before the end of the line.",
                    NULL);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (expr[i] == '\'') {
            int end = skip_char_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an invalid char literal",
                    "write a single-byte character literal such as `'A'`.",
                    NULL);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + i, (size_t)(end - i));
            i = end;
            continue;
        }
        if (!is_ident_start(expr[i])) {
            sb_append_n(&out, expr + i, 1);
            i++;
            continue;
        }
        int start = i;
        i++;
        while (is_ident_continue(expr[i])) i++;
        char *name = substr_copy(expr + start, (size_t)(i - start));
        const char *base_type = direct_names_type(locals, name);
        int cursor = i;
        while (expr[cursor] == ' ' || expr[cursor] == '\t') cursor++;
        if (expr[cursor] == '(') {
            DirectFnInfo *fn = direct_find_fn(fns, fn_count, name);
            if (fn != NULL) {
                int close = find_matching_paren_c(expr, cursor);
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` to close the function call",
                        "close the call arguments.",
                        NULL);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                char *inside = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
                char *args[16] = {0};
                int argc = split_top_level_commas_c(inside, args, 16);
                free(inside);
                if (argc == 1 && args[0] != NULL && strlen(skip_ws(args[0])) == 0) {
                    free(args[0]);
                    args[0] = NULL;
                    argc = 0;
                }
                if (argc < 0 || argc != fn->param_count) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter function call argument count does not match",
                        "pass exactly the parameters declared by the direct helper.",
                        NULL);
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                sb_append(&out, name);
                sb_append(&out, "(");
                for (int a = 0; a < argc; a++) {
                    char *rewritten_arg = NULL;
                    if (direct_is_list_type(fn->param_types[a])) {
                        rewritten_arg = direct_rewrite_list_arg_expr(path, line_no, line, args[a], fn->param_types[a], locals, fns, fn_count, structs, struct_count);
                    } else if (direct_is_map_type(fn->param_types[a])) {
                        rewritten_arg = direct_rewrite_map_arg_expr(path, line_no, line, args[a], locals);
                    } else {
                        rewritten_arg = direct_rewrite_expr(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count);
                    }
                    if (rewritten_arg == NULL) {
                        for (int k = 0; k < 16; k++) free(args[k]);
                        free(name);
                        free(out.data);
                        return NULL;
                    }
                    if (a > 0) sb_append(&out, ", ");
                    sb_append(&out, rewritten_arg);
                    free(rewritten_arg);
                }
                sb_append(&out, ")");
                for (int k = 0; k < 16; k++) free(args[k]);
                free(name);
                i = close + 1;
                continue;
            }
        }
        if (direct_is_str_type(base_type)) {
            if (expr[cursor] == '[') {
                int close = find_matching_bracket_c(expr, cursor);
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `]` to close a Str index",
                        "write indexes as `s[i]`.",
                        NULL);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                char *index_expr = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
                char *rewritten_index = direct_rewrite_expr(path, line_no, line, index_expr, locals, fns, fn_count, structs, struct_count);
                free(index_expr);
                if (rewritten_index == NULL) {
                    free(name);
                    free(out.data);
                    return NULL;
                }
                sb_append(&out, "__vais_str_byte(");
                sb_append(&out, name);
                sb_append(&out, ", (long)(");
                sb_append(&out, rewritten_index);
                sb_append(&out, "))");
                free(rewritten_index);
                free(name);
                i = close + 1;
                continue;
            }
            if (expr[cursor] == '.') {
                int field_start = cursor + 1;
                while (expr[field_start] == ' ' || expr[field_start] == '\t') field_start++;
                int field_end = field_start;
                while (is_ident_continue(expr[field_end])) field_end++;
                char *field = substr_copy(expr + field_start, (size_t)(field_end - field_start));
                const char *after = skip_ws(expr + field_end);
                if (strcmp(field, "len") == 0) {
                    int next_i = field_end;
                    if (*after == '(') {
                        int close = find_matching_paren_c(expr, (int)(after - expr));
                        if (close < 0) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter expected `)` after Str.len",
                                "write `s.len()` or `s.len`.",
                                NULL);
                            free(field);
                            free(name);
                            free(out.data);
                            return NULL;
                        }
                        char *args = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                        char *trimmed_args = trim_copy(args);
                        int has_args = trimmed_args[0] != '\0';
                        free(trimmed_args);
                        free(args);
                        if (has_args) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter Str.len takes no arguments",
                                "write `s.len()`.",
                                NULL);
                            free(field);
                            free(name);
                            free(out.data);
                            return NULL;
                        }
                        next_i = close + 1;
                    }
                    sb_append(&out, "__vais_str_len(");
                    sb_append(&out, name);
                    sb_append(&out, ")");
                    free(field);
                    free(name);
                    i = next_i;
                    continue;
                }
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter supports Str len and index expressions",
                    "write `s.len()` or `s[i]`.",
                    NULL);
                free(field);
                free(name);
                free(out.data);
                return NULL;
            }
            sb_append_n(&out, expr + start, (size_t)(i - start));
            free(name);
            continue;
        }
        if (!direct_is_list_type(base_type)) {
            if (direct_is_map_type(base_type)) {
                if (expr[cursor] == '.') {
                    int field_start = cursor + 1;
                    while (expr[field_start] == ' ' || expr[field_start] == '\t') field_start++;
                    int field_end = field_start;
                    while (is_ident_continue(expr[field_end])) field_end++;
                    char *field = substr_copy(expr + field_start, (size_t)(field_end - field_start));
                    const char *after = skip_ws(expr + field_end);
                    if (strcmp(field, "len") == 0) {
                        if (*after != '(') {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter Map.len must be called",
                                "write `m.len()`.",
                                NULL);
                            free(field);
                            free(name);
                            free(out.data);
                            return NULL;
                        }
                        int close = find_matching_paren_c(expr, (int)(after - expr));
                        if (close < 0) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter expected `)` after Map.len",
                                "write `m.len()`.",
                                NULL);
                            free(field);
                            free(name);
                            free(out.data);
                            return NULL;
                        }
                        sb_append(&out, direct_map_helper_name(base_type, "len"));
                        sb_append(&out, "(");
                        direct_append_map_ptr_ref(&out, name, direct_names_is_ref(locals, name));
                        sb_append(&out, ")");
                        free(field);
                        free(name);
                        i = close + 1;
                        continue;
                    }
                    if ((strcmp(field, "contains") == 0 || strcmp(field, "get") == 0 || strcmp(field, "get_opt") == 0) && *after == '(') {
                        int close = find_matching_paren_c(expr, (int)(after - expr));
                        if (close < 0) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter expected `)` after Map method",
                                "write `m.contains(key)`, `m.get(key, default)`, or `m.get_opt(key)`.",
                                NULL);
                            free(field);
                            free(name);
                            free(out.data);
                            return NULL;
                        }
                        char *inside = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                        char *args[16] = {0};
                        int argc = split_top_level_commas_c(inside, args, 16);
                        free(inside);
                        int expected = strcmp(field, "get") == 0 ? 2 : 1;
                        if (argc != expected) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter Map method argument count does not match",
                                "write `m.contains(key)`, `m.get(key, default)`, or `m.get_opt(key)`.",
                                NULL);
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(field);
                            free(name);
                            free(out.data);
                            return NULL;
                        }
                        sb_append(&out, direct_map_helper_name(base_type, field));
                        sb_append(&out, "(");
                        direct_append_map_ptr_ref(&out, name, direct_names_is_ref(locals, name));
                        for (int a = 0; a < argc; a++) {
                            char *rewritten_arg = direct_rewrite_expr(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count);
                            if (rewritten_arg == NULL) {
                                for (int k = 0; k < 16; k++) free(args[k]);
                                free(field);
                                free(name);
                                free(out.data);
                                return NULL;
                            }
                            sb_append(&out, ", ");
                            sb_append(&out, rewritten_arg);
                            free(rewritten_arg);
                        }
                        sb_append(&out, ")");
                        for (int k = 0; k < 16; k++) free(args[k]);
                        free(field);
                        free(name);
                        i = close + 1;
                        continue;
                    }
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter supports Map get, get_opt, contains, and len expressions",
                            "write `m.get(key, default)`, `m.get_opt(key)`, `m.contains(key)`, or `m.len()` for Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, local Map<Str,Int>, local Map<Str,Bool>, or local Map<Str,Char>.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                sb_append_n(&out, expr + start, (size_t)(i - start));
                free(name);
                continue;
            }
            sb_append_n(&out, expr + start, (size_t)(i - start));
            free(name);
            continue;
        }

        int is_ref = direct_names_is_ref(locals, name);
        if (expr[cursor] == '[') {
            int close = find_matching_bracket_c(expr, cursor);
            if (close < 0) {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter expected `]` to close a List<Int> index",
                    "write indexes as `xs[i]`.",
                    NULL);
                free(name);
                free(out.data);
                return NULL;
            }
            char *index_expr = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
            char *rewritten_index = direct_rewrite_list_expr(path, line_no, line, index_expr, locals, fns, fn_count, structs, struct_count);
            free(index_expr);
            if (rewritten_index == NULL) {
                free(name);
                free(out.data);
                return NULL;
            }
            direct_append_list_data_ref(&out, name, is_ref);
            sb_append(&out, "[__vais_list_checked_index((long)(");
            sb_append(&out, rewritten_index);
            sb_append(&out, "), ");
            direct_append_list_len_ref(&out, name, is_ref);
            sb_append(&out, ")]");
            free(rewritten_index);
            free(name);
            i = close + 1;
            continue;
        }

        if (expr[cursor] == '.') {
            int field_start = cursor + 1;
            while (expr[field_start] == ' ' || expr[field_start] == '\t') field_start++;
            int field_end = field_start;
            while (is_ident_continue(expr[field_end])) field_end++;
            char *field = substr_copy(expr + field_start, (size_t)(field_end - field_start));
            const char *after = skip_ws(expr + field_end);
            if (strcmp(field, "len") == 0) {
                int next_i = field_end;
                if (*after == '(') {
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after List<Int>.len",
                            "write `xs.len()` or `xs.len`.",
                            NULL);
                        free(field);
                        free(name);
                        free(out.data);
                        return NULL;
                    }
                    char *args = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                    char *trimmed_args = trim_copy(args);
                    int has_args = trimmed_args[0] != '\0';
                    free(trimmed_args);
                    free(args);
                    if (has_args) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter List<Int>.len takes no arguments",
                            "write `xs.len()`.",
                            NULL);
                        free(field);
                        free(name);
                        free(out.data);
                        return NULL;
                    }
                    next_i = close + 1;
                }
                direct_append_list_len_ref(&out, name, is_ref);
                free(field);
                free(name);
                i = next_i;
                continue;
            }
            if (strcmp(field, "is_empty") == 0) {
                if (*after != '(') {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List.is_empty must be called",
                        "write `xs.is_empty()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                int close = find_matching_paren_c(expr, (int)(after - expr));
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` after List.is_empty",
                        "write `xs.is_empty()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                char *args = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                char *trimmed_args = trim_copy(args);
                int has_args = trimmed_args[0] != '\0';
                free(trimmed_args);
                free(args);
                if (has_args) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List.is_empty takes no arguments",
                        "write `xs.is_empty()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                direct_append_list_is_empty_ref(&out, name, is_ref);
                free(field);
                free(name);
                i = close + 1;
                continue;
            }
            if (strcmp(field, "last") == 0) {
                if (*after != '(') {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List.last must be called",
                        "write `xs.last()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                int close = find_matching_paren_c(expr, (int)(after - expr));
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` after List.last",
                        "write `xs.last()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                char *args = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                char *trimmed_args = trim_copy(args);
                int has_args = trimmed_args[0] != '\0';
                free(trimmed_args);
                free(args);
                if (has_args) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List.last takes no arguments",
                        "write `xs.last()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                direct_append_list_last_ref(&out, name, is_ref);
                free(field);
                free(name);
                i = close + 1;
                continue;
            }
            if (strcmp(field, "pop") == 0) {
                if (*after != '(') {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List.pop must be called",
                        "write `xs.pop()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                int close = find_matching_paren_c(expr, (int)(after - expr));
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` after List.pop",
                        "write `xs.pop()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                char *args = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                char *trimmed_args = trim_copy(args);
                int has_args = trimmed_args[0] != '\0';
                free(trimmed_args);
                free(args);
                if (has_args) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List.pop takes no arguments",
                        "write `xs.pop()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                direct_append_list_pop_ref(&out, name, is_ref, base_type, locals);
                free(field);
                free(name);
                i = close + 1;
                continue;
            }
            if (strcmp(field, "sum") == 0) {
                if (!direct_is_list_int_type(base_type)) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter supports sum only on List<Int>",
                        "read struct list fields explicitly before summing.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                if (*after != '(') {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List<Int>.sum must be called",
                        "write `xs.sum()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                int close = find_matching_paren_c(expr, (int)(after - expr));
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` after List<Int>.sum",
                        "write `xs.sum()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                char *args = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                char *trimmed_args = trim_copy(args);
                int has_args = trimmed_args[0] != '\0';
                free(trimmed_args);
                free(args);
                if (has_args) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter List<Int>.sum takes no arguments",
                        "write `xs.sum()`.",
                        NULL);
                    free(field);
                    free(name);
                    free(out.data);
                    return NULL;
                }
                direct_append_list_sum_ref(&out, name, is_ref);
                free(field);
                free(name);
                i = close + 1;
                continue;
            }
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter supports List len, is_empty, last, pop, index, and List<Int> sum expressions",
                "write `xs.len()`, `xs.is_empty()`, `xs.last()`, `xs.pop()`, `xs[i]`, or `xs.sum()` for List<Int>.",
                NULL);
            free(field);
            free(name);
            free(out.data);
            return NULL;
        }

        sb_append_n(&out, expr + start, (size_t)(i - start));
        free(name);
    }
    return sb_take(&out);
}

static char *direct_rewrite_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    char *struct_rewritten = direct_rewrite_struct_literals(path, line_no, line, expr, structs, struct_count);
    if (struct_rewritten == NULL) return NULL;
    char *eq_rewritten = direct_rewrite_str_equality_expr(path, line_no, line, struct_rewritten, locals, fns, fn_count, structs, struct_count);
    if (eq_rewritten != NULL) {
        free(struct_rewritten);
        return eq_rewritten;
    }
    char *list_rewritten = direct_rewrite_list_expr(path, line_no, line, struct_rewritten, locals, fns, fn_count, structs, struct_count);
    free(struct_rewritten);
    if (list_rewritten == NULL) return NULL;
    char *parse_rewritten = direct_rewrite_parse_builtin_calls(path, line_no, line, list_rewritten, locals, fns, fn_count, structs, struct_count);
    free(list_rewritten);
    if (parse_rewritten == NULL) return NULL;
    char *string_rewritten = direct_rewrite_string_literals(path, line_no, line, parse_rewritten);
    free(parse_rewritten);
    return string_rewritten;
}

static char *direct_infer_expr_type(
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    char *trimmed = trim_copy(expr);
    const char *s = skip_ws(trimmed);
    if (direct_expr_is_string_literal(trimmed)) {
        free(trimmed);
        return strdup("Str");
    }
    if (direct_expr_is_char_literal(trimmed)) {
        free(trimmed);
        return strdup("Char");
    }
    if (direct_is_list_initializer_expr(trimmed)) {
        free(trimmed);
        return strdup("List<Int>");
    }
    if (strcmp(s, "true") == 0 || strcmp(s, "false") == 0) {
        free(trimmed);
        return strdup("Bool");
    }
    if (is_ident_start(*s)) {
        const char *name_start = s;
        s++;
        while (is_ident_continue(*s)) s++;
        char *name = substr_copy(name_start, (size_t)(s - name_start));
        const char *rest = skip_ws(s);
        if (*rest == '\0') {
            const char *local_type = direct_names_type(locals, name);
            if (local_type != NULL) {
                char *out = strdup(local_type);
                free(name);
                free(trimmed);
                return out;
            }
        }
        if (*rest == '[') {
            int close = find_matching_bracket_c(trimmed, (int)(rest - trimmed));
            if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                const char *local_type = direct_names_type(locals, name);
                if (direct_is_str_type(local_type)) {
                    free(name);
                    free(trimmed);
                    return strdup("Int");
                }
                char *elem_type = direct_list_element_type(local_type);
                if (elem_type != NULL) {
                    free(name);
                    free(trimmed);
                    return elem_type;
                }
            }
        }
        if (*rest == '.') {
            int field_start = (int)(rest - trimmed) + 1;
            while (trimmed[field_start] == ' ' || trimmed[field_start] == '\t') field_start++;
            int field_end = field_start;
            while (is_ident_continue(trimmed[field_end])) field_end++;
            char *field = substr_copy(trimmed + field_start, (size_t)(field_end - field_start));
            const char *after = skip_ws(trimmed + field_end);
            const char *local_type = direct_names_type(locals, name);
            if (direct_is_str_type(local_type) && strcmp(field, "len") == 0) {
                if (*after == '\0') {
                    free(field);
                    free(name);
                    free(trimmed);
                    return strdup("Int");
                }
                if (*after == '(') {
                    int close = find_matching_paren_c(trimmed, (int)(after - trimmed));
                    if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                        free(field);
                        free(name);
                        free(trimmed);
                        return strdup("Int");
                    }
                }
            }
            if (direct_is_list_type(local_type) && (strcmp(field, "last") == 0 || strcmp(field, "pop") == 0) && *after == '(') {
                int close = find_matching_paren_c(trimmed, (int)(after - trimmed));
                if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                    char *elem_type = direct_list_element_type(local_type);
                    free(field);
                    free(name);
                    free(trimmed);
                    return elem_type;
                }
            }
            if (direct_is_map_type(local_type)) {
                if (strcmp(field, "get") == 0 && *after == '(') {
                    int close = find_matching_paren_c(trimmed, (int)(after - trimmed));
                    if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                        const char *value_type = direct_map_value_type(local_type);
                        free(field);
                        free(name);
                        free(trimmed);
                        return strdup(value_type == NULL ? "Int" : value_type);
                    }
                }
                if ((strcmp(field, "get_opt") == 0 || strcmp(field, "len") == 0) && *after == '(') {
                    int close = find_matching_paren_c(trimmed, (int)(after - trimmed));
                    if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                        free(field);
                        free(name);
                        free(trimmed);
                        return strdup("Int");
                    }
                }
                if (strcmp(field, "contains") == 0 && *after == '(') {
                    int close = find_matching_paren_c(trimmed, (int)(after - trimmed));
                    if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                        free(field);
                        free(name);
                        free(trimmed);
                        return strdup("Bool");
                    }
                }
            }
            free(field);
        }
        if (*rest == '{' && direct_find_struct(structs, struct_count, name) != NULL) {
            char *out = strdup(name);
            free(name);
            free(trimmed);
            return out;
        }
        if (*rest == '(') {
            int close = find_matching_paren_c(trimmed, (int)(rest - trimmed));
            if (close >= 0 && *skip_ws(trimmed + close + 1) == '\0') {
                if (direct_is_parse_builtin_name(name)) {
                    free(name);
                    free(trimmed);
                    return strdup("Int");
                }
                DirectFnInfo *fn = direct_find_fn(fns, fn_count, name);
                if (fn != NULL) {
                    char *out = strdup(fn->return_type);
                    free(name);
                    free(trimmed);
                    return out;
                }
            }
        }
        free(name);
    }
    free(trimmed);
    return strdup("Int");
}

static int direct_is_keyword(const char *name) {
    return strcmp(name, "fn") == 0 || strcmp(name, "return") == 0 ||
        strcmp(name, "let") == 0 || strcmp(name, "mut") == 0 ||
        strcmp(name, "if") == 0 || strcmp(name, "else") == 0 ||
        strcmp(name, "while") == 0 || strcmp(name, "for") == 0 ||
        strcmp(name, "break") == 0 || strcmp(name, "continue") == 0 ||
        strcmp(name, "in") == 0 || strcmp(name, "Int") == 0 ||
        strcmp(name, "Str") == 0 || strcmp(name, "Bool") == 0 ||
        strcmp(name, "Char") == 0 ||
        strcmp(name, "true") == 0 || strcmp(name, "false") == 0 ||
        strcmp(name, "and") == 0 || strcmp(name, "or") == 0 ||
        strcmp(name, "not") == 0;
}

static int direct_check_expr_inner(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count,
    const char *allowed_list_type
) {
    if (allowed_list_type != NULL) {
        char *allowed_list_elem_type = direct_list_element_type(allowed_list_type);
        if (allowed_list_elem_type == NULL) return 1;
        char *items[16] = {0};
        int item_count = 0;
        int literal_state = direct_parse_list_initializer_items(expr, items, 16, &item_count);
        if (literal_state < 0) {
            report_issue(path, line_no, find_col(line, "List"), line,
                "direct native emitter supports up to 16 List literal items",
                "bind a local list and call `push` for larger direct-engine lists.",
                NULL);
            for (int k = 0; k < 16; k++) free(items[k]);
            free(allowed_list_elem_type);
            return 1;
        }
        if (literal_state == 1) {
            for (int item = 0; item < item_count; item++) {
                if (direct_check_expr_inner(path, line_no, line, items[item], locals, fns, fn_count, structs, struct_count, NULL)) {
                    for (int k = item; k < 16; k++) free(items[k]);
                    free(allowed_list_elem_type);
                    return 1;
                }
                char *item_type = direct_infer_expr_type(items[item], locals, fns, fn_count, structs, struct_count);
                if (strcmp(item_type, allowed_list_elem_type) != 0) {
                    report_issue(path, line_no, find_col(line, "List"), line,
                        "direct native emitter list literal item type does not match the expected list element type",
                        "use items with the element type declared by the surrounding list.",
                        NULL);
                    free(item_type);
                    for (int k = item; k < 16; k++) free(items[k]);
                    free(allowed_list_elem_type);
                    return 1;
                }
                free(item_type);
                free(items[item]);
            }
            free(allowed_list_elem_type);
            return 0;
        }
        free(allowed_list_elem_type);
    }
    for (int i = 0; expr[i] != '\0';) {
        if (is_string_delim_c(expr[i])) {
            int end = skip_string_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an unterminated string literal",
                    "close the string before the end of the line.",
                    NULL);
                return 1;
            }
            i = end;
            continue;
        }
        if (expr[i] == '\'') {
            int end = skip_char_literal_c(expr, i);
            if (end < 0) {
                report_issue(path, line_no, 1, line,
                    "direct native emitter found an invalid char literal",
                    "write a single-byte character literal such as `'A'`.",
                    NULL);
                return 1;
            }
            i = end;
            continue;
        }
        if (!is_ident_start(expr[i])) {
            i++;
            continue;
        }
        int prev = i - 1;
        while (prev >= 0 && (expr[prev] == ' ' || expr[prev] == '\t')) prev--;
        if (prev >= 0 && expr[prev] == '.') {
            i++;
            while (is_ident_continue(expr[i])) i++;
            continue;
        }
        int start = i;
        i++;
        while (is_ident_continue(expr[i])) i++;
        char *name = substr_copy(expr + start, (size_t)(i - start));
        int cursor = i;
        while (expr[cursor] == ' ' || expr[cursor] == '\t') cursor++;
        if (expr[cursor] == ':') {
            free(name);
            i = cursor + 1;
            continue;
        }
        if (expr[cursor] == '.') {
            int field_start = cursor + 1;
            while (expr[field_start] == ' ' || expr[field_start] == '\t') field_start++;
            int field_end = field_start;
            while (is_ident_continue(expr[field_end])) field_end++;
            char *field = substr_copy(expr + field_start, (size_t)(field_end - field_start));
            const char *base_type = direct_names_type(locals, name);
            if (direct_is_str_type(base_type)) {
                const char *after = skip_ws(expr + field_end);
                if (strcmp(field, "len") == 0) {
                    if (*after == '(') {
                        int close = find_matching_paren_c(expr, (int)(after - expr));
                        if (close < 0) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter expected `)` after Str.len",
                                "write `s.len()` or `s.len`.",
                                NULL);
                            free(field);
                            free(name);
                            return 1;
                        }
                        i = close + 1;
                    } else {
                        i = field_end;
                    }
                    free(field);
                    free(name);
                    continue;
                }
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter supports Str len and index expressions",
                    "write `s.len()` or `s[i]`.",
                    NULL);
                free(field);
                free(name);
                return 1;
            }
            if (direct_is_list_type(base_type)) {
                const char *after = skip_ws(expr + field_end);
                if (strcmp(field, "len") == 0) {
                    if (*after == '(') {
                        int close = find_matching_paren_c(expr, (int)(after - expr));
                        if (close < 0) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter expected `)` after List<Int>.len",
                                "write `xs.len()` or `xs.len`.",
                                NULL);
                            free(field);
                            free(name);
                            return 1;
                        }
                        i = close + 1;
                    } else {
                        i = field_end;
                    }
                    free(field);
                    free(name);
                    continue;
                }
                if (strcmp(field, "is_empty") == 0 && *after == '(') {
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after List.is_empty",
                            "write `xs.is_empty()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    i = close + 1;
                    free(field);
                    free(name);
                    continue;
                }
                if (strcmp(field, "last") == 0 && *after == '(') {
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after List.last",
                            "write `xs.last()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    i = close + 1;
                    free(field);
                    free(name);
                    continue;
                }
                if (strcmp(field, "pop") == 0 && *after == '(') {
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after List.pop",
                            "write `xs.pop()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    i = close + 1;
                    free(field);
                    free(name);
                    continue;
                }
                if (strcmp(field, "sum") == 0 && *after == '(') {
                    if (!direct_is_list_int_type(base_type)) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter supports sum only on List<Int>",
                            "read struct list fields explicitly before summing.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after List<Int>.sum",
                            "write `xs.sum()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    i = close + 1;
                    free(field);
                    free(name);
                    continue;
                }
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter supports List push, len, is_empty, last, pop, index, and List<Int> sum",
                    "write `xs.push(value)`, `xs.len()`, `xs.is_empty()`, `xs.last()`, `xs.pop()`, `xs[i]`, or `xs.sum()` for List<Int>.",
                    "xs.push(value)");
                free(field);
                free(name);
                return 1;
            }
            if (direct_is_map_type(base_type)) {
                const char *after = skip_ws(expr + field_end);
                if (strcmp(field, "len") == 0) {
                    if (*after != '(') {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter Map.len must be called",
                            "write `m.len()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after Map.len",
                            "write `m.len()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    char *args_text = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                    char *trimmed_args = trim_copy(args_text);
                    int has_args = trimmed_args[0] != '\0';
                    free(trimmed_args);
                    free(args_text);
                    if (has_args) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter Map.len takes no arguments",
                            "write `m.len()`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    i = close + 1;
                    free(field);
                    free(name);
                    continue;
                }
                if ((strcmp(field, "contains") == 0 || strcmp(field, "get") == 0 || strcmp(field, "get_opt") == 0) && *after == '(') {
                    int close = find_matching_paren_c(expr, (int)(after - expr));
                    if (close < 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter expected `)` after Map method",
                            "write `m.contains(key)`, `m.get(key, default)`, or `m.get_opt(key)`.",
                            NULL);
                        free(field);
                        free(name);
                        return 1;
                    }
                    char *inside = substr_copy(after + 1, (size_t)(close - (after - expr) - 1));
                    char *args[16] = {0};
                    int argc = split_top_level_commas_c(inside, args, 16);
                    free(inside);
                    int expected = strcmp(field, "get") == 0 ? 2 : 1;
                    if (argc != expected) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter Map method argument count does not match",
                            "write `m.contains(key)`, `m.get(key, default)`, or `m.get_opt(key)`.",
                            NULL);
                        for (int k = 0; k < 16; k++) free(args[k]);
                        free(field);
                        free(name);
                        return 1;
                    }
                    for (int a = 0; a < argc; a++) {
                        if (args[a] == NULL || strlen(skip_ws(args[a])) == 0) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter Map arguments cannot be empty",
                                "pass key/value expressions to the Map method.",
                                NULL);
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(field);
                            free(name);
                            return 1;
                        }
                        if (direct_check_expr_inner(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count, NULL)) {
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(field);
                            free(name);
                            return 1;
                        }
                        char *arg_type = direct_infer_expr_type(args[a], locals, fns, fn_count, structs, struct_count);
                        const char *expected_arg_type = a == 0 ? direct_map_key_type(base_type) : direct_map_value_type(base_type);
                        if (!direct_map_arg_type_compatible(expected_arg_type, arg_type)) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter Map arguments must match key/value types",
                                "use a key and value matching the Map's concrete key/value types.",
                                NULL);
                            free(arg_type);
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(field);
                            free(name);
                            return 1;
                        }
                        free(arg_type);
                    }
                    for (int k = 0; k < 16; k++) free(args[k]);
                    i = close + 1;
                    free(field);
                    free(name);
                    continue;
                }
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter supports Map insert/remove/clear statements and get/get_opt/contains/len expressions",
                    "write `m.insert(key, value)`, `m.remove(key)`, `m.clear()`, `m.get(key, default)`, `m.get_opt(key)`, `m.contains(key)`, or `m.len()` for Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, local Map<Str,Int>, local Map<Str,Bool>, or local Map<Str,Char>.",
                    NULL);
                free(field);
                free(name);
                return 1;
            }
            DirectStructInfo *st = base_type == NULL ? NULL : direct_find_struct(structs, struct_count, base_type);
            if (st == NULL || !direct_struct_has_field(st, field)) {
                int col = find_col(line, name);
                report_issue(path, line_no, col, line,
                    "direct native emitter found an unknown struct field access",
                    "read fields from a known direct-engine struct local.",
                    "return b.value");
                free(field);
                free(name);
                return 1;
            }
            free(field);
            free(name);
            i = field_end;
            continue;
        }
        int is_call = expr[cursor] == '(';
        if (is_call) {
            if (direct_is_parse_builtin_name(name)) {
                int close = find_matching_paren_c(expr, cursor);
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` to close the parse call",
                        "close the call arguments.",
                        NULL);
                    free(name);
                    return 1;
                }
                char *inside = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
                char *args[16] = {0};
                int argc = split_top_level_commas_c(inside, args, 16);
                free(inside);
                if (argc != 1 || args[0] == NULL || strlen(skip_ws(args[0])) == 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter parse helpers take one Str argument",
                        "write `parse_uint(s)` or `parse_int(s)`.",
                        NULL);
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(name);
                    return 1;
                }
                if (direct_check_expr_inner(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count, NULL)) {
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(name);
                    return 1;
                }
                char *arg_type = direct_infer_expr_type(args[0], locals, fns, fn_count, structs, struct_count);
                if (!direct_is_str_type(arg_type)) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter parse helpers require a Str argument",
                        "pass a string literal or `Str` local.",
                        NULL);
                    free(arg_type);
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(name);
                    return 1;
                }
                free(arg_type);
                for (int k = 0; k < 16; k++) free(args[k]);
                free(name);
                i = close + 1;
                continue;
            }
            DirectFnInfo *fn = direct_find_fn(fns, fn_count, name);
            if (fn != NULL) {
                int close = find_matching_paren_c(expr, cursor);
                if (close < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter expected `)` to close the function call",
                        "close the call arguments.",
                        NULL);
                    free(name);
                    return 1;
                }
                char *inside = substr_copy(expr + cursor + 1, (size_t)(close - cursor - 1));
                char *args[16] = {0};
                int argc = split_top_level_commas_c(inside, args, 16);
                free(inside);
                if (argc == 1 && args[0] != NULL && strlen(skip_ws(args[0])) == 0) {
                    free(args[0]);
                    args[0] = NULL;
                    argc = 0;
                }
                if (argc < 0 || argc != fn->param_count) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter function call argument count does not match",
                        "pass exactly the parameters declared by the direct helper.",
                        NULL);
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(name);
                    return 1;
                }
                for (int a = 0; a < argc; a++) {
                    int allow_arg_list = direct_is_list_type(fn->param_types[a]);
                    int allow_arg_map = direct_is_map_type(fn->param_types[a]);
                    int inline_list_arg = 0;
                    int returned_list_arg = 0;
                    if (allow_arg_list) {
                        int literal_state = direct_list_initializer_state(args[a]);
                        if (literal_state < 0) {
                            report_issue(path, line_no, find_col(line, "List"), line,
                                "direct native emitter supports up to 16 List literal items",
                                "bind a local list and call `push` for larger direct-engine lists.",
                                NULL);
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(name);
                            return 1;
                        }
                        inline_list_arg = literal_state == 1;
                        returned_list_arg = direct_expr_exact_list_return_call(args[a], fns, fn_count);
                    }
                    if (allow_arg_list && inline_list_arg) {
                        if (direct_check_expr_inner(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count, fn->param_types[a])) {
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(name);
                            return 1;
                        }
                        continue;
                    }
                    char *arg_type = direct_infer_expr_type(args[a], locals, fns, fn_count, structs, struct_count);
                    if (!direct_type_compatible(fn->param_types[a], arg_type)) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter function call argument type does not match",
                            "pass a value with the parameter type declared by the direct helper.",
                            "pass a matching argument");
                        free(arg_type);
                        for (int k = 0; k < 16; k++) free(args[k]);
                        free(name);
                        return 1;
                    }
                    free(arg_type);
                    if (allow_arg_map) {
                        if (!direct_expr_bare_map_local(locals, args[a], NULL)) {
                            report_issue(path, line_no, find_col(line, name), line,
                                "direct native emitter requires a local Map argument",
                                "bind the Map value to a local before passing it.",
                                "let m: Map<Int,Int> = {}");
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(name);
                            return 1;
                        }
                        continue;
                    }
                    if (allow_arg_list && returned_list_arg) {
                        if (direct_check_expr_inner(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count, fn->param_types[a])) {
                            for (int k = 0; k < 16; k++) free(args[k]);
                            free(name);
                            return 1;
                        }
                        continue;
                    }
                    if (allow_arg_list && !direct_expr_bare_list_local(locals, args[a], NULL)) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter requires a local List argument",
                            "bind the list value to a local before passing it.",
                            "let xs: List<Int> = []");
                        for (int k = 0; k < 16; k++) free(args[k]);
                        free(name);
                        return 1;
                    }
                    if (direct_check_expr_inner(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count, allow_arg_list ? fn->param_types[a] : NULL)) {
                        for (int k = 0; k < 16; k++) free(args[k]);
                        free(name);
                        return 1;
                    }
                }
                for (int k = 0; k < 16; k++) free(args[k]);
                free(name);
                i = close + 1;
                continue;
            }
        }
        const char *local_type = direct_names_type(locals, name);
        if (direct_is_list_type(local_type) && expr[cursor] != '[' && allowed_list_type == NULL) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter cannot use a List value as an Int expression",
                "read a list element, length, or sum value instead.",
                "xs.len()");
            free(name);
            return 1;
        }
        if (direct_is_map_type(local_type)) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter cannot use a Map value as an Int expression",
                "read a map value with `m.get(key, default)`, `m.get_opt(key)`, `m.contains(key)`, or `m.len()`.",
                "m.get(key, 0)");
            free(name);
            return 1;
        }
        int is_struct_literal = expr[cursor] == '{' && direct_find_struct(structs, struct_count, name) != NULL;
        int ok = direct_is_keyword(name) || direct_names_has(locals, name) ||
            (is_call && direct_find_fn(fns, fn_count, name) != NULL) ||
            is_struct_literal;
        if (!ok) {
            int col = find_col(line, name);
            report_issue(path, line_no, col, line,
                "direct native emitter found an unknown Int identifier",
                "define it with `let`, pass it as an Int parameter, or call a known helper function.",
                "return 40 + 2");
            free(name);
            return 1;
        }
        free(name);
    }
    return 0;
}

static int direct_check_expr(
    const char *path,
    int line_no,
    const char *line,
    const char *expr,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count
) {
    return direct_check_expr_inner(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count, NULL);
}

static char *direct_after_keyword_expr(const char *s, const char *keyword, char stop) {
    s = skip_ws(s);
    size_t n = strlen(keyword);
    if (strncmp(s, keyword, n) != 0) return NULL;
    const char *expr = skip_ws(s + n);
    const char *end = strrchr(expr, stop);
    if (end == NULL) return NULL;
    return trim_copy(substr_copy(expr, (size_t)(end - expr)));
}

static int direct_find_top_level_char(const char *text, char target) {
    int paren_depth = 0;
    int bracket_depth = 0;
    int brace_depth = 0;
    for (int i = 0; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == target && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0) return i;
        if (text[i] == '(') paren_depth++;
        else if (text[i] == ')') paren_depth--;
        else if (text[i] == '[') bracket_depth++;
        else if (text[i] == ']') bracket_depth--;
        else if (text[i] == '{') brace_depth++;
        else if (text[i] == '}') brace_depth--;
    }
    return -1;
}

static int direct_find_top_level_range_op(const char *text, int *inclusive) {
    int paren_depth = 0;
    int bracket_depth = 0;
    int brace_depth = 0;
    for (int i = 0; text[i] != '\0'; i++) {
        if (is_string_delim_c(text[i])) {
            int end = skip_string_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (text[i] == '\'') {
            int end = skip_char_literal_c(text, i);
            if (end < 0) return -1;
            i = end - 1;
            continue;
        }
        if (paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 &&
            text[i] == '.' && text[i + 1] == '.') {
            *inclusive = text[i + 2] == '=';
            return i;
        }
        if (text[i] == '(') paren_depth++;
        else if (text[i] == ')') paren_depth--;
        else if (text[i] == '[') bracket_depth++;
        else if (text[i] == ']') bracket_depth--;
        else if (text[i] == '{') brace_depth++;
        else if (text[i] == '}') brace_depth--;
    }
    return -1;
}

static int direct_parse_for_header(
    const char *s,
    char **name_out,
    char **lo_out,
    char **hi_out,
    int *inclusive_out
) {
    s = skip_ws(s);
    if (!starts_with(s, "for") || is_ident_continue(s[3])) return 0;
    const char *p = skip_ws(s + 3);
    if (!is_ident_start(*p)) return -1;
    const char *name_start = p;
    p++;
    while (is_ident_continue(*p)) p++;
    char *name = substr_copy(name_start, (size_t)(p - name_start));
    p = skip_ws(p);
    if (!(p[0] == 'i' && p[1] == 'n' && !is_ident_continue(p[2]))) {
        free(name);
        return -1;
    }
    p = skip_ws(p + 2);
    int open_rel = direct_find_top_level_char(p, '{');
    if (open_rel < 0 || *skip_ws(p + open_rel + 1) != '\0') {
        free(name);
        return -1;
    }
    char *range = substr_copy(p, (size_t)open_rel);
    int inclusive = 0;
    int op = direct_find_top_level_range_op(range, &inclusive);
    if (op < 0) {
        free(range);
        free(name);
        return -1;
    }
    int op_len = inclusive ? 3 : 2;
    char *lo_raw = substr_copy(range, (size_t)op);
    char *hi_raw = substr_copy(range + op + op_len, strlen(range + op + op_len));
    char *lo = trim_copy(lo_raw);
    char *hi = trim_copy(hi_raw);
    free(lo_raw);
    free(hi_raw);
    free(range);
    if (lo[0] == '\0' || hi[0] == '\0') {
        free(lo);
        free(hi);
        free(name);
        return -1;
    }
    *name_out = name;
    *lo_out = lo;
    *hi_out = hi;
    *inclusive_out = inclusive;
    return 1;
}

static int direct_parse_for_each_header(
    const char *s,
    char **name_out,
    char **iterable_out
) {
    s = skip_ws(s);
    if (!starts_with(s, "for") || is_ident_continue(s[3])) return 0;
    const char *p = skip_ws(s + 3);
    if (!is_ident_start(*p)) return -1;
    const char *name_start = p;
    p++;
    while (is_ident_continue(*p)) p++;
    char *name = substr_copy(name_start, (size_t)(p - name_start));
    p = skip_ws(p);
    if (!(p[0] == 'i' && p[1] == 'n' && !is_ident_continue(p[2]))) {
        free(name);
        return -1;
    }
    p = skip_ws(p + 2);
    int open_rel = direct_find_top_level_char(p, '{');
    if (open_rel < 0 || *skip_ws(p + open_rel + 1) != '\0') {
        free(name);
        return -1;
    }
    char *iter_raw = substr_copy(p, (size_t)open_rel);
    char *iterable = trim_copy(iter_raw);
    free(iter_raw);
    if (iterable[0] == '\0') {
        free(iterable);
        free(name);
        return -1;
    }
    int inclusive = 0;
    if (direct_find_top_level_range_op(iterable, &inclusive) >= 0) {
        free(iterable);
        free(name);
        return -1;
    }
    *name_out = name;
    *iterable_out = iterable;
    return 1;
}

static int direct_parse_inline_block(const char *s, const char *keyword, char **expr_out, char **body_out) {
    s = skip_ws(s);
    size_t n = strlen(keyword);
    if (strncmp(s, keyword, n) != 0) return 0;
    char next = s[n];
    if (!(next == ' ' || next == '\t')) return 0;
    const char *expr = skip_ws(s + n);
    int open_rel = direct_find_top_level_char(expr, '{');
    if (open_rel < 0) return 0;
    int open_abs = (int)(expr - s) + open_rel;
    int close_abs = find_matching_brace_c(s, open_abs);
    if (close_abs < 0) return 0;
    if (*skip_ws(s + close_abs + 1) != '\0') return 0;
    char *expr_raw = substr_copy(expr, (size_t)open_rel);
    char *body_raw = substr_copy(s + open_abs + 1, (size_t)(close_abs - open_abs - 1));
    *expr_out = trim_copy(expr_raw);
    *body_out = trim_copy(body_raw);
    free(expr_raw);
    free(body_raw);
    return 1;
}

static int direct_parse_field_target(const char *lhs, char **base, char **field) {
    const char *s = skip_ws(lhs);
    if (!is_ident_start(*s)) return 0;
    const char *base_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *base_end = s;
    const char *dot = skip_ws(s);
    if (*dot != '.') return 0;
    s = skip_ws(dot + 1);
    if (!is_ident_start(*s)) return 0;
    const char *field_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    if (*skip_ws(s) != '\0') return 0;
    *base = substr_copy(base_start, (size_t)(base_end - base_start));
    *field = substr_copy(field_start, (size_t)(s - field_start));
    return 1;
}

static int direct_parse_list_field_target(const char *lhs, char **base, char **field) {
    const char *s = skip_ws(lhs);
    if (!is_ident_start(*s)) return 0;
    const char *base_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *base_end = s;
    const char *open = skip_ws(s);
    if (*open != '[') return 0;
    int close = find_matching_bracket_c(lhs, (int)(open - lhs));
    if (close < 0) return 0;
    s = skip_ws(lhs + close + 1);
    if (*s != '.') return 0;
    s = skip_ws(s + 1);
    if (!is_ident_start(*s)) return 0;
    const char *field_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    if (*skip_ws(s) != '\0') return 0;
    *base = substr_copy(base_start, (size_t)(base_end - base_start));
    *field = substr_copy(field_start, (size_t)(s - field_start));
    return 1;
}

static int direct_parse_list_index_target(const char *lhs, char **base) {
    const char *s = skip_ws(lhs);
    if (!is_ident_start(*s)) return 0;
    const char *base_start = s;
    s++;
    while (is_ident_continue(*s)) s++;
    const char *base_end = s;
    const char *open = skip_ws(s);
    if (*open != '[') return 0;
    int close = find_matching_bracket_c(lhs, (int)(open - lhs));
    if (close < 0 || *skip_ws(lhs + close + 1) != '\0') return 0;
    *base = substr_copy(base_start, (size_t)(base_end - base_start));
    return 1;
}

static int direct_check_assignment_target(
    const char *path,
    int line_no,
    const char *line,
    const char *lhs,
    DirectNameSet *locals,
    DirectStructInfo *structs,
    int struct_count
) {
    if (direct_names_has(locals, lhs)) return 0;
    char *base = NULL;
    char *field = NULL;
    if (direct_parse_field_target(lhs, &base, &field)) {
        const char *base_type = direct_names_type(locals, base);
        DirectStructInfo *st = base_type == NULL ? NULL : direct_find_struct(structs, struct_count, base_type);
        if (st != NULL && direct_struct_has_field(st, field)) {
            free(base);
            free(field);
            return 0;
        }
        report_issue(path, line_no, find_col(line, base), line,
            "direct native emitter assignment target is not a known struct field",
            "assign to a field declared on a direct-engine struct local.",
            NULL);
        free(base);
        free(field);
        return 1;
    }
    if (direct_parse_list_index_target(lhs, &base)) {
        const char *base_type = direct_names_type(locals, base);
        char *elem_type = direct_list_element_type(base_type);
        int ok = elem_type != NULL && (strcmp(elem_type, "Int") == 0 || direct_find_struct(structs, struct_count, elem_type) != NULL);
        if (ok) {
            free(elem_type);
            free(base);
            return 0;
        }
        report_issue(path, line_no, find_col(line, base), line,
            "direct native emitter assignment target is not a known List element",
            "assign to an element of a known direct-engine list.",
            "xs[0] = value");
        free(elem_type);
        free(base);
        return 1;
    }
    if (direct_parse_list_field_target(lhs, &base, &field)) {
        const char *base_type = direct_names_type(locals, base);
        char *elem_type = direct_list_element_type(base_type);
        DirectStructInfo *st = elem_type == NULL ? NULL : direct_find_struct(structs, struct_count, elem_type);
        if (st != NULL && direct_struct_has_field(st, field)) {
            free(elem_type);
            free(base);
            free(field);
            return 0;
        }
        report_issue(path, line_no, find_col(line, base), line,
            "direct native emitter assignment target is not a known List<Struct> field",
            "assign to a field declared on the list element struct.",
            "xs[0].value = 42");
        free(elem_type);
        free(base);
        free(field);
        return 1;
    }
    report_issue(path, line_no, find_col(line, lhs), line,
        "direct native emitter assignment target is not a known local",
        "define the target with `let` before assigning to it.",
        NULL);
    return 1;
}

static const char *direct_assignment_target_type(
    const char *lhs,
    DirectNameSet *locals,
    DirectStructInfo *structs,
    int struct_count
) {
    const char *local_type = direct_names_type(locals, lhs);
    if (local_type != NULL) return local_type;
    char *base = NULL;
    char *field = NULL;
    if (direct_parse_field_target(lhs, &base, &field)) {
        const char *base_type = direct_names_type(locals, base);
        DirectStructInfo *st = base_type == NULL ? NULL : direct_find_struct(structs, struct_count, base_type);
        int ok = st != NULL && direct_struct_has_field(st, field);
        free(base);
        free(field);
        return ok ? "Int" : NULL;
    }
    if (direct_parse_list_index_target(lhs, &base)) {
        const char *base_type = direct_names_type(locals, base);
        static char bufs[4][128];
        static int slot = 0;
        char *elem_type = direct_list_element_type(base_type);
        if (elem_type == NULL) {
            free(base);
            return NULL;
        }
        slot = (slot + 1) % 4;
        snprintf(bufs[slot], sizeof(bufs[slot]), "%s", elem_type);
        free(elem_type);
        free(base);
        return bufs[slot];
    }
    if (direct_parse_list_field_target(lhs, &base, &field)) {
        const char *base_type = direct_names_type(locals, base);
        char *elem_type = direct_list_element_type(base_type);
        DirectStructInfo *st = elem_type == NULL ? NULL : direct_find_struct(structs, struct_count, elem_type);
        int ok = st != NULL && direct_struct_has_field(st, field);
        free(elem_type);
        free(base);
        free(field);
        return ok ? "Int" : NULL;
    }
    return NULL;
}

static int direct_parse_method_statement(const char *s, char **base, char **method, char **args) {
    const char *p = skip_ws(s);
    if (!is_ident_start(*p)) return 0;
    const char *base_start = p;
    p++;
    while (is_ident_continue(*p)) p++;
    const char *base_end = p;
    p = skip_ws(p);
    if (*p != '.') return 0;
    p = skip_ws(p + 1);
    if (!is_ident_start(*p)) return 0;
    const char *method_start = p;
    p++;
    while (is_ident_continue(*p)) p++;
    const char *method_end = p;
    p = skip_ws(p);
    if (*p != '(') return 0;
    int close = find_matching_paren_c(s, (int)(p - s));
    if (close < 0) return 0;
    const char *after = skip_ws(s + close + 1);
    if (*after == ';') after = skip_ws(after + 1);
    if (*after != '\0') return 0;
    *base = substr_copy(base_start, (size_t)(base_end - base_start));
    *method = substr_copy(method_start, (size_t)(method_end - method_start));
    *args = substr_copy(p + 1, (size_t)(close - (p - s) - 1));
    return 1;
}

static int direct_parse_option_some_pattern(const char *pattern, char **binder_out) {
    char *trimmed = trim_copy(pattern);
    const char *s = skip_ws(trimmed);
    if (!starts_with(s, "Some") || is_ident_continue(s[4])) {
        free(trimmed);
        return 0;
    }
    const char *p = skip_ws(s + 4);
    if (*p != '(') {
        free(trimmed);
        return -1;
    }
    int close = find_matching_paren_c(trimmed, (int)(p - trimmed));
    if (close < 0 || *skip_ws(trimmed + close + 1) != '\0') {
        free(trimmed);
        return -1;
    }
    char *inside_raw = substr_copy(p + 1, (size_t)(close - (p - trimmed) - 1));
    char *inside = trim_copy(inside_raw);
    free(inside_raw);
    if (!direct_is_plain_ident(inside)) {
        free(inside);
        free(trimmed);
        return -1;
    }
    *binder_out = inside;
    free(trimmed);
    return 1;
}

static int direct_is_option_none_pattern(const char *pattern) {
    char *trimmed = trim_copy(pattern);
    const char *s = skip_ws(trimmed);
    int ok = starts_with(s, "None") && !is_ident_continue(s[4]) && *skip_ws(s + 4) == '\0';
    free(trimmed);
    return ok;
}

static int direct_lower_option_match_let(
    const char *path,
    int line_no,
    const char *line,
    const char *name,
    const char *match_expr,
    const char *arms_text,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count,
    StrBuf *out
) {
    if (strstr(match_expr, "get_opt") == NULL) {
        report_issue(path, line_no, find_col(line, "match"), line,
            "direct native emitter supports Option match only for Map.get_opt results",
            "write `let value = match m.get_opt(key) { Some(v) => v, None => fallback }`.",
            NULL);
        return 1;
    }

    char *arm_parts[16] = {0};
    char *patterns[16] = {0};
    char *exprs[16] = {0};
    int arm_count = split_top_level_commas_c(arms_text, arm_parts, 16);
    char *some_binder = NULL;
    char *some_expr = NULL;
    char *none_expr = NULL;
    int failed = 0;

    if (arm_count != 2) {
        failed = 1;
    }
    for (int a = 0; a < arm_count && a < 16 && failed == 0; a++) {
        if (!parse_arm(arm_parts[a], &patterns[a], &exprs[a])) {
            failed = 1;
            break;
        }
        char *binder = NULL;
        int some = direct_parse_option_some_pattern(patterns[a], &binder);
        if (some == 1) {
            if (some_binder != NULL || some_expr != NULL) {
                free(binder);
                failed = 1;
                break;
            }
            some_binder = binder;
            some_expr = exprs[a];
            exprs[a] = NULL;
        } else if (some < 0) {
            failed = 1;
            break;
        } else if (direct_is_option_none_pattern(patterns[a])) {
            if (none_expr != NULL) {
                failed = 1;
                break;
            }
            none_expr = exprs[a];
            exprs[a] = NULL;
        } else {
            failed = 1;
            break;
        }
    }
    if (some_binder == NULL || some_expr == NULL || none_expr == NULL) failed = 1;
    if (failed) {
        report_issue(path, line_no, find_col(line, "match"), line,
            "direct native emitter supports Option match with one Some arm and one None arm",
            "write `let value = match m.get_opt(key) { Some(v) => v, None => fallback }`.",
            NULL);
        for (int k = 0; k < 16; k++) {
            free(arm_parts[k]);
            free(patterns[k]);
            free(exprs[k]);
        }
        free(some_binder);
        free(some_expr);
        free(none_expr);
        return 1;
    }

    if (direct_check_expr(path, line_no, line, match_expr, locals, fns, fn_count, structs, struct_count)) {
        for (int k = 0; k < 16; k++) {
            free(arm_parts[k]);
            free(patterns[k]);
            free(exprs[k]);
        }
        free(some_binder);
        free(some_expr);
        free(none_expr);
        return 1;
    }
    char *match_rewritten = direct_rewrite_expr(path, line_no, line, match_expr, locals, fns, fn_count, structs, struct_count);
    if (match_rewritten == NULL) {
        for (int k = 0; k < 16; k++) {
            free(arm_parts[k]);
            free(patterns[k]);
            free(exprs[k]);
        }
        free(some_binder);
        free(some_expr);
        free(none_expr);
        return 1;
    }
    char *match_c = direct_translate_expr(match_rewritten);
    free(match_rewritten);

    char match_tmp[80];
    char payload_tmp[80];
    snprintf(match_tmp, sizeof(match_tmp), "__vais_match_%d", locals->temp_count++);
    snprintf(payload_tmp, sizeof(payload_tmp), "__vais_match_payload_%d", locals->temp_count++);

    char *some_bound = replace_word_all(some_expr, some_binder, payload_tmp);
    direct_names_add_typed(locals, payload_tmp, "Int");
    int some_bad = direct_check_expr(path, line_no, line, some_bound, locals, fns, fn_count, structs, struct_count);
    char *some_type = some_bad ? NULL : direct_infer_expr_type(some_bound, locals, fns, fn_count, structs, struct_count);
    if (!some_bad && !direct_type_compatible("Int", some_type)) {
        report_issue(path, line_no, find_col(line, "Some"), line,
            "direct native emitter Option Some arm must produce an Int value",
            "return an Int-compatible expression from the Some arm.",
            NULL);
        some_bad = 1;
    }
    char *some_rewritten = some_bad ? NULL : direct_rewrite_expr(path, line_no, line, some_bound, locals, fns, fn_count, structs, struct_count);
    direct_names_remove(locals, payload_tmp);
    free(some_type);
    free(some_bound);
    if (some_bad || some_rewritten == NULL) {
        free(match_c);
        for (int k = 0; k < 16; k++) {
            free(arm_parts[k]);
            free(patterns[k]);
            free(exprs[k]);
        }
        free(some_binder);
        free(some_expr);
        free(none_expr);
        free(some_rewritten);
        return 1;
    }
    char *some_c = direct_translate_expr(some_rewritten);
    free(some_rewritten);

    int none_bad = direct_check_expr(path, line_no, line, none_expr, locals, fns, fn_count, structs, struct_count);
    char *none_type = none_bad ? NULL : direct_infer_expr_type(none_expr, locals, fns, fn_count, structs, struct_count);
    if (!none_bad && !direct_type_compatible("Int", none_type)) {
        report_issue(path, line_no, find_col(line, "None"), line,
            "direct native emitter Option None arm must produce an Int value",
            "return an Int-compatible expression from the None arm.",
            NULL);
        none_bad = 1;
    }
    char *none_rewritten = none_bad ? NULL : direct_rewrite_expr(path, line_no, line, none_expr, locals, fns, fn_count, structs, struct_count);
    free(none_type);
    if (none_bad || none_rewritten == NULL) {
        free(match_c);
        free(some_c);
        for (int k = 0; k < 16; k++) {
            free(arm_parts[k]);
            free(patterns[k]);
            free(exprs[k]);
        }
        free(some_binder);
        free(some_expr);
        free(none_expr);
        free(none_rewritten);
        return 1;
    }
    char *none_c = direct_translate_expr(none_rewritten);
    free(none_rewritten);

    sb_append(out, "long ");
    sb_append(out, name);
    sb_append(out, " = 0;\n");
    sb_append(out, "{\nlong ");
    sb_append(out, match_tmp);
    sb_append(out, " = ");
    sb_append(out, match_c);
    sb_append(out, ";\nif (");
    sb_append(out, match_tmp);
    sb_append(out, " % 2 == 0) {\nlong ");
    sb_append(out, payload_tmp);
    sb_append(out, " = (");
    sb_append(out, match_tmp);
    sb_append(out, " / 2) % 1000000;\n");
    sb_append(out, name);
    sb_append(out, " = ");
    sb_append(out, some_c);
    sb_append(out, ";\n} else {\n");
    sb_append(out, name);
    sb_append(out, " = ");
    sb_append(out, none_c);
    sb_append(out, ";\n}\n}\n");
    direct_names_add_typed(locals, name, "Int");

    free(match_c);
    free(some_c);
    free(none_c);
    for (int k = 0; k < 16; k++) {
        free(arm_parts[k]);
        free(patterns[k]);
        free(exprs[k]);
    }
    free(some_binder);
    free(some_expr);
    free(none_expr);
    return 0;
}

static int direct_parse_named_call_statement(const char *s, const char *name, char **args_out) {
    size_t name_len = strlen(name);
    if (strncmp(s, name, name_len) != 0 || is_ident_continue(s[name_len])) return 0;
    const char *p = skip_ws(s + name_len);
    if (*p != '(') return -1;
    int close = find_matching_paren_c(s, (int)(p - s));
    if (close < 0) return -1;
    const char *after = skip_ws(s + close + 1);
    if (*after == ';') after = skip_ws(after + 1);
    if (*after != '\0') return -1;
    *args_out = substr_copy(p + 1, (size_t)(close - (p - s) - 1));
    return 1;
}

static void sb_append_printf_literal_byte(StrBuf *out, char ch) {
    if (ch == '%') {
        sb_append(out, "%%");
    } else {
        sb_append_c_escaped_byte(out, ch);
    }
}

static int direct_append_print_interpolation(
    const char *path,
    int line_no,
    const char *line,
    const char *arg,
    DirectNameSet *locals,
    StrBuf *out,
    int *handled
) {
    *handled = 0;
    char *trimmed = trim_copy(arg);
    const char *s = skip_ws(trimmed);
    if (!direct_expr_is_string_literal(s)) {
        free(trimmed);
        return 0;
    }
    int end = skip_string_literal_c(s, 0);
    int has_interpolation = 0;
    for (int i = 1; i < end - 1; i++) {
        if (s[0] == '"' && s[i] == '\\' && i + 1 < end - 1) {
            i++;
            continue;
        }
        if (s[i] == '{') {
            has_interpolation = 1;
            break;
        }
    }
    if (!has_interpolation) {
        free(trimmed);
        return 0;
    }

    StrBuf fmt;
    StrBuf args;
    sb_init(&fmt);
    sb_init(&args);
    sb_append(&fmt, "\"");
    for (int i = 1; i < end - 1;) {
        if (s[0] == '"' && s[i] == '\\' && i + 1 < end - 1) {
            i++;
            sb_append_printf_literal_byte(&fmt, s[i]);
            i++;
            continue;
        }
        if (s[i] == '{') {
            int close = i + 1;
            while (close < end - 1 && s[close] != '}') close++;
            if (close >= end - 1) {
                report_issue(path, line_no, find_col(line, "{"), line,
                    "direct native emitter print interpolation is missing `}`",
                    "write placeholders as `{name}` inside the string literal.",
                    NULL);
                free(fmt.data);
                free(args.data);
                free(trimmed);
                return 1;
            }
            char *raw = substr_copy(s + i + 1, (size_t)(close - i - 1));
            char *name = trim_copy(raw);
            free(raw);
            if (!direct_is_plain_ident(name)) {
                report_issue(path, line_no, find_col(line, "{"), line,
                    "direct native emitter print interpolation supports simple identifiers",
                    "write placeholders as `{name}` for a local Int, Bool, Char, or Str value.",
                    NULL);
                free(name);
                free(fmt.data);
                free(args.data);
                free(trimmed);
                return 1;
            }
            const char *ty = direct_names_type(locals, name);
            if (ty == NULL) {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter print interpolation name is not a known local",
                    "define the value with `let` or pass it as a parameter before printing it.",
                    NULL);
                free(name);
                free(fmt.data);
                free(args.data);
                free(trimmed);
                return 1;
            }
            if (direct_is_str_type(ty)) {
                sb_append(&fmt, "%s");
            } else if (direct_is_intlike_scalar_type(ty)) {
                sb_append(&fmt, "%ld");
            } else {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter print interpolation supports scalar and Str values",
                    "interpolate an Int, Bool, Char, or Str value.",
                    NULL);
                free(name);
                free(fmt.data);
                free(args.data);
                free(trimmed);
                return 1;
            }
            sb_append(&args, ", ");
            sb_append(&args, name);
            free(name);
            i = close + 1;
            continue;
        }
        if (s[i] == '}') {
            report_issue(path, line_no, find_col(line, "}"), line,
                "direct native emitter print interpolation found an unmatched `}`",
                "write placeholders as `{name}` inside the string literal.",
                NULL);
            free(fmt.data);
            free(args.data);
            free(trimmed);
            return 1;
        }
        sb_append_printf_literal_byte(&fmt, s[i]);
        i++;
    }
    sb_append(&fmt, "\\n\"");
    sb_append(out, "printf(");
    sb_append(out, fmt.data);
    sb_append(out, args.data);
    sb_append(out, ");\n");
    free(fmt.data);
    free(args.data);
    free(trimmed);
    *handled = 1;
    return 0;
}

static int direct_lower_print_statement(
    const char *path,
    int line_no,
    const char *line,
    const char *args_text,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count,
    StrBuf *out
) {
    char *args[16] = {0};
    int argc = split_top_level_commas_c(args_text, args, 16);
    if (argc != 1 || args[0] == NULL || strlen(skip_ws(args[0])) == 0) {
        report_issue(path, line_no, find_col(line, "print"), line,
            "direct native emitter print expects one Str argument",
            "write `print(\"text\")` or `print(\"value={name}\")`.",
            NULL);
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    int handled = 0;
    if (direct_append_print_interpolation(path, line_no, line, args[0], locals, out, &handled)) {
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    if (handled) {
        for (int k = 0; k < 16; k++) free(args[k]);
        return 0;
    }
    if (direct_check_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count)) {
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    char *arg_type = direct_infer_expr_type(args[0], locals, fns, fn_count, structs, struct_count);
    if (!direct_is_str_type(arg_type)) {
        report_issue(path, line_no, find_col(line, "print"), line,
            "direct native emitter print expects a Str argument",
            "print a string literal or Str value; use interpolation for scalar values.",
            "print(\"value={x}\")");
        free(arg_type);
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    free(arg_type);
    StrBuf prelude;
    sb_init(&prelude);
    direct_current_prelude = &prelude;
    char *rewritten = direct_rewrite_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count);
    direct_current_prelude = NULL;
    if (rewritten == NULL) {
        free(prelude.data);
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    char *c_arg = direct_translate_expr(rewritten);
    sb_append(out, prelude.data);
    sb_append(out, "puts(");
    sb_append(out, c_arg);
    sb_append(out, ");\n");
    free(prelude.data);
    free(c_arg);
    free(rewritten);
    for (int k = 0; k < 16; k++) free(args[k]);
    return 0;
}

static int direct_lower_putchar_statement(
    const char *path,
    int line_no,
    const char *line,
    const char *args_text,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count,
    StrBuf *out
) {
    char *args[16] = {0};
    int argc = split_top_level_commas_c(args_text, args, 16);
    if (argc != 1 || args[0] == NULL || strlen(skip_ws(args[0])) == 0) {
        report_issue(path, line_no, find_col(line, "putchar"), line,
            "direct native emitter putchar expects one Int-compatible argument",
            "write `putchar(33)` or pass an Int, Bool, or Char value.",
            NULL);
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    if (direct_check_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count)) {
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    char *arg_type = direct_infer_expr_type(args[0], locals, fns, fn_count, structs, struct_count);
    if (!direct_type_compatible("Int", arg_type)) {
        report_issue(path, line_no, find_col(line, "putchar"), line,
            "direct native emitter putchar expects an Int-compatible argument",
            "pass an Int, Bool, or Char value.",
            "putchar(33)");
        free(arg_type);
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    free(arg_type);
    StrBuf prelude;
    sb_init(&prelude);
    direct_current_prelude = &prelude;
    char *rewritten = direct_rewrite_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count);
    direct_current_prelude = NULL;
    if (rewritten == NULL) {
        free(prelude.data);
        for (int k = 0; k < 16; k++) free(args[k]);
        return 1;
    }
    char *c_arg = direct_translate_expr(rewritten);
    sb_append(out, prelude.data);
    sb_append(out, "putchar((int)(");
    sb_append(out, c_arg);
    sb_append(out, "));\n");
    free(prelude.data);
    free(c_arg);
    free(rewritten);
    for (int k = 0; k < 16; k++) free(args[k]);
    return 0;
}

static int direct_lower_line(
    const char *path,
    int line_no,
    const char *line,
    DirectNameSet *locals,
    DirectFnInfo *fns,
    int fn_count,
    DirectStructInfo *structs,
    int struct_count,
    StrBuf *out
) {
    char *stripped = strip_line_comment(line, strlen(line));
    const char *s = skip_ws(stripped);
    if (*s == '\0') {
        free(stripped);
        return 0;
    }

    DirectFnInfo header;
    int parsed_header = parse_direct_fn_header(stripped, &header);
    if (parsed_header == 1) {
        direct_names_free(locals);
        locals->current_return_type = strdup(header.return_type);
        for (int p = 0; p < header.param_count; p++) {
            direct_names_add_typed_ref(locals, header.params[p], header.param_types[p], direct_is_list_type(header.param_types[p]) || direct_is_map_type(header.param_types[p]));
        }
        sb_append(out, direct_c_type(header.return_type));
        sb_append(out, " ");
        sb_append(out, header.name);
        sb_append(out, "(");
        for (int p = 0; p < header.param_count; p++) {
            if (p > 0) sb_append(out, ", ");
            sb_append(out, direct_c_param_type(header.param_types[p]));
            sb_append(out, " ");
            sb_append(out, header.params[p]);
        }
        sb_append(out, ") {\n");
        direct_fns_free(&header, 1);
        free(stripped);
        return 0;
    }
    if (parsed_header < 0 || starts_with(s, "fn ")) {
        report_issue(path, line_no, find_col(line, "fn"), line,
            "direct native emitter supports scalar/List/Struct function headers, verified concrete Map returns, and verified Map parameters",
            "write functions with `Int`, `Bool`, `Char`, `Str`, `List<Int>`, `List<Struct>`, `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, or declared struct return types.",
            NULL);
        free(stripped);
        return 1;
    }

    if (strcmp(s, "}") == 0) {
        sb_append(out, "}\n");
        free(stripped);
        return 0;
    }
    if (starts_with(s, "} else if ")) {
        char *expr = direct_after_keyword_expr(s + 2, "else if", '{');
        if (expr == NULL || direct_check_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count)) {
            free(expr);
            free(stripped);
            return 1;
        }
        char *rewritten = direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        if (rewritten == NULL) {
            free(expr);
            free(stripped);
            return 1;
        }
        char *c_expr = direct_translate_expr(rewritten);
        sb_append(out, "} else if (");
        sb_append(out, c_expr);
        sb_append(out, ") {\n");
        free(c_expr);
        free(rewritten);
        free(expr);
        free(stripped);
        return 0;
    }
    if (starts_with(s, "} else")) {
        sb_append(out, "} else {\n");
        free(stripped);
        return 0;
    }
    char *inline_expr = NULL;
    char *inline_body = NULL;
    if (direct_parse_inline_block(s, "if", &inline_expr, &inline_body)) {
        if (inline_expr == NULL || inline_body == NULL ||
            direct_check_expr(path, line_no, line, inline_expr, locals, fns, fn_count, structs, struct_count)) {
            free(inline_expr);
            free(inline_body);
            free(stripped);
            return 1;
        }
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten = direct_rewrite_expr(path, line_no, line, inline_expr, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten == NULL) {
            free(prelude.data);
            free(inline_expr);
            free(inline_body);
            free(stripped);
            return 1;
        }
        char *c_expr = direct_translate_expr(rewritten);
        sb_append(out, prelude.data);
        sb_append(out, "if (");
        sb_append(out, c_expr);
        sb_append(out, ") {\n");
        int body_rc = direct_lower_line(path, line_no, inline_body, locals, fns, fn_count, structs, struct_count, out);
        sb_append(out, "}\n");
        free(prelude.data);
        free(c_expr);
        free(rewritten);
        free(inline_expr);
        free(inline_body);
        free(stripped);
        return body_rc;
    }
    if (starts_with(s, "if ")) {
        char *expr = direct_after_keyword_expr(s, "if", '{');
        if (expr == NULL || direct_check_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count)) {
            free(expr);
            free(stripped);
            return 1;
        }
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten = direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten == NULL) {
            free(prelude.data);
            free(expr);
            free(stripped);
            return 1;
        }
        char *c_expr = direct_translate_expr(rewritten);
        sb_append(out, prelude.data);
        sb_append(out, "if (");
        sb_append(out, c_expr);
        sb_append(out, ") {\n");
        free(prelude.data);
        free(c_expr);
        free(rewritten);
        free(expr);
        free(stripped);
        return 0;
    }
    if (starts_with(s, "while ")) {
        char *expr = direct_after_keyword_expr(s, "while", '{');
        if (expr == NULL || direct_check_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count)) {
            free(expr);
            free(stripped);
            return 1;
        }
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten = direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten == NULL) {
            free(prelude.data);
            free(expr);
            free(stripped);
            return 1;
        }
        char *c_expr = direct_translate_expr(rewritten);
        if (prelude.len > 0) {
            sb_append(out, "while (1) {\n");
            sb_append(out, prelude.data);
            sb_append(out, "if (!(");
            sb_append(out, c_expr);
            sb_append(out, ")) break;\n");
        } else {
            sb_append(out, "while (");
            sb_append(out, c_expr);
            sb_append(out, ") {\n");
        }
        free(prelude.data);
        free(c_expr);
        free(rewritten);
        free(expr);
        free(stripped);
        return 0;
    }
    char *for_name = NULL;
    char *for_lo = NULL;
    char *for_hi = NULL;
    int for_inclusive = 0;
    int parsed_for = direct_parse_for_header(s, &for_name, &for_lo, &for_hi, &for_inclusive);
    if (parsed_for == 1) {
        const char *existing_type = direct_names_type(locals, for_name);
        if (existing_type != NULL && !direct_is_intlike_scalar_type(existing_type)) {
            report_issue(path, line_no, find_col(line, for_name), line,
                "direct native emitter for-loop variable must be an Int-compatible scalar",
                "use a fresh loop variable name or an `Int` local.",
                "for i in 0..n {");
            free(for_name);
            free(for_lo);
            free(for_hi);
            free(stripped);
            return 1;
        }
        if (direct_check_expr(path, line_no, line, for_lo, locals, fns, fn_count, structs, struct_count)) {
            free(for_name);
            free(for_lo);
            free(for_hi);
            free(stripped);
            return 1;
        }
        char *lo_type = direct_infer_expr_type(for_lo, locals, fns, fn_count, structs, struct_count);
        if (!direct_type_compatible("Int", lo_type)) {
            report_issue(path, line_no, find_col(line, "for"), line,
                "direct native emitter for-loop lower bound must be Int-compatible",
                "use an `Int`, `Bool`, or `Char` expression for the lower bound.",
                "for i in 0..n {");
            free(lo_type);
            free(for_name);
            free(for_lo);
            free(for_hi);
            free(stripped);
            return 1;
        }
        free(lo_type);
        int declare_loop_var = existing_type == NULL;
        if (declare_loop_var) direct_names_add_typed(locals, for_name, "Int");
        if (direct_check_expr(path, line_no, line, for_hi, locals, fns, fn_count, structs, struct_count)) {
            free(for_name);
            free(for_lo);
            free(for_hi);
            free(stripped);
            return 1;
        }
        char *hi_type = direct_infer_expr_type(for_hi, locals, fns, fn_count, structs, struct_count);
        if (!direct_type_compatible("Int", hi_type)) {
            report_issue(path, line_no, find_col(line, "for"), line,
                "direct native emitter for-loop upper bound must be Int-compatible",
                "use an `Int`, `Bool`, or `Char` expression for the upper bound.",
                "for i in 0..n {");
            free(hi_type);
            free(for_name);
            free(for_lo);
            free(for_hi);
            free(stripped);
            return 1;
        }
        free(hi_type);
        StrBuf start_prelude;
        StrBuf end_prelude;
        sb_init(&start_prelude);
        sb_init(&end_prelude);
        direct_current_prelude = &start_prelude;
        char *rewritten_lo = direct_rewrite_expr(path, line_no, line, for_lo, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = &end_prelude;
        char *rewritten_hi = rewritten_lo == NULL ? NULL : direct_rewrite_expr(path, line_no, line, for_hi, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten_lo == NULL || rewritten_hi == NULL) {
            free(start_prelude.data);
            free(end_prelude.data);
            free(rewritten_lo);
            free(rewritten_hi);
            free(for_name);
            free(for_lo);
            free(for_hi);
            free(stripped);
            return 1;
        }
        char *c_lo = direct_translate_expr(rewritten_lo);
        char *c_hi = direct_translate_expr(rewritten_hi);
        if (declare_loop_var) {
            sb_append(out, "long ");
            sb_append(out, for_name);
            sb_append(out, ";\n");
        }
        sb_append(out, start_prelude.data);
        sb_append(out, "for (");
        sb_append(out, for_name);
        sb_append(out, " = ");
        sb_append(out, c_lo);
        sb_append(out, "; ; ");
        sb_append(out, for_name);
        sb_append(out, "++) {\n");
        sb_append(out, end_prelude.data);
        sb_append(out, "if (!(");
        sb_append(out, for_name);
        sb_append(out, for_inclusive ? " <= " : " < ");
        sb_append(out, c_hi);
        sb_append(out, ")) break;\n");
        free(start_prelude.data);
        free(end_prelude.data);
        free(c_lo);
        free(c_hi);
        free(rewritten_lo);
        free(rewritten_hi);
        free(for_name);
        free(for_lo);
        free(for_hi);
        free(stripped);
        return 0;
    }
    char *each_name = NULL;
    char *each_iterable = NULL;
    int parsed_each = direct_parse_for_each_header(s, &each_name, &each_iterable);
    if (parsed_each == 1) {
        const char *existing_type = direct_names_type(locals, each_name);
        if (existing_type != NULL && !direct_is_intlike_scalar_type(existing_type)) {
            report_issue(path, line_no, find_col(line, each_name), line,
                "direct native emitter for-each variable must be an Int-compatible scalar",
                "use a fresh loop variable name or an `Int` local.",
                "for x in xs {");
            free(each_name);
            free(each_iterable);
            free(stripped);
            return 1;
        }
        char *list_name = NULL;
        if (!direct_expr_bare_list_local(locals, each_iterable, &list_name)) {
            report_issue(path, line_no, find_col(line, each_iterable), line,
                "direct native emitter for-each expects a local List<Int>",
                "iterate a named local or parameter with type `List<Int>`.",
                "for x in xs {");
            free(each_name);
            free(each_iterable);
            free(stripped);
            return 1;
        }
        const char *iter_type = direct_names_type(locals, list_name);
        char *elem_type = direct_list_element_type(iter_type);
        if (elem_type == NULL || strcmp(elem_type, "Int") != 0) {
            report_issue(path, line_no, find_col(line, list_name), line,
                "direct native emitter for-each currently supports List<Int>",
                "iterate a `List<Int>` in this direct-engine slice.",
                "for x in xs {");
            free(elem_type);
            free(list_name);
            free(each_name);
            free(each_iterable);
            free(stripped);
            return 1;
        }
        free(elem_type);
        int declare_loop_var = existing_type == NULL;
        if (declare_loop_var) direct_names_add_typed(locals, each_name, "Int");
        char idx_name[64];
        snprintf(idx_name, sizeof(idx_name), "__vais_for_each_%d", locals->temp_count++);
        if (declare_loop_var) {
            sb_append(out, "long ");
            sb_append(out, each_name);
            sb_append(out, ";\n");
        }
        sb_append(out, "for (long ");
        sb_append(out, idx_name);
        sb_append(out, " = 0; ");
        sb_append(out, idx_name);
        sb_append(out, " < ");
        direct_append_list_len_ref(out, list_name, direct_names_is_ref(locals, list_name));
        sb_append(out, "; ");
        sb_append(out, idx_name);
        sb_append(out, "++) {\n");
        sb_append(out, each_name);
        sb_append(out, " = ");
        direct_append_list_data_ref(out, list_name, direct_names_is_ref(locals, list_name));
        sb_append(out, "[");
        sb_append(out, idx_name);
        sb_append(out, "];\n");
        free(list_name);
        free(each_name);
        free(each_iterable);
        free(stripped);
        return 0;
    }
    if (parsed_for < 0 || starts_with(s, "for")) {
        report_issue(path, line_no, find_col(line, "for"), line,
            "direct native emitter expected a for-loop range or List<Int> header",
            "write `for i in 0..n {`, `for i in 0..=n {`, or `for x in xs {` where `xs` is `List<Int>`.",
            "for i in 0..n {");
        free(each_name);
        free(each_iterable);
        free(for_name);
        free(for_lo);
        free(for_hi);
        free(stripped);
        return 1;
    }
    if (strcmp(s, "break") == 0 || strcmp(s, "break;") == 0) {
        sb_append(out, "break;\n");
        free(stripped);
        return 0;
    }
    if (strcmp(s, "continue") == 0 || strcmp(s, "continue;") == 0) {
        sb_append(out, "continue;\n");
        free(stripped);
        return 0;
    }
    if (starts_with(s, "return ")) {
        char *expr = trim_copy(s + 7);
        size_t n = strlen(expr);
        if (n > 0 && expr[n - 1] == ';') expr[n - 1] = '\0';
        const char *return_type = locals->current_return_type == NULL ? "Int" : locals->current_return_type;
        int allow_list_return = direct_is_list_type(return_type);
        int allow_map_return = direct_is_supported_map_return_type(return_type);
        int inline_list_return = allow_list_return && direct_is_list_initializer_expr(expr);
        if (allow_map_return) {
            if (direct_check_map_value_expr(path, line_no, line, expr, return_type, locals, fns, fn_count, structs, struct_count)) {
                free(expr);
                free(stripped);
                return 1;
            }
        } else if (!inline_list_return) {
            char *expr_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
            if (!direct_type_compatible(return_type, expr_type)) {
                report_issue(path, line_no, find_col(line, "return"), line,
                    "direct native emitter return expression type does not match the function return type",
                    "return a value with the type declared in the function header.",
                    "return 40 + 2");
                free(expr_type);
                free(expr);
                free(stripped);
                return 1;
            }
            free(expr_type);
        }
        if (!allow_map_return && direct_check_expr_inner(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count, allow_list_return ? return_type : NULL)) {
            free(expr);
            free(stripped);
            return 1;
        }
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten = allow_list_return
            ? direct_rewrite_list_value_expr(path, line_no, line, expr, return_type, locals, fns, fn_count, structs, struct_count)
            : allow_map_return
            ? direct_rewrite_map_value_expr(path, line_no, line, expr, return_type, locals, fns, fn_count, structs, struct_count)
            : direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten == NULL) {
            free(prelude.data);
            free(expr);
            free(stripped);
            return 1;
        }
        char *c_expr = direct_translate_expr(rewritten);
        sb_append(out, prelude.data);
        sb_append(out, "return ");
        sb_append(out, c_expr);
        sb_append(out, ";\n");
        free(prelude.data);
        free(c_expr);
        free(rewritten);
        free(expr);
        free(stripped);
        return 0;
    }
    char *match_name = NULL;
    char *match_expr = NULL;
    char *match_arms = NULL;
    if (parse_inline_match_let(s, &match_name, &match_expr, &match_arms)) {
        int rc = direct_lower_option_match_let(path, line_no, line, match_name, match_expr, match_arms, locals, fns, fn_count, structs, struct_count, out);
        free(match_name);
        free(match_expr);
        free(match_arms);
        free(stripped);
        return rc;
    }
    free(match_name);
    free(match_expr);
    free(match_arms);
    if (starts_with(s, "let ")) {
        const char *p = skip_ws(s + 4);
        if (starts_with(p, "mut ")) p = skip_ws(p + 4);
        if (!is_ident_start(*p)) {
            report_issue(path, line_no, find_col(line, "let"), line,
                "direct native emitter expected an Int local binding",
                "write `let name = expr` or `let mut name = expr`.",
                NULL);
            free(stripped);
            return 1;
        }
        const char *name_start = p;
        while (is_ident_continue(*p)) p++;
        char *name = substr_copy(name_start, (size_t)(p - name_start));
        p = skip_ws(p);
        char *local_type = NULL;
        if (*p == ':') {
            p = skip_ws(p + 1);
            local_type = direct_parse_local_type(&p);
            if (local_type == NULL) {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter expected a local type",
                    "use `Int`, `Bool`, `Char`, `Str`, `List<Int>`, `List<Struct>`, `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, or a declared struct type in this direct-engine slice.",
                    NULL);
                free(name);
                free(stripped);
                return 1;
            }
            p = skip_ws(p);
            if (!direct_local_type_allowed(structs, struct_count, local_type)) {
                report_issue(path, line_no, find_col(line, local_type), line,
                    "direct native emitter supports Int, Bool, Char, Str, List<Int>, List<Struct>, Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, Map<Str,Char>, and declared struct locals only",
                    "use `let name: Int = expr`, `let name: Bool = expr`, `let name: Char = expr`, `let name: Str = expr`, `let name: List<Int> = []`, `let name: Map<Int,Int> = {}`, `let name: Map<Int,Bool> = {}`, `let name: Map<Int,Char> = {}`, `let name: Map<Str,Int> = {}`, `let name: Map<Str,Bool> = {}`, `let name: Map<Str,Char> = {}`, `let name: Struct = expr`, or omit the annotation.",
                    NULL);
                free(local_type);
                free(name);
                free(stripped);
                return 1;
            }
        }
        if (*p != '=') {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter expected an initialized local",
                "write `let name = expr`.",
                NULL);
            free(local_type);
            free(name);
            free(stripped);
            return 1;
        }
        char *expr = trim_copy(p + 1);
        size_t n = strlen(expr);
        if (n > 0 && expr[n - 1] == ';') expr[n - 1] = '\0';
        if (local_type == NULL && direct_is_map_empty_initializer_expr(expr)) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter requires a Map type annotation for `{}`",
                "write `let name: Map<Int,Int> = {}`, `let name: Map<Int,Bool> = {}`, `let name: Map<Int,Char> = {}`, `let name: Map<Str,Int> = {}`, `let name: Map<Str,Bool> = {}`, or `let name: Map<Str,Char> = {}` for the verified local Map slices.",
                "let m: Map<Int,Int> = {}");
            free(expr);
            free(name);
            free(stripped);
            return 1;
        }
        if (local_type == NULL) local_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
        if (direct_is_map_type(local_type)) {
            if (direct_is_map_empty_initializer_expr(expr)) {
                sb_append(out, direct_c_type(local_type));
                sb_append(out, " ");
                sb_append(out, name);
                sb_append(out, " = {0};\n");
                direct_names_add_typed(locals, name, local_type);
                free(expr);
                free(local_type);
                free(name);
                free(stripped);
                return 0;
            }
            if (!direct_is_supported_map_return_type(local_type)) {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter Map locals must start with `{}`",
                    "initialize Map locals with `{}`; verified return-capable concrete maps may also use a same-type helper returning Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, or Map<Str,Char>.",
                    "let m: Map<Int,Int> = {}");
                free(expr);
                free(local_type);
                free(name);
                free(stripped);
                return 1;
            }
            if (direct_check_map_value_expr(path, line_no, line, expr, local_type, locals, fns, fn_count, structs, struct_count)) {
                free(expr);
                free(local_type);
                free(name);
                free(stripped);
                return 1;
            }
            StrBuf prelude;
            sb_init(&prelude);
            direct_current_prelude = &prelude;
            char *rewritten = direct_rewrite_map_value_expr(path, line_no, line, expr, local_type, locals, fns, fn_count, structs, struct_count);
            direct_current_prelude = NULL;
            if (rewritten == NULL) {
                free(prelude.data);
                free(expr);
                free(local_type);
                free(name);
                free(stripped);
                return 1;
            }
            char *c_expr = direct_translate_expr(rewritten);
            sb_append(out, prelude.data);
            sb_append(out, direct_c_type(local_type));
            sb_append(out, " ");
            sb_append(out, name);
            sb_append(out, " = ");
            sb_append(out, c_expr);
            sb_append(out, ";\n");
            direct_names_add_typed(locals, name, local_type);
            free(prelude.data);
            free(c_expr);
            free(rewritten);
            free(expr);
            free(local_type);
            free(name);
            free(stripped);
            return 0;
        }
        if (direct_is_list_type(local_type)) {
            char *list_elem_type = direct_list_element_type(local_type);
            if (direct_is_list_initializer_expr(expr)) {
                sb_append(out, direct_c_type(local_type));
                sb_append(out, " ");
                sb_append(out, name);
                sb_append(out, " = {{0}, 0};\n");
                direct_names_add_typed(locals, name, local_type);
                char *items[16] = {0};
                int item_count = 0;
                int literal_state = direct_parse_list_literal_items(expr, items, 16, &item_count);
                if (literal_state < 0) {
                    report_issue(path, line_no, find_col(line, name), line,
                        "direct native emitter supports up to 16 List literal items",
                        "start with `[]` and call `push` for larger direct-engine lists.",
                        NULL);
                    for (int k = 0; k < 16; k++) free(items[k]);
                    free(expr);
                    free(local_type);
                    free(list_elem_type);
                    free(name);
                    free(stripped);
                    return 1;
                }
                for (int item = 0; item < item_count; item++) {
                    if (direct_check_expr(path, line_no, line, items[item], locals, fns, fn_count, structs, struct_count)) {
                        for (int k = item; k < item_count; k++) free(items[k]);
                        free(expr);
                        free(local_type);
                        free(list_elem_type);
                        free(name);
                        free(stripped);
                        return 1;
                    }
                    char *item_type = direct_infer_expr_type(items[item], locals, fns, fn_count, structs, struct_count);
                    if (strcmp(item_type, list_elem_type) != 0) {
                        report_issue(path, line_no, find_col(line, name), line,
                            "direct native emitter list literal item type does not match the local list element type",
                            "push values with the element type declared by the local list.",
                            NULL);
                        free(item_type);
                        for (int k = item; k < item_count; k++) free(items[k]);
                        free(expr);
                        free(local_type);
                        free(list_elem_type);
                        free(name);
                        free(stripped);
                        return 1;
                    }
                    free(item_type);
                    StrBuf prelude;
                    sb_init(&prelude);
                    direct_current_prelude = &prelude;
                    char *rewritten_item = direct_rewrite_expr(path, line_no, line, items[item], locals, fns, fn_count, structs, struct_count);
                    direct_current_prelude = NULL;
                    if (rewritten_item == NULL) {
                        free(prelude.data);
                        for (int k = item; k < item_count; k++) free(items[k]);
                        free(expr);
                        free(local_type);
                        free(list_elem_type);
                        free(name);
                        free(stripped);
                        return 1;
                    }
                    char *c_item = direct_translate_expr(rewritten_item);
                    sb_append(out, prelude.data);
                    sb_append(out, name);
                    sb_append(out, ".data[");
                    sb_append(out, name);
                    sb_append(out, ".len++] = ");
                    sb_append(out, c_item);
                    sb_append(out, ";\n");
                    free(prelude.data);
                    free(c_item);
                    free(rewritten_item);
                    free(items[item]);
                }
                free(expr);
                free(local_type);
                free(list_elem_type);
                free(name);
                free(stripped);
                return 0;
            }
            if (direct_check_expr_inner(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count, local_type)) {
                free(expr);
                free(local_type);
                free(list_elem_type);
                free(name);
                free(stripped);
                return 1;
            }
            char *expr_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
            if (!direct_type_compatible(local_type, expr_type)) {
                report_issue(path, line_no, find_col(line, name), line,
                    "direct native emitter local initializer type does not match the annotation",
                    "initialize the local with a value of the annotated type.",
                    NULL);
                free(expr_type);
                free(expr);
                free(local_type);
                free(list_elem_type);
                free(name);
                free(stripped);
                return 1;
            }
            free(expr_type);
            StrBuf prelude;
            sb_init(&prelude);
            direct_current_prelude = &prelude;
            char *rewritten = direct_rewrite_list_value_expr(path, line_no, line, expr, local_type, locals, fns, fn_count, structs, struct_count);
            direct_current_prelude = NULL;
            if (rewritten == NULL) {
                free(prelude.data);
                free(expr);
                free(local_type);
                free(list_elem_type);
                free(name);
                free(stripped);
                return 1;
            }
            char *c_expr = direct_translate_expr(rewritten);
            sb_append(out, prelude.data);
            sb_append(out, direct_c_type(local_type));
            sb_append(out, " ");
            sb_append(out, name);
            sb_append(out, " = ");
            sb_append(out, c_expr);
            sb_append(out, ";\n");
            direct_names_add_typed(locals, name, local_type);
            free(prelude.data);
            free(c_expr);
            free(rewritten);
            free(expr);
            free(local_type);
            free(list_elem_type);
            free(name);
            free(stripped);
            return 0;
        }
        if (direct_check_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count)) {
            free(expr);
            free(local_type);
            free(name);
            free(stripped);
            return 1;
        }
        char *expr_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
        if (!direct_type_compatible(local_type, expr_type)) {
            report_issue(path, line_no, find_col(line, name), line,
                "direct native emitter local initializer type does not match the annotation",
                "initialize the local with a value of the annotated type.",
                NULL);
            free(expr_type);
            free(expr);
            free(local_type);
            free(name);
            free(stripped);
            return 1;
        }
        free(expr_type);
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten = direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten == NULL) {
            free(prelude.data);
            free(expr);
            free(local_type);
            free(name);
            free(stripped);
            return 1;
        }
        char *c_expr = direct_translate_expr(rewritten);
        sb_append(out, prelude.data);
        sb_append(out, direct_c_type(local_type));
        sb_append(out, " ");
        sb_append(out, name);
        sb_append(out, " = ");
        sb_append(out, c_expr);
        sb_append(out, ";\n");
        direct_names_add_typed(locals, name, local_type);
        free(prelude.data);
        free(c_expr);
        free(rewritten);
        free(expr);
        free(local_type);
        free(name);
        free(stripped);
        return 0;
    }

    char *call_args = NULL;
    int parsed_print = direct_parse_named_call_statement(s, "print", &call_args);
    if (parsed_print == 1) {
        int rc = direct_lower_print_statement(path, line_no, line, call_args, locals, fns, fn_count, structs, struct_count, out);
        free(call_args);
        free(stripped);
        return rc;
    }
    if (parsed_print < 0) {
        report_issue(path, line_no, find_col(line, "print"), line,
            "direct native emitter expected a valid print call statement",
            "write `print(\"text\")` or `print(\"value={name}\")`.",
            NULL);
        free(stripped);
        return 1;
    }
    int parsed_putchar = direct_parse_named_call_statement(s, "putchar", &call_args);
    if (parsed_putchar == 1) {
        int rc = direct_lower_putchar_statement(path, line_no, line, call_args, locals, fns, fn_count, structs, struct_count, out);
        free(call_args);
        free(stripped);
        return rc;
    }
    if (parsed_putchar < 0) {
        report_issue(path, line_no, find_col(line, "putchar"), line,
            "direct native emitter expected a valid putchar call statement",
            "write `putchar(33)` or pass an Int, Bool, or Char value.",
            NULL);
        free(stripped);
        return 1;
    }

    char *method_base = NULL;
    char *method_name = NULL;
    char *method_args = NULL;
    if (direct_parse_method_statement(s, &method_base, &method_name, &method_args)) {
        const char *base_type = direct_names_type(locals, method_base);
        if (direct_is_map_type(base_type)) {
            int is_insert = strcmp(method_name, "insert") == 0;
            int is_remove = strcmp(method_name, "remove") == 0;
            int is_clear = strcmp(method_name, "clear") == 0;
            if (!is_insert && !is_remove && !is_clear) {
                report_issue(path, line_no, find_col(line, method_base), line,
                    "direct native emitter supports Map.insert, Map.remove, and Map.clear statements only",
                    "write `m.insert(key, value)`, `m.remove(key)`, or `m.clear()` on a local Map<Int,Int>, Map<Int,Bool>, Map<Int,Char>, Map<Str,Int>, Map<Str,Bool>, or Map<Str,Char>.",
                    NULL);
                free(method_base);
                free(method_name);
                free(method_args);
                free(stripped);
                return 1;
            }
            char *args[16] = {0};
            int argc = split_top_level_commas_c(method_args, args, 16);
            if (is_clear && argc == 1 && args[0] != NULL && strlen(skip_ws(args[0])) == 0) argc = 0;
            int expected_argc = is_insert ? 2 : (is_clear ? 0 : 1);
            int bad_args = argc != expected_argc;
            if (!is_clear && (args[0] == NULL || strlen(skip_ws(args[0])) == 0)) bad_args = 1;
            if (is_insert && (args[1] == NULL || strlen(skip_ws(args[1])) == 0)) bad_args = 1;
            if (bad_args) {
                report_issue(path, line_no, find_col(line, method_name), line,
                    is_insert ? "direct native emitter Map.insert expects key and value arguments" : (is_clear ? "direct native emitter Map.clear expects no arguments" : "direct native emitter Map.remove expects one key argument"),
                    is_insert ? "write `m.insert(key, value)`." : (is_clear ? "write `m.clear()`." : "write `m.remove(key)`."),
                    NULL);
                for (int k = 0; k < 16; k++) free(args[k]);
                free(method_base);
                free(method_name);
                free(method_args);
                free(stripped);
                return 1;
            }
            for (int a = 0; a < argc; a++) {
                if (direct_check_expr(path, line_no, line, args[a], locals, fns, fn_count, structs, struct_count)) {
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(method_base);
                    free(method_name);
                    free(method_args);
                    free(stripped);
                    return 1;
                }
                char *arg_type = direct_infer_expr_type(args[a], locals, fns, fn_count, structs, struct_count);
                const char *expected_arg_type = a == 0 ? direct_map_key_type(base_type) : direct_map_value_type(base_type);
                if (!direct_map_arg_type_compatible(expected_arg_type, arg_type)) {
                    report_issue(path, line_no, find_col(line, method_name), line,
                        is_insert ? "direct native emitter Map.insert arguments must match key/value types" : "direct native emitter Map.remove key must match the Map key type",
                        is_insert ? "use a key and value matching the Map's concrete key/value types." : "use a key matching the Map's concrete key type.",
                        NULL);
                    free(arg_type);
                    for (int k = 0; k < 16; k++) free(args[k]);
                    free(method_base);
                    free(method_name);
                    free(method_args);
                    free(stripped);
                    return 1;
                }
                free(arg_type);
            }
            StrBuf prelude;
            sb_init(&prelude);
            direct_current_prelude = &prelude;
            char *rewritten_key = is_clear ? NULL : direct_rewrite_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count);
            char *rewritten_value = (is_insert && rewritten_key != NULL) ? direct_rewrite_expr(path, line_no, line, args[1], locals, fns, fn_count, structs, struct_count) : NULL;
            direct_current_prelude = NULL;
            if ((!is_clear && rewritten_key == NULL) || (is_insert && rewritten_value == NULL)) {
                free(prelude.data);
                free(rewritten_key);
                free(rewritten_value);
                for (int k = 0; k < 16; k++) free(args[k]);
                free(method_base);
                free(method_name);
                free(method_args);
                free(stripped);
                return 1;
            }
            char *c_key = is_clear ? NULL : direct_translate_expr(rewritten_key);
            char *c_value = is_insert ? direct_translate_expr(rewritten_value) : NULL;
            sb_append(out, prelude.data);
            sb_append(out, direct_map_helper_name(base_type, is_insert ? "insert" : (is_clear ? "clear" : "remove")));
            sb_append(out, "(");
            direct_append_map_ptr_ref(out, method_base, direct_names_is_ref(locals, method_base));
            if (!is_clear) {
                sb_append(out, ", ");
                sb_append(out, c_key);
            }
            if (is_insert) {
                sb_append(out, ", ");
                sb_append(out, c_value);
            }
            sb_append(out, ");\n");
            free(prelude.data);
            free(c_key);
            free(c_value);
            free(rewritten_key);
            free(rewritten_value);
            for (int k = 0; k < 16; k++) free(args[k]);
            free(method_base);
            free(method_name);
            free(method_args);
            free(stripped);
            return 0;
        }
        if (!direct_is_list_type(base_type) || strcmp(method_name, "push") != 0) {
            report_issue(path, line_no, find_col(line, method_base), line,
                "direct native emitter supports List.push and Map.insert/Map.remove/Map.clear statements only",
                "write `xs.push(value)`, `m.insert(key, value)`, `m.remove(key)`, or `m.clear()`.",
                NULL);
            free(method_base);
            free(method_name);
            free(method_args);
            free(stripped);
            return 1;
        }
        char *list_elem_type = direct_list_element_type(base_type);
        char *args[16] = {0};
        int argc = split_top_level_commas_c(method_args, args, 16);
        if (argc != 1 || strlen(skip_ws(args[0])) == 0) {
            report_issue(path, line_no, find_col(line, method_name), line,
                "direct native emitter List.push expects one argument",
                "write `xs.push(value)`.",
                NULL);
            for (int k = 0; k < 16; k++) free(args[k]);
            free(list_elem_type);
            free(method_base);
            free(method_name);
            free(method_args);
            free(stripped);
            return 1;
        }
        if (direct_check_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count)) {
            for (int k = 0; k < 16; k++) free(args[k]);
            free(list_elem_type);
            free(method_base);
            free(method_name);
            free(method_args);
            free(stripped);
            return 1;
        }
        char *arg_type = direct_infer_expr_type(args[0], locals, fns, fn_count, structs, struct_count);
        if (strcmp(arg_type, list_elem_type) != 0) {
            report_issue(path, line_no, find_col(line, method_name), line,
                "direct native emitter List.push argument type does not match the list element type",
                "push a value with the element type declared by the local list.",
                NULL);
            free(arg_type);
            for (int k = 0; k < 16; k++) free(args[k]);
            free(list_elem_type);
            free(method_base);
            free(method_name);
            free(method_args);
            free(stripped);
            return 1;
        }
        free(arg_type);
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten_arg = direct_rewrite_expr(path, line_no, line, args[0], locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten_arg == NULL) {
            free(prelude.data);
            for (int k = 0; k < 16; k++) free(args[k]);
            free(list_elem_type);
            free(method_base);
            free(method_name);
            free(method_args);
            free(stripped);
            return 1;
        }
        char *c_arg = direct_translate_expr(rewritten_arg);
        int base_is_ref = direct_names_is_ref(locals, method_base);
        sb_append(out, prelude.data);
        direct_append_list_data_ref(out, method_base, base_is_ref);
        sb_append(out, "[");
        direct_append_list_len_ref(out, method_base, base_is_ref);
        sb_append(out, "++] = ");
        sb_append(out, c_arg);
        sb_append(out, ";\n");
        free(prelude.data);
        free(c_arg);
        free(rewritten_arg);
        for (int k = 0; k < 16; k++) free(args[k]);
        free(list_elem_type);
        free(method_base);
        free(method_name);
        free(method_args);
        free(stripped);
        return 0;
    }

    const char *eq = strchr(s, '=');
    const char *semi = strchr(s, ';');
    if (eq != NULL && (semi == NULL || eq < semi)) {
        char *lhs = trim_copy(substr_copy(s, (size_t)(eq - s)));
        char *expr = trim_copy(eq + 1);
        size_t n = strlen(expr);
        if (n > 0 && expr[n - 1] == ';') expr[n - 1] = '\0';
        if (direct_check_assignment_target(path, line_no, line, lhs, locals, structs, struct_count)) {
            free(lhs);
            free(expr);
            free(stripped);
            return 1;
        }
        const char *target_type = direct_assignment_target_type(lhs, locals, structs, struct_count);
        int allow_list_assignment = direct_is_list_type(target_type);
        if (direct_is_map_type(target_type)) {
            if (direct_expr_exact_map_return_call(expr, fns, fn_count)) {
                char *expr_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
                int ok_type = expr_type != NULL && strcmp(target_type, expr_type) == 0;
                free(expr_type);
                if (!ok_type) {
                    report_issue(path, line_no, find_col(line, lhs), line,
                        "direct native emitter Map assignment source type must match the target Map type",
                        "copy from a Map-returning call with the same concrete Map type.",
                        "scores = make_scores()");
                    free(lhs);
                    free(expr);
                    free(stripped);
                    return 1;
                }
                StrBuf prelude;
                sb_init(&prelude);
                direct_current_prelude = &prelude;
                char *rewritten = direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
                direct_current_prelude = NULL;
                if (rewritten == NULL) {
                    free(prelude.data);
                    free(lhs);
                    free(expr);
                    free(stripped);
                    return 1;
                }
                char *c_expr = direct_translate_expr(rewritten);
                sb_append(out, prelude.data);
                if (direct_names_is_ref(locals, lhs)) sb_append(out, "*");
                sb_append(out, lhs);
                sb_append(out, " = ");
                sb_append(out, c_expr);
                sb_append(out, ";\n");
                free(prelude.data);
                free(c_expr);
                free(rewritten);
                free(lhs);
                free(expr);
                free(stripped);
                return 0;
            }
            char *rhs_name = trim_copy(expr);
            const char *rhs_type = direct_is_plain_ident(rhs_name) ? direct_names_type(locals, rhs_name) : NULL;
            if (!direct_is_map_type(rhs_type)) {
                report_issue(path, line_no, find_col(line, lhs), line,
                    "direct native emitter Map assignment requires another local or parameter Map value",
                    "assign from a local, same-type parameter, or same-type Map-returning call for `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, or `Map<Str,Char>`; broader generic key/value forms are not in this direct slice.",
                    "scores = other");
                free(rhs_name);
                free(lhs);
                free(expr);
                free(stripped);
                return 1;
            }
            if (strcmp(target_type, rhs_type) != 0) {
                report_issue(path, line_no, find_col(line, lhs), line,
                    "direct native emitter Map assignment source type must match the target Map type",
                    "copy between locals with the same concrete Map type.",
                    "scores = other");
                free(rhs_name);
                free(lhs);
                free(expr);
                free(stripped);
                return 1;
            }
            sb_append(out, direct_map_helper_name(target_type, "copy"));
            sb_append(out, "(");
            direct_append_map_ptr_ref(out, lhs, direct_names_is_ref(locals, lhs));
            sb_append(out, ", ");
            direct_append_map_ptr_ref(out, rhs_name, direct_names_is_ref(locals, rhs_name));
            sb_append(out, ");\n");
            free(rhs_name);
            free(lhs);
            free(expr);
            free(stripped);
            return 0;
        }
        if (direct_check_expr_inner(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count, allow_list_assignment ? target_type : NULL)) {
            free(lhs);
            free(expr);
            free(stripped);
            return 1;
        }
        int contextual_list_assignment = allow_list_assignment && direct_is_list_initializer_expr(expr);
        if (!contextual_list_assignment) {
            char *expr_type = direct_infer_expr_type(expr, locals, fns, fn_count, structs, struct_count);
            if (target_type != NULL && !direct_type_compatible(target_type, expr_type)) {
                report_issue(path, line_no, find_col(line, lhs), line,
                    "direct native emitter assignment expression type does not match the target",
                    "assign a value with the same type as the target local.",
                    NULL);
                free(expr_type);
                free(lhs);
                free(expr);
                free(stripped);
                return 1;
            }
            free(expr_type);
        }
        StrBuf prelude;
        sb_init(&prelude);
        direct_current_prelude = &prelude;
        char *rewritten_lhs = direct_rewrite_expr(path, line_no, line, lhs, locals, fns, fn_count, structs, struct_count);
        if (rewritten_lhs == NULL) {
            direct_current_prelude = NULL;
            free(prelude.data);
            free(lhs);
            free(expr);
            free(stripped);
            return 1;
        }
        char *rewritten = allow_list_assignment
            ? direct_rewrite_list_value_expr(path, line_no, line, expr, target_type, locals, fns, fn_count, structs, struct_count)
            : direct_rewrite_expr(path, line_no, line, expr, locals, fns, fn_count, structs, struct_count);
        direct_current_prelude = NULL;
        if (rewritten == NULL) {
            free(prelude.data);
            free(rewritten_lhs);
            free(lhs);
            free(expr);
            free(stripped);
            return 1;
        }
        char *c_lhs = direct_translate_expr(rewritten_lhs);
        char *c_expr = direct_translate_expr(rewritten);
        sb_append(out, prelude.data);
        if (allow_list_assignment && direct_names_is_ref(locals, lhs)) sb_append(out, "*");
        sb_append(out, c_lhs);
        sb_append(out, " = ");
        sb_append(out, c_expr);
        sb_append(out, ";\n");
        free(prelude.data);
        free(c_lhs);
        free(c_expr);
        free(rewritten_lhs);
        free(rewritten);
        free(lhs);
        free(expr);
        free(stripped);
        return 0;
    }

    report_issue(path, line_no, 1, line,
        "direct native emitter supports Int functions, struct locals, List locals, concrete Map locals/parameters, lets, assignment, if, while, for, break, continue, calls, and return",
        "use the full engine for syntax outside the direct subset.",
        NULL);
    free(stripped);
    return 1;
}

static char *direct_lower_to_c(const char *path, const char *raw) {
    LineVec lines = split_lines(raw);
    DirectFnInfo fns[64];
    DirectStructInfo structs[32];
    int fn_count = 0;
    int struct_count = 0;
    memset(fns, 0, sizeof(fns));
    memset(structs, 0, sizeof(structs));
    int *skip_lines = (int *)calloc(lines.len == 0 ? 1 : lines.len, sizeof(int));
    if (skip_lines == NULL) die_oom();
    int has_main = 0;

    for (size_t i = 0; i < lines.len; i++) {
        char *code = strip_line_comment(lines.items[i], strlen(lines.items[i]));
        const char *trim = skip_ws(code);
        if (starts_with(trim, "import") && !is_ident_continue(trim[6])) {
            report_issue(path, (int)i + 1, find_col(lines.items[i], "import"), lines.items[i],
                "direct native emitter does not support imports",
                "use the full engine for local imports; direct engine builds stay single-file.",
                NULL);
            free(code);
            free(skip_lines);
            lines_free(&lines);
            direct_structs_free(structs, struct_count);
            direct_fns_free(fns, fn_count);
            return NULL;
        }
        if ((starts_with(trim, "module") && !is_ident_continue(trim[6])) ||
            (starts_with(trim, "package") && !is_ident_continue(trim[7]))) {
            const char *kw = starts_with(trim, "module") ? "module" : "package";
            report_issue(path, (int)i + 1, find_col(lines.items[i], kw), lines.items[i],
                "module and package declarations are not implemented yet",
                "omit the declaration; module names are derived from file paths in the first import slice.",
                NULL);
            free(code);
            free(skip_lines);
            lines_free(&lines);
            direct_structs_free(structs, struct_count);
            direct_fns_free(fns, fn_count);
            return NULL;
        }
        free(code);

        DirectStructInfo st;
        memset(&st, 0, sizeof(st));
        size_t struct_end = i;
        int parsed_struct = direct_parse_struct_decl(&lines, i, &st, &struct_end, path);
        if (parsed_struct == 1) {
            if (struct_count >= 32) {
                fprintf(stderr, "error: too many direct structs\n");
                direct_struct_free_one(&st);
                free(skip_lines);
                lines_free(&lines);
                direct_structs_free(structs, struct_count);
                direct_fns_free(fns, fn_count);
                return NULL;
            }
            if (direct_find_struct(structs, struct_count, st.name) != NULL) {
                report_issue(path, st.line_no, find_col(lines.items[i], st.name), lines.items[i],
                    "direct native emitter found a duplicate struct name",
                    "use a unique name for each direct-engine struct declaration.",
                    NULL);
                direct_struct_free_one(&st);
                free(skip_lines);
                lines_free(&lines);
                direct_structs_free(structs, struct_count);
                direct_fns_free(fns, fn_count);
                return NULL;
            }
            for (size_t j = i; j <= struct_end && j < lines.len; j++) skip_lines[j] = 1;
            structs[struct_count++] = st;
            i = struct_end;
            continue;
        }
        if (parsed_struct < 0) {
            free(skip_lines);
            lines_free(&lines);
            direct_structs_free(structs, struct_count);
            direct_fns_free(fns, fn_count);
            return NULL;
        }

        DirectFnInfo info;
        memset(&info, 0, sizeof(info));
        int parsed = parse_direct_fn_header(lines.items[i], &info);
        if (parsed == 1) {
            if (fn_count >= 64) {
                fprintf(stderr, "error: too many direct functions\n");
                free(skip_lines);
                lines_free(&lines);
                direct_structs_free(structs, struct_count);
                direct_fns_free(fns, fn_count);
                return NULL;
            }
            info.line_no = (int)i + 1;
            if (strcmp(info.name, "main") == 0) {
                if (info.param_count != 0 || strcmp(info.return_type, "Int") != 0) {
                    report_issue(path, (int)i + 1, find_col(lines.items[i], "main"), lines.items[i],
                        "direct native emitter requires `fn main() -> Int`",
                        "write the entrypoint without parameters.",
                        NULL);
                    direct_fns_free(&info, 1);
                    free(skip_lines);
                    lines_free(&lines);
                    direct_structs_free(structs, struct_count);
                    direct_fns_free(fns, fn_count);
                    return NULL;
                }
                has_main = 1;
            }
            fns[fn_count++] = info;
        } else if (parsed < 0) {
            report_issue(path, (int)i + 1, 1, lines.items[i],
                "direct native emitter supports scalar/List/Struct function headers, verified concrete Map returns, and verified Map parameters",
                "write functions with `Int`, `Bool`, `Char`, `Str`, `List<Int>`, `List<Struct>`, `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, or declared struct return types.",
                NULL);
            free(skip_lines);
            lines_free(&lines);
            direct_structs_free(structs, struct_count);
            direct_fns_free(fns, fn_count);
            return NULL;
        }
    }

    for (int f = 0; f < fn_count; f++) {
        if (direct_validate_fn_types(path, lines.items[fns[f].line_no - 1], &fns[f], structs, struct_count) != 0) {
            free(skip_lines);
            lines_free(&lines);
            direct_structs_free(structs, struct_count);
            direct_fns_free(fns, fn_count);
            return NULL;
        }
    }

    if (!has_main) {
        report_issue(path, 1, 1, lines.len ? lines.items[0] : "",
            "direct native emitter requires `fn main() -> Int`",
            "add `fn main() -> Int { return <int> }` as the program entrypoint.",
            NULL);
        free(skip_lines);
        lines_free(&lines);
        direct_structs_free(structs, struct_count);
        direct_fns_free(fns, fn_count);
        return NULL;
    }

    StrBuf out;
    sb_init(&out);
    sb_append(&out, "#include <stdbool.h>\n");
    sb_append(&out, "#include <stdio.h>\n");
    sb_append(&out, "#include <string.h>\n");
    sb_append(&out, "typedef long Int;\n");
    sb_append(&out, "typedef long Bool;\n");
    sb_append(&out, "typedef const char *Str;\n");
    sb_append(&out, "static long __vais_str_len(const char *s) { return (long)strlen(s); }\n");
    sb_append(&out, "static long __vais_str_byte(const char *s, long index) { return (long)(unsigned char)s[index]; }\n");
    sb_append(&out, "static long __vais_str_eq(const char *a, const char *b) { return strcmp(a, b) == 0 ? 1 : 0; }\n");
    sb_append(&out, "static long __vais_parse_uint(const char *s) { long value = 0; for (long i = 0; s[i] != '\\0'; i++) { unsigned char b = (unsigned char)s[i]; if (b < '0' || b > '9') break; value = value * 10 + (long)(b - '0'); } return value; }\n");
    sb_append(&out, "static long __vais_parse_int(const char *s) { if (s[0] == '-') return 0 - __vais_parse_uint(s + 1); return __vais_parse_uint(s); }\n");
    sb_append(&out, "static long __vais_list_checked_index(long index, long len) { if (index < 0 || index >= len) __builtin_trap(); return index; }\n");
    sb_append(&out, "static long __vais_list_checked_last(long len) { if (len <= 0) __builtin_trap(); return len - 1; }\n");
    sb_append(&out, "static long __vais_list_checked_pop_index(long *len) { if (*len <= 0) __builtin_trap(); *len -= 1; return *len; }\n");
    sb_append(&out, "typedef struct { long data[256]; long len; } DirectListInt;\n");
    sb_append(&out, "static long __vais_list_int_sum(DirectListInt *xs) { long total = 0; for (long i = 0; i < xs->len; i++) total += xs->data[i]; return total; }\n");
    sb_append(&out, "typedef struct { long keys[256]; long values[256]; unsigned char present[256]; long len; } DirectMapIntInt;\n");
    sb_append(&out, "static long __vais_map_int_int_find(DirectMapIntInt *m, long key) { for (long i = 0; i < m->len; i++) if (m->present[i] && m->keys[i] == key) return i; return -1; }\n");
    sb_append(&out, "static void __vais_map_int_int_insert(DirectMapIntInt *m, long key, long value) { long i = __vais_map_int_int_find(m, key); if (i >= 0) { m->values[i] = value; return; } if (m->len >= 256) __builtin_trap(); i = m->len++; m->present[i] = 1; m->keys[i] = key; m->values[i] = value; }\n");
    sb_append(&out, "static void __vais_map_int_int_remove(DirectMapIntInt *m, long key) { long i = __vais_map_int_int_find(m, key); if (i < 0) return; long last = --m->len; if (i != last) { m->present[i] = m->present[last]; m->keys[i] = m->keys[last]; m->values[i] = m->values[last]; } m->present[last] = 0; }\n");
    sb_append(&out, "static void __vais_map_int_int_clear(DirectMapIntInt *m) { m->len = 0; }\n");
    sb_append(&out, "static void __vais_map_int_int_copy(DirectMapIntInt *dst, DirectMapIntInt *src) { *dst = *src; }\n");
    sb_append(&out, "static long __vais_map_int_int_get(DirectMapIntInt *m, long key, long fallback) { long i = __vais_map_int_int_find(m, key); return i >= 0 ? m->values[i] : fallback; }\n");
    sb_append(&out, "static long __vais_map_int_int_get_opt(DirectMapIntInt *m, long key) { long i = __vais_map_int_int_find(m, key); return i >= 0 ? (m->values[i] * 2) : 1; }\n");
    sb_append(&out, "static long __vais_map_int_int_contains(DirectMapIntInt *m, long key) { return __vais_map_int_int_find(m, key) >= 0 ? 1 : 0; }\n");
    sb_append(&out, "static long __vais_map_int_int_len(DirectMapIntInt *m) { return m->len; }\n");
    sb_append(&out, "typedef struct { const char *keys[256]; long values[256]; unsigned char present[256]; long len; } DirectMapStrInt;\n");
    sb_append(&out, "static long __vais_map_str_int_find(DirectMapStrInt *m, const char *key) { for (long i = 0; i < m->len; i++) if (m->present[i] && strcmp(m->keys[i], key) == 0) return i; return -1; }\n");
    sb_append(&out, "static void __vais_map_str_int_insert(DirectMapStrInt *m, const char *key, long value) { long i = __vais_map_str_int_find(m, key); if (i >= 0) { m->values[i] = value; return; } if (m->len >= 256) __builtin_trap(); i = m->len++; m->present[i] = 1; m->keys[i] = key; m->values[i] = value; }\n");
    sb_append(&out, "static void __vais_map_str_int_remove(DirectMapStrInt *m, const char *key) { long i = __vais_map_str_int_find(m, key); if (i < 0) return; long last = --m->len; if (i != last) { m->present[i] = m->present[last]; m->keys[i] = m->keys[last]; m->values[i] = m->values[last]; } m->present[last] = 0; }\n");
    sb_append(&out, "static void __vais_map_str_int_clear(DirectMapStrInt *m) { m->len = 0; }\n");
    sb_append(&out, "static void __vais_map_str_int_copy(DirectMapStrInt *dst, DirectMapStrInt *src) { *dst = *src; }\n");
    sb_append(&out, "static long __vais_map_str_int_get(DirectMapStrInt *m, const char *key, long fallback) { long i = __vais_map_str_int_find(m, key); return i >= 0 ? m->values[i] : fallback; }\n");
    sb_append(&out, "static long __vais_map_str_int_get_opt(DirectMapStrInt *m, const char *key) { long i = __vais_map_str_int_find(m, key); return i >= 0 ? (m->values[i] * 2) : 1; }\n");
    sb_append(&out, "static long __vais_map_str_int_contains(DirectMapStrInt *m, const char *key) { return __vais_map_str_int_find(m, key) >= 0 ? 1 : 0; }\n");
    sb_append(&out, "static long __vais_map_str_int_len(DirectMapStrInt *m) { return m->len; }\n");
    for (int s = 0; s < struct_count; s++) {
        sb_append(&out, "typedef struct {");
        for (int f = 0; f < structs[s].field_count; f++) {
            sb_append(&out, " long ");
            sb_append(&out, structs[s].fields[f]);
            sb_append(&out, ";");
        }
        sb_append(&out, " } ");
        sb_append(&out, structs[s].name);
        sb_append(&out, ";\n");
    }
    for (int s = 0; s < struct_count; s++) {
        sb_append(&out, "typedef struct { ");
        sb_append(&out, structs[s].name);
        sb_append(&out, " data[256]; long len; } DirectList_");
        sb_append(&out, structs[s].name);
        sb_append(&out, ";\n");
    }
    for (int f = 0; f < fn_count; f++) {
        sb_append(&out, direct_c_type(fns[f].return_type));
        sb_append(&out, " ");
        sb_append(&out, fns[f].name);
        sb_append(&out, "(");
        for (int p = 0; p < fns[f].param_count; p++) {
            if (p > 0) sb_append(&out, ", ");
            sb_append(&out, direct_c_param_type(fns[f].param_types[p]));
            sb_append(&out, " ");
            sb_append(&out, fns[f].params[p]);
        }
        sb_append(&out, ");\n");
    }
    DirectNameSet locals;
    memset(&locals, 0, sizeof(locals));
    int in_main = 0;
    int main_depth = 0;
    int main_has_return = 0;
    for (size_t i = 0; i < lines.len; i++) {
        if (skip_lines[i]) continue;
        DirectFnInfo line_header;
        memset(&line_header, 0, sizeof(line_header));
        int header_state = parse_direct_fn_header(lines.items[i], &line_header);
        if (header_state == 1 && strcmp(line_header.name, "main") == 0) {
            in_main = 1;
            main_depth = 0;
        }
        if (in_main && strstr(skip_ws(lines.items[i]), "return ") != NULL) main_has_return = 1;
        if (direct_lower_line(path, (int)i + 1, lines.items[i], &locals, fns, fn_count, structs, struct_count, &out) != 0) {
            if (header_state == 1) {
                direct_fns_free(&line_header, 1);
            }
            free(out.data);
            free(skip_lines);
            direct_names_free(&locals);
            lines_free(&lines);
            direct_structs_free(structs, struct_count);
            direct_fns_free(fns, fn_count);
            return NULL;
        }
        if (header_state == 1) {
            direct_fns_free(&line_header, 1);
        }
        if (in_main) {
            const char *code = lines.items[i];
            for (int c = 0; code[c] != '\0'; c++) {
                if (code[c] == '{') main_depth++;
                if (code[c] == '}') main_depth--;
            }
            if (main_depth <= 0) {
                in_main = 0;
                main_depth = 0;
            }
        }
    }
    if (!main_has_return) {
        report_issue(path, 1, 1, lines.len ? lines.items[0] : "",
            "direct native emitter requires at least one `return` statement",
            "write `fn main() -> Int { return 40 + 2 }` for the direct Int subset.",
            "fn main() -> Int { return 40 + 2 }");
        free(out.data);
        free(skip_lines);
        direct_names_free(&locals);
        lines_free(&lines);
        direct_structs_free(structs, struct_count);
        direct_fns_free(fns, fn_count);
        return NULL;
    }

    free(skip_lines);
    direct_names_free(&locals);
    lines_free(&lines);
    direct_structs_free(structs, struct_count);
    direct_fns_free(fns, fn_count);
    return sb_take(&out);
}

static int copy_file_to_stdout(const char *path) {
    char *text = read_file(path);
    if (text == NULL) return 1;
    fputs(text, stdout);
    free(text);
    return 0;
}

static int direct_emit_ir_file(const char *source, const char *out_path, const char *clang) {
    if (!has_vais_suffix(source)) {
        fprintf(stderr, "error: Vais source files must use the .vais extension: %s\n", source);
        return 1;
    }
    char *raw = read_file(source);
    if (raw == NULL) return 1;
    char *c_src = direct_lower_to_c(source, raw);
    free(raw);
    if (c_src == NULL) return 1;

    char c_path[512];
    char ll_path[512];
    if (make_tmp_path(c_path, sizeof(c_path), "direct.c") != 0) {
        free(c_src);
        return 1;
    }
    const char *emit_path = out_path;
    if (strcmp(out_path, "-") == 0) {
        if (make_tmp_path(ll_path, sizeof(ll_path), "direct.ll") != 0) {
            free(c_src);
            return 1;
        }
        emit_path = ll_path;
    }
    FILE *fp = fopen(c_path, "wb");
    if (fp == NULL) {
        fprintf(stderr, "error: cannot write %s: %s\n", c_path, strerror(errno));
        free(c_src);
        return 1;
    }
    fputs(c_src, fp);
    fclose(fp);
    free(c_src);

    char *const argv[] = {(char *)clang, "-S", "-emit-llvm", "-Wno-main-return-type", "-O0", "-o", (char *)emit_path, c_path, NULL};
    int rc = run_program_wait(argv);
    if (rc != 0) {
        fprintf(stderr, "error: clang direct emit failed with exit code %d\n", rc);
        return 1;
    }
    if (strcmp(out_path, "-") == 0) return copy_file_to_stdout(emit_path);
    return 0;
}

static int compile_to_stream(char *prepared) {
    fputs(HOST_INTRINSIC_IR, stdout);
    int64_t rc = compile(prepared);
    fflush(stdout);
    if (rc != 0) {
        fprintf(stderr, "error: self-host compiler exited %lld\n", (long long)rc);
        return 1;
    }
    return 0;
}

static int compile_to_file(char *prepared, const char *out_path) {
    FILE *out = fopen(out_path, "wb");
    if (out == NULL) {
        fprintf(stderr, "error: cannot write %s: %s\n", out_path, strerror(errno));
        return 1;
    }
    int saved = dup(STDOUT_FILENO);
    if (saved < 0) {
        fprintf(stderr, "error: cannot save stdout\n");
        fclose(out);
        return 1;
    }
    if (dup2(fileno(out), STDOUT_FILENO) < 0) {
        fprintf(stderr, "error: cannot redirect stdout\n");
        close(saved);
        fclose(out);
        return 1;
    }
    int rc = compile_to_stream(prepared);
    fflush(stdout);
    dup2(saved, STDOUT_FILENO);
    close(saved);
    fclose(out);
    return rc;
}

static int line_start_match(const char *text, const char *pos) {
    return pos == text || pos[-1] == '\n';
}

static int write_link_ir_entrypoint(const char *ir_path, const char *link_ir_path) {
    char *text = read_file(ir_path);
    if (text == NULL) return 1;
    const char *needle = "define i64 @main()";
    const char *replace = "define i64 @vais_user_main()";
    const char *target = NULL;
    int count = 0;
    const char *p = text;
    while ((p = strstr(p, needle)) != NULL) {
        if (line_start_match(text, p)) {
            target = p;
            count++;
        }
        p++;
    }
    if (count != 1 || target == NULL) {
        fprintf(stderr, "error: invalid Vais build IR: expected one @main, found %d\n", count);
        free(text);
        return 1;
    }

    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, text, (size_t)(target - text));
    sb_append(&out, replace);
    sb_append(&out, target + strlen(needle));
    char *rewritten = sb_take(&out);
    int rc = write_file_text(link_ir_path, rewritten);
    free(rewritten);
    free(text);
    return rc;
}

static int write_host_runtime_c(const char *path) {
    FILE *fp = fopen(path, "wb");
    if (fp == NULL) {
        fprintf(stderr, "error: cannot write %s: %s\n", path, strerror(errno));
        return 1;
    }
    fputs(
        "#include <errno.h>\n"
        "#include <fcntl.h>\n"
        "#include <stdint.h>\n"
        "#include <stdio.h>\n"
        "#include <string.h>\n"
        "#include <stdlib.h>\n"
        "#include <sys/stat.h>\n"
        "#include <sys/wait.h>\n"
        "#include <unistd.h>\n"
        "\n"
        "int64_t fs_exists(char *path) {\n"
        "    if (path == 0) return 0;\n"
        "    return access(path, F_OK) == 0 ? 1 : 0;\n"
        "}\n"
        "\n"
        "static void fs_host_trap(const char *op, const char *path) {\n"
        "    fprintf(stderr, \"vais host %s failed: %s: %s\\n\", op, path == 0 ? \"<null>\" : path, strerror(errno == 0 ? EIO : errno));\n"
        "    abort();\n"
        "}\n"
        "\n"
        "static char *fs_host_copy_n(const char *text, size_t len) {\n"
        "    char *out = (char *)malloc(len + 1);\n"
        "    if (out == 0) fs_host_trap(\"alloc\", text);\n"
        "    memcpy(out, text, len);\n"
        "    out[len] = '\\0';\n"
        "    return out;\n"
        "}\n"
        "\n"
        "static char *fs_host_copy(const char *text) {\n"
        "    if (text == 0) fs_host_trap(\"copy\", text);\n"
        "    return fs_host_copy_n(text, strlen(text));\n"
        "}\n"
        "\n"
        "static int vais_host_argc = 0;\n"
        "static char **vais_host_argv = 0;\n"
        "\n"
        "extern int64_t vais_user_main(void);\n"
        "\n"
        "int main(int argc, char **argv) {\n"
        "    if (argc > 0) {\n"
        "        vais_host_argc = argc - 1;\n"
        "        vais_host_argv = argv + 1;\n"
        "    } else {\n"
        "        vais_host_argc = 0;\n"
        "        vais_host_argv = argv;\n"
        "    }\n"
        "    return (int)vais_user_main();\n"
        "}\n"
        "\n"
        "int64_t proc_argc(void) {\n"
        "    if (vais_host_argv != 0) return (int64_t)vais_host_argc;\n"
        "    const char *raw = getenv(\"VAIS_RUN_ARGC\");\n"
        "    if (raw == 0 || raw[0] == '\\0') return 0;\n"
        "    char *end = 0;\n"
        "    long n = strtol(raw, &end, 10);\n"
        "    if (end == raw || n < 0) return 0;\n"
        "    return (int64_t)n;\n"
        "}\n"
        "\n"
        "char *proc_arg(int64_t index) {\n"
        "    int64_t count = proc_argc();\n"
        "    if (index < 0 || index >= count) fs_host_trap(\"proc_arg\", \"\");\n"
        "    if (vais_host_argv != 0) return fs_host_copy(vais_host_argv[index]);\n"
        "    char name[64];\n"
        "    snprintf(name, sizeof(name), \"VAIS_RUN_ARG_%lld\", (long long)index);\n"
        "    const char *value = getenv(name);\n"
        "    if (value == 0) fs_host_trap(\"proc_arg\", name);\n"
        "    return fs_host_copy(value);\n"
        "}\n"
        "\n"
        "static int path_is_absolute(const char *path) {\n"
        "    if (path == 0 || path[0] == '\\0') return 0;\n"
        "    return path[0] == '/';\n"
        "}\n"
        "\n"
        "char *fs_read_text(char *path) {\n"
        "    if (path == 0) fs_host_trap(\"read\", path);\n"
        "    FILE *fp = fopen(path, \"rb\");\n"
        "    if (fp == 0) fs_host_trap(\"read\", path);\n"
        "    if (fseek(fp, 0, SEEK_END) != 0) { fclose(fp); fs_host_trap(\"read\", path); }\n"
        "    long size = ftell(fp);\n"
        "    if (size < 0) { fclose(fp); fs_host_trap(\"read\", path); }\n"
        "    if (fseek(fp, 0, SEEK_SET) != 0) { fclose(fp); fs_host_trap(\"read\", path); }\n"
        "    char *buf = (char *)malloc((size_t)size + 1);\n"
        "    if (buf == 0) { fclose(fp); fs_host_trap(\"read\", path); }\n"
        "    size_t got = fread(buf, 1, (size_t)size, fp);\n"
        "    if (got != (size_t)size) { free(buf); fclose(fp); fs_host_trap(\"read\", path); }\n"
        "    buf[size] = '\\0';\n"
        "    if (fclose(fp) != 0) { free(buf); fs_host_trap(\"read\", path); }\n"
        "    return buf;\n"
        "}\n"
        "\n"
        "char *fs_cwd(void) {\n"
        "    char *cwd = getcwd(0, 0);\n"
        "    if (cwd == 0) fs_host_trap(\"cwd\", \"\");\n"
        "    return cwd;\n"
        "}\n"
        "\n"
        "char *fs_temp_dir(void) {\n"
        "    const char *tmp = getenv(\"TMPDIR\");\n"
        "    if (tmp == 0 || tmp[0] == '\\0') tmp = getenv(\"TEMP\");\n"
        "    if (tmp == 0 || tmp[0] == '\\0') tmp = getenv(\"TMP\");\n"
        "    if (tmp == 0 || tmp[0] == '\\0') tmp = \"/tmp\";\n"
        "    return fs_host_copy(tmp);\n"
        "}\n"
        "\n"
        "char *path_join(char *base, char *child) {\n"
        "    if (base == 0 || child == 0) fs_host_trap(\"path_join\", base == 0 ? base : child);\n"
        "    if (child[0] == '\\0') return fs_host_copy(base);\n"
        "    if (path_is_absolute(child)) return fs_host_copy(child);\n"
        "    size_t blen = strlen(base);\n"
        "    size_t clen = strlen(child);\n"
        "    if (blen == 0) return fs_host_copy(child);\n"
        "    int need_sep = base[blen - 1] != '/';\n"
        "    char *out = (char *)malloc(blen + (size_t)need_sep + clen + 1);\n"
        "    if (out == 0) fs_host_trap(\"path_join\", base);\n"
        "    memcpy(out, base, blen);\n"
        "    size_t pos = blen;\n"
        "    if (need_sep) {\n"
        "        out[pos] = '/';\n"
        "        pos = pos + 1;\n"
        "    }\n"
        "    memcpy(out + pos, child, clen);\n"
        "    out[pos + clen] = '\\0';\n"
        "    return out;\n"
        "}\n"
        "\n"
        "static size_t path_trim_trailing(const char *path, size_t len) {\n"
        "    while (len > 1 && path[len - 1] == '/') len = len - 1;\n"
        "    return len;\n"
        "}\n"
        "\n"
        "char *path_basename(char *path) {\n"
        "    if (path == 0) fs_host_trap(\"path_basename\", path);\n"
        "    size_t len = path_trim_trailing(path, strlen(path));\n"
        "    if (len == 0) return fs_host_copy(\"\");\n"
        "    if (len == 1 && path[0] == '/') return fs_host_copy_n(path, 1);\n"
        "    size_t start = len;\n"
        "    while (start > 0 && path[start - 1] != '/') start = start - 1;\n"
        "    return fs_host_copy_n(path + start, len - start);\n"
        "}\n"
        "\n"
        "char *path_dirname(char *path) {\n"
        "    if (path == 0) fs_host_trap(\"path_dirname\", path);\n"
        "    size_t len = path_trim_trailing(path, strlen(path));\n"
        "    if (len == 0) return fs_host_copy(\".\");\n"
        "    if (len == 1 && path[0] == '/') return fs_host_copy(\"/\");\n"
        "    size_t end = len;\n"
        "    while (end > 0 && path[end - 1] != '/') end = end - 1;\n"
        "    if (end == 0) return fs_host_copy(\".\");\n"
        "    while (end > 1 && path[end - 1] == '/') end = end - 1;\n"
        "    if (end == 0) return fs_host_copy(\".\");\n"
        "    return fs_host_copy_n(path, end);\n"
        "}\n"
        "\n"
        "char *str_concat(char *left, char *right) {\n"
        "    if (left == 0 || right == 0) fs_host_trap(\"str_concat\", left == 0 ? left : right);\n"
        "    size_t llen = strlen(left);\n"
        "    size_t rlen = strlen(right);\n"
        "    char *out = (char *)malloc(llen + rlen + 1);\n"
        "    if (out == 0) fs_host_trap(\"str_concat\", left);\n"
        "    memcpy(out, left, llen);\n"
        "    memcpy(out + llen, right, rlen);\n"
        "    out[llen + rlen] = '\\0';\n"
        "    return out;\n"
        "}\n"
        "\n"
        "char *str_slice(char *text, int64_t start, int64_t len) {\n"
        "    if (text == 0) fs_host_trap(\"str_slice\", text);\n"
        "    if (start < 0 || len < 0) fs_host_trap(\"str_slice\", text);\n"
        "    size_t n = strlen(text);\n"
        "    if ((size_t)start > n || (size_t)len > n - (size_t)start) fs_host_trap(\"str_slice\", text);\n"
        "    return fs_host_copy_n(text + start, (size_t)len);\n"
        "}\n"
        "\n"
        "char *str_byte(int64_t value) {\n"
        "    if (value < 0 || value > 255) fs_host_trap(\"str_byte\", \"\");\n"
        "    char tmp[2];\n"
        "    tmp[0] = (char)value;\n"
        "    tmp[1] = '\\0';\n"
        "    return fs_host_copy_n(tmp, 1);\n"
        "}\n"
        "\n"
        "typedef struct {\n"
        "    char *data;\n"
        "    size_t len;\n"
        "    size_t cap;\n"
        "} VaisStrBuilder;\n"
        "\n"
        "static VaisStrBuilder *str_builder_ptr(int64_t handle) {\n"
        "    if (handle == 0) fs_host_trap(\"str_builder\", \"\");\n"
        "    return (VaisStrBuilder *)(intptr_t)handle;\n"
        "}\n"
        "\n"
        "static void str_builder_reserve(VaisStrBuilder *builder, size_t extra) {\n"
        "    if (builder == 0) fs_host_trap(\"str_builder\", \"\");\n"
        "    if (extra > (size_t)-1 - builder->len - 1) fs_host_trap(\"str_builder\", \"\");\n"
        "    size_t need = builder->len + extra + 1;\n"
        "    if (need <= builder->cap) return;\n"
        "    size_t cap = builder->cap == 0 ? 64 : builder->cap;\n"
        "    while (cap < need) {\n"
        "        if (cap > (size_t)-1 / 2) {\n"
        "            cap = need;\n"
        "            break;\n"
        "        }\n"
        "        cap = cap * 2;\n"
        "    }\n"
        "    char *data = (char *)realloc(builder->data, cap);\n"
        "    if (data == 0) fs_host_trap(\"str_builder_alloc\", \"\");\n"
        "    builder->data = data;\n"
        "    builder->cap = cap;\n"
        "}\n"
        "\n"
        "int64_t str_builder_new(void) {\n"
        "    VaisStrBuilder *builder = (VaisStrBuilder *)calloc(1, sizeof(VaisStrBuilder));\n"
        "    if (builder == 0) fs_host_trap(\"str_builder_new\", \"\");\n"
        "    str_builder_reserve(builder, 0);\n"
        "    builder->data[0] = '\\0';\n"
        "    return (int64_t)(intptr_t)builder;\n"
        "}\n"
        "\n"
        "int64_t str_builder_push(int64_t handle, int64_t value) {\n"
        "    if (value < 0 || value > 255) fs_host_trap(\"str_builder_push\", \"\");\n"
        "    VaisStrBuilder *builder = str_builder_ptr(handle);\n"
        "    str_builder_reserve(builder, 1);\n"
        "    builder->data[builder->len] = (char)value;\n"
        "    builder->len += 1;\n"
        "    builder->data[builder->len] = '\\0';\n"
        "    return 0;\n"
        "}\n"
        "\n"
        "int64_t str_builder_append(int64_t handle, char *text) {\n"
        "    if (text == 0) fs_host_trap(\"str_builder_append\", \"\");\n"
        "    VaisStrBuilder *builder = str_builder_ptr(handle);\n"
        "    size_t len = strlen(text);\n"
        "    str_builder_reserve(builder, len);\n"
        "    memcpy(builder->data + builder->len, text, len);\n"
        "    builder->len += len;\n"
        "    builder->data[builder->len] = '\\0';\n"
        "    return 0;\n"
        "}\n"
        "\n"
        "char *str_builder_finish(int64_t handle) {\n"
        "    VaisStrBuilder *builder = str_builder_ptr(handle);\n"
        "    return fs_host_copy_n(builder->data, builder->len);\n"
        "}\n"
        "\n"
        "char *proc_capture_stdout(int64_t *argv_buf) {\n"
        "    if (argv_buf == 0) return fs_host_copy(\"\");\n"
        "    int64_t argc = argv_buf[4095];\n"
        "    if (argc <= 0 || argc > 4095) return fs_host_copy(\"\");\n"
        "    char **argv = (char **)malloc((size_t)(argc + 1) * sizeof(char *));\n"
        "    if (argv == 0) fs_host_trap(\"proc_capture_stdout\", \"argv\");\n"
        "    for (int64_t i = 0; i < argc; i++) {\n"
        "        argv[i] = (char *)(intptr_t)argv_buf[i];\n"
        "        if (argv[i] == 0) {\n"
        "            free(argv);\n"
        "            return fs_host_copy(\"\");\n"
        "        }\n"
        "    }\n"
        "    argv[argc] = 0;\n"
        "    int pipefd[2];\n"
        "    if (pipe(pipefd) != 0) {\n"
        "        free(argv);\n"
        "        fs_host_trap(\"proc_capture_stdout\", \"pipe\");\n"
        "    }\n"
        "    pid_t pid = fork();\n"
        "    if (pid < 0) {\n"
        "        close(pipefd[0]);\n"
        "        close(pipefd[1]);\n"
        "        free(argv);\n"
        "        fs_host_trap(\"proc_capture_stdout\", \"fork\");\n"
        "    }\n"
        "    if (pid == 0) {\n"
        "        close(pipefd[0]);\n"
        "        if (dup2(pipefd[1], STDOUT_FILENO) < 0) _exit(127);\n"
        "        close(pipefd[1]);\n"
        "        execvp(argv[0], argv);\n"
        "        fprintf(stderr, \"vais host proc_capture_stdout failed: %s: %s\\n\", argv[0], strerror(errno == 0 ? EIO : errno));\n"
        "        _exit(127);\n"
        "    }\n"
        "    close(pipefd[1]);\n"
        "    size_t len = 0;\n"
        "    size_t cap = 256;\n"
        "    char *out = (char *)malloc(cap);\n"
        "    if (out == 0) fs_host_trap(\"proc_capture_stdout\", \"alloc\");\n"
        "    char buf[512];\n"
        "    for (;;) {\n"
        "        ssize_t n = read(pipefd[0], buf, sizeof(buf));\n"
        "        if (n < 0 && errno == EINTR) continue;\n"
        "        if (n < 0) {\n"
        "            close(pipefd[0]);\n"
        "            free(out);\n"
        "            free(argv);\n"
        "            fs_host_trap(\"proc_capture_stdout\", \"read\");\n"
        "        }\n"
        "        if (n == 0) break;\n"
        "        if (len + (size_t)n + 1 > cap) {\n"
        "            while (len + (size_t)n + 1 > cap) cap = cap * 2;\n"
        "            char *next = (char *)realloc(out, cap);\n"
        "            if (next == 0) {\n"
        "                close(pipefd[0]);\n"
        "                free(out);\n"
        "                free(argv);\n"
        "                fs_host_trap(\"proc_capture_stdout\", \"alloc\");\n"
        "            }\n"
        "            out = next;\n"
        "        }\n"
        "        memcpy(out + len, buf, (size_t)n);\n"
        "        len = len + (size_t)n;\n"
        "    }\n"
        "    close(pipefd[0]);\n"
        "    int status = 0;\n"
        "    while (waitpid(pid, &status, 0) < 0) {\n"
        "        if (errno == EINTR) continue;\n"
        "        break;\n"
        "    }\n"
        "    free(argv);\n"
        "    out[len] = '\\0';\n"
        "    return out;\n"
        "}\n"
        "\n"
        "char *proc_capture_stderr(int64_t *argv_buf) {\n"
        "    if (argv_buf == 0) return fs_host_copy(\"\");\n"
        "    int64_t argc = argv_buf[4095];\n"
        "    if (argc <= 0 || argc > 4095) return fs_host_copy(\"\");\n"
        "    char **argv = (char **)malloc((size_t)(argc + 1) * sizeof(char *));\n"
        "    if (argv == 0) fs_host_trap(\"proc_capture_stderr\", \"argv\");\n"
        "    for (int64_t i = 0; i < argc; i++) {\n"
        "        argv[i] = (char *)(intptr_t)argv_buf[i];\n"
        "        if (argv[i] == 0) {\n"
        "            free(argv);\n"
        "            return fs_host_copy(\"\");\n"
        "        }\n"
        "    }\n"
        "    argv[argc] = 0;\n"
        "    int pipefd[2];\n"
        "    if (pipe(pipefd) != 0) {\n"
        "        free(argv);\n"
        "        fs_host_trap(\"proc_capture_stderr\", \"pipe\");\n"
        "    }\n"
        "    pid_t pid = fork();\n"
        "    if (pid < 0) {\n"
        "        close(pipefd[0]);\n"
        "        close(pipefd[1]);\n"
        "        free(argv);\n"
        "        fs_host_trap(\"proc_capture_stderr\", \"fork\");\n"
        "    }\n"
        "    if (pid == 0) {\n"
        "        close(pipefd[0]);\n"
        "        if (dup2(pipefd[1], STDERR_FILENO) < 0) _exit(127);\n"
        "        close(pipefd[1]);\n"
        "        execvp(argv[0], argv);\n"
        "        fprintf(stderr, \"vais host proc_capture_stderr failed: %s: %s\\n\", argv[0], strerror(errno == 0 ? EIO : errno));\n"
        "        _exit(127);\n"
        "    }\n"
        "    close(pipefd[1]);\n"
        "    size_t len = 0;\n"
        "    size_t cap = 256;\n"
        "    char *out = (char *)malloc(cap);\n"
        "    if (out == 0) fs_host_trap(\"proc_capture_stderr\", \"alloc\");\n"
        "    char buf[512];\n"
        "    for (;;) {\n"
        "        ssize_t n = read(pipefd[0], buf, sizeof(buf));\n"
        "        if (n < 0 && errno == EINTR) continue;\n"
        "        if (n < 0) {\n"
        "            close(pipefd[0]);\n"
        "            free(out);\n"
        "            free(argv);\n"
        "            fs_host_trap(\"proc_capture_stderr\", \"read\");\n"
        "        }\n"
        "        if (n == 0) break;\n"
        "        if (len + (size_t)n + 1 > cap) {\n"
        "            while (len + (size_t)n + 1 > cap) cap = cap * 2;\n"
        "            char *next = (char *)realloc(out, cap);\n"
        "            if (next == 0) {\n"
        "                close(pipefd[0]);\n"
        "                free(out);\n"
        "                free(argv);\n"
        "                fs_host_trap(\"proc_capture_stderr\", \"alloc\");\n"
        "            }\n"
        "            out = next;\n"
        "        }\n"
        "        memcpy(out + len, buf, (size_t)n);\n"
        "        len = len + (size_t)n;\n"
        "    }\n"
        "    close(pipefd[0]);\n"
        "    int status = 0;\n"
        "    while (waitpid(pid, &status, 0) < 0) {\n"
        "        if (errno == EINTR) continue;\n"
        "        break;\n"
        "    }\n"
        "    free(argv);\n"
        "    out[len] = '\\0';\n"
        "    return out;\n"
        "}\n"
        "\n"
        "static int proc_redirect_path(const char *path, int fd) {\n"
        "    if (path == 0 || path[0] == '\\0') return 0;\n"
        "    int out = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0666);\n"
        "    if (out < 0) return 1;\n"
        "    if (dup2(out, fd) < 0) { close(out); return 1; }\n"
        "    close(out);\n"
        "    return 0;\n"
        "}\n"
        "\n"
        "int64_t proc_capture_to(int64_t *argv_buf, char *stdout_path, char *stderr_path) {\n"
        "    if (argv_buf == 0) return 1;\n"
        "    int64_t argc = argv_buf[4095];\n"
        "    if (argc <= 0 || argc > 4095) return 1;\n"
        "    char **argv = (char **)malloc((size_t)(argc + 1) * sizeof(char *));\n"
        "    if (argv == 0) return errno == 0 ? 1 : errno;\n"
        "    for (int64_t i = 0; i < argc; i++) {\n"
        "        argv[i] = (char *)(intptr_t)argv_buf[i];\n"
        "        if (argv[i] == 0) {\n"
        "            free(argv);\n"
        "            return 1;\n"
        "        }\n"
        "    }\n"
        "    argv[argc] = 0;\n"
        "    pid_t pid = fork();\n"
        "    if (pid < 0) {\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        free(argv);\n"
        "        return err;\n"
        "    }\n"
        "    if (pid == 0) {\n"
        "        if (proc_redirect_path(stdout_path, STDOUT_FILENO) != 0) _exit(127);\n"
        "        if (proc_redirect_path(stderr_path, STDERR_FILENO) != 0) _exit(127);\n"
        "        execvp(argv[0], argv);\n"
        "        fprintf(stderr, \"vais host proc_capture_to failed: %s: %s\\n\", argv[0], strerror(errno == 0 ? EIO : errno));\n"
        "        _exit(127);\n"
        "    }\n"
        "    int status = 0;\n"
        "    while (waitpid(pid, &status, 0) < 0) {\n"
        "        if (errno == EINTR) continue;\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        free(argv);\n"
        "        return err;\n"
        "    }\n"
        "    free(argv);\n"
        "    if (WIFEXITED(status)) return WEXITSTATUS(status);\n"
        "    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);\n"
        "    return 1;\n"
        "}\n"
        "\n"
        "static int proc_apply_env(int64_t *env_buf) {\n"
        "    if (env_buf == 0) return 0;\n"
        "    int64_t envc = env_buf[4095];\n"
        "    if (envc < 0 || envc > 4095) return 1;\n"
        "    for (int64_t i = 0; i < envc; i++) {\n"
        "        char *entry = (char *)(intptr_t)env_buf[i];\n"
        "        if (entry == 0) return 1;\n"
        "        char *eq = strchr(entry, '=');\n"
        "        if (eq == 0 || eq == entry) return 1;\n"
        "        size_t key_len = (size_t)(eq - entry);\n"
        "        char *key = (char *)malloc(key_len + 1);\n"
        "        if (key == 0) return 1;\n"
        "        memcpy(key, entry, key_len);\n"
        "        key[key_len] = '\\0';\n"
        "        if (setenv(key, eq + 1, 1) != 0) {\n"
        "            free(key);\n"
        "            return 1;\n"
        "        }\n"
        "        free(key);\n"
        "    }\n"
        "    return 0;\n"
        "}\n"
        "\n"
        "int64_t proc_run(int64_t *argv_buf) {\n"
        "    if (argv_buf == 0) return 1;\n"
        "    int64_t argc = argv_buf[4095];\n"
        "    if (argc <= 0 || argc > 4095) return 1;\n"
        "    char **argv = (char **)malloc((size_t)(argc + 1) * sizeof(char *));\n"
        "    if (argv == 0) return errno == 0 ? 1 : errno;\n"
        "    for (int64_t i = 0; i < argc; i++) {\n"
        "        argv[i] = (char *)(intptr_t)argv_buf[i];\n"
        "        if (argv[i] == 0) {\n"
        "            free(argv);\n"
        "            return 1;\n"
        "        }\n"
        "    }\n"
        "    argv[argc] = 0;\n"
        "    pid_t pid = fork();\n"
        "    if (pid < 0) {\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        free(argv);\n"
        "        return err;\n"
        "    }\n"
        "    if (pid == 0) {\n"
        "        execvp(argv[0], argv);\n"
        "        fprintf(stderr, \"vais host proc_run failed: %s: %s\\n\", argv[0], strerror(errno == 0 ? EIO : errno));\n"
        "        _exit(127);\n"
        "    }\n"
        "    int status = 0;\n"
        "    while (waitpid(pid, &status, 0) < 0) {\n"
        "        if (errno == EINTR) continue;\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        free(argv);\n"
        "        return err;\n"
        "    }\n"
        "    free(argv);\n"
        "    if (WIFEXITED(status)) return WEXITSTATUS(status);\n"
        "    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);\n"
        "    return 1;\n"
        "}\n"
        "\n"
        "int64_t proc_run_env(int64_t *argv_buf, int64_t *env_buf) {\n"
        "    if (argv_buf == 0) return 1;\n"
        "    int64_t argc = argv_buf[4095];\n"
        "    if (argc <= 0 || argc > 4095) return 1;\n"
        "    char **argv = (char **)malloc((size_t)(argc + 1) * sizeof(char *));\n"
        "    if (argv == 0) return errno == 0 ? 1 : errno;\n"
        "    for (int64_t i = 0; i < argc; i++) {\n"
        "        argv[i] = (char *)(intptr_t)argv_buf[i];\n"
        "        if (argv[i] == 0) {\n"
        "            free(argv);\n"
        "            return 1;\n"
        "        }\n"
        "    }\n"
        "    argv[argc] = 0;\n"
        "    pid_t pid = fork();\n"
        "    if (pid < 0) {\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        free(argv);\n"
        "        return err;\n"
        "    }\n"
        "    if (pid == 0) {\n"
        "        if (proc_apply_env(env_buf) != 0) _exit(127);\n"
        "        execvp(argv[0], argv);\n"
        "        fprintf(stderr, \"vais host proc_run_env failed: %s: %s\\n\", argv[0], strerror(errno == 0 ? EIO : errno));\n"
        "        _exit(127);\n"
        "    }\n"
        "    int status = 0;\n"
        "    while (waitpid(pid, &status, 0) < 0) {\n"
        "        if (errno == EINTR) continue;\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        free(argv);\n"
        "        return err;\n"
        "    }\n"
        "    free(argv);\n"
        "    if (WIFEXITED(status)) return WEXITSTATUS(status);\n"
        "    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);\n"
        "    return 1;\n"
        "}\n"
        "\n"
        "int64_t fs_write_text(char *path, char *text) {\n"
        "    if (path == 0 || text == 0) return 1;\n"
        "    FILE *fp = fopen(path, \"wb\");\n"
        "    if (fp == 0) return errno == 0 ? 1 : errno;\n"
        "    if (fputs(text, fp) < 0) {\n"
        "        int err = errno == 0 ? 1 : errno;\n"
        "        fclose(fp);\n"
        "        return err;\n"
        "    }\n"
        "    if (fclose(fp) != 0) return errno == 0 ? 1 : errno;\n"
        "    return 0;\n"
        "}\n"
        "\n"
        "static int64_t fs_mkdir_one(const char *path) {\n"
        "    if (path == 0 || path[0] == '\\0') return 0;\n"
        "    if (mkdir(path, 0777) == 0) return 0;\n"
        "    if (errno == EEXIST) {\n"
        "        struct stat st;\n"
        "        if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;\n"
        "        return ENOTDIR;\n"
        "    }\n"
        "    return errno == 0 ? 1 : errno;\n"
        "}\n"
        "\n"
        "int64_t fs_mkdirs(char *path) {\n"
        "    if (path == 0 || path[0] == '\\0') return 1;\n"
        "    char buf[4096];\n"
        "    size_t len = strlen(path);\n"
        "    if (len >= sizeof(buf)) return ENAMETOOLONG;\n"
        "    memcpy(buf, path, len + 1);\n"
        "    while (len > 1 && buf[len - 1] == '/') {\n"
        "        buf[len - 1] = '\\0';\n"
        "        len = len - 1;\n"
        "    }\n"
        "    for (char *p = buf + 1; *p != '\\0'; p++) {\n"
        "        if (*p == '/') {\n"
        "            *p = '\\0';\n"
        "            int64_t rc = fs_mkdir_one(buf);\n"
        "            *p = '/';\n"
        "            if (rc != 0) return rc;\n"
        "        }\n"
        "    }\n"
        "    return fs_mkdir_one(buf);\n"
        "}\n"
        "\n"
        "int64_t fs_remove(char *path) {\n"
        "    if (path == 0 || path[0] == '\\0') return 1;\n"
        "    if (unlink(path) == 0) return 0;\n"
        "    if (errno == ENOENT) return 0;\n"
        "    return errno == 0 ? 1 : errno;\n"
        "}\n",
        fp
    );
    if (fclose(fp) != 0) {
        fprintf(stderr, "error: cannot close %s: %s\n", path, strerror(errno));
        return 1;
    }
    return 0;
}

static int run_program_wait(char *const argv[]) {
    pid_t pid = fork();
    if (pid < 0) {
        fprintf(stderr, "error: fork failed: %s\n", strerror(errno));
        return 1;
    }
    if (pid == 0) {
        execvp(argv[0], argv);
        fprintf(stderr, "error: cannot exec %s: %s\n", argv[0], strerror(errno));
        _exit(127);
    }
    int status = 0;
    if (waitpid(pid, &status, 0) < 0) {
        fprintf(stderr, "error: waitpid failed: %s\n", strerror(errno));
        return 1;
    }
    if (WIFEXITED(status)) return WEXITSTATUS(status);
    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
    return 1;
}

static int clang_build(const char *clang, const char *ir_path, const char *out_path) {
    char runtime_path[512];
    char link_ir_path[512];
    if (make_tmp_path(runtime_path, sizeof(runtime_path), "host_runtime.c") != 0) return 1;
    if (make_tmp_path(link_ir_path, sizeof(link_ir_path), "link.ll") != 0) return 1;
    if (write_link_ir_entrypoint(ir_path, link_ir_path) != 0) return 1;
    if (write_host_runtime_c(runtime_path) != 0) return 1;
    char *const argv[] = {
        (char *)clang,
        "-Wno-override-module",
        "-o",
        (char *)out_path,
        link_ir_path,
        runtime_path,
        NULL
    };
    int rc = run_program_wait(argv);
    if (rc != 0) {
        fprintf(stderr, "error: clang failed with exit code %d\n", rc);
        return 1;
    }
    return 0;
}

static int make_tmp_path(char *buf, size_t buflen, const char *suffix) {
    char tmpl[] = "/tmp/vaisc-native-XXXXXX";
    char *dir = mkdtemp(tmpl);
    if (dir == NULL) {
        fprintf(stderr, "error: mkdtemp failed: %s\n", strerror(errno));
        return 1;
    }
    if (snprintf(buf, buflen, "%s/%s", dir, suffix) >= (int)buflen) {
        fprintf(stderr, "error: temporary path too long\n");
        return 1;
    }
    return 0;
}

static void print_help(void) {
    printf("Vais compiler %s\n", VAIS_VERSION);
    printf("usage:\n");
    printf("  vaisc emit-ir <source.vais> [-o out.ll] [--engine full|direct]\n");
    printf("  vaisc build <source.vais> -o out [--ir-out out.ll] [--clang clang] [--engine full|direct]\n");
    printf("  vaisc run <source.vais> [--clang clang] [--engine full|direct]\n");
    printf("  vaisc doctor\n");
    printf("  vaisc --version\n");
}

static int command_doctor(void) {
    printf("vaisc %s\n", VAIS_VERSION);
    fflush(stdout);
    const char *clang = getenv("CLANG");
    if (clang == NULL || clang[0] == '\0') clang = "clang";
    char *const argv[] = {(char *)clang, "--version", NULL};
    int rc = run_program_wait(argv);
    if (rc != 0) {
        fprintf(stderr, "error: clang is required on PATH or pass --clang to build/run\n");
        return 1;
    }
    printf("self-host core: linked\n");
    return 0;
}

static int command_emit_ir(int argc, char **argv) {
    const char *source = NULL;
    const char *output = "-";
    const char *engine = "full";
    const char *clang = getenv("CLANG");
    if (clang == NULL || clang[0] == '\0') clang = "clang";
    for (int i = 2; i < argc; i++) {
        if ((strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "--output") == 0) && i + 1 < argc) {
            output = argv[++i];
        } else if (strcmp(argv[i], "--clang") == 0 && i + 1 < argc) {
            clang = argv[++i];
        } else if (strcmp(argv[i], "--engine") == 0 && i + 1 < argc) {
            engine = argv[++i];
        } else if (starts_with(argv[i], "--engine=")) {
            engine = argv[i] + 9;
        } else if (strcmp(argv[i], "--keep-tmp") == 0) {
            continue;
        } else if (source == NULL) {
            source = argv[i];
        } else {
            fprintf(stderr, "error: unexpected argument: %s\n", argv[i]);
            return 1;
        }
    }
    if (source == NULL) {
        fprintf(stderr, "error: emit-ir needs a source path\n");
        return 1;
    }
    if (strcmp(engine, "direct") == 0) return direct_emit_ir_file(source, output, clang);
    if (strcmp(engine, "full") != 0) {
        fprintf(stderr, "error: unknown engine: %s\n", engine);
        return 1;
    }
    char *prepared = prepare_source_file(source);
    if (prepared == NULL) return 1;
    int rc = strcmp(output, "-") == 0 ? compile_to_stream(prepared) : compile_to_file(prepared, output);
    free(prepared);
    return rc;
}

static int command_build(int argc, char **argv) {
    const char *source = NULL;
    const char *output = NULL;
    const char *ir_out = NULL;
    const char *engine = "full";
    const char *clang = getenv("CLANG");
    if (clang == NULL || clang[0] == '\0') clang = "clang";
    for (int i = 2; i < argc; i++) {
        if ((strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "--output") == 0) && i + 1 < argc) {
            output = argv[++i];
        } else if (strcmp(argv[i], "--ir-out") == 0 && i + 1 < argc) {
            ir_out = argv[++i];
        } else if (strcmp(argv[i], "--clang") == 0 && i + 1 < argc) {
            clang = argv[++i];
        } else if (strcmp(argv[i], "--engine") == 0 && i + 1 < argc) {
            engine = argv[++i];
        } else if (starts_with(argv[i], "--engine=")) {
            engine = argv[i] + 9;
        } else if (strcmp(argv[i], "--keep-tmp") == 0) {
            continue;
        } else if (source == NULL) {
            source = argv[i];
        } else {
            fprintf(stderr, "error: unexpected argument: %s\n", argv[i]);
            return 1;
        }
    }
    if (source == NULL || output == NULL) {
        fprintf(stderr, "error: build needs <source.vais> and -o <out>\n");
        return 1;
    }
    char tmp_ir[512];
    const char *ir_path = ir_out;
    if (ir_path == NULL) {
        if (make_tmp_path(tmp_ir, sizeof(tmp_ir), "out.ll") != 0) return 1;
        ir_path = tmp_ir;
    }
    int rc = 0;
    if (strcmp(engine, "direct") == 0) {
        rc = direct_emit_ir_file(source, ir_path, clang);
    } else if (strcmp(engine, "full") == 0) {
        char *prepared = prepare_source_file(source);
        if (prepared == NULL) return 1;
        rc = compile_to_file(prepared, ir_path);
        free(prepared);
    } else {
        fprintf(stderr, "error: unknown engine: %s\n", engine);
        return 1;
    }
    if (rc != 0) return rc;
    return clang_build(clang, ir_path, output);
}

static int command_run(int argc, char **argv) {
    const char *source = NULL;
    const char *engine = "full";
    const char *clang = getenv("CLANG");
    const char *program_args[256];
    int program_argc = 0;
    if (clang == NULL || clang[0] == '\0') clang = "clang";
    for (int i = 2; i < argc; i++) {
        if (strcmp(argv[i], "--") == 0) {
            for (int j = i + 1; j < argc; j++) {
                if (program_argc >= 256) {
                    fprintf(stderr, "error: too many program arguments\n");
                    return 1;
                }
                program_args[program_argc++] = argv[j];
            }
            break;
        } else if (strcmp(argv[i], "--clang") == 0 && i + 1 < argc) {
            clang = argv[++i];
        } else if (strcmp(argv[i], "--engine") == 0 && i + 1 < argc) {
            engine = argv[++i];
        } else if (starts_with(argv[i], "--engine=")) {
            engine = argv[i] + 9;
        } else if (strcmp(argv[i], "--keep-tmp") == 0) {
            continue;
        } else if (source == NULL) {
            source = argv[i];
        } else {
            fprintf(stderr, "error: unexpected argument: %s\n", argv[i]);
            return 1;
        }
    }
    if (source == NULL) {
        fprintf(stderr, "error: run needs a source path\n");
        return 1;
    }
    char bin_path[512];
    if (make_tmp_path(bin_path, sizeof(bin_path), "a.out") != 0) return 1;
    char *build_argv[] = {"vaisc", "build", (char *)source, "-o", bin_path, "--clang", (char *)clang, "--engine", (char *)engine, NULL};
    int build_argc = 9;
    int rc = command_build(build_argc, build_argv);
    if (rc != 0) return rc;
    char *run_argv[258];
    run_argv[0] = bin_path;
    for (int i = 0; i < program_argc; i++) {
        run_argv[i + 1] = (char *)program_args[i];
    }
    run_argv[program_argc + 1] = NULL;
    return run_program_wait(run_argv);
}

int main(int argc, char **argv) {
    if (argc < 2 || strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0) {
        print_help();
        return argc < 2 ? 1 : 0;
    }
    if (strcmp(argv[1], "--version") == 0 || strcmp(argv[1], "version") == 0) {
        printf("vaisc %s\n", VAIS_VERSION);
        return 0;
    }
    if (strcmp(argv[1], "doctor") == 0) return command_doctor();
    if (strcmp(argv[1], "emit-ir") == 0) return command_emit_ir(argc, argv);
    if (strcmp(argv[1], "build") == 0) return command_build(argc, argv);
    if (strcmp(argv[1], "run") == 0) return command_run(argc, argv);
    fprintf(stderr, "error: unknown command: %s\n", argv[1]);
    print_help();
    return 1;
}
