# RavensOne Real-Time Chat Example

A production-ready chat experience showcasing RavensOne's built-in support for
real-time APIs.

## 🔥 Feature Highlights

- Multi-room chat with instant WebSocket fan-out
- Presence tracking and typing indicators
- Server-side message moderation hooks
- Resilient reconnect logic and optimistic UI updates

## 🏗️ Architecture Overview

- **`join_room` / `leave_room`** orchestrate WebSocket subscriptions.
- **`send_message`** persists messages and broadcasts to room subscribers.
- **`typing`** emits presence events to keep UI in sync.
- **`UIState`** keeps the client hydrated with messages, presence, and typing
  state while switching between rooms.

## 🚀 Running the Example

```bash
cd examples/apps/realtime-chat
raven compile app.raven --minify
cd dist
node server.js
```

Open `http://localhost:3000` in multiple browser tabs to experience:

1. **Room selection** with per-room message history.
2. **Live presence list** showing connected avatars.
3. **Typing indicators** that disappear when users stop typing.

## 🔌 Integration Checklist

- Connect `send_message` to a moderation or spam filtering service.
- Persist history in Redis Streams, PostgreSQL, or DynamoDB.
- Use the presence hooks to integrate with Slack or email digests.
- Add ephemeral rooms for direct messages.

## ➕ Enhancements

- Add read receipts with message timestamps.
- Record metrics with Prometheus or OpenTelemetry.
- Integrate push notifications via Firebase Cloud Messaging.
