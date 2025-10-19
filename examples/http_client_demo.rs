/// HTTP Client Demo
///
/// This example demonstrates using the RavensOne HTTP client to:
/// 1. Fetch posts from the Bluebird backend
/// 2. Like a post
/// 3. Post a comment
///
/// Prerequisites:
/// - Bluebird backend must be running on http://localhost:9000
/// - Run: cd examples/bluebird-backend && ./target/release/bluebird-backend

use ravensone_compiler::stdlib::http;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🐦 Bluebird API Demo - HTTP Client");
    println!("=====================================\n");

    // Create HTTP client with base URL
    let client = http::HttpClient::new()
        .with_base_url("http://localhost:9000".to_string())
        .with_header("Content-Type".to_string(), "application/json".to_string());

    // 1. Fetch all posts
    println!("📥 Fetching posts from API...");
    match client.get("/api/posts").send().await {
        Ok(response) => {
            if response.is_ok() {
                match response.json() {
                    Ok(posts) => {
                        println!("✅ Successfully fetched posts!");
                        println!("Response: {}\n", serde_json::to_string_pretty(&posts).unwrap());

                        // Extract first post ID for next examples
                        if let Some(first_post) = posts.as_array().and_then(|arr| arr.first()) {
                            let post_id = first_post["id"].as_str().unwrap_or("");

                            // 2. Like a post
                            println!("❤️  Liking post {}...", post_id);
                            let like_data = json!({
                                "user_id": "6ecfe611-2677-4eb6-8d2a-f7e627e23d6a"  // Alex Chen's ID
                            });

                            match http::post_json(&format!("http://localhost:9000/api/posts/{}/like", post_id), like_data).await {
                                Ok(like_response) => {
                                    if like_response.is_ok() {
                                        println!("✅ Successfully liked post!");
                                        println!("Response: {}\n", like_response.text());
                                    } else {
                                        println!("❌ Failed to like post: HTTP {}", like_response.status);
                                    }
                                }
                                Err(e) => println!("❌ Error liking post: {}\n", e),
                            }

                            // 3. Get comments for the post
                            println!("💬 Fetching comments for post {}...", post_id);
                            match http::get(&format!("http://localhost:9000/api/posts/{}/comments", post_id)).await {
                                Ok(comments_response) => {
                                    if comments_response.is_ok() {
                                        match comments_response.json() {
                                            Ok(comments) => {
                                                println!("✅ Successfully fetched comments!");
                                                println!("Response: {}\n", serde_json::to_string_pretty(&comments).unwrap());
                                            }
                                            Err(e) => println!("❌ Failed to parse comments JSON: {}\n", e),
                                        }
                                    }
                                }
                                Err(e) => println!("❌ Error fetching comments: {}\n", e),
                            }

                            // 4. Post a comment
                            println!("💬 Posting a comment...");
                            let comment_data = json!({
                                "user_id": "6ecfe611-2677-4eb6-8d2a-f7e627e23d6a",
                                "content": "This is amazing! Posted from RavensOne HTTP client 🚀"
                            });

                            match http::post_json(&format!("http://localhost:9000/api/posts/{}/comments", post_id), comment_data).await {
                                Ok(comment_response) => {
                                    if comment_response.is_ok() {
                                        println!("✅ Successfully posted comment!");
                                        match comment_response.json() {
                                            Ok(comment) => {
                                                println!("Response: {}\n", serde_json::to_string_pretty(&comment).unwrap());
                                            }
                                            Err(e) => println!("Failed to parse comment JSON: {}", e),
                                        }
                                    } else {
                                        println!("❌ Failed to post comment: HTTP {}", comment_response.status);
                                    }
                                }
                                Err(e) => println!("❌ Error posting comment: {}\n", e),
                            }
                        }
                    }
                    Err(e) => println!("❌ Failed to parse JSON: {}", e),
                }
            } else {
                println!("❌ HTTP error: {}", response.status);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("\n⚠️  Make sure the Bluebird backend is running:");
            println!("   cd examples/bluebird-backend");
            println!("   ./target/release/bluebird-backend");
        }
    }

    println!("\n🎉 Demo complete!");
}
