-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name VARCHAR(255) NOT NULL,
    address TEXT,
    phone VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Products table
CREATE TABLE IF NOT EXISTS products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DOUBLE PRECISION NOT NULL,
    category VARCHAR(100) NOT NULL,
    image_url TEXT,
    stock INTEGER DEFAULT 0,
    featured BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Carts table
CREATE TABLE IF NOT EXISTS carts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Cart items table
CREATE TABLE IF NOT EXISTS cart_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id UUID NOT NULL REFERENCES carts(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(cart_id, product_id)
);

-- Orders table
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total DOUBLE PRECISION NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    shipping_address TEXT NOT NULL,
    payment_method VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Order items table
CREATE TABLE IF NOT EXISTS order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL,
    price_at_purchase DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Reviews table
CREATE TABLE IF NOT EXISTS reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(product_id, user_id)
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);
CREATE INDEX IF NOT EXISTS idx_products_featured ON products(featured);
CREATE INDEX IF NOT EXISTS idx_cart_items_cart_id ON cart_items(cart_id);
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_reviews_product_id ON reviews(product_id);
CREATE INDEX IF NOT EXISTS idx_reviews_user_id ON reviews(user_id);

-- Sample products for demo
INSERT INTO products (name, description, price, category, image_url, stock, featured) VALUES
('Wireless Headphones', 'Premium noise-cancelling wireless headphones with 30-hour battery life', 299.99, 'Electronics', 'https://images.unsplash.com/photo-1505740420928-5e560c06d30e?w=400', 25, true),
('Smart Watch', 'Fitness tracking smartwatch with heart rate monitor and GPS', 199.99, 'Electronics', 'https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=400', 40, true),
('Running Shoes', 'Lightweight running shoes with advanced cushioning technology', 129.99, 'Apparel', 'https://images.unsplash.com/photo-1542291026-7eec264c27ff?w=400', 60, false),
('Laptop Backpack', 'Durable laptop backpack with multiple compartments and USB charging port', 79.99, 'Accessories', 'https://images.unsplash.com/photo-1553062407-98eeb64c6a62?w=400', 35, false),
('Yoga Mat', 'Premium non-slip yoga mat with carrying strap', 49.99, 'Fitness', 'https://images.unsplash.com/photo-1601925260368-ae2f83cf8b7f?w=400', 50, false),
('Coffee Maker', 'Programmable coffee maker with thermal carafe and auto-shutoff', 89.99, 'Home', 'https://images.unsplash.com/photo-1517668808822-9ebb02f2a0e6?w=400', 20, false),
('Gaming Mouse', 'RGB gaming mouse with 16,000 DPI and programmable buttons', 69.99, 'Electronics', 'https://images.unsplash.com/photo-1527814050087-3793815479db?w=400', 45, true),
('Water Bottle', 'Insulated stainless steel water bottle keeps drinks cold for 24 hours', 29.99, 'Fitness', 'https://images.unsplash.com/photo-1602143407151-7111542de6e8?w=400', 100, false),
('Desk Lamp', 'LED desk lamp with adjustable brightness and USB charging port', 39.99, 'Home', 'https://images.unsplash.com/photo-1507473885765-e6ed057f782c?w=400', 30, false),
('Bluetooth Speaker', 'Portable waterproof Bluetooth speaker with 360-degree sound', 79.99, 'Electronics', 'https://images.unsplash.com/photo-1608043152269-423dbba4e7e1?w=400', 55, true),
('Sunglasses', 'Polarized UV protection sunglasses with classic design', 149.99, 'Accessories', 'https://images.unsplash.com/photo-1572635196237-14b3f281503f?w=400', 70, false),
('Phone Case', 'Slim protective phone case with shock absorption', 24.99, 'Accessories', 'https://images.unsplash.com/photo-1601784551446-20c9e07cdbdb?w=400', 200, false)
ON CONFLICT DO NOTHING;
