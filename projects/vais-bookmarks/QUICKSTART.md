# Quick Start Guide - Vais Bookmarks API

Get up and running with the Vais Bookmarks API server in 5 minutes.

## Prerequisites

- Vais compiler installed (`vaisc`)
- LLVM 17
- clang

## Step 1: Build

```bash
cd /Users/sswoo/study/projects/vais/projects/vais-bookmarks
vaisc compile src/main.vais -o bookmark-server
```

## Step 2: Run

```bash
./bookmark-server
```

You should see:
```
=================================
  Vais Bookmark API Server v1.0
=================================

Configuration:
  Port: 8080
  TLS: Disabled (HTTP)

API Endpoints:
  GET    /api/health          - Health check
  GET    /api/bookmarks       - List all bookmarks
  ...

Listening on port 8080
```

## Step 3: Test

Open a new terminal and try these commands:

### Health Check
```bash
curl http://localhost:8080/api/health
# {"status":"ok","service":"vais-bookmarks"}
```

### Create a Bookmark
```bash
curl -X POST http://localhost:8080/api/bookmarks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Vais Language",
    "url": "https://github.com/vais-lang/vais",
    "tags": "programming,compiler,systems"
  }'
# {"success":true,"id":1}
```

### List All Bookmarks
```bash
curl http://localhost:8080/api/bookmarks
# [{"id":1,"title":"Vais Language","url":"https://github.com/vais-lang/vais",...}]
```

### Get Single Bookmark
```bash
curl http://localhost:8080/api/bookmarks/1
# {"id":1,"title":"Vais Language",...}
```

### Update Bookmark
```bash
curl -X PUT http://localhost:8080/api/bookmarks/1 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Vais - AI-Optimized Language",
    "url": "https://github.com/vais-lang/vais",
    "tags": "programming,compiler,systems,ai"
  }'
# {"success":true,"message":"Bookmark updated"}
```

### Search Bookmarks
```bash
curl "http://localhost:8080/api/search?q=vais"
# [{"id":1,"title":"Vais - AI-Optimized Language",...}]
```

### Delete Bookmark
```bash
curl -X DELETE http://localhost:8080/api/bookmarks/1
# {"success":true,"message":"Bookmark deleted"}
```

## Step 4: Enable HTTPS (Optional)

Edit `src/main.vais` and change:
```vais
C use_tls := 0  # Change to 1
```

Recompile and run. The server will now use TLS/HTTPS.

## Troubleshooting

### Port Already in Use
Change the port in `src/main.vais`:
```vais
C port := 8080  # Change to 8081 or another port
```

### Compilation Errors
Ensure you have:
- Latest Vais compiler
- LLVM 17 installed
- Vais standard library available

### Connection Refused
Check that the server is running:
```bash
ps aux | grep bookmark-server
```

Check the port is open:
```bash
lsof -i :8080
```

## Next Steps

- Read [README.md](README.md) for full API documentation
- Read [IMPLEMENTATION.md](IMPLEMENTATION.md) for technical details
- Modify the code to add new features
- Deploy to production with systemd/docker

## Common Tasks

### Add More Sample Data
Modify `src/bookmark.vais` to pre-populate bookmarks on startup.

### Change Default Port
Edit `src/main.vais`, change `C port := 8080`.

### Enable Debug Logging
Add more `log_info()` calls in handlers and server.

### Add New Endpoint
1. Add route in `src/server.vais` (`route_request()`)
2. Add handler in `src/handler.vais`
3. Update README.md documentation

## Performance Tips

- Enable compression for large responses
- Use TLS only if needed (adds overhead)
- Monitor with `/api/health` endpoint
- Profile with `vais-profiler` (if available)

## Support

For issues or questions:
- Check the documentation files
- Review the source code comments
- Examine the Vais language guide

---

**Enjoy building with Vais!** 🚀
