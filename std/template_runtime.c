// Template engine runtime support for Vais
// Provides template parsing, rendering, context management, and HTML escaping
// for the std/template.vais standard library module.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

// ============================================
// Constants
// ============================================

#define TPL_MAX_NODES     256
#define TPL_MAX_VARS      64
#define TPL_BUFFER_SIZE   65536
#define TPL_KEY_MAX       128
#define TPL_VAL_MAX       4096
#define TPL_MAX_PARTIALS  32

// Node types
#define NODE_TEXT    0
#define NODE_VAR    1
#define NODE_IF     2
#define NODE_FOR    3
#define NODE_INCLUDE 4
#define NODE_FILTER  5

// ============================================
// Template Context (key-value store)
// ============================================

typedef struct {
    char* key;
    char* value;
} CtxEntry;

typedef struct {
    CtxEntry entries[TPL_MAX_VARS];
    int count;
} TemplateCtx;

// Create a new context
long __template_ctx_new(void) {
    TemplateCtx* ctx = (TemplateCtx*)calloc(1, sizeof(TemplateCtx));
    if (!ctx) return 0;
    ctx->count = 0;
    return (long)ctx;
}

// Set a key-value pair in the context
long __template_ctx_set(long ctx_handle, const char* key, const char* value) {
    if (ctx_handle == 0 || key == NULL) return 0;
    TemplateCtx* ctx = (TemplateCtx*)ctx_handle;

    // Check if key already exists, update if so
    for (int i = 0; i < ctx->count; i++) {
        if (ctx->entries[i].key && strcmp(ctx->entries[i].key, key) == 0) {
            free(ctx->entries[i].value);
            ctx->entries[i].value = value ? strdup(value) : strdup("");
            return 1;
        }
    }

    // Add new entry
    if (ctx->count >= TPL_MAX_VARS) return 0;

    ctx->entries[ctx->count].key = strdup(key);
    ctx->entries[ctx->count].value = value ? strdup(value) : strdup("");
    ctx->count++;
    return 1;
}

// Get a value by key from the context
const char* __template_ctx_get(long ctx_handle, const char* key) {
    if (ctx_handle == 0 || key == NULL) return NULL;
    TemplateCtx* ctx = (TemplateCtx*)ctx_handle;

    for (int i = 0; i < ctx->count; i++) {
        if (ctx->entries[i].key && strcmp(ctx->entries[i].key, key) == 0) {
            return ctx->entries[i].value;
        }
    }
    return NULL;
}

// Free the context
long __template_ctx_free(long ctx_handle) {
    if (ctx_handle == 0) return 0;
    TemplateCtx* ctx = (TemplateCtx*)ctx_handle;

    for (int i = 0; i < ctx->count; i++) {
        free(ctx->entries[i].key);
        free(ctx->entries[i].value);
    }
    free(ctx);
    return 0;
}

// ============================================
// Template Node
// ============================================

typedef struct {
    int type;             // NODE_TEXT, NODE_VAR, NODE_IF, NODE_FOR, NODE_FILTER
    char* text;           // For TEXT: the raw text content
                          // For VAR: the variable name
                          // For IF: the condition variable name
                          // For FOR: "item" variable name
                          // For INCLUDE: partial name
                          // For FILTER: variable name
    char* extra;          // For FOR: the list variable name
                          // For FILTER: the filter name
    int children_start;   // Index of first child node (for IF/FOR blocks)
    int children_count;   // Number of child nodes
    int else_start;       // Index of else branch children (for IF blocks)
    int else_count;       // Number of else branch children
} TemplateNode;

typedef struct {
    TemplateNode nodes[TPL_MAX_NODES];
    int node_count;
    char* source;         // Original source (kept for reference)
} ParsedTemplate;

// ============================================
// Partial Registry (simple global store)
// ============================================

static struct {
    char* name;
    char* source;
} g_partials[TPL_MAX_PARTIALS];
static int g_partial_count = 0;

long __template_register_partial(const char* name, const char* source) {
    if (!name || !source || g_partial_count >= TPL_MAX_PARTIALS) return 0;
    g_partials[g_partial_count].name = strdup(name);
    g_partials[g_partial_count].source = strdup(source);
    g_partial_count++;
    return 1;
}

// Forward declarations for functions used before definition
const char* __html_escape(const char* input);

static const char* find_partial(const char* name) {
    for (int i = 0; i < g_partial_count; i++) {
        if (g_partials[i].name && strcmp(g_partials[i].name, name) == 0) {
            return g_partials[i].source;
        }
    }
    return NULL;
}

// ============================================
// Template Parsing Helpers
// ============================================

// Skip whitespace in a string starting at position pos
static int skip_ws(const char* s, int pos, int len) {
    while (pos < len && (s[pos] == ' ' || s[pos] == '\t')) pos++;
    return pos;
}

// Extract a word (identifier) from position pos
// Returns length of the word, stores word start in *start
static int extract_word(const char* s, int pos, int len) {
    int start = pos;
    while (pos < len && s[pos] != ' ' && s[pos] != '\t' &&
           s[pos] != '}' && s[pos] != '%' && s[pos] != '|') {
        pos++;
    }
    return pos - start;
}

// Duplicate a substring as a null-terminated string
static char* substr_dup(const char* s, int start, int length) {
    char* result = (char*)malloc((size_t)(length + 1));
    if (!result) return NULL;
    memcpy(result, s + start, (size_t)length);
    result[length] = '\0';
    return result;
}

// ============================================
// Template Parser
// ============================================

// Forward declaration for recursive parsing
static int parse_nodes(ParsedTemplate* tpl, const char* src, int len,
                       int pos, int* node_start, int* node_count,
                       const char* end_tag);

// Parse the template source into nodes
// Returns the position after parsing
static int parse_nodes(ParsedTemplate* tpl, const char* src, int len,
                       int pos, int* node_start, int* node_count,
                       const char* end_tag) {
    *node_start = tpl->node_count;
    *node_count = 0;

    while (pos < len && tpl->node_count < TPL_MAX_NODES) {
        // Check for {{ (variable interpolation)
        if (pos + 1 < len && src[pos] == '{' && src[pos + 1] == '{') {
            int var_start = pos + 2;
            // Find closing }}
            int var_end = var_start;
            while (var_end + 1 < len && !(src[var_end] == '}' && src[var_end + 1] == '}')) {
                var_end++;
            }
            if (var_end + 1 >= len) break; // Unclosed {{

            // Extract and trim variable name
            int vs = skip_ws(src, var_start, var_end);
            int wlen = extract_word(src, vs, var_end);
            char* var_name = substr_dup(src, vs, wlen);

            // Check for filter: {{ var | filter }}
            int after_var = vs + wlen;
            after_var = skip_ws(src, after_var, var_end);
            if (after_var < var_end && src[after_var] == '|') {
                // Has filter
                int fs = skip_ws(src, after_var + 1, var_end);
                int flen = extract_word(src, fs, var_end);
                char* filter_name = substr_dup(src, fs, flen);

                TemplateNode* node = &tpl->nodes[tpl->node_count];
                node->type = NODE_FILTER;
                node->text = var_name;
                node->extra = filter_name;
                node->children_start = 0;
                node->children_count = 0;
                node->else_start = 0;
                node->else_count = 0;
                tpl->node_count++;
                (*node_count)++;
            } else {
                // Plain variable
                TemplateNode* node = &tpl->nodes[tpl->node_count];
                node->type = NODE_VAR;
                node->text = var_name;
                node->extra = NULL;
                node->children_start = 0;
                node->children_count = 0;
                node->else_start = 0;
                node->else_count = 0;
                tpl->node_count++;
                (*node_count)++;
            }

            pos = var_end + 2; // Skip }}
            continue;
        }

        // Check for {% (block tag)
        if (pos + 1 < len && src[pos] == '{' && src[pos + 1] == '%') {
            int tag_start = pos + 2;
            // Find closing %}
            int tag_end = tag_start;
            while (tag_end + 1 < len && !(src[tag_end] == '%' && src[tag_end + 1] == '}')) {
                tag_end++;
            }
            if (tag_end + 1 >= len) break; // Unclosed {%

            // Extract tag keyword
            int ts = skip_ws(src, tag_start, tag_end);
            int kwlen = extract_word(src, ts, tag_end);

            // Check for end tags
            if (end_tag != NULL) {
                if (kwlen == (int)strlen(end_tag) &&
                    strncmp(src + ts, end_tag, (size_t)kwlen) == 0) {
                    // Found our end tag, return position after %}
                    return tag_end + 2;
                }
            }

            if (kwlen == 2 && strncmp(src + ts, "if", 2) == 0) {
                // {% if condition %}
                int cs = skip_ws(src, ts + 2, tag_end);
                int clen = extract_word(src, cs, tag_end);
                char* cond_var = substr_dup(src, cs, clen);

                TemplateNode* node = &tpl->nodes[tpl->node_count];
                node->type = NODE_IF;
                node->text = cond_var;
                node->extra = NULL;
                int node_idx = tpl->node_count;
                tpl->node_count++;
                (*node_count)++;

                // Parse children until {% else %} or {% endif %}
                int after_body = tag_end + 2;

                // Parse body nodes
                int body_start_idx = tpl->node_count;
                int body_count = 0;

                // Parse body until we hit {% else %} or {% endif %}
                int cur_pos = parse_nodes(tpl, src, len, after_body,
                                          &body_start_idx, &body_count, "else");

                // After parsing body, try to parse else block until endif
                {
                    int else_body_start = 0, else_body_count = 0;
                    int next_pos = parse_nodes(tpl, src, len, cur_pos,
                                               &else_body_start, &else_body_count, "endif");

                    if (else_body_count > 0) {
                        // We had an else block
                        tpl->nodes[node_idx].else_start = else_body_start;
                        tpl->nodes[node_idx].else_count = else_body_count;
                    }
                    cur_pos = next_pos;
                }

                tpl->nodes[node_idx].children_start = body_start_idx;
                tpl->nodes[node_idx].children_count = body_count;

                pos = cur_pos;
                continue;

            } else if (kwlen == 3 && strncmp(src + ts, "for", 3) == 0) {
                // {% for item in list %}
                int is_ = skip_ws(src, ts + 3, tag_end);
                int ilen = extract_word(src, is_, tag_end);
                char* item_var = substr_dup(src, is_, ilen);

                // Skip "in"
                int in_s = skip_ws(src, is_ + ilen, tag_end);
                // Expect "in"
                if (in_s + 2 <= tag_end && src[in_s] == 'i' && src[in_s + 1] == 'n') {
                    in_s += 2;
                }
                int ls = skip_ws(src, in_s, tag_end);
                int llen = extract_word(src, ls, tag_end);
                char* list_var = substr_dup(src, ls, llen);

                TemplateNode* node = &tpl->nodes[tpl->node_count];
                node->type = NODE_FOR;
                node->text = item_var;
                node->extra = list_var;
                int node_idx = tpl->node_count;
                tpl->node_count++;
                (*node_count)++;

                // Parse children until {% endfor %}
                int child_start = 0, child_count = 0;
                int after_body = tag_end + 2;
                pos = parse_nodes(tpl, src, len, after_body,
                                  &child_start, &child_count, "endfor");

                tpl->nodes[node_idx].children_start = child_start;
                tpl->nodes[node_idx].children_count = child_count;
                continue;

            } else if (kwlen == 7 && strncmp(src + ts, "include", 7) == 0) {
                // {% include "partial_name" %}
                int ns = skip_ws(src, ts + 7, tag_end);
                // Skip opening quote if present
                if (ns < tag_end && (src[ns] == '"' || src[ns] == '\'')) ns++;
                int nlen = 0;
                int ne = ns;
                while (ne < tag_end && src[ne] != '"' && src[ne] != '\'' &&
                       src[ne] != '%' && src[ne] != ' ') {
                    ne++;
                    nlen++;
                }
                char* partial_name = substr_dup(src, ns, nlen);

                TemplateNode* node = &tpl->nodes[tpl->node_count];
                node->type = NODE_INCLUDE;
                node->text = partial_name;
                node->extra = NULL;
                node->children_start = 0;
                node->children_count = 0;
                node->else_start = 0;
                node->else_count = 0;
                tpl->node_count++;
                (*node_count)++;

                pos = tag_end + 2;
                continue;

            } else if (kwlen == 5 && strncmp(src + ts, "endif", 5) == 0) {
                // Unexpected endif (not matching any if)
                pos = tag_end + 2;
                continue;

            } else if (kwlen == 6 && strncmp(src + ts, "endfor", 6) == 0) {
                // Unexpected endfor
                pos = tag_end + 2;
                continue;

            } else if (kwlen == 4 && strncmp(src + ts, "else", 4) == 0) {
                // Unexpected else
                pos = tag_end + 2;
                continue;
            }

            // Unknown tag, treat as text
            pos = tag_end + 2;
            continue;
        }

        // Regular text: collect until next {{ or {% or end
        int text_start = pos;
        while (pos < len) {
            if (pos + 1 < len && src[pos] == '{' &&
                (src[pos + 1] == '{' || src[pos + 1] == '%')) {
                break;
            }
            pos++;
        }

        if (pos > text_start) {
            TemplateNode* node = &tpl->nodes[tpl->node_count];
            node->type = NODE_TEXT;
            node->text = substr_dup(src, text_start, pos - text_start);
            node->extra = NULL;
            node->children_start = 0;
            node->children_count = 0;
            node->else_start = 0;
            node->else_count = 0;
            tpl->node_count++;
            (*node_count)++;
        }
    }

    return pos;
}

// ============================================
// Template Parse Entry Point
// ============================================

long __template_parse(const char* source, long len) {
    if (!source || len <= 0) return 0;

    ParsedTemplate* tpl = (ParsedTemplate*)calloc(1, sizeof(ParsedTemplate));
    if (!tpl) return 0;

    tpl->node_count = 0;
    tpl->source = strdup(source);

    int node_start = 0, node_count = 0;
    parse_nodes(tpl, source, (int)len, 0, &node_start, &node_count, NULL);

    return (long)tpl;
}

// ============================================
// Template Rendering
// ============================================

// Forward declaration
static int render_nodes(ParsedTemplate* tpl, TemplateCtx* ctx,
                        int start, int count, char* buf, int buf_size);

// Render a range of nodes into a buffer
// Returns the number of bytes written
static int render_nodes(ParsedTemplate* tpl, TemplateCtx* ctx,
                        int start, int count, char* buf, int buf_size) {
    int pos = 0;

    for (int i = start; i < start + count && i < tpl->node_count; i++) {
        TemplateNode* node = &tpl->nodes[i];

        switch (node->type) {
            case NODE_TEXT: {
                if (node->text) {
                    int tlen = (int)strlen(node->text);
                    if (pos + tlen < buf_size) {
                        memcpy(buf + pos, node->text, (size_t)tlen);
                        pos += tlen;
                    }
                }
                break;
            }

            case NODE_VAR: {
                if (node->text) {
                    const char* val = __template_ctx_get((long)ctx, node->text);
                    if (val) {
                        int vlen = (int)strlen(val);
                        if (pos + vlen < buf_size) {
                            memcpy(buf + pos, val, (size_t)vlen);
                            pos += vlen;
                        }
                    }
                }
                break;
            }

            case NODE_FILTER: {
                if (node->text) {
                    const char* val = __template_ctx_get((long)ctx, node->text);
                    if (val && node->extra) {
                        // Apply filter
                        char* filtered = NULL;
                        if (strcmp(node->extra, "upper") == 0) {
                            filtered = strdup(val);
                            if (filtered) {
                                for (int j = 0; filtered[j]; j++)
                                    filtered[j] = (char)toupper((unsigned char)filtered[j]);
                            }
                        } else if (strcmp(node->extra, "lower") == 0) {
                            filtered = strdup(val);
                            if (filtered) {
                                for (int j = 0; filtered[j]; j++)
                                    filtered[j] = (char)tolower((unsigned char)filtered[j]);
                            }
                        } else if (strcmp(node->extra, "escape") == 0) {
                            filtered = (char*)__html_escape(val);
                        } else if (strcmp(node->extra, "trim") == 0) {
                            // Trim leading/trailing whitespace
                            int s = 0, e = (int)strlen(val) - 1;
                            while (s <= e && isspace((unsigned char)val[s])) s++;
                            while (e >= s && isspace((unsigned char)val[e])) e--;
                            filtered = substr_dup(val, s, e - s + 1);
                        } else if (strcmp(node->extra, "length") == 0) {
                            filtered = (char*)malloc(32);
                            if (filtered) sprintf(filtered, "%d", (int)strlen(val));
                        } else {
                            // Unknown filter, pass through
                            filtered = strdup(val);
                        }

                        if (filtered) {
                            int flen = (int)strlen(filtered);
                            if (pos + flen < buf_size) {
                                memcpy(buf + pos, filtered, (size_t)flen);
                                pos += flen;
                            }
                            free(filtered);
                        }
                    } else if (val) {
                        // No filter, just output value
                        int vlen = (int)strlen(val);
                        if (pos + vlen < buf_size) {
                            memcpy(buf + pos, val, (size_t)vlen);
                            pos += vlen;
                        }
                    }
                }
                break;
            }

            case NODE_IF: {
                if (node->text) {
                    const char* val = __template_ctx_get((long)ctx, node->text);
                    int truthy = 0;
                    if (val && strlen(val) > 0) {
                        // Check for falsy values: "0", "false", ""
                        if (strcmp(val, "0") != 0 && strcmp(val, "false") != 0) {
                            truthy = 1;
                        }
                    }

                    if (truthy) {
                        // Render body children
                        pos += render_nodes(tpl, ctx, node->children_start,
                                            node->children_count, buf + pos, buf_size - pos);
                    } else if (node->else_count > 0) {
                        // Render else children
                        pos += render_nodes(tpl, ctx, node->else_start,
                                            node->else_count, buf + pos, buf_size - pos);
                    }
                }
                break;
            }

            case NODE_FOR: {
                if (node->text && node->extra) {
                    // For loop: iterate over comma-separated values in the list variable
                    // The list variable should contain comma-separated values
                    const char* list_val = __template_ctx_get((long)ctx, node->extra);
                    if (list_val && strlen(list_val) > 0) {
                        const char* item_name = node->text;
                        int llen = (int)strlen(list_val);
                        int lpos = 0;

                        while (lpos < llen) {
                            // Find next comma or end
                            int item_start = lpos;
                            while (lpos < llen && list_val[lpos] != ',') lpos++;
                            int item_len = lpos - item_start;

                            // Trim whitespace from item
                            int is_ = item_start;
                            int ie = item_start + item_len - 1;
                            while (is_ <= ie && isspace((unsigned char)list_val[is_])) is_++;
                            while (ie >= is_ && isspace((unsigned char)list_val[ie])) ie--;

                            char* item_val = substr_dup(list_val, is_, ie - is_ + 1);

                            // Temporarily set the item variable in context
                            const char* old_val = __template_ctx_get((long)ctx, item_name);
                            char* saved_val = old_val ? strdup(old_val) : NULL;

                            __template_ctx_set((long)ctx, item_name, item_val);

                            // Render body for this iteration
                            pos += render_nodes(tpl, ctx, node->children_start,
                                                node->children_count, buf + pos, buf_size - pos);

                            // Restore old value
                            if (saved_val) {
                                __template_ctx_set((long)ctx, item_name, saved_val);
                                free(saved_val);
                            }

                            free(item_val);

                            // Skip comma
                            if (lpos < llen) lpos++;
                        }
                    }
                }
                break;
            }

            case NODE_INCLUDE: {
                if (node->text) {
                    const char* partial_src = find_partial(node->text);
                    if (partial_src) {
                        // Parse and render the partial inline
                        ParsedTemplate* partial_tpl = (ParsedTemplate*)calloc(1, sizeof(ParsedTemplate));
                        if (partial_tpl) {
                            partial_tpl->node_count = 0;
                            partial_tpl->source = strdup(partial_src);
                            int ps = 0, pc = 0;
                            parse_nodes(partial_tpl, partial_src, (int)strlen(partial_src),
                                        0, &ps, &pc, NULL);
                            pos += render_nodes(partial_tpl, ctx, ps, pc,
                                                buf + pos, buf_size - pos);
                            // Free partial template
                            for (int j = 0; j < partial_tpl->node_count; j++) {
                                free(partial_tpl->nodes[j].text);
                                free(partial_tpl->nodes[j].extra);
                            }
                            free(partial_tpl->source);
                            free(partial_tpl);
                        }
                    }
                    // If partial not found, silently skip
                }
                break;
            }
        }
    }

    return pos;
}

// Render template with context
// Returns a newly allocated string (caller must free)
const char* __template_render(long tmpl_handle, long ctx_handle) {
    if (tmpl_handle == 0 || ctx_handle == 0) return NULL;

    ParsedTemplate* tpl = (ParsedTemplate*)tmpl_handle;
    TemplateCtx* ctx = (TemplateCtx*)ctx_handle;

    char* buf = (char*)malloc(TPL_BUFFER_SIZE);
    if (!buf) return NULL;

    // Find the top-level nodes: they start at 0 and go to node_count,
    // but we need to skip children that are part of if/for blocks
    // Actually, top-level nodes are those parsed at the outermost level
    // In our parser, the first call to parse_nodes stores the top-level range

    // Render all top-level nodes
    // The top-level nodes are those NOT nested inside if/for blocks
    // In our flat array, the top-level parsing starts at index 0
    // and the count is whatever parse_nodes returned as node_count

    // We need to find the actual top-level node indices
    // Since parse_nodes returns node_count for the top level, we need
    // to track that. Let's just render from 0 with the count of
    // nodes that were at the top level.

    // Simple approach: re-parse to find top-level count, or just
    // store it. For now, let's scan the source to find top-level nodes.
    // Actually, the node_count after initial parse IS the total.
    // The top-level nodes are stored first, then children.
    // But children are also mixed in...

    // Better approach: just render from index 0 with the total count,
    // but skip nodes that are children of other nodes.
    // Actually in our parser, children of if/for are stored AFTER
    // the parent, and the parent has children_start/count.
    // Top-level nodes are those not referenced as children of any parent.

    // Simplest correct approach: collect top-level indices
    int top_level[TPL_MAX_NODES];
    int top_count = 0;
    int is_child[TPL_MAX_NODES];
    memset(is_child, 0, sizeof(is_child));

    // Mark all children
    for (int i = 0; i < tpl->node_count; i++) {
        if (tpl->nodes[i].type == NODE_IF || tpl->nodes[i].type == NODE_FOR) {
            for (int j = tpl->nodes[i].children_start;
                 j < tpl->nodes[i].children_start + tpl->nodes[i].children_count; j++) {
                if (j < TPL_MAX_NODES) is_child[j] = 1;
            }
            for (int j = tpl->nodes[i].else_start;
                 j < tpl->nodes[i].else_start + tpl->nodes[i].else_count; j++) {
                if (j < TPL_MAX_NODES) is_child[j] = 1;
            }
        }
    }

    // Collect top-level nodes
    for (int i = 0; i < tpl->node_count; i++) {
        if (!is_child[i]) {
            top_level[top_count++] = i;
        }
    }

    // Render top-level nodes by iterating through them
    int written = 0;
    for (int i = 0; i < top_count; i++) {
        written += render_nodes(tpl, ctx, top_level[i], 1, buf + written, TPL_BUFFER_SIZE - written);
    }

    buf[written] = '\0';

    // Return a right-sized copy
    char* result = strdup(buf);
    free(buf);
    return result;
}

// ============================================
// Template Free
// ============================================

long __template_free(long tmpl_handle) {
    if (tmpl_handle == 0) return 0;
    ParsedTemplate* tpl = (ParsedTemplate*)tmpl_handle;

    for (int i = 0; i < tpl->node_count; i++) {
        free(tpl->nodes[i].text);
        free(tpl->nodes[i].extra);
    }
    free(tpl->source);
    free(tpl);
    return 0;
}

// ============================================
// HTML Escaping
// ============================================

const char* __html_escape(const char* input) {
    if (!input) return NULL;

    int len = (int)strlen(input);
    // Worst case: every char becomes "&quot;" (6 chars)
    int buf_size = len * 6 + 1;
    char* buf = (char*)malloc((size_t)buf_size);
    if (!buf) return NULL;

    int pos = 0;
    for (int i = 0; i < len; i++) {
        char c = input[i];
        switch (c) {
            case '&':
                memcpy(buf + pos, "&amp;", 5);
                pos += 5;
                break;
            case '<':
                memcpy(buf + pos, "&lt;", 4);
                pos += 4;
                break;
            case '>':
                memcpy(buf + pos, "&gt;", 4);
                pos += 4;
                break;
            case '"':
                memcpy(buf + pos, "&quot;", 6);
                pos += 6;
                break;
            case '\'':
                memcpy(buf + pos, "&#39;", 5);
                pos += 5;
                break;
            default:
                buf[pos++] = c;
                break;
        }
    }
    buf[pos] = '\0';

    // Return right-sized copy
    char* result = strdup(buf);
    free(buf);
    return result;
}

// ============================================
// String Filter Helpers
// ============================================

const char* __str_to_upper(const char* s) {
    if (!s) return NULL;
    int len = (int)strlen(s);
    char* result = (char*)malloc((size_t)(len + 1));
    if (!result) return NULL;
    for (int i = 0; i < len; i++) {
        result[i] = (char)toupper((unsigned char)s[i]);
    }
    result[len] = '\0';
    return result;
}

const char* __str_to_lower(const char* s) {
    if (!s) return NULL;
    int len = (int)strlen(s);
    char* result = (char*)malloc((size_t)(len + 1));
    if (!result) return NULL;
    for (int i = 0; i < len; i++) {
        result[i] = (char)tolower((unsigned char)s[i]);
    }
    result[len] = '\0';
    return result;
}

const char* __str_trim(const char* s) {
    if (!s) return NULL;
    int len = (int)strlen(s);
    int start = 0, end = len - 1;
    while (start <= end && isspace((unsigned char)s[start])) start++;
    while (end >= start && isspace((unsigned char)s[end])) end--;
    return substr_dup(s, start, end - start + 1);
}
