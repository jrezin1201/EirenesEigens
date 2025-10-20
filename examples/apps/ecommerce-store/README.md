# RavensOne E-commerce Store Example

Demonstrates how to build a complete shopping experience with RavensOne. The
entire catalog, cart, checkout, and Stripe integration live in a single
`app.raven` file.

## 🛍️ Features

- Product catalog with tag filtering and type-ahead search
- Persistent client cart with quantity controls
- Server-side checkout flow that creates Stripe payment intents
- Order history view with live payment status updates
- Transactional email hook for order confirmations

## 🧱 Architecture

- **Shared models** define `Product`, `CartLine`, `CheckoutSession`, and `Order`
  so both server and client work with the same types.
- **Server functions** handle catalog queries, checkout orchestration, Stripe
  integration, and transactional email triggers.
- **Client functions** render the storefront UI, manage the cart state, and call
  server RPCs for checkout and payment confirmation.

## 🚀 Getting Started

```bash
cd examples/apps/ecommerce-store
raven compile app.raven --minify
cd dist
node server.js
```

Visit `http://localhost:3000` to browse the storefront, add items to the cart,
and walk through the mocked Stripe checkout flow.

## 🔌 Integration Points

- Replace `create_stripe_payment_intent` with the official Stripe SDK.
- Persist orders in Postgres, DynamoDB, or Supabase using RavensOne's database
  bridge.
- Connect `send_checkout_email` to Postmark, Sendgrid, or AWS SES.
- Hook up inventory to a warehouse API for real-time stock levels.

## 📦 Next Improvements

- Support discount codes and loyalty programs
- Add address autocompletion with the Google Places API
- Implement webhooks to receive Stripe payment events
