# 🧠 Rust Async + Threading Mastery — Syllabus
## 🎯 Goal

Develop a deep, practical understanding of:

- Threading (shared memory + message passing)
- Async (futures, executors, scheduling)
- Hybrid systems (async + threads)
- Multi-process architectures

Each lesson = learn → build → break → understand

## 📚 Lesson 1 — Threads & Ownership
Learn
std::thread::spawn
Ownership across threads (move)
Send and Sync (high-level intuition)
Basic Arc
Project

Parallel Job Runner (Thread Pool v1)

Requirements
Spawn N worker threads
Send jobs via channel
Workers execute jobs
Print results
You should struggle with:
“value moved” errors
sharing data safely

## 📚 Lesson 2 — Shared State & Synchronization
Learn
Arc<Mutex<T>>
Locking & contention
Deadlocks (what causes them)
Project

Concurrent File Processor

Requirements
Walk directory
Process files in parallel
Aggregate results safely
Break it on purpose:
Create a deadlock
Hold locks too long

## 📚 Lesson 3 — Message Passing vs Shared Memory
Learn
std::sync::mpsc
When channels beat mutexes
Backpressure basics
Project

Bounded Work Queue System

Requirements
Producer → bounded queue → workers
Block or drop when full
Key insight:
Systems fail when producers outrun consumers
📚 Lesson 4 — Async Fundamentals (Under the Hood)
Learn
Future
Poll
Waker concept (high-level)
Why async ≠ threads
Project

Mini Async Runtime

Requirements
Implement simple executor
Poll futures in loop
Outcome:
You understand what .await actually does
📚 Lesson 5 — Async in Practice
Learn
tokio Rust crate basics
async fn, .await
Tasks (spawn)
Async I/O
Project

Async TCP Chat Server

Requirements
Multiple clients
Broadcast messages
Handle disconnects
Pain points:
Shared state in async
Lifetimes + Arc
📚 Lesson 6 — Async State Management
Learn
Arc<Mutex<T>> in async
Channels in async (tokio::mpsc)
Avoiding blocking
Project

Chat Server v2 (Rooms + State)

Requirements
Rooms/channels
User tracking
Message routing
📚 Lesson 7 — Async + CPU Work
Learn
Why async fails for CPU work
spawn_blocking
Thread pools vs async tasks
Project

Hybrid Server (IO + CPU)

Requirements
Async API receives jobs
CPU-heavy tasks run in threads
Return results async
📚 Lesson 8 — System Design with Concurrency
Learn
Event-driven architecture
Decoupling via messaging
Throughput vs latency
Project

Event Bus / Message Broker

Requirements
Producers + consumers
Topics
Message distribution
Stretch:
Persistence
Replay
📚 Lesson 9 — Multi-Process Systems
Learn
std::process
IPC (pipes, sockets)
Serialization basics
Project

Process Orchestrator

Requirements
Spawn worker processes
Communicate with them
Restart on crash
📚 Lesson 10 — Distributed Thinking
Learn
Task queues
Failure handling
Retries
Project

Distributed Task System

Requirements
Workers (separate processes)
Task submission API
Queue system
🔑 Core Concepts You Must Master

You should be able to explain these without guessing:

Ownership across threads
Send / Sync
Arc, Mutex, RwLock
Channels vs shared memory
Futures + polling
Executors
Backpressure
Blocking vs non-blocking
🧪 How Each Lesson Should Go (IMPORTANT)

When you start a lesson, ask:

“Start Lesson X”

And we will:

Explain only what you need
Design the project together
You implement
I help debug / refine
Then we break it on purpose
🚀 Final Note

By Lesson 7–10, you’ll be building systems very similar to:

PLC simulation engines
Real-time tag systems
Multi-component industrial software
