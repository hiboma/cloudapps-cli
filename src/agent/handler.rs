use crate::agent::protocol::{AgentRequest, AgentResponse};
use crate::agent::security::{
    AuditLog, AuditResult, CommandWhitelist, RateLimiter, constant_time_eq, validate_command_name,
};
use crate::config::CloudAppsCredentials;
use crate::dispatch;

/// Handle an incoming agent request with security checks.
pub async fn handle_request(
    request: AgentRequest,
    session_token: &str,
    whitelist: &CommandWhitelist,
    rate_limiter: &RateLimiter,
    audit_log: &AuditLog,
    credentials: &CloudAppsCredentials,
) -> AgentResponse {
    let request_id = request.request_id.clone();

    // 1. Token verification (constant-time).
    if !constant_time_eq(&request.token, session_token) {
        audit_log.log(AuditLog::entry(
            request_id.clone(),
            request.command.clone(),
            request.action.clone(),
            None,
            AuditResult::Denied("invalid token".to_string()),
        ));
        return AgentResponse::denied(request_id, "authentication failed".to_string());
    }

    // 2. Command name validation.
    if !validate_command_name(&request.command) {
        audit_log.log(AuditLog::entry(
            request_id.clone(),
            request.command.clone(),
            request.action.clone(),
            None,
            AuditResult::Denied("invalid command name".to_string()),
        ));
        return AgentResponse::denied(request_id, "invalid command".to_string());
    }

    if !validate_command_name(&request.action) {
        audit_log.log(AuditLog::entry(
            request_id.clone(),
            request.command.clone(),
            request.action.clone(),
            None,
            AuditResult::Denied("invalid action name".to_string()),
        ));
        return AgentResponse::denied(request_id, "invalid command".to_string());
    }

    // 3. Whitelist check.
    if !whitelist.is_allowed(&request.command) {
        audit_log.log(AuditLog::entry(
            request_id.clone(),
            request.command.clone(),
            request.action.clone(),
            None,
            AuditResult::Denied("command not whitelisted".to_string()),
        ));
        return AgentResponse::denied(request_id, "command not allowed".to_string());
    }

    // 4. Rate limit check.
    if !rate_limiter.try_acquire() {
        audit_log.log(AuditLog::entry(
            request_id.clone(),
            request.command.clone(),
            request.action.clone(),
            None,
            AuditResult::Denied("rate limited".to_string()),
        ));
        return AgentResponse::denied(request_id, "rate limited".to_string());
    }

    // 5. Build CLI args and dispatch.
    let cli_args = build_cli_args(&request);

    match dispatch::dispatch_from_args(&cli_args, credentials).await {
        Ok(output) => {
            audit_log.log(AuditLog::entry(
                request_id.clone(),
                request.command.clone(),
                request.action.clone(),
                None,
                AuditResult::Allowed,
            ));
            AgentResponse::success(request_id, output)
        }
        Err(e) => {
            audit_log.log(AuditLog::entry(
                request_id.clone(),
                request.command.clone(),
                request.action.clone(),
                None,
                AuditResult::Error(e.to_string()),
            ));
            // The client is already authenticated via peer UID/binary verification,
            // so returning the detailed error is safe and aids debugging.
            AgentResponse::error(request_id, e.to_string())
        }
    }
}

/// Build CLI argument vector from an AgentRequest.
/// Reconstructs: ["cloudapps", <command>, <action>, ...args]
fn build_cli_args(request: &AgentRequest) -> Vec<String> {
    let mut args = vec![
        "cloudapps".to_string(),
        request.command.clone(),
        request.action.clone(),
    ];
    args.extend(request.args.iter().cloned());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cli_args() {
        let req = AgentRequest {
            token: "token".to_string(),
            request_id: "req-1".to_string(),
            command: "alerts".to_string(),
            action: "list".to_string(),
            args: vec!["--severity".to_string(), "HIGH".to_string()],
        };
        let args = build_cli_args(&req);
        assert_eq!(
            args,
            vec!["cloudapps", "alerts", "list", "--severity", "HIGH"]
        );
    }

    #[tokio::test]
    async fn test_invalid_token_denied() {
        let whitelist = CommandWhitelist::new(["alerts"].iter().map(|s| s.to_string()).collect());
        let rate_limiter = RateLimiter::new(60);
        let audit_log = AuditLog::new();
        let credentials = CloudAppsCredentials::default();

        let req = AgentRequest {
            token: "wrong-token".to_string(),
            request_id: "req-1".to_string(),
            command: "alerts".to_string(),
            action: "list".to_string(),
            args: vec![],
        };

        let resp = handle_request(
            req,
            "correct-token",
            &whitelist,
            &rate_limiter,
            &audit_log,
            &credentials,
        )
        .await;
        assert_eq!(resp.status, crate::agent::protocol::ResponseStatus::Denied);
        assert_eq!(resp.error.unwrap(), "authentication failed");
    }

    #[tokio::test]
    async fn test_command_not_whitelisted() {
        let whitelist = CommandWhitelist::new(["alerts"].iter().map(|s| s.to_string()).collect());
        let rate_limiter = RateLimiter::new(60);
        let audit_log = AuditLog::new();
        let credentials = CloudAppsCredentials::default();

        let req = AgentRequest {
            token: "valid-token".to_string(),
            request_id: "req-1".to_string(),
            command: "files".to_string(),
            action: "list".to_string(),
            args: vec![],
        };

        let resp = handle_request(
            req,
            "valid-token",
            &whitelist,
            &rate_limiter,
            &audit_log,
            &credentials,
        )
        .await;
        assert_eq!(resp.status, crate::agent::protocol::ResponseStatus::Denied);
        assert_eq!(resp.error.unwrap(), "command not allowed");
    }

    #[tokio::test]
    async fn test_unknown_args_rejected() {
        let whitelist = CommandWhitelist::new(["alerts"].iter().map(|s| s.to_string()).collect());
        let rate_limiter = RateLimiter::new(60);
        let audit_log = AuditLog::new();
        let credentials = CloudAppsCredentials::default();

        let req = AgentRequest {
            token: "valid-token".to_string(),
            request_id: "req-1".to_string(),
            command: "alerts".to_string(),
            action: "list".to_string(),
            args: vec!["--unknown-flag".to_string(), "some-value".to_string()],
        };

        let resp = handle_request(
            req,
            "valid-token",
            &whitelist,
            &rate_limiter,
            &audit_log,
            &credentials,
        )
        .await;
        assert_eq!(resp.status, crate::agent::protocol::ResponseStatus::Error);
        assert!(resp.error.unwrap().contains("unexpected argument"));
    }

    #[tokio::test]
    async fn test_invalid_command_name_rejected() {
        let whitelist = CommandWhitelist::new(["alerts"].iter().map(|s| s.to_string()).collect());
        let rate_limiter = RateLimiter::new(60);
        let audit_log = AuditLog::new();
        let credentials = CloudAppsCredentials::default();

        let req = AgentRequest {
            token: "valid-token".to_string(),
            request_id: "req-1".to_string(),
            command: "../etc/passwd".to_string(),
            action: "list".to_string(),
            args: vec![],
        };

        let resp = handle_request(
            req,
            "valid-token",
            &whitelist,
            &rate_limiter,
            &audit_log,
            &credentials,
        )
        .await;
        assert_eq!(resp.status, crate::agent::protocol::ResponseStatus::Denied);
        assert_eq!(resp.error.unwrap(), "invalid command");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let whitelist = CommandWhitelist::new(["alerts"].iter().map(|s| s.to_string()).collect());
        let rate_limiter = RateLimiter::new(1); // 1 per minute
        let audit_log = AuditLog::new();
        let credentials = CloudAppsCredentials::default();

        // First request exhausts the token.
        let req1 = AgentRequest {
            token: "valid-token".to_string(),
            request_id: "req-1".to_string(),
            command: "alerts".to_string(),
            action: "list".to_string(),
            args: vec![],
        };
        let _ = handle_request(
            req1,
            "valid-token",
            &whitelist,
            &rate_limiter,
            &audit_log,
            &credentials,
        )
        .await;

        // Second request should be rate limited.
        let req2 = AgentRequest {
            token: "valid-token".to_string(),
            request_id: "req-2".to_string(),
            command: "alerts".to_string(),
            action: "list".to_string(),
            args: vec![],
        };
        let resp = handle_request(
            req2,
            "valid-token",
            &whitelist,
            &rate_limiter,
            &audit_log,
            &credentials,
        )
        .await;
        assert_eq!(resp.status, crate::agent::protocol::ResponseStatus::Denied);
        assert_eq!(resp.error.unwrap(), "rate limited");
    }
}
