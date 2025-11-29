#!/usr/bin/env lua
--[[
Simple HTTP server in Lua for benchmarking
Equivalent to the Lisp http_server_simple.lisp
Requires luasocket: luarocks install luasocket
--]]

local socket = require("socket")

local function handle_request(client)
    -- Read request line
    local request_line = client:receive()
    if not request_line then
        return false
    end

    -- Read headers until empty line
    while true do
        local header = client:receive()
        if not header or header == "" then
            break
        end
    end

    -- Send response
    local body = "Hello from Lua!"
    local response = string.format(
        "HTTP/1.1 200 OK\r\n" ..
        "Content-Length: %d\r\n" ..
        "Content-Type: text/plain\r\n" ..
        "\r\n" ..
        "%s",
        #body, body
    )

    client:send(response)
    client:close()
    return true
end

local function main()
    local port = arg[1] or 8080
    local max_requests = 1000

    local server = assert(socket.bind("0.0.0.0", port))
    print("Lua HTTP server listening on port " .. port)

    local count = 0
    while count < max_requests do
        local client = server:accept()
        client:settimeout(10)

        if handle_request(client) then
            count = count + 1
        end
    end

    server:close()
end

main()
