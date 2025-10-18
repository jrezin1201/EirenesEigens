-- Bluebird Social Feed Database Schema

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    avatar_gradient VARCHAR(20) NOT NULL, -- e.g., "gradient-a"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Posts table
CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    caption TEXT NOT NULL,
    image_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Likes table
CREATE TABLE likes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(post_id, user_id) -- One like per user per post
);

-- Comments table
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX idx_likes_post_id ON likes(post_id);
CREATE INDEX idx_likes_user_id ON likes(user_id);
CREATE INDEX idx_comments_post_id ON comments(post_id);
CREATE INDEX idx_comments_created_at ON comments(created_at DESC);

-- Seed data with the 6 users from our UI
INSERT INTO users (username, display_name, avatar_gradient) VALUES
('alexchen', 'Alex Chen', 'gradient-b'),
('sarahkim', 'Sarah Kim', 'gradient-c'),
('mikejohnson', 'Mike Johnson', 'gradient-d'),
('emmadavis', 'Emma Davis', 'gradient-e'),
('chrislee', 'Chris Lee', 'gradient-f'),
('mayapatel', 'Maya Patel', 'gradient-g');

-- Seed posts
INSERT INTO posts (user_id, caption, image_url) VALUES
((SELECT id FROM users WHERE username = 'alexchen'),
 'Chasing sunsets and salt air. 🌅 #travel #beach',
 'https://images.unsplash.com/photo-1507525428034-b723cf961d3e?q=80&w=1600&auto=format&fit=crop'),

((SELECT id FROM users WHERE username = 'sarahkim'),
 'New tasting menu drop. Crispy skin, cloud-light puree. 🍽️',
 'https://images.unsplash.com/photo-1544025162-d76694265947?q=80&w=1600&auto=format&fit=crop'),

((SELECT id FROM users WHERE username = 'mikejohnson'),
 'Prototype v2: brighter, thinner, smarter. ⚙️📱',
 'https://images.unsplash.com/photo-1517336714731-489689fd1ca8?q=80&w=1600&auto=format&fit=crop'),

((SELECT id FROM users WHERE username = 'emmadavis'),
 'Morning light over the ridge. Nature resets everything. 🏔️',
 'https://images.unsplash.com/photo-1501785888041-af3ef285b470?q=80&w=1600&auto=format&fit=crop'),

((SELECT id FROM users WHERE username = 'chrislee'),
 'City hum + neon rain. 🎞️ #street',
 'https://images.unsplash.com/photo-1517999349371-c43520457b23?q=80&w=1600&auto=format&fit=crop'),

((SELECT id FROM users WHERE username = 'mayapatel'),
 'Pour-over mornings, latte-art wins. ☕️✨',
 'https://images.unsplash.com/photo-1504754524776-8f4f37790ca0?q=80&w=1600&auto=format&fit=crop');

-- Seed some likes (312, 468, 221, 377, 259, 198 likes respectively)
DO $$
DECLARE
    post_record RECORD;
    like_counts INTEGER[] := ARRAY[312, 468, 221, 377, 259, 198];
    i INTEGER := 1;
BEGIN
    FOR post_record IN SELECT id FROM posts ORDER BY created_at LOOP
        -- Add likes from random users (we'll just use the same user for simplicity)
        FOR j IN 1..like_counts[i] LOOP
            INSERT INTO likes (post_id, user_id)
            VALUES (post_record.id, (SELECT id FROM users ORDER BY RANDOM() LIMIT 1))
            ON CONFLICT DO NOTHING;
        END LOOP;
        i := i + 1;
    END LOOP;
END $$;
