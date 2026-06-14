#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

#define VAIS_VERSION "0.2.0"

extern int64_t compile(char *src);

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
    int tag;
    int field_count;
} VariantInfo;

typedef struct {
    char *name;
    int is_payload;
    int count;
    VariantInfo variants[16];
} EnumInfo;

typedef struct {
    char *maker;
    char *apply;
} ClosureMaker;

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
        if (delim == '"' && ch == '\\') {
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
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(name_start - line));
    sb_append_n(&out, name_start, (size_t)(p - name_start));
    if (*q == ',') sb_append(&out, ",");
    return out.data;
}

static char *replace_print_token(const char *line) {
    const char *p = strstr(line, "print(");
    if (p == NULL) return strdup(line);
    StrBuf out;
    sb_init(&out);
    sb_append_n(&out, line, (size_t)(p - line));
    sb_append(&out, "puts(");
    sb_append(&out, p + 6);
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

static int split_top_level_commas_c(const char *text, char **parts, int max_parts) {
    int count = 0;
    int depth = 0;
    const char *start = text;
    for (const char *p = text; ; p++) {
        char ch = *p;
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

static VariantInfo *find_variant(EnumInfo *info, const char *name) {
    for (int i = 0; i < info->count; i++) {
        if (strcmp(info->variants[i].name, name) == 0) return &info->variants[i];
    }
    return NULL;
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
                    if (strcmp(field_parts[k], "Int") != 0 && strcmp(field_parts[k], info->name) != 0) {
                        fields = -1;
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
    memset(info, 0, sizeof(*info));
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

static char *replace_enum_types(const char *line, EnumInfo *info) {
    char needle[160];
    snprintf(needle, sizeof(needle), ": %s", info->name);
    char *step = replace_exact(line, needle, ": Int");
    snprintf(needle, sizeof(needle), "-> %s", info->name);
    char *out = replace_exact(step, needle, "-> Int");
    free(step);
    return out;
}

static int find_matching_paren_c(const char *text, int open_index) {
    int depth = 0;
    for (int i = open_index; text[i] != '\0'; i++) {
        if (text[i] == '(') depth++;
        if (text[i] == ')') {
            depth--;
            if (depth == 0) return i;
        }
    }
    return -1;
}

static char *rewrite_constructors(const char *line, EnumInfo *info);

static char *encode_payload_call(VariantInfo *variant, char **args, int argc, EnumInfo *info) {
    if (argc != variant->field_count) return strdup("0");
    StrBuf payload;
    sb_init(&payload);
    for (int i = 0; i < argc; i++) {
        char *rewritten = rewrite_constructors(args[i], info);
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

static int parse_arm(const char *line, char **pattern, char **expr) {
    const char *arrow = strstr(line, "=>");
    if (arrow == NULL) return 0;
    char *pat_raw = substr_copy(line, (size_t)(arrow - line));
    *pattern = trim_copy(pat_raw);
    free(pat_raw);
    const char *body = skip_ws(arrow + 2);
    if (!starts_with(body, "return ")) return 0;
    body += 7;
    *expr = trim_copy(body);
    return 1;
}

static char *lower_enum_text(const char *text) {
    LineVec lines = split_lines(text);
    EnumInfo info;
    memset(&info, 0, sizeof(info));
    int enum_line = -1;
    for (size_t i = 0; i < lines.len; i++) {
        if (parse_enum_line(lines.items[i], &info)) {
            enum_line = (int)i;
            break;
        }
    }
    if (enum_line < 0) {
        lines_free(&lines);
        return strdup(text);
    }

    LineVec out;
    lines_init(&out);
    for (size_t i = 0; i < lines.len; i++) {
        if ((int)i == enum_line) continue;
        char *typed = replace_enum_types(lines.items[i], &info);
        char *match_expr = parse_match_expr(typed);
        if (match_expr == NULL) {
            char *rewritten = rewrite_constructors(typed, &info);
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
            char *rewritten_expr = rewrite_constructors(expr, &info);
            char *rewritten_pattern = rewrite_constructors(pattern, &info);
            VariantInfo *variant = NULL;
            char *binders[8] = {0};
            int binder_count = 0;
            if (info.is_payload) {
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
            StrBuf b;
            sb_init(&b);
            sb_append(&b, arm_index == 0 ? "    if " : "    else if ");
            if (info.is_payload && variant != NULL) {
                char tag_text[64];
                snprintf(tag_text, sizeof(tag_text), "%% %d == %d {", info.count, variant->tag);
                sb_append(&b, match_expr);
                sb_append(&b, " ");
                sb_append(&b, tag_text);
            } else {
                sb_append(&b, match_expr);
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

    char *joined = join_lines(&out, text[strlen(text) - 1] == '\n');
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

        if (!in_struct && starts_with(trim, "struct ") && strchr(trim, '{') != NULL && strchr(trim, '}') == NULL) {
            in_struct = 1;
            struct_depth = 1;
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

static int is_valid_int_params(const char *params) {
    char *copy = strdup(params);
    if (copy == NULL) die_oom();
    char *parts[16] = {0};
    int n = split_top_level_commas_c(copy, parts, 16);
    free(copy);
    if (n < 0) return 0;
    for (int i = 0; i < n; i++) {
        char *colon = strchr(parts[i], ':');
        if (colon == NULL) {
            for (int k = 0; k < n; k++) free(parts[k]);
            return 0;
        }
        char *ty = trim_copy(colon + 1);
        int ok = strcmp(ty, "Int") == 0;
        free(ty);
        if (!ok) {
            for (int k = 0; k < n; k++) free(parts[k]);
            return 0;
        }
    }
    for (int i = 0; i < n; i++) free(parts[i]);
    return 1;
}

static int check_fn_contract_line(
    const char *path,
    int line_no,
    const char *line,
    int *has_main,
    int *has_bad_main
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
    char *ret = NULL;
    if (arrow != NULL) {
        const char *r = skip_ws(arrow + 2);
        const char *rend = r;
        while (is_ident_continue(*rend)) rend++;
        ret = substr_copy(r, (size_t)(rend - r));
    }
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
        if (ret == NULL || strcmp(ret, "Int") != 0) {
            report_issue(path, line_no, find_col(line, "fn "), line,
                "Vais native day-1 helper functions must return `Int`",
                "write helpers as `fn name(a: Int, ...) -> Int { ... }`.",
                NULL);
            issue = 1;
        }
        if (params != NULL && strlen(skip_ws(params)) > 0 && !is_valid_int_params(params)) {
            report_issue(path, line_no, find_col(line, params), line,
                "Vais native day-1 helper parameters must be `name: Int`",
                "use Int-typed helper parameters in this slice, e.g. `fn add(a: Int, b: Int) -> Int`.",
                NULL);
            issue = 1;
        }
    }
    free(name);
    free(ret);
    free(params);
    return issue;
}

static int check_front_contract_text(const char *text, const char *path) {
    LineVec lines = split_lines(text);
    int issues = 0;
    int has_main = 0;
    int has_bad_main = 0;
    for (size_t i = 0; i < lines.len; i++) {
        const char *line = lines.items[i];
        int line_no = (int)i + 1;
        issues += check_fn_contract_line(path, line_no, line, &has_main, &has_bad_main);

        char *fix = NULL;
        if (strstr(line, "&&") != NULL) {
            fix = replace_once_for_fix(line, "&&", "and");
            report_issue(path, line_no, find_col(line, "&&"), line,
                "logical AND uses the word `and`, not `&&`",
                "replace `&&` with `and`.", fix);
            free(fix);
            issues++;
        } else if (strstr(line, "||") != NULL) {
            fix = replace_once_for_fix(line, "||", "or");
            report_issue(path, line_no, find_col(line, "||"), line,
                "logical OR uses the word `or`, not `||`",
                "replace `||` with `or`.", fix);
            free(fix);
            issues++;
        } else if (strstr(line, " as Int") != NULL) {
            fix = fix_as_cast(line);
            report_issue(path, line_no, find_col(line, " as Int"), line,
                "type conversion is explicit `Type(x)`, not `x as Type`",
                "write `Type(expr)` instead of `expr as Type`.", fix);
            free(fix);
            issues++;
        } else if (strstr(line, "Vec<") != NULL && strstr(line, "::new") != NULL) {
            fix = replace_once_for_fix(line, "Vec<Int>::new()", "[]");
            report_issue(path, line_no, find_col(line, "Vec<"), line,
                "no turbofish constructor; use a literal instead of `Type<...>::new()`",
                "use a list/map literal such as `[]`, `[1, 2]`, or `{}`.", fix);
            free(fix);
            issues++;
        } else if (strstr(line, "::") != NULL) {
            fix = replace_once_for_fix(line, "::", ".");
            report_issue(path, line_no, find_col(line, "::"), line,
                "enum/path access uses `.`, not `::`",
                "replace `::` with `.`.", fix);
            free(fix);
            issues++;
        } else if (strstr(line, "i32") != NULL) {
            fix = replace_once_for_fix(line, "i32", "Int");
            report_issue(path, line_no, find_col(line, "i32"), line,
                "Vais scalar types are capitalized, not Rust scalar names",
                "use `Int` for the verified release scalar type.", fix);
            free(fix);
            issues++;
        } else if (strstr(line, "enum ") != NULL) {
            report_issue(path, line_no, find_col(line, "enum"), line,
                "enum declarations beyond payload-free tags or small Int-coded payload enums are not in the Vais native front subset yet",
                "use payload-free enum tags or Int/self-recursive payload enums with simple return-arm match; keep broader payload enums on the full compiler path.",
                NULL);
            issues++;
        } else if (strstr(line, "match ") != NULL) {
            report_issue(path, line_no, find_col(line, "match"), line,
                "`match` beyond simple enum return arms is not in the Vais native front subset yet",
                "use if/else for native sources, or keep payload match code on the full compiler path.",
                NULL);
            issues++;
        } else if (strstr(line, "for ") != NULL) {
            report_issue(path, line_no, find_col(line, "for"), line,
                "`for` loops are not in the Vais native day-1 front subset yet",
                "use `while` with an explicit mutable index for now.",
                NULL);
            issues++;
        } else if (strstr(line, "Str") != NULL || strstr(line, "Char") != NULL || strstr(line, "Bool") != NULL || strstr(line, "String") != NULL) {
            report_issue(path, line_no, 1, line,
                "only Int scalar typing is in the Vais native day-1 front subset",
                "use Int parameters/locals for this slice; string, char, and bool surface types come later.",
                NULL);
            issues++;
        } else if (strstr(line, "|") != NULL) {
            report_issue(path, line_no, find_col(line, "|"), line,
                "closures beyond the single-Int closure-return slice are not in the Vais native front subset yet",
                "use a single Int capture returning `fn(Int) -> Int`, or write a named function for broader closure cases.",
                NULL);
            issues++;
        } else if (strstr(line, ".clear(") != NULL) {
            report_issue(path, line_no, find_col(line, ".clear("), line,
                "method calls beyond push/len/sum are not in the Vais native front subset yet",
                "use a plain function call, or keep this source on the full compiler path until that method is promoted.",
                NULL);
            issues++;
        }
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
    lines_free(&lines);
    return issues == 0 ? 0 : 1;
}

static char *prepare_source_file(const char *path) {
    if (!has_vais_suffix(path)) {
        fprintf(stderr, "error: Vais source files must use the .vais extension: %s\n", path);
        return NULL;
    }
    char *raw = read_file(path);
    if (raw == NULL) return NULL;
    char *normalized = normalize_source_text(raw, 0);
    char *enum_lowered = lower_enum_text(normalized);
    char *closure_lowered = lower_closure_text(enum_lowered);
    if (check_front_contract_text(closure_lowered, path) != 0) {
        free(raw);
        free(normalized);
        free(enum_lowered);
        free(closure_lowered);
        return NULL;
    }
    char *prepared = normalize_source_text(closure_lowered, 1);
    free(raw);
    free(normalized);
    free(enum_lowered);
    free(closure_lowered);
    return prepared;
}

static int compile_to_stream(char *prepared) {
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
    char *const argv[] = {(char *)clang, "-Wno-override-module", "-o", (char *)out_path, (char *)ir_path, NULL};
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
    printf("  vaisc emit-ir <source.vais> [-o out.ll]\n");
    printf("  vaisc build <source.vais> -o out [--ir-out out.ll] [--clang clang]\n");
    printf("  vaisc run <source.vais> [--clang clang]\n");
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
    for (int i = 2; i < argc; i++) {
        if ((strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "--output") == 0) && i + 1 < argc) {
            output = argv[++i];
        } else if (strcmp(argv[i], "--engine") == 0 && i + 1 < argc) {
            const char *engine = argv[++i];
            if (strcmp(engine, "full") != 0) {
                fprintf(stderr, "error: native vaisc currently supports --engine full only\n");
                return 1;
            }
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
            const char *engine = argv[++i];
            if (strcmp(engine, "full") != 0) {
                fprintf(stderr, "error: native vaisc currently supports --engine full only\n");
                return 1;
            }
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
    char *prepared = prepare_source_file(source);
    if (prepared == NULL) return 1;
    int rc = compile_to_file(prepared, ir_path);
    free(prepared);
    if (rc != 0) return rc;
    return clang_build(clang, ir_path, output);
}

static int command_run(int argc, char **argv) {
    const char *source = NULL;
    const char *clang = getenv("CLANG");
    if (clang == NULL || clang[0] == '\0') clang = "clang";
    for (int i = 2; i < argc; i++) {
        if (strcmp(argv[i], "--clang") == 0 && i + 1 < argc) {
            clang = argv[++i];
        } else if (strcmp(argv[i], "--engine") == 0 && i + 1 < argc) {
            const char *engine = argv[++i];
            if (strcmp(engine, "full") != 0) {
                fprintf(stderr, "error: native vaisc currently supports --engine full only\n");
                return 1;
            }
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
    char *build_argv[] = {"vaisc", "build", (char *)source, "-o", bin_path, "--clang", (char *)clang, NULL};
    int build_argc = 7;
    int rc = command_build(build_argc, build_argv);
    if (rc != 0) return rc;
    char *const run_argv[] = {bin_path, NULL};
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
