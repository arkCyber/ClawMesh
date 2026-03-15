//! Simplified translations module for email templates
//! This replaces the rosetta-i18n generated translations

use lemmy_db_schema_file::enums::ModlogKind;
use lemmy_diesel_utils::dburl::DbUrl;
use rosetta_i18n::{Language, LanguageId};

/// Language struct for email translations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lang;

/// English language constant
pub const En: Lang = Lang;

impl Default for Lang {
    fn default() -> Self {
        Self
    }
}

impl Lang {
    /// English language constant
    pub const En: Lang = Lang;
    
    /// Get language from language ID
    pub fn from_language_id(_id: &LanguageId) -> Option<Self> {
        Some(Self::default())
    }
    
    /// Create default English language
    pub fn new() -> Self {
        Self
    }

    // ============ Account emails ============
    
    pub fn password_reset_subject(&self, name: &str) -> String {
        format!("Password reset for {}", name)
    }

    pub fn password_reset_body(&self, link: String, name: &str) -> String {
        format!(
            "Click here to reset your password: {}\n\nUsername: {}",
            link, name
        )
    }

    pub fn verify_email_subject(&self, hostname: &str) -> String {
        format!("Verify your email for {}", hostname)
    }

    pub fn verify_email_body(&self, hostname: &str, name: &str, link: String) -> String {
        format!(
            "Welcome to {}!\n\nUsername: {}\n\nClick here to verify your email: {}",
            hostname, name, link
        )
    }

    pub fn verify_email_body_with_application(&self, hostname: &str, name: &str, link: String) -> String {
        format!(
            "Welcome to {}!\n\nUsername: {}\n\nClick here to verify your email: {}\n\nNote: Your application is pending review.",
            hostname, name, link
        )
    }

    pub fn registration_approved_subject(&self, name: &str) -> String {
        format!("Registration approved for {}", name)
    }

    pub fn registration_approved_body(&self, hostname: &str) -> String {
        format!(
            "Your registration for {} has been approved. You can now log in.",
            hostname
        )
    }

    pub fn registration_denied_subject(&self, name: &str) -> String {
        format!("Registration denied for {}", name)
    }

    pub fn registration_denied_body(&self, hostname: &str) -> String {
        format!("Your registration for {} has been denied.", hostname)
    }

    pub fn registration_denied_reason_body(&self, hostname: &str, reason: &str) -> String {
        format!(
            "Your registration for {} has been denied.\n\nReason: {}",
            hostname, reason
        )
    }

    pub fn email_verified_subject(&self, name: &str) -> String {
        format!("Email verified for {}", name)
    }

    pub fn email_verified_body(&self) -> &'static str {
        "Your email has been verified successfully."
    }

    // ============ Admin emails ============
    
    pub fn new_report_subject(&self, hostname: &str, reported: &str, reporter: &str) -> String {
        format!("New report on {}: {} reported by {}", hostname, reported, reporter)
    }

    pub fn new_report_body(&self, link: &str) -> String {
        format!("A new report has been submitted.\n\nView reports: {}", link)
    }

    pub fn new_application_subject(&self, hostname: &str, applicant: &str) -> String {
        format!("New registration application on {} from {}", hostname, applicant)
    }

    pub fn new_application_body(&self, link: &str) -> String {
        format!("A new user has applied to join.\n\nView applications: {}", link)
    }

    // ============ Notification emails ============
    
    pub fn notification_mentioned_by_subject(&self, name: &str) -> String {
        format!("{} mentioned you", name)
    }

    pub fn notification_mentioned_by_body(&self, link: &DbUrl, content: &str, inbox_link: &str, name: &str) -> String {
        format!(
            "{} mentioned you:\n\n{}\n\nView: {}\n\nInbox: {}",
            name, content, link, inbox_link
        )
    }

    pub fn notification_post_subscribed_subject(&self, post_name: &str) -> String {
        format!("New comment on: {}", post_name)
    }

    pub fn notification_post_subscribed_body(&self, content: &str, link: &DbUrl, inbox_link: String) -> String {
        format!(
            "New comment:\n\n{}\n\nView: {}\n\nInbox: {}",
            content, link, inbox_link
        )
    }

    pub fn notification_community_subscribed_subject(&self, post_name: &str, community: &str) -> String {
        format!("New post in {}: {}", community, post_name)
    }

    pub fn notification_community_subscribed_body(&self, content: &str, link: &DbUrl, inbox_link: String) -> String {
        format!(
            "New post:\n\n{}\n\nView: {}\n\nInbox: {}",
            content, link, inbox_link
        )
    }

    pub fn notification_comment_reply_subject(&self, name: &str) -> String {
        format!("{} replied to your comment", name)
    }

    pub fn notification_comment_reply_body(
        &self,
        link: DbUrl,
        content: &str,
        inbox_link: &str,
        parent_content: &str,
        post_name: &str,
        name: &str,
    ) -> String {
        format!(
            "{} replied to your comment on \"{}\":\n\nYour comment: {}\n\nReply: {}\n\nView: {}\n\nInbox: {}",
            name, post_name, parent_content, content, link, inbox_link
        )
    }

    pub fn notification_post_reply_subject(&self, name: &str) -> String {
        format!("{} replied to your post", name)
    }

    pub fn notification_post_reply_body(
        &self,
        link: DbUrl,
        content: &str,
        inbox_link: &str,
        post_name: &str,
        name: &str,
    ) -> String {
        format!(
            "{} replied to your post \"{}\":\n\n{}\n\nView: {}\n\nInbox: {}",
            name, post_name, content, link, inbox_link
        )
    }

    pub fn notification_private_message_subject(&self, sender: &str) -> String {
        format!("New private message from {}", sender)
    }

    pub fn notification_private_message_body(&self, inbox_link: String, content: &str, sender: &str) -> String {
        format!(
            "{} sent you a private message:\n\n{}\n\nInbox: {}",
            sender, content, inbox_link
        )
    }

    pub fn notification_mod_action_subject(&self, kind: ModlogKind) -> String {
        format!("Moderation action: {:?}", kind)
    }

    pub fn notification_mod_action_body(&self, reason: &str, inbox_link: String) -> String {
        format!(
            "A moderation action was taken.\n\nReason: {}\n\nInbox: {}",
            reason, inbox_link
        )
    }

    pub fn notification_mod_action_reverted_subject(&self, kind: ModlogKind) -> String {
        format!("Moderation action reverted: {:?}", kind)
    }

    pub fn notification_mod_action_reverted_body(&self, reason: &str, inbox_link: String) -> String {
        format!(
            "A moderation action was reverted.\n\nReason: {}\n\nInbox: {}",
            reason, inbox_link
        )
    }

    // ============ Welcome post ============
    
    pub fn welcome_post_title(&self) -> &'static str {
        "Welcome to ClawMesh!"
    }

    pub fn welcome_post_body(&self) -> &'static str {
        "Welcome to ClawMesh, a federated social platform powered by Lemmy.\n\nFeel free to explore communities, create posts, and engage with others.\n\nEnjoy your stay!"
    }

    // ============ Feed titles ============
    
    pub fn subscribed(&self) -> &'static str {
        "Subscribed"
    }

    pub fn notifications(&self) -> &'static str {
        "Notifications"
    }

    pub fn modlog(&self) -> &'static str {
        "Modlog"
    }

    pub fn local(&self) -> &'static str {
        "Local"
    }

    pub fn all(&self) -> &'static str {
        "All"
    }

    // ============ Modlog actions ============
    
    pub fn admin_disallowed_instance_x(&self, instance: &str) -> String {
        format!("Admin disallowed instance: {}", instance)
    }

    pub fn admin_allowed_instance_x(&self, instance: &str) -> String {
        format!("Admin allowed instance: {}", instance)
    }

    pub fn admin_unblocked_instance_x(&self, instance: &str) -> String {
        format!("Admin unblocked instance: {}", instance)
    }

    pub fn admin_blocked_instance_x(&self, instance: &str) -> String {
        format!("Admin blocked instance: {}", instance)
    }

    pub fn admin_purged_comment(&self) -> &'static str {
        "Admin purged comment"
    }

    pub fn admin_purged_community(&self) -> &'static str {
        "Admin purged community"
    }

    pub fn admin_purged_person(&self) -> &'static str {
        "Admin purged person"
    }

    pub fn admin_purged_post(&self) -> &'static str {
        "Admin purged post"
    }

    pub fn added_admin_x(&self, person: &str) -> String {
        format!("Added admin: {}", person)
    }

    pub fn removed_admin_x(&self, person: &str) -> String {
        format!("Removed admin: {}", person)
    }

    pub fn added_mod_x_to_community_y(&self, community: &str, person: &str) -> String {
        format!("Added mod {} to community {}", person, community)
    }

    pub fn removed_mod_x_from_community_y(&self, community: &str, person: &str) -> String {
        format!("Removed mod {} from community {}", person, community)
    }

    pub fn unbanned_user_x(&self, person: &str) -> String {
        format!("Unbanned user: {}", person)
    }

    pub fn banned_user_x(&self, person: &str) -> String {
        format!("Banned user: {}", person)
    }

    pub fn unbanned_user_x_from_community_y(&self, community: &str, person: &str) -> String {
        format!("Unbanned user {} from community {}", person, community)
    }

    pub fn banned_user_x_from_community_y(&self, community: &str, person: &str) -> String {
        format!("Banned user {} from community {}", person, community)
    }

    pub fn featured_post_x(&self, post: &str) -> String {
        format!("Featured post: {}", post)
    }

    pub fn unfeatured_post_x(&self, post: &str) -> String {
        format!("Unfeatured post: {}", post)
    }

    pub fn changed_community_x_visibility(&self, community: &str) -> String {
        format!("Changed community visibility: {}", community)
    }

    pub fn unlocked_post_x(&self, post: &str) -> String {
        format!("Unlocked post: {}", post)
    }

    pub fn locked_post_x(&self, post: &str) -> String {
        format!("Locked post: {}", post)
    }

    pub fn restored_comment_x(&self, comment: &str) -> String {
        format!("Restored comment: {}", comment)
    }

    pub fn removed_comment_x(&self, comment: &str) -> String {
        format!("Removed comment: {}", comment)
    }

    pub fn restored_community_x(&self, community: &str) -> String {
        format!("Restored community: {}", community)
    }

    pub fn removed_community_x(&self, community: &str) -> String {
        format!("Removed community: {}", community)
    }

    pub fn restored_post_x(&self, post: &str) -> String {
        format!("Restored post: {}", post)
    }

    pub fn removed_post_x(&self, post: &str) -> String {
        format!("Removed post: {}", post)
    }

    pub fn transferred_community_x_to_user_y(&self, community: &str, person: &str) -> String {
        format!("Transferred community {} to user {}", community, person)
    }

    pub fn unlocked_comment_x(&self, comment: &str) -> String {
        format!("Unlocked comment: {}", comment)
    }

    pub fn locked_comment_x(&self, comment: &str) -> String {
        format!("Locked comment: {}", comment)
    }

    // ============ Notification types ============
    
    pub fn mention_from_x(&self, name: String) -> String {
        format!("Mention from {}", name)
    }

    pub fn reply_from_x(&self, name: String) -> String {
        format!("Reply from {}", name)
    }

    pub fn private_message_from_x(&self, name: String) -> String {
        format!("Private message from {}", name)
    }

    pub fn mod_action(&self) -> &'static str {
        "Mod action"
    }

    pub fn submitted_post_with_meta_info(
        &self,
        creator_url: url::Url,
        community_name: &str,
        community_url: &url::Url,
        creator_name: &str,
        comments: i32,
        score: i32,
        post_url: &url::Url,
    ) -> String {
        format!(
            "Submitted by <a href=\"{}\">{}</a> to <a href=\"{}\">{}</a><br>Comments: {} | Score: {}<br><a href=\"{}\">View post</a>",
            creator_url, creator_name, community_url, community_name, comments, score, post_url
        )
    }
}

impl Language for Lang {
    fn language_id(&self) -> LanguageId<'_> {
        LanguageId::new("en")
    }
    
    fn from_language_id(_id: &LanguageId<'_>) -> Option<Self> {
        Some(Self)
    }
    
    fn fallback() -> Self {
        Self
    }
}
