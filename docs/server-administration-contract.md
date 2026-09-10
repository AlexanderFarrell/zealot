# Server Administration, Enrolment, and Scope Sharing Contract

**Ticket:** Z179  
**Status:** approved design; implementation is deliberately split into Z180 and
Z181.  
**Prerequisites:** [Core Model Foundation Design](core-model-foundation-design.md)
(Z137/Z138) and [Scope-Based Sync Contract](scope-sync-contract.md) (Z141).

This contract defines server-local administration, human enrolment, invitations,
service principals, and exceptional membership recovery. It is an additive
policy contract, not a migration, API, UI, or invitation-storage implementation.
The core-model contract remains authoritative for servers, principals, scopes,
and ordinary scope roles; the sync contract remains authoritative for
cross-server replication.

## 1. Boundaries and roles

The following roles are orthogonal:

| Role | Meaning | May hold server administration or operate the server? |
|---|---|---|
| human account | A server-local authentication and profile record for a person. | An active account's human principal may receive either or both grants. |
| service principal | A named, revocable, non-human server principal. | No. |
| server administrator | A grant to administer server policy and administration records. | Yes, if granted to an active human account. |
| server operator | A grant to run approved operational functions such as backup, restore, health, and incident procedures. | Yes, if granted to an active human account. |

Server-administrator and server-operator grants are server-local. They do not
convey membership or ordinary read/write access to any unrelated scope. A
service principal cannot hold either grant, has no personal scope, and obtains
scope access only through explicit scope membership. An administrator cannot
make themself an owner of a scope merely because they administer the server.

The built-in scope roles are deliberately small:

| Scope role | Authority |
|---|---|
| owner | Manages scope membership and scope settings, and has the content authority of an editor. |
| editor | Creates and updates content permitted by the scope's schema and lifecycle rules. |
| viewer | Reads permitted scope content. |

Only an active owner may manage membership in v1, except where a future,
explicitly granted `manage_members` capability says otherwise. A recipient may
not invite or grant a role greater than that recipient's active grant. Scope
owners cannot create server administrators.

## 2. Server policy and human-account lifecycle

Each server has one enrolment policy: `closed`, `admin-approved`,
`invite-enabled`, or `open-registration`. `closed` is the default. Policies
are evaluated by server services, never by a client-side shortcut.

* `closed`: an administrator provisions the account; no self-registration or
  invitation creates an active account without administrator action.
* `admin-approved`: a person may request an account, but it requires any
  configured verification and administrator approval.
* `invite-enabled`: a valid invitation may begin admission; verification and
  approval still apply when required by server policy.
* `open-registration`: a person may request an account without an invitation;
  configured verification and approval requirements still apply.

The lifecycle is:

```text
provisioned or requested -> verification and/or approval -> active
active <-> suspended
active or suspended -> recovery -> active or suspended
active or suspended -> disabled or anonymized
```

The exact verification and approval steps are policy configuration, but no
subject becomes active until all required steps succeed. A human principal,
personal scope, and active owner membership are created exactly once when a
human account becomes active. That bootstrap is not a collaboration invitation
flow, and neither a pending account nor a service principal receives a personal
scope.

Disabling prevents authentication and new authorization. Recovery is a
server-local, audited process that restores only the state permitted by policy;
it does not silently reinstate a revoked scope membership.

## 3. Account deletion and retention

Deletion is a controlled server-local lifecycle operation. Before an account is
disabled or anonymized, the service must transfer shared-scope ownership to an
eligible active owner or revoke/close the affected membership while preserving
the invariant that every active scope has an active owner. It must never remove
the final active owner.

The account's personal scope is archived, retained only for the configured
recovery period, then disposed of under the server retention policy. At the end
of retention, the local account is anonymized and audit history retains only
redacted attribution sufficient for integrity, compliance, and incident
investigation. Password hashes, sessions, credentials, and contact details are
not retained as audit data.

## 4. Scope invitations and membership

An invitation is a scope-bound, role-bound, expiring, single-use capability;
it is not an account credential. Its bearer value is opaque and stored only as
a secure hash. The authoritative record includes its scope, bounded role,
issuer, creation and expiry times, state, and redacted audit correlation. A
token is never logged, exported in clear text, or placed in replicated scope
payloads.

The membership lifecycle is:

```text
create -> pending admission (when required) -> accepted -> active
create or pending admission -> cancelled, expired, or revoked
accepted or active -> revoked
```

An active owner creates an invitation only for a role it may grant. For an
existing active local account, acceptance consumes the invitation and creates
or updates the permitted membership. For a new person, acceptance begins the
enrolment path allowed by the current server policy: closed requires
administrator provision/approval; admin-approved requires approval;
invite-enabled permits the invitation to start admission; and open-registration
also permits uninvited registration. Verification and pending approval are
preserved rather than bypassed. Membership becomes active only when both the
account and admission requirements are satisfied.

The issuer or another authorized owner may cancel an unused invitation; an
administrator may revoke it under server policy. Expiry, revocation, replay,
and cancellation make acceptance fail atomically. Successful acceptance
consumes the capability exactly once. Notifications are sent only through
configured local notification channels and must not expose the invitation token
or unnecessary membership details.

Every unauthorised lookup or failed action involving an account, scope,
membership, invitation, or service principal returns a privacy-preserving
response that does not confirm whether the target exists. Authorized audit
views may distinguish the internal reason using redacted data.

## 5. Service principals and credentials

An authorized server-administration service creates a service principal with a
stable local identity, owner/administrator attribution, purpose, status, and
expiry/review policy. It is granted each scope role explicitly by an active
owner; it has neither a personal scope nor inherited access from its creator.

Credentials are issued only to that service principal through approved server
services. Issuance, use where safely recordable, rotation, revocation, expiry,
and failed authentication all produce auditable events. Credential material is
shown only at issuance when the mechanism requires it, stored with the
server's secret-handling mechanism, and redacted from logs, exports, events,
and receipts. Revoking a credential prevents future authentication; revoking a
scope grant prevents access even if another credential remains valid.

## 6. Break-glass membership recovery

Break-glass is a configurable server policy with modes `disabled` (the
default), `one-administrator`, and `two-administrators`. A request must state a
reason, target scope and recovery action, requestor, and correlation ID. In
two-administrator mode, approval requires two distinct active administrators.

A successful request produces an immutable receipt and a time-limited,
explicitly scoped membership-recovery capability. It is not ordinary content
access and cannot read, edit, or export scope items by itself. The configured
lifetime is at most 24 hours and defaults to 60 minutes. The recovery action,
approvals, expiry, use, and outcome are all audited. Where notification is
available, active scope owners and members receive notice; notification failure
does not broaden the capability and is itself recorded.

## 7. Sync and data-locality rules

Accounts, password hashes, sessions, API keys, service credentials, server
policy, administrator/operator grants, local notification settings, and
operator configuration are server-local and never replicate as scope data.
Under the scope-sync contract, only the scope's membership descriptors,
external member-subject mappings, role grants/revocations, and authorization
events needed to evaluate that scope may replicate. Replication does not grant
an incoming subject a local identity or authentication capability.

A replicated external member subject maps to a local human or service principal
only through verified local mapping. Implementations must not match accounts by
name or email. An unmapped descriptor remains a safe audit/display identity and
cannot access local APIs.

## 8. Follow-on command boundaries

Z180 owns server-policy, account-lifecycle, service-principal, credential,
audit, and break-glass commands. Z181 owns membership and invitation commands:
create, approve where required, accept, cancel, revoke, expire, change role,
and remove. Each command receives an authorization context, evaluates policy in
a server service, changes state atomically, and emits a redacted audit event
for every success, denial, and safe failure.

HTTP, CLI, MCP, UI, rules, imports, and sync are consumers of these server
services. None may implement independent authorization, direct database
mutation, or different policy semantics.

SQLite and PostgreSQL implementations must have identical state transitions
and idempotent migrations. Their test matrix must cover every enrolment policy,
invitation replay/expiry/revocation, owner preservation, service credential
redaction and rotation, break-glass approval and expiry, audit emission, and
denied-enumeration behavior. Z138 compatibility requires preserving local
account flows while mapping authenticated local accounts to local principals;
replicated external subjects require verified mappings as above.

## 9. Acceptance gates

Z180 and Z181 are ready to implement only when their command/API surfaces can
be derived from this document without making a new policy decision. Their
verification must also show that personal-scope bootstrap is distinct from
collaboration invitations, that ordinary scope authorization remains
scope-local, and that no local authentication or credential data crosses the
Z141 replication boundary.
