//! ClawMesh Database Schema
//! 
//! Diesel schema definitions for ClawMesh-specific tables

// @generated automatically by Diesel CLI.

diesel::table! {
    friendship (id) {
        id -> Int4,
        user_id_1 -> Int4,
        user_id_2 -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    friend_request (id) {
        id -> Int4,
        sender_id -> Int4,
        recipient_id -> Int4,
        message -> Nullable<Text>,
        status -> Varchar,
        created_at -> Timestamp,
        responded_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_block (id) {
        id -> Int4,
        blocker_id -> Int4,
        blocked_id -> Int4,
        reason -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    friend_nickname (id) {
        id -> Int4,
        user_id -> Int4,
        friend_id -> Int4,
        nickname -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    encryption_key (id) {
        id -> Varchar,
        user_id -> Int4,
        key_data -> Bytea,
        algorithm -> Varchar,
        created_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
        revoked_at -> Nullable<Timestamp>,
        last_used_at -> Nullable<Timestamp>,
        usage_count -> Int8,
        is_active -> Bool,
    }
}

diesel::table! {
    key_rotation_history (id) {
        id -> Int4,
        user_id -> Int4,
        old_key_id -> Varchar,
        new_key_id -> Varchar,
        rotation_reason -> Nullable<Varchar>,
        rotated_at -> Timestamp,
    }
}

diesel::joinable!(friendship -> person (user_id_1));
diesel::joinable!(friend_request -> person (sender_id));
diesel::joinable!(user_block -> person (blocker_id));
diesel::joinable!(friend_nickname -> person (user_id));
diesel::joinable!(encryption_key -> person (user_id));
diesel::joinable!(key_rotation_history -> person (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    friendship,
    friend_request,
    user_block,
    friend_nickname,
    encryption_key,
    key_rotation_history,
);
