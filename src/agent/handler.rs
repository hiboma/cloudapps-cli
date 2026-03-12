use crate::agent::protocol::{AgentRequest, AgentResponse};
use crate::agent::security::{
    AuditLog, AuditResult, CommandWhitelist, RateLimiter, constant_time_eq, validate_command_name,
};
use crate::dispatch;

/// Handle an incoming agent request with security checks.
pub async fn handle_request(
    request: AgentRequest,
    session_token: &str,
    whitelist: &CommandWhitelist,
    rate_limiter: &RateLimiter,
    audit_log: &AuditLog,
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

    match dispatch::dispatch_from_args(&cli_args).await {
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
            // Do not leak detailed error messages to the client.
            AgentResponse::error(request_id, "command execution failed".to_string())
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
}
