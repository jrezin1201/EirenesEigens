# RavensOne Blog Platform Example

A full-featured publishing workflow implemented in a single `.raven` file. This
example demonstrates how to build a collaborative editorial experience with
RavensOne's compiler bridge.

## ✨ Highlights

- **Live Markdown editing** with two-way binding between the textarea and a
  rendered preview component.
- **Media uploads** backed by `@server` handlers that can stream files to S3,
  Vercel Blob, or any custom storage backend.
- **Comment threads** with optimistic UI updates, moderation queues, and
  publication workflows.
- **Shared validation** logic to keep business rules consistent across server
  and client.
- **Type-safe RPCs** generated automatically by the RavensOne compiler bridge.

## 📂 Files

| File | Description |
|------|-------------|
| `app.raven` | All server, client, and shared logic in one file |
| `styles.css` | Tailored styles for the editor and public reader views |
| `README.md` | This document |

## 🚀 Running the Example

```bash
cd examples/apps/blog-platform
raven compile app.raven --minify
cd dist
node server.js
```

Open `http://localhost:3000` to explore:

1. **Editorial Dashboard** – Draft, preview, and publish posts.
2. **Media Manager** – Upload hero images and inline assets.
3. **Public Reader** – Browse published posts and leave comments.

## 🧠 Key Patterns

- **`save_post`** + **`publish_post`** show how to keep drafts and published
  content separate without duplicating logic.
- **`upload_media`** demonstrates streaming binary data to the server from a
  client event handler.
- **`add_comment`** and **`moderate_comment`** illustrate two-step moderation.
- **`bind_markdown_preview`** wires up real-time markdown previews with a few
  lines of client code.

## 🔌 Integrations

Replace the stubbed helpers with real services:

- Swap `render_markdown` with `@server` calls to a WASM Markdown engine.
- Forward uploads to object storage using presigned URLs.
- Persist data in Postgres, Fauna, or DynamoDB via RavensOne's database bridge.

## ✅ Next Steps

- Add role-based permissions for multi-author newsrooms.
- Integrate scheduled publishing with CRON triggers.
- Push notifications when new comments require moderation.
