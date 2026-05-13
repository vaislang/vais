// HTTP types with error handling
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

typedef enum {
    HTTP_OK = 0,
    HTTP_BAD_REQUEST,
    HTTP_NOT_FOUND,
    HTTP_INTERNAL,
} HttpErrorKind;

typedef struct {
    HttpErrorKind kind;
    const char *message;
} HttpError;

typedef struct {
    const char *name;
    const char *value;
} Header;

typedef struct {
    const char *method;
    const char *path;
    Header *headers;
    int header_count;
    const char *body;
} Request;

typedef struct {
    int status;
    Header *headers;
    int header_count;
    const char *body;
} Response;

int parse_method(const char *s, HttpError *err) {
    if (strcmp(s, "GET") == 0 || strcmp(s, "POST") == 0 ||
        strcmp(s, "PUT") == 0 || strcmp(s, "DELETE") == 0) {
        return 1;
    }
    err->kind = HTTP_BAD_REQUEST;
    err->message = "invalid method";
    return 0;
}

Response route(const Request *req) {
    Response res;
    res.headers = NULL;
    res.header_count = 0;
    if (strcmp(req->path, "/") == 0) {
        res.status = 200;
        res.body = "OK";
    } else if (strcmp(req->path, "/health") == 0) {
        res.status = 200;
        res.body = "healthy";
    } else {
        res.status = 404;
        res.body = "not found";
    }
    return res;
}

void format_response(const Response *res, char *buf, int buf_size) {
    snprintf(buf, buf_size, "HTTP/1.1 %d %s", res->status, res->body);
}

int main() {
    Request req = { "GET", "/", NULL, 0, "" };
    Response res = route(&req);
    char buf[256];
    format_response(&res, buf, sizeof(buf));
    printf("%s\n", buf);
    return 0;
}
