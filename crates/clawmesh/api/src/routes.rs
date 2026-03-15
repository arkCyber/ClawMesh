use actix_web::{web, web::ServiceConfig};
use crate::{agent, agent_auth, agent_list, agent_reputation, agent_skills, credit, direct_message, friendship, permissions, stats};

/// Configure ClawMesh API routes
pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api/v3")
            // Agent endpoints
            .service(
                web::scope("/agent")
                    // Agent CRUD operations
                    .route("/install", web::post().to(agent::agent_install))
                    .route("/{person_id}", web::put().to(agent::update_agent))
                    .route("/{person_id}/status", web::patch().to(agent::update_agent_status))
                    .route("/{person_id}", web::delete().to(agent::delete_agent))
                    
                    // Heartbeat monitoring
                    .route("/heartbeat/{person_id}", web::get().to(agent::get_agent_heartbeat))
                    .route("/heartbeat/{person_id}", web::post().to(agent::update_agent_heartbeat))
                    
                    // Agent queries
                    .route("/list", web::get().to(agent_list::list_all_agents))
                    .route("/info/{person_id}", web::get().to(agent_list::get_agent_details))
                    .route("/count", web::get().to(agent_list::get_agent_count))
                    .route("/stale", web::get().to(agent_list::get_stale_agents_list))
                    .route("/skill", web::get().to(agent::get_skill))
                    
                    // Authentication
                    .route("/auth/token", web::post().to(agent_auth::generate_agent_token))
                    .route("/auth/refresh", web::post().to(agent_auth::refresh_agent_token))
                    .route("/auth/token/{token_id}", web::delete().to(agent_auth::revoke_agent_token))
                    
                    // Reputation system
                    .route("/{person_id}/reputation", web::get().to(agent_reputation::get_reputation))
                    .route("/{person_id}/reputation/vote", web::post().to(agent_reputation::vote_for_agent))
                    .route("/{person_id}/reputation/history", web::get().to(agent_reputation::get_reputation_history))
                    .route("/{person_id}/reputation/stats", web::get().to(agent_reputation::get_reputation_stats))
                    .route("/reputation/leaderboard", web::get().to(agent_reputation::get_leaderboard))
                    
                    // Skills system
                    .route("/{person_id}/skills", web::post().to(agent_skills::register_agent_skill))
                    .route("/{person_id}/skills", web::get().to(agent_skills::list_agent_skills))
                    .route("/skills/{skill_id}/install", web::post().to(agent_skills::install_agent_skill))
                    .route("/skills/{skill_id}", web::delete().to(agent_skills::delete_agent_skill))
                    .route("/skills/{skill_id}/execute", web::post().to(agent_skills::execute_agent_skill))
                    .route("/skills/{skill_id}/publish", web::post().to(agent_skills::publish_to_marketplace))
                    .route("/skills/marketplace", web::get().to(agent_skills::search_marketplace))
                    .route("/skills/marketplace/stats", web::get().to(agent_skills::get_marketplace_statistics))
            )
            // Credit endpoints
            .service(
                web::scope("/credit")
                    .route("/user/{person_id}", web::get().to(credit::get_user_credit))
                    .route("/history/{person_id}", web::get().to(credit::get_credit_history))
                    .route("/stats/global", web::get().to(stats::get_global_credit_stats))
                    .route("/stats/{person_id}", web::get().to(stats::get_person_credit_stats))
                    .route("/check_permission", web::post().to(permissions::check_permission))
            )
            // Friendship endpoints
            .service(
                web::scope("/friendship")
                    // Friend requests
                    .route("/request", web::post().to(friendship::send_friend_request))
                    .route("/request/respond", web::post().to(friendship::respond_to_request))
                    .route("/request/{id}/cancel", web::delete().to(friendship::cancel_request))
                    .route("/requests/incoming", web::get().to(friendship::get_incoming_requests))
                    .route("/requests/outgoing", web::get().to(friendship::get_outgoing_requests))
                    // Friends list
                    .route("/friends", web::get().to(friendship::get_friends))
                    .route("/friends/{id}", web::delete().to(friendship::remove_friend))
                    .route("/friends/{id}/nickname", web::put().to(friendship::update_friend_nickname))
                    // Blocking
                    .route("/block/{id}", web::post().to(friendship::block_user))
                    .route("/block/{id}", web::delete().to(friendship::unblock_user))
                    .route("/blocked", web::get().to(friendship::get_blocked_users))
                    // Stats
                    .route("/stats", web::get().to(friendship::get_friendship_stats))
            )
            // Direct messaging endpoints
            .service(
                web::scope("/messages")
                    // Send message
                    .route("/send", web::post().to(direct_message::send_direct_message))
                    // Conversations
                    .route("/conversations", web::get().to(direct_message::get_conversations))
                    .route("/conversations/{user_id}", web::get().to(direct_message::get_conversation_messages))
                    .route("/conversations/{user_id}/read", web::post().to(direct_message::mark_conversation_read))
                    .route("/conversations/{user_id}/mute", web::post().to(direct_message::mute_conversation))
                    // Messages
                    .route("/{message_id}", web::delete().to(direct_message::delete_message))
                    // Unread count
                    .route("/unread", web::get().to(direct_message::get_unread_count))
                    // Search
                    .route("/search", web::get().to(direct_message::search_messages))
            )
    );
}
