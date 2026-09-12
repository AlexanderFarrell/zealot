-- Z181: scope-bound invitation capabilities and redacted membership lifecycle
-- evidence. Server enrolment policy is shared with Z180; closed is the safe
-- default until an administrator changes it through the server policy API.
ALTER TABLE server
    ADD COLUMN enrolment_policy text NOT NULL DEFAULT 'closed'
    CHECK (enrolment_policy IN ('closed', 'admin-approved', 'invite-enabled', 'open-registration'));
ALTER TABLE server ADD COLUMN invitation_signing_key text;

CREATE TABLE scope_invitation (
    invitation_id uuid PRIMARY KEY,
    scope_id uuid NOT NULL REFERENCES scope(scope_id) ON DELETE CASCADE,
    issuer_principal_id uuid NOT NULL REFERENCES server_principal(principal_id),
    recipient_principal_id uuid REFERENCES server_principal(principal_id),
    permitted_role text NOT NULL CHECK (permitted_role IN ('owner', 'editor', 'viewer')),
    token_hash text NOT NULL UNIQUE,
    token_signature text NOT NULL,
    status text NOT NULL DEFAULT 'pending' CHECK (
        status IN ('pending', 'pending_admission', 'accepted', 'active', 'cancelled', 'expired', 'revoked')
    ),
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    accepted_at timestamptz,
    terminal_at timestamptz,
    correlation_id uuid NOT NULL
);
CREATE INDEX idx_scope_invitation_scope_status
    ON scope_invitation(scope_id, status, created_at DESC);
CREATE INDEX idx_scope_invitation_recipient
    ON scope_invitation(recipient_principal_id, status);

CREATE TABLE scope_lifecycle_event (
    event_id uuid PRIMARY KEY,
    scope_id uuid NOT NULL REFERENCES scope(scope_id) ON DELETE CASCADE,
    actor_principal_id uuid REFERENCES server_principal(principal_id),
    subject_principal_id uuid REFERENCES server_principal(principal_id),
    invitation_id uuid REFERENCES scope_invitation(invitation_id) ON DELETE SET NULL,
    event_type text NOT NULL,
    outcome text NOT NULL,
    reason_code text NOT NULL,
    correlation_id uuid NOT NULL,
    occurred_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_scope_lifecycle_event_scope_time
    ON scope_lifecycle_event(scope_id, occurred_at DESC);
