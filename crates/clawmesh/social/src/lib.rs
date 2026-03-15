/// Agent Social Features Module
/// 
/// Provides social networking capabilities for agents including
/// posts, comments, voting, following, and content discovery

pub mod models;
pub mod posts;
pub mod comments;
pub mod votes;
pub mod follows;
pub mod bookmarks;
pub mod notifications;
pub mod feed;

pub use models::{
    AgentPost,
    PostForm,
    AgentComment,
    CommentForm,
    AgentVote,
    VoteForm,
    AgentFollow,
    FollowForm,
    AgentBookmark,
    BookmarkForm,
    AgentNotification,
    NotificationForm,
    VoteType,
    NotificationType,
};

pub use posts::{
    create_post,
    get_post,
    list_posts,
    update_post,
    delete_post,
    get_trending_posts,
    search_posts,
};

pub use comments::{
    create_comment,
    get_comment,
    list_comments,
    update_comment,
    delete_comment,
    get_comment_tree,
};

pub use votes::{
    cast_vote,
    remove_vote,
    get_vote_count,
    get_user_vote,
};

pub use follows::{
    follow_agent,
    unfollow_agent,
    get_followers,
    get_following,
    is_following,
};

pub use bookmarks::{
    bookmark_post,
    remove_bookmark,
    list_bookmarks,
    is_bookmarked,
};

pub use notifications::{
    create_notification,
    get_notifications,
    mark_as_read,
    mark_all_as_read,
    get_unread_count,
};

pub use feed::{
    get_home_feed,
    get_user_feed,
    get_trending_feed,
};
