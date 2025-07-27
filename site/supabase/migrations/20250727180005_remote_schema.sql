drop trigger if exists "update_agent_ratings_updated_at" on "public"."agent_ratings";

drop trigger if exists "agent_version_published" on "public"."agent_versions";

drop trigger if exists "update_agent_versions_updated_at" on "public"."agent_versions";

drop trigger if exists "update_api_tokens_updated_at" on "public"."api_tokens";

drop policy "Agent packages are viewable by everyone for public agents" on "public"."agent_packages";

drop policy "Users can create packages for their own agent versions" on "public"."agent_packages";

drop policy "Users can delete packages for their own agent versions" on "public"."agent_packages";

drop policy "Users can update packages for their own agent versions" on "public"."agent_packages";

drop policy "Agent ratings are viewable by everyone" on "public"."agent_ratings";

drop policy "Users can create their own ratings" on "public"."agent_ratings";

drop policy "Users can delete their own ratings" on "public"."agent_ratings";

drop policy "Users can update their own ratings" on "public"."agent_ratings";

drop policy "Agent versions are viewable by everyone for public agents" on "public"."agent_versions";

drop policy "Users can create versions for their own agents" on "public"."agent_versions";

drop policy "Users can delete versions of their own agents" on "public"."agent_versions";

drop policy "Users can update versions of their own agents" on "public"."agent_versions";

drop policy "Users can create their own API tokens" on "public"."api_tokens";

drop policy "Users can delete their own API tokens" on "public"."api_tokens";

drop policy "Users can update their own API tokens" on "public"."api_tokens";

drop policy "Users can view their own API tokens" on "public"."api_tokens";

drop policy "System can insert download stats" on "public"."download_stats";

drop policy "Users can view download stats for their own agents" on "public"."download_stats";

drop policy "System only access to rate limits" on "public"."rate_limits";

drop policy "Users can create their own follows" on "public"."user_follows";

drop policy "Users can delete their own follows" on "public"."user_follows";

drop policy "Users can view their own follows" on "public"."user_follows";

drop policy "System only access to webhook events" on "public"."webhook_events";

revoke delete on table "public"."agent_packages" from "anon";

revoke insert on table "public"."agent_packages" from "anon";

revoke references on table "public"."agent_packages" from "anon";

revoke select on table "public"."agent_packages" from "anon";

revoke trigger on table "public"."agent_packages" from "anon";

revoke truncate on table "public"."agent_packages" from "anon";

revoke update on table "public"."agent_packages" from "anon";

revoke delete on table "public"."agent_packages" from "authenticated";

revoke insert on table "public"."agent_packages" from "authenticated";

revoke references on table "public"."agent_packages" from "authenticated";

revoke select on table "public"."agent_packages" from "authenticated";

revoke trigger on table "public"."agent_packages" from "authenticated";

revoke truncate on table "public"."agent_packages" from "authenticated";

revoke update on table "public"."agent_packages" from "authenticated";

revoke delete on table "public"."agent_packages" from "service_role";

revoke insert on table "public"."agent_packages" from "service_role";

revoke references on table "public"."agent_packages" from "service_role";

revoke select on table "public"."agent_packages" from "service_role";

revoke trigger on table "public"."agent_packages" from "service_role";

revoke truncate on table "public"."agent_packages" from "service_role";

revoke update on table "public"."agent_packages" from "service_role";

revoke delete on table "public"."agent_ratings" from "anon";

revoke insert on table "public"."agent_ratings" from "anon";

revoke references on table "public"."agent_ratings" from "anon";

revoke select on table "public"."agent_ratings" from "anon";

revoke trigger on table "public"."agent_ratings" from "anon";

revoke truncate on table "public"."agent_ratings" from "anon";

revoke update on table "public"."agent_ratings" from "anon";

revoke delete on table "public"."agent_ratings" from "authenticated";

revoke insert on table "public"."agent_ratings" from "authenticated";

revoke references on table "public"."agent_ratings" from "authenticated";

revoke select on table "public"."agent_ratings" from "authenticated";

revoke trigger on table "public"."agent_ratings" from "authenticated";

revoke truncate on table "public"."agent_ratings" from "authenticated";

revoke update on table "public"."agent_ratings" from "authenticated";

revoke delete on table "public"."agent_ratings" from "service_role";

revoke insert on table "public"."agent_ratings" from "service_role";

revoke references on table "public"."agent_ratings" from "service_role";

revoke select on table "public"."agent_ratings" from "service_role";

revoke trigger on table "public"."agent_ratings" from "service_role";

revoke truncate on table "public"."agent_ratings" from "service_role";

revoke update on table "public"."agent_ratings" from "service_role";

revoke delete on table "public"."agent_versions" from "anon";

revoke insert on table "public"."agent_versions" from "anon";

revoke references on table "public"."agent_versions" from "anon";

revoke select on table "public"."agent_versions" from "anon";

revoke trigger on table "public"."agent_versions" from "anon";

revoke truncate on table "public"."agent_versions" from "anon";

revoke update on table "public"."agent_versions" from "anon";

revoke delete on table "public"."agent_versions" from "authenticated";

revoke insert on table "public"."agent_versions" from "authenticated";

revoke references on table "public"."agent_versions" from "authenticated";

revoke select on table "public"."agent_versions" from "authenticated";

revoke trigger on table "public"."agent_versions" from "authenticated";

revoke truncate on table "public"."agent_versions" from "authenticated";

revoke update on table "public"."agent_versions" from "authenticated";

revoke delete on table "public"."agent_versions" from "service_role";

revoke insert on table "public"."agent_versions" from "service_role";

revoke references on table "public"."agent_versions" from "service_role";

revoke select on table "public"."agent_versions" from "service_role";

revoke trigger on table "public"."agent_versions" from "service_role";

revoke truncate on table "public"."agent_versions" from "service_role";

revoke update on table "public"."agent_versions" from "service_role";

revoke delete on table "public"."api_tokens" from "anon";

revoke insert on table "public"."api_tokens" from "anon";

revoke references on table "public"."api_tokens" from "anon";

revoke select on table "public"."api_tokens" from "anon";

revoke trigger on table "public"."api_tokens" from "anon";

revoke truncate on table "public"."api_tokens" from "anon";

revoke update on table "public"."api_tokens" from "anon";

revoke delete on table "public"."api_tokens" from "authenticated";

revoke insert on table "public"."api_tokens" from "authenticated";

revoke references on table "public"."api_tokens" from "authenticated";

revoke select on table "public"."api_tokens" from "authenticated";

revoke trigger on table "public"."api_tokens" from "authenticated";

revoke truncate on table "public"."api_tokens" from "authenticated";

revoke update on table "public"."api_tokens" from "authenticated";

revoke delete on table "public"."api_tokens" from "service_role";

revoke insert on table "public"."api_tokens" from "service_role";

revoke references on table "public"."api_tokens" from "service_role";

revoke select on table "public"."api_tokens" from "service_role";

revoke trigger on table "public"."api_tokens" from "service_role";

revoke truncate on table "public"."api_tokens" from "service_role";

revoke update on table "public"."api_tokens" from "service_role";

revoke delete on table "public"."download_stats" from "anon";

revoke insert on table "public"."download_stats" from "anon";

revoke references on table "public"."download_stats" from "anon";

revoke select on table "public"."download_stats" from "anon";

revoke trigger on table "public"."download_stats" from "anon";

revoke truncate on table "public"."download_stats" from "anon";

revoke update on table "public"."download_stats" from "anon";

revoke delete on table "public"."download_stats" from "authenticated";

revoke insert on table "public"."download_stats" from "authenticated";

revoke references on table "public"."download_stats" from "authenticated";

revoke select on table "public"."download_stats" from "authenticated";

revoke trigger on table "public"."download_stats" from "authenticated";

revoke truncate on table "public"."download_stats" from "authenticated";

revoke update on table "public"."download_stats" from "authenticated";

revoke delete on table "public"."download_stats" from "service_role";

revoke insert on table "public"."download_stats" from "service_role";

revoke references on table "public"."download_stats" from "service_role";

revoke select on table "public"."download_stats" from "service_role";

revoke trigger on table "public"."download_stats" from "service_role";

revoke truncate on table "public"."download_stats" from "service_role";

revoke update on table "public"."download_stats" from "service_role";

revoke delete on table "public"."rate_limits" from "anon";

revoke insert on table "public"."rate_limits" from "anon";

revoke references on table "public"."rate_limits" from "anon";

revoke select on table "public"."rate_limits" from "anon";

revoke trigger on table "public"."rate_limits" from "anon";

revoke truncate on table "public"."rate_limits" from "anon";

revoke update on table "public"."rate_limits" from "anon";

revoke delete on table "public"."rate_limits" from "authenticated";

revoke insert on table "public"."rate_limits" from "authenticated";

revoke references on table "public"."rate_limits" from "authenticated";

revoke select on table "public"."rate_limits" from "authenticated";

revoke trigger on table "public"."rate_limits" from "authenticated";

revoke truncate on table "public"."rate_limits" from "authenticated";

revoke update on table "public"."rate_limits" from "authenticated";

revoke delete on table "public"."rate_limits" from "service_role";

revoke insert on table "public"."rate_limits" from "service_role";

revoke references on table "public"."rate_limits" from "service_role";

revoke select on table "public"."rate_limits" from "service_role";

revoke trigger on table "public"."rate_limits" from "service_role";

revoke truncate on table "public"."rate_limits" from "service_role";

revoke update on table "public"."rate_limits" from "service_role";

revoke delete on table "public"."user_follows" from "anon";

revoke insert on table "public"."user_follows" from "anon";

revoke references on table "public"."user_follows" from "anon";

revoke select on table "public"."user_follows" from "anon";

revoke trigger on table "public"."user_follows" from "anon";

revoke truncate on table "public"."user_follows" from "anon";

revoke update on table "public"."user_follows" from "anon";

revoke delete on table "public"."user_follows" from "authenticated";

revoke insert on table "public"."user_follows" from "authenticated";

revoke references on table "public"."user_follows" from "authenticated";

revoke select on table "public"."user_follows" from "authenticated";

revoke trigger on table "public"."user_follows" from "authenticated";

revoke truncate on table "public"."user_follows" from "authenticated";

revoke update on table "public"."user_follows" from "authenticated";

revoke delete on table "public"."user_follows" from "service_role";

revoke insert on table "public"."user_follows" from "service_role";

revoke references on table "public"."user_follows" from "service_role";

revoke select on table "public"."user_follows" from "service_role";

revoke trigger on table "public"."user_follows" from "service_role";

revoke truncate on table "public"."user_follows" from "service_role";

revoke update on table "public"."user_follows" from "service_role";

revoke delete on table "public"."webhook_events" from "anon";

revoke insert on table "public"."webhook_events" from "anon";

revoke references on table "public"."webhook_events" from "anon";

revoke select on table "public"."webhook_events" from "anon";

revoke trigger on table "public"."webhook_events" from "anon";

revoke truncate on table "public"."webhook_events" from "anon";

revoke update on table "public"."webhook_events" from "anon";

revoke delete on table "public"."webhook_events" from "authenticated";

revoke insert on table "public"."webhook_events" from "authenticated";

revoke references on table "public"."webhook_events" from "authenticated";

revoke select on table "public"."webhook_events" from "authenticated";

revoke trigger on table "public"."webhook_events" from "authenticated";

revoke truncate on table "public"."webhook_events" from "authenticated";

revoke update on table "public"."webhook_events" from "authenticated";

revoke delete on table "public"."webhook_events" from "service_role";

revoke insert on table "public"."webhook_events" from "service_role";

revoke references on table "public"."webhook_events" from "service_role";

revoke select on table "public"."webhook_events" from "service_role";

revoke trigger on table "public"."webhook_events" from "service_role";

revoke truncate on table "public"."webhook_events" from "service_role";

revoke update on table "public"."webhook_events" from "service_role";

alter table "public"."agent_packages" drop constraint "agent_packages_version_id_file_name_key";

alter table "public"."agent_packages" drop constraint "agent_packages_version_id_fkey";

alter table "public"."agent_ratings" drop constraint "agent_ratings_agent_id_fkey";

alter table "public"."agent_ratings" drop constraint "agent_ratings_agent_id_user_id_key";

alter table "public"."agent_ratings" drop constraint "agent_ratings_rating_check";

alter table "public"."agent_ratings" drop constraint "agent_ratings_user_id_fkey";

alter table "public"."agent_versions" drop constraint "agent_versions_agent_id_fkey";

alter table "public"."agent_versions" drop constraint "agent_versions_agent_id_version_key";

alter table "public"."agents" drop constraint "fk_agents_latest_version";

alter table "public"."api_tokens" drop constraint "api_tokens_token_hash_key";

alter table "public"."api_tokens" drop constraint "api_tokens_user_id_fkey";

alter table "public"."download_stats" drop constraint "download_stats_agent_id_fkey";

alter table "public"."download_stats" drop constraint "download_stats_package_id_fkey";

alter table "public"."download_stats" drop constraint "download_stats_user_id_fkey";

alter table "public"."download_stats" drop constraint "download_stats_version_id_fkey";

alter table "public"."rate_limits" drop constraint "rate_limits_identifier_endpoint_window_start_key";

alter table "public"."user_follows" drop constraint "user_follows_check";

alter table "public"."user_follows" drop constraint "user_follows_follower_id_fkey";

alter table "public"."user_follows" drop constraint "user_follows_follower_id_following_agent_id_key";

alter table "public"."user_follows" drop constraint "user_follows_follower_id_following_user_id_key";

alter table "public"."user_follows" drop constraint "user_follows_following_agent_id_fkey";

alter table "public"."user_follows" drop constraint "user_follows_following_user_id_fkey";

alter table "public"."webhook_events" drop constraint "webhook_events_agent_id_fkey";

alter table "public"."webhook_events" drop constraint "webhook_events_user_id_fkey";

alter table "public"."webhook_events" drop constraint "webhook_events_version_id_fkey";

drop index if exists "public"."idx_trending_agents_author";

drop index if exists "public"."idx_trending_agents_name";

drop index if exists "public"."idx_trending_agents_score";

drop function if exists "public"."agent_search_text"(name text, description text, author_name text, tags text[], keywords text[]);

drop view if exists "public"."agent_stats";

drop function if exists "public"."check_rate_limit"(identifier text, endpoint text, max_requests integer, window_minutes integer);

drop function if exists "public"."cleanup_old_data"();

drop function if exists "public"."create_agent"(agent_name text, description text, author_name text, tags text[], keywords text[], license text, homepage text, repository text, readme text, is_public boolean);

drop function if exists "public"."get_agent_dependencies"(agent_name text);

drop function if exists "public"."get_agent_details"(agent_name text, agent_author text);

drop function if exists "public"."get_popular_tags"(limit_count integer);

drop function if exists "public"."get_user_agent_stats"(target_user_id uuid);

drop function if exists "public"."log_webhook_event"(event_type text, agent_id uuid, version_id uuid, user_id uuid, payload jsonb);

drop function if exists "public"."publish_agent_version"(agent_name text, version text, description text, changelog text, definition_data jsonb, package_data jsonb);

drop function if exists "public"."record_download"(agent_name text, version_text text, user_agent_text text, ip_addr inet);

drop function if exists "public"."refresh_trending_agents"();

drop function if exists "public"."search_agents"(search_query text, tags_filter text[], author_filter text, sort_by text, sort_order text, page_num integer, page_size integer);

drop materialized view if exists "public"."trending_agents";

drop function if exists "public"."trigger_agent_published"();

drop function if exists "public"."validate_api_token"(token_hash text);

alter table "public"."agent_packages" drop constraint "agent_packages_pkey";

alter table "public"."agent_ratings" drop constraint "agent_ratings_pkey";

alter table "public"."agent_versions" drop constraint "agent_versions_pkey";

alter table "public"."api_tokens" drop constraint "api_tokens_pkey";

alter table "public"."download_stats" drop constraint "download_stats_pkey";

alter table "public"."rate_limits" drop constraint "rate_limits_pkey";

alter table "public"."user_follows" drop constraint "user_follows_pkey";

alter table "public"."webhook_events" drop constraint "webhook_events_pkey";

drop index if exists "public"."agent_packages_pkey";

drop index if exists "public"."agent_packages_version_id_file_name_key";

drop index if exists "public"."agent_ratings_agent_id_user_id_key";

drop index if exists "public"."agent_ratings_pkey";

drop index if exists "public"."agent_versions_agent_id_version_key";

drop index if exists "public"."agent_versions_pkey";

drop index if exists "public"."api_tokens_pkey";

drop index if exists "public"."api_tokens_token_hash_key";

drop index if exists "public"."download_stats_pkey";

drop index if exists "public"."idx_agent_packages_version_id";

drop index if exists "public"."idx_agent_ratings_agent_id";

drop index if exists "public"."idx_agent_ratings_created_at";

drop index if exists "public"."idx_agent_ratings_rating";

drop index if exists "public"."idx_agent_ratings_user_id";

drop index if exists "public"."idx_agent_versions_agent_created";

drop index if exists "public"."idx_agent_versions_agent_id";

drop index if exists "public"."idx_agent_versions_created_at";

drop index if exists "public"."idx_agent_versions_download_count";

drop index if exists "public"."idx_agent_versions_not_yanked";

drop index if exists "public"."idx_agent_versions_version";

drop index if exists "public"."idx_agents_author_name";

drop index if exists "public"."idx_agents_current_version";

drop index if exists "public"."idx_agents_download_count";

drop index if exists "public"."idx_agents_keywords";

drop index if exists "public"."idx_agents_public_created";

drop index if exists "public"."idx_agents_public_downloads";

drop index if exists "public"."idx_agents_public_updated";

drop index if exists "public"."idx_agents_search";

drop index if exists "public"."idx_agents_user_name";

drop index if exists "public"."idx_api_tokens_active";

drop index if exists "public"."idx_api_tokens_expires_at";

drop index if exists "public"."idx_api_tokens_is_active";

drop index if exists "public"."idx_api_tokens_token_hash";

drop index if exists "public"."idx_api_tokens_token_prefix";

drop index if exists "public"."idx_api_tokens_user_id";

drop index if exists "public"."idx_download_stats_agent_date";

drop index if exists "public"."idx_download_stats_agent_id";

drop index if exists "public"."idx_download_stats_downloaded_at";

drop index if exists "public"."idx_download_stats_ip_address";

drop index if exists "public"."idx_download_stats_user_id";

drop index if exists "public"."idx_download_stats_version_id";

drop index if exists "public"."idx_rate_limits_created_at";

drop index if exists "public"."idx_rate_limits_identifier_endpoint";

drop index if exists "public"."idx_rate_limits_window_start";

drop index if exists "public"."idx_user_follows_follower_id";

drop index if exists "public"."idx_user_follows_following_agent_id";

drop index if exists "public"."idx_user_follows_following_user_id";

drop index if exists "public"."idx_webhook_events_agent_id";

drop index if exists "public"."idx_webhook_events_processed";

drop index if exists "public"."idx_webhook_events_type";

drop index if exists "public"."rate_limits_identifier_endpoint_window_start_key";

drop index if exists "public"."rate_limits_pkey";

drop index if exists "public"."user_follows_follower_id_following_agent_id_key";

drop index if exists "public"."user_follows_follower_id_following_user_id_key";

drop index if exists "public"."user_follows_pkey";

drop index if exists "public"."webhook_events_pkey";

drop table "public"."agent_packages";

drop table "public"."agent_ratings";

drop table "public"."agent_versions";

drop table "public"."api_tokens";

drop table "public"."download_stats";

drop table "public"."rate_limits";

drop table "public"."user_follows";

drop table "public"."webhook_events";

alter table "public"."agents" drop column "author_name";

alter table "public"."agents" drop column "current_version";

alter table "public"."agents" drop column "download_count";

alter table "public"."agents" drop column "homepage";

alter table "public"."agents" drop column "keywords";

alter table "public"."agents" drop column "latest_version_id";

alter table "public"."agents" drop column "license";

alter table "public"."agents" drop column "readme";

alter table "public"."agents" drop column "repository";

CREATE INDEX idx_agents_name_search ON public.agents USING gin (to_tsvector('english'::regconfig, ((name || ' '::text) || description)));


