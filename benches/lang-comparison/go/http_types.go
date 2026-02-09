// HTTP types with error handling
package main

import (
	"errors"
	"fmt"
)

type HttpError struct {
	Kind    string
	Message string
}

func (e *HttpError) Error() string {
	return fmt.Sprintf("%s: %s", e.Kind, e.Message)
}

type Header struct {
	Name  string
	Value string
}

type Request struct {
	Method  string
	Path    string
	Headers []Header
	Body    string
}

type Response struct {
	Status  int
	Headers []Header
	Body    string
}

func parseMethod(s string) (string, error) {
	switch s {
	case "GET", "POST", "PUT", "DELETE":
		return s, nil
	default:
		return "", errors.New("invalid method")
	}
}

func route(req *Request) Response {
	switch req.Path {
	case "/":
		return Response{Status: 200, Headers: nil, Body: "OK"}
	case "/health":
		return Response{Status: 200, Headers: nil, Body: "healthy"}
	default:
		return Response{Status: 404, Headers: nil, Body: "not found"}
	}
}

func formatResponse(res Response) string {
	return fmt.Sprintf("HTTP/1.1 %d %s", res.Status, res.Body)
}

func main() {
	req := &Request{
		Method:  "GET",
		Path:    "/",
		Headers: nil,
		Body:    "",
	}
	res := route(req)
	fmt.Println(formatResponse(res))
}
