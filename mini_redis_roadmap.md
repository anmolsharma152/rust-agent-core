# mini-redis: High-Concurrency RESP TCP Server & Pub/Sub Broker in Go

A lightweight, high-throughput Redis server and Pub/Sub event broker built from scratch in Go to demonstrate concurrent TCP networking, protocol parsing, and lock-free/mutex-optimized data structures.

---

## 🎯 Architectural Philosophy

- **Zero Third-Party Redis Libraries**: Pure Go standard library (`net`, `sync`, `bytes`, `time`).
- **Goroutine-Per-Connection Pipeline**: Scalable concurrent connection handling using Go's lightweight user-space goroutines.
- **RESP2/RESP3 Protocol Parser**: Zero-allocation byte-level parser for the Redis Serialization Protocol.
- **Thread-Safe In-Memory Store**: Mutex-protected key-value store supporting TTL expiration timers and background eviction.

---

## 🚀 Milestones & Implementation Stages

### Stage 1: RESP Protocol Parser & TCP Listener
- Establish TCP listener (`net.Listen("tcp", ":6379")`).
- Implement zero-allocation RESP protocol reader/writer:
  - Simple Strings (`+OK\r\n`)
  - Errors (`-ERR ...\r\n`)
  - Integers (`:1000\r\n`)
  - Bulk Strings (`$6\r\nfoobar\r\n`)
  - Arrays (`*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n`)

### Stage 2: Core Key-Value Operations (`SET`, `GET`, `DEL`, `EXISTS`)
- Implement thread-safe `Store` using `sync.RWMutex` over Go maps.
- Handle core commands: `PING`, `ECHO`, `SET`, `GET`, `DEL`, `EXISTS`.

### Stage 3: Key Expiration (TTL) & Background Eviction Loop
- Add `EXPIRE`, `TTL`, `PERSIST` command support.
- Implement background cleanup goroutine using `time.Ticker` (active probabilistic expiration of expired keys to prevent memory leaks).

### Stage 4: Advanced Data Types (Hashes & Lists)
- Add Hash operations: `HSET`, `HGET`, `HGETALL`, `HDEL`.
- Add List operations: `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LRANGE`.

### Stage 5: Concurrent Pub/Sub Messaging Engine
- Implement `SUBSCRIBE`, `UNSUBSCRIBE`, `PUBLISH` commands.
- Manage client subscriptions using concurrent channel mappings (`map[string]map[chan Message]bool`).
- Deliver broadcast messages asynchronously to subscribed client connections without blocking the main storage mutex.

### Stage 6: Persistence (RDB Snapshotting & AOF Log)
- Implement Append-Only File (`AOF`) write-ahead logging for durability.
- Implement background RDB snapshotting (`SAVE` / `BGSAVE` using process fork/child goroutine).

---

## 🧩 Core Architectural Structure (Go Spec)

```go
type Client struct {
	conn net.Conn
	reader *bufio.Reader
	writer *bufio.Writer
}

type Store struct {
	mu   sync.RWMutex
	data map[string]Value
	ttl  map[string]time.Time
}

type PubSubBroker struct {
	mu          sync.RWMutex
	subscribers map[string]map[*Client]struct{}
}
```

---

## 🧪 Verification & Benchmarking

- Test with standard `redis-cli` tool:
  ```bash
  redis-cli -p 6379 PING
  redis-cli -p 6379 SET mykey "hello world"
  redis-cli -p 6379 GET mykey
  ```
- Run standard Redis benchmark tool:
  ```bash
  redis-benchmark -p 6379 -n 100000 -c 50
  ```
