-- Z181: scope-bound invitation capabilities and redacted membership lifecycle
-- evidence. Server enrolment policy is shared with Z180; closed is the safe
-- default until an administrator changes it through the server policy API.
ALTER TABLE server ADD COLUMN enrolment_policy TEXT NOT NULL DEFAULT 'closed'
    CHECK (enrolment_policy IN ('closed', 'admin-approved', 'invite-enabled', 'open-registration'));
ALTER TABLE server ADD COLUMN invitation_signing_key TEXT;

CREATE TABLE scope_invitation (
    invitation_id TEXT PRIMARY KEY NOT NULL,
    scope_id TEXT NOT NULL REFERENCES scope(scope_id) ON DELETE CASCADE,
    issuer_principal_id TEXT NOT NULL REFERENCES server_principal(principal_id),
    recipient_principal_id TEXT REFERENCES server_principal(principal_id),
    permitted_role TEXT NOT NULL CHECK (permitted_role IN ('owner', 'editor', 'viewer')),
    token_hash TEXT NOT NULL UNIQUE,
    token_signature TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (
        status IN ('pending', 'pending_admission', 'accepted', 'active', 'cancelled', 'expired', 'revoked')
    ),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    expires_at INTEGER NOT NULL,
    accepted_at INTEGER,
    terminal_at INTEGER,
    correlation_id TEXT NOT NULL
);
CREATE INDEX idx_scope_invitation_scope_status
    ON scope_invitation(scope_id, status, created_at DESC);
CREATE INDEX idx_scope_invitation_recipient
    ON scope_invitation(recipient_principal_id, status);

CREATE TABLE scope_lifecycle_event (
    event_id TEXT PRIMARY KEY NOT NULL,
    scope_id TEXT NOT NULL REFERENCES scope(scope_id) ON DELETE CASCADE,
    actor_principal_id TEXT REFERENCES server_principal(principal_id),
    subject_principal_id TEXT REFERENCES server_principal(principal_id),
    invitation_id TEXT REFERENCES scope_invitation(invitation_id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    outcome TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    occurred_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
CREATE INDEX idx_scope_lifecycle_event_scope_time
    ON scope_lifecycle_event(scope_id, occurred_at DESC);
