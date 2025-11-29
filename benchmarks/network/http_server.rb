#!/usr/bin/env ruby
# Simple HTTP server in Ruby for benchmarking
# Equivalent to the Lisp http_server_simple.lisp

require 'socket'

def handle_request(client)
  # Read request line
  request_line = client.gets
  return false unless request_line

  # Read headers until empty line
  loop do
    header = client.gets
    break if header.nil? || header.strip.empty?
  end

  # Send response
  body = "Hello from Ruby!"
  response = "HTTP/1.1 200 OK\r\n" \
             "Content-Length: #{body.bytesize}\r\n" \
             "Content-Type: text/plain\r\n" \
             "\r\n" \
             "#{body}"

  client.write(response)
  client.close
  true
rescue => e
  client.close rescue nil
  false
end

def main
  port = ARGV[0]&.to_i || 8080
  max_requests = 100000

  server = TCPServer.new('0.0.0.0', port)
  puts "Ruby HTTP server listening on port #{port}"

  count = 0
  while count < max_requests
    client = server.accept
    count += 1 if handle_request(client)
  end

  server.close
end

main
