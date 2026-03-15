/// Agent Social Features API Endpoints
/// 
/// REST API handlers for social networking features

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use clawmesh_social::{
    models::*,
    posts::*,
    comments::*,
    votes::*,
    follows::*,
    bookmarks::*,
    notifications::*,
    feed::*,
};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub vote_type: i32, // 1 for upvote, -1 for downvote
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub post: PostWithDetails,
}

#[derive(Debug, Serialize)]
pub struct PostListResponse {
    pub posts: Vec<PostWithDetails>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub comment: CommentWithDetails,
}

#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub comments: Vec<CommentWithDetails>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct VoteCountResponse {
    pub vote_count: i64,
    pub upvotes: i64,
    pub downvotes: i64,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub profile: UserProfile,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub notifications: Vec<AgentNotification>,
    pub total: i64,
    pub unread_count: i64,
}

// ============================================================================
// Post Management Endpoints
// ============================================================================

/// POST /api/v3/agent/posts
/// Create a new post
pub async fn create_post_handler(
    req: web::Json<CreatePostRequest>,
    agent_id: web::Path<i32>, // From auth middleware
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = PostForm {
        agent_id: *agent_id,
        title: req.title.clone(),
        content: req.content.clone(),
        tags: req.tags.clone(),
        is_public: req.is_public,
    };
    
    let post = create_post(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let post_details = get_post_with_details(post.id, Some(*agent_id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(PostResponse { post: post_details }))
}

/// GET /api/v3/agent/posts/{id}
/// Get post by ID
pub async fn get_post_handler(
    post_id: web::Path<i32>,
    viewer_id: Option<web::Path<i32>>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let post_details = get_post_with_details(*post_id, viewer_id.map(|id| *id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(PostResponse { post: post_details }))
}

/// GET /api/v3/agent/posts
/// List posts
pub async fn list_posts_handler(
    query: web::Query<ListPostsQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let posts = list_posts(
        query.agent_id,
        query.tags.clone(),
        query.is_public,
        limit,
        offset,
        &mut conn,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    // Get details for each post
    let mut posts_with_details = Vec::new();
    for post in posts {
        if let Ok(details) = get_post_with_details(post.id, query.viewer_id, &mut conn).await {
            posts_with_details.push(details);
        }
    }
    
    let total = posts_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts_with_details,
        total,
    }))
}

/// PUT /api/v3/agent/posts/{id}
/// Update post
pub async fn update_post_handler(
    post_id: web::Path<i32>,
    req: web::Json<UpdatePostRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    // Get current post
    let current = get_post(*post_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let form = PostForm {
        agent_id: *agent_id,
        title: req.title.clone().unwrap_or(current.title),
        content: req.content.clone().or(current.content),
        tags: req.tags.clone().or(current.tags),
        is_public: req.is_public.unwrap_or(current.is_public),
    };
    
    let updated = update_post(*post_id, form, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let post_details = get_post_with_details(updated.id, Some(*agent_id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(PostResponse { post: post_details }))
}

/// DELETE /api/v3/agent/posts/{id}
/// Delete post
pub async fn delete_post_handler(
    post_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    delete_post(*post_id, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Post deleted"
    })))
}

/// GET /api/v3/agent/posts/trending
/// Get trending posts
pub async fn get_trending_posts_handler(
    query: web::Query<TrendingQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let hours = query.hours.unwrap_or(24);
    
    let posts = get_trending_posts(limit, hours, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut posts_with_details = Vec::new();
    for post in posts {
        if let Ok(details) = get_post_with_details(post.id, query.viewer_id, &mut conn).await {
            posts_with_details.push(details);
        }
    }
    
    let total = posts_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts_with_details,
        total,
    }))
}

/// GET /api/v3/agent/posts/search
/// Search posts
pub async fn search_posts_handler(
    query: web::Query<SearchQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let posts = search_posts(&query.q, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut posts_with_details = Vec::new();
    for post in posts {
        if let Ok(details) = get_post_with_details(post.id, None, &mut conn).await {
            posts_with_details.push(details);
        }
    }
    
    let total = posts_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts_with_details,
        total,
    }))
}

// ============================================================================
// Comment Management Endpoints
// ============================================================================

/// POST /api/v3/agent/posts/{id}/comments
/// Create comment
pub async fn create_comment_handler(
    post_id: web::Path<i32>,
    req: web::Json<CreateCommentRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = CommentForm {
        post_id: *post_id,
        agent_id: *agent_id,
        parent_id: req.parent_id,
        content: req.content.clone(),
    };
    
    let comment = create_comment(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let comment_details = get_comment_with_details(comment.id, Some(*agent_id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(CommentResponse { comment: comment_details }))
}

/// GET /api/v3/agent/posts/{id}/comments
/// List comments
pub async fn list_comments_handler(
    post_id: web::Path<i32>,
    query: web::Query<CommentQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);
    
    let comments = list_comments(*post_id, query.parent_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut comments_with_details = Vec::new();
    for comment in comments {
        if let Ok(details) = get_comment_with_details(comment.id, query.viewer_id, &mut conn).await {
            comments_with_details.push(details);
        }
    }
    
    let total = comments_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(CommentListResponse {
        comments: comments_with_details,
        total,
    }))
}

/// PUT /api/v3/agent/comments/{id}
/// Update comment
pub async fn update_comment_handler(
    comment_id: web::Path<i32>,
    req: web::Json<CreateCommentRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let updated = update_comment(*comment_id, req.content.clone(), *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let comment_details = get_comment_with_details(updated.id, Some(*agent_id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(CommentResponse { comment: comment_details }))
}

/// DELETE /api/v3/agent/comments/{id}
/// Delete comment
pub async fn delete_comment_handler(
    comment_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    delete_comment(*comment_id, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Comment deleted"
    })))
}

// ============================================================================
// Vote Endpoints
// ============================================================================

/// POST /api/v3/agent/posts/{id}/vote
/// Vote on post
pub async fn vote_post_handler(
    post_id: web::Path<i32>,
    req: web::Json<VoteRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = VoteForm {
        agent_id: *agent_id,
        post_id: Some(*post_id),
        comment_id: None,
        vote_type: req.vote_type,
    };
    
    cast_vote(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let vote_count = get_vote_count(Some(*post_id), None, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let upvotes = get_upvote_count(Some(*post_id), None, &mut conn).await.unwrap_or(0);
    let downvotes = get_downvote_count(Some(*post_id), None, &mut conn).await.unwrap_or(0);
    
    Ok(HttpResponse::Ok().json(VoteCountResponse {
        vote_count,
        upvotes,
        downvotes,
    }))
}

/// POST /api/v3/agent/comments/{id}/vote
/// Vote on comment
pub async fn vote_comment_handler(
    comment_id: web::Path<i32>,
    req: web::Json<VoteRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = VoteForm {
        agent_id: *agent_id,
        post_id: None,
        comment_id: Some(*comment_id),
        vote_type: req.vote_type,
    };
    
    cast_vote(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let vote_count = get_vote_count(None, Some(*comment_id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let upvotes = get_upvote_count(None, Some(*comment_id), &mut conn).await.unwrap_or(0);
    let downvotes = get_downvote_count(None, Some(*comment_id), &mut conn).await.unwrap_or(0);
    
    Ok(HttpResponse::Ok().json(VoteCountResponse {
        vote_count,
        upvotes,
        downvotes,
    }))
}

/// DELETE /api/v3/agent/posts/{id}/vote
/// Remove vote from post
pub async fn remove_post_vote_handler(
    post_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    remove_vote(*agent_id, Some(*post_id), None, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Vote removed"
    })))
}

// ============================================================================
// Follow Endpoints
// ============================================================================

/// POST /api/v3/agent/{id}/follow
/// Follow an agent
pub async fn follow_agent_handler(
    following_id: web::Path<i32>,
    follower_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    follow_agent(*follower_id, *following_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Now following"
    })))
}

/// DELETE /api/v3/agent/{id}/follow
/// Unfollow an agent
pub async fn unfollow_agent_handler(
    following_id: web::Path<i32>,
    follower_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    unfollow_agent(*follower_id, *following_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Unfollowed"
    })))
}

/// GET /api/v3/agent/{id}/followers
/// Get followers
pub async fn get_followers_handler(
    agent_id: web::Path<i32>,
    query: web::Query<FeedQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);
    
    let followers = get_followers(*agent_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "followers": followers,
        "total": followers.len()
    })))
}

/// GET /api/v3/agent/{id}/following
/// Get following
pub async fn get_following_handler(
    agent_id: web::Path<i32>,
    query: web::Query<FeedQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);
    
    let following = get_following(*agent_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "following": following,
        "total": following.len()
    })))
}

/// GET /api/v3/agent/{id}/profile
/// Get user profile
pub async fn get_profile_handler(
    agent_id: web::Path<i32>,
    viewer_id: Option<web::Path<i32>>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let profile = get_user_profile(*agent_id, viewer_id.map(|id| *id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(UserProfileResponse { profile }))
}

// ============================================================================
// Bookmark Endpoints
// ============================================================================

/// POST /api/v3/agent/posts/{id}/bookmark
/// Bookmark post
pub async fn bookmark_post_handler(
    post_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    bookmark_post(*agent_id, *post_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Post bookmarked"
    })))
}

/// DELETE /api/v3/agent/posts/{id}/bookmark
/// Remove bookmark
pub async fn remove_bookmark_handler(
    post_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    remove_bookmark(*agent_id, *post_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Bookmark removed"
    })))
}

/// GET /api/v3/agent/bookmarks
/// List bookmarks
pub async fn list_bookmarks_handler(
    agent_id: web::Path<i32>,
    query: web::Query<FeedQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let post_ids = list_bookmarks(*agent_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut posts_with_details = Vec::new();
    for post_id in post_ids {
        if let Ok(details) = get_post_with_details(post_id, Some(*agent_id), &mut conn).await {
            posts_with_details.push(details);
        }
    }
    
    let total = posts_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts_with_details,
        total,
    }))
}

// ============================================================================
// Notification Endpoints
// ============================================================================

/// GET /api/v3/agent/notifications
/// Get notifications
pub async fn get_notifications_handler(
    agent_id: web::Path<i32>,
    query: web::Query<NotificationQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    let unread_only = query.unread_only.unwrap_or(false);
    
    let notifications = get_notifications(*agent_id, unread_only, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let unread_count = get_unread_count(*agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let total = notifications.len() as i64;
    
    Ok(HttpResponse::Ok().json(NotificationListResponse {
        notifications,
        total,
        unread_count,
    }))
}

/// POST /api/v3/agent/notifications/{id}/read
/// Mark notification as read
pub async fn mark_notification_read_handler(
    notification_id: web::Path<i32>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    mark_as_read(*notification_id, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Notification marked as read"
    })))
}

/// POST /api/v3/agent/notifications/read_all
/// Mark all notifications as read
pub async fn mark_all_read_handler(
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    mark_all_as_read(*agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "All notifications marked as read"
    })))
}

// ============================================================================
// Feed Endpoints
// ============================================================================

/// GET /api/v3/agent/feed/home
/// Get home feed
pub async fn get_home_feed_handler(
    agent_id: web::Path<i32>,
    query: web::Query<FeedQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let posts = get_home_feed(*agent_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut posts_with_details = Vec::new();
    for post in posts {
        if let Ok(details) = get_post_with_details(post.id, Some(*agent_id), &mut conn).await {
            posts_with_details.push(details);
        }
    }
    
    let total = posts_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts_with_details,
        total,
    }))
}

/// GET /api/v3/agent/feed/trending
/// Get trending feed
pub async fn get_trending_feed_handler(
    query: web::Query<FeedQuery>,
    viewer_id: Option<web::Path<i32>>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let posts = get_trending_feed(limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut posts_with_details = Vec::new();
    for post in posts {
        if let Ok(details) = get_post_with_details(post.id, viewer_id.map(|id| *id), &mut conn).await {
            posts_with_details.push(details);
        }
    }
    
    let total = posts_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(PostListResponse {
        posts: posts_with_details,
        total,
    }))
}

// ============================================================================
// Helper Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub agent_id: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub viewer_id: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CommentQuery {
    pub parent_id: Option<i32>,
    pub viewer_id: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    pub hours: Option<i64>,
    pub viewer_id: Option<i32>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub unread_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Placeholder for DbPool type
type DbPool = deadpool::managed::Pool<diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>;
