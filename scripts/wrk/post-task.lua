-- Lua script for wrk to POST task creation requests
wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.body = '{"title":"Benchmark Task","description":"Load testing","priority":"medium","status":"todo"}'
