# Server Administration, Enrolment, and Scope Sharing Contract

**Ticket:** Z179  
**Status:** implementation-ready contract revision; live Z179 acceptance remains
pending.
**Revision:** 2026-09-12; reconciled with the Z137/Z138/Z139 foundation and the
merged Z181 membership implementation.
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

### Effective capability matrix

The identity kind, server grant, and scope membership are separate inputs to an
authorization decision. A role label never implies a server grant, and a
server grant never implies scope membership.

| Subject or grant | Server-local capability | Scope-local capability | Explicitly not granted |
|---|---|---|---|
| active human account | Authenticate and resolve to its local human principal | Only the active memberships held by that principal; bootstrap creates one personal owner membership | No scope access from account existence alone; no server administration unless separately granted |
| service principal | Authenticate with an active server-local credential | Only explicitly granted `editor` or `viewer` membership in v1; `owner` is disallowed for service principals | No account, personal scope, inherited creator access, server-admin grant, or server-operator grant |
| server administrator | Set server enrolment policy; provision, approve, suspend, recover, disable, or anonymize accounts; manage server principals, credentials, audit, and break-glass policy | None by this grant | No standing membership, content read/write, or invitation/membership management in unrelated scopes |
| server operator | Run approved health, backup, restore, and incident procedures | None by this grant | No account policy, credential issuance, scope membership, or content access by this grant |
| scope owner | None unless separately granted at server level | `view_items`, `create_items`, `update_items`, `delete_items`, `manage_members`, and `manage_scope_settings` in the active scope | No authority in another scope and no ability to create server administrators |
| scope editor | None unless separately granted at server level | `view_items`, `create_items`, `update_items`, and `delete_items` in the active scope | No membership, scope-settings, or server-administration authority |
| scope viewer | None unless separately granted at server level | `view_items` in the active scope | No content mutation, membership, scope-settings, or server-administration authority |

The server-administrator and server-operator grants may overlap on one active
human account, but their capabilities remain independently auditable. The v1
owner preset is the only built-in `manage_members` grant; adding another
principal or capability requires an explicit policy revision.

## 2. Server policy and human-account lifecycle

Each server has one enrolment policy: `closed`, `admin-approved`,
`invite-enabled`, or `open-registration`. `closed` is the default. Policies
are evaluated by server services, never by a client-side shortcut.

| Policy | Admission entry | Activation authority and requirements | Invitation relationship |
|---|---|---|---|
| `closed` | Server administrator provisions the account; self-registration is unavailable | The server-admin lifecycle service completes any required verification and approval before activation | An invitation may identify a pending admission, but cannot activate an account or bypass administrator action |
| `admin-approved` | A person may request an account, or an invitation may begin admission | Configured verification and explicit server-administrator approval are required | Invitation acceptance preserves the pending approval state |
| `invite-enabled` | A valid invitation may begin admission; uninvited self-registration is unavailable | Configured verification and any required approval are required | The invitation is an admission capability, not authentication or approval by itself |
| `open-registration` | A person may request an account without an invitation; an invitation may also begin admission | Configured verification and approval requirements still apply | The invitation does not bypass the normal account activation checks |

An admission request can remain pending without creating an active principal.
No client may infer activation from an invitation, and no invitation may change
the server's selected policy.

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

In v1, creating, approving, cancelling, and revoking invitations, changing or
removing memberships, and listing invitation metadata require the active
principal's scope-local `manage_members` grant (the owner preset). Listing
current membership requires only the target scope's `view_items` grant so a
scope reader can understand the collaboration boundary. A server-administrator
grant does not create ordinary invitation or membership authority, and server
administrators do not receive a hidden invitation-revocation shortcut. Expiry,
revocation, replay, and cancellation make acceptance fail atomically.
Successful acceptance consumes the capability exactly once. Notifications are
sent only through configured local notification channels and must not expose the
invitation token or unnecessary membership details.

Every unauthorised lookup or failed action involving an account, scope,
membership, invitation, or service principal returns a privacy-preserving
response that does not confirm whether the target exists. Authorized audit
views may distinguish the internal reason using redacted data.

## 5. Service principals and credentials

Z180 owns the server-level service-principal lifecycle: creation, retirement,
purpose/display metadata, status, review/expiry policy, and server-admin audit
attribution. Scope membership remains scope-local and is granted explicitly by
an active owner or another principal with `manage_members`; a service principal
has neither a personal scope nor inherited access from its creator.

The existing `POST /account/scopes/{scope_id}/service-keys` compatibility path
is a bounded v1 exception to the server-level creation sequence. An active
scope owner may use it to create a service principal, grant it only `editor` or
`viewer` in that one scope, and receive a server-local credential once. It
cannot grant `owner`, server administration, or access to another scope. This
path is not the full Z180 service-principal lifecycle; Z180 must preserve these
least-privilege rules and make the principal/membership/credential sequence
transactional and auditable while converging credential issuance, rotation,
revocation, and expiry onto its server service.

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

## 7. Transition authority, audit, and safe failure

The following matrix is normative. A downstream implementation may add an
endpoint or command only when it preserves the listed authority, state, audit,
and failure behavior.

| Subject | Allowed transitions or decisions | Required authority | Audit and safe-failure rule |
|---|---|---|---|
| human account | `requested`/`provisioned` → verification/approval → `active`; `active` ↔ `suspended`; `active`/`suspended` → recovery or `disabled`/`anonymized` | Z180 server services, configured verification, and server-administrator approval where the policy requires it | Record actor, subject, policy, outcome, reason, and correlation. Unauthorized account lookup is neutral and never confirms an account-directory entry. |
| invitation | `pending` → `pending_admission` → `active`, or terminal `cancelled`/`expired`/`revoked`; acceptance is one-time | Scope-local `manage_members` for management; the bearer plus an active eligible principal for acceptance; anonymous acceptance may only create `pending_admission` | Record invitation/scope/actor/outcome/reason/correlation without the bearer. Unauthorized targets use a privacy-safe `404`; authorized expired or terminal capabilities use a non-sensitive conflict response. |
| membership | Active role changes or revocation; final active owner cannot be downgraded or removed | Scope-local `manage_members`, with the requested role bounded by the actor's grant | Record actor, subject, prior/resulting role, outcome, reason, and correlation. Denied or missing targets must not disclose an inaccessible scope or principal. |
| service principal and credential | Provision/retire the principal; issue, rotate, revoke, or expire credentials; add or remove explicit scope grants | Z180 server administration for global lifecycle and credentials; scope-local `manage_members` for scope grants | Record lifecycle and credential metadata without secret material. Unknown or unauthorized principals use a neutral response; credential values never appear in records, logs, events, exports, or replication. |
| break-glass capability | `requested` → `approved` → `active` → `consumed`/`expired`, or `denied` | One active administrator or two distinct active administrators according to configured mode | Immutable receipt, reason, target scope, action, approvals, expiry, use, outcome, and notice result are recorded. The capability cannot become ordinary content access. |

Every state-changing service operation is atomic with its authoritative record
and redacted audit event. A denied operation may expose a safe reason code to
an authorized audit reader, but never a secret, account directory, or
cross-scope existence signal to the caller.

## 8. Sync and data-locality rules

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

## 9. Follow-on command and consumer boundaries

### Ownership matrix

| Owner | Owns | Does not own or infer |
|---|---|---|
| Z179 | This policy, role/capability matrix, enrolment matrix, lifecycle authority, privacy rules, replication locality, and downstream decision gates | Runtime commands, migrations, invitation storage, UI, or peer transport |
| Z180 | Server policy; account admission, verification, suspension, recovery, deletion/anonymization; server grants; service-principal lifecycle and credentials; audit; break-glass | Ordinary scope membership, invitation UI, peer trust, and replication transport |
| Z181 | Scope membership and invitation commands and their current HTTP compatibility surface: list, create, approve, accept, cancel, revoke, role change, and removal | Account creation/authentication, server-admin grants, credential lifecycle, and peer trust/transport |
| Z139 scope authorization service | Resolve an authenticated principal's accessible scopes, default-scope compatibility, effective scope capabilities, and item-level scope guards | Client-side authorization or a server-admin bypass; Z180 supplies account/principal lifecycle inputs and Z181 changes membership state |
| Z140 | Active-scope indicator/switcher, scope-management and member-management UX, scope context, filters, and safe empty/denied states | Server account administration, enrolment policy, role calculation, account-directory enumeration, or direct database mutation |
| Z141 | Cross-server scope manifest, membership/event replication boundary, and sync compatibility | Server credentials, accounts, server policy, admin/operator grants, or peer-trust implementation |
| Z172 | Peer trust, signed transport, verified external-subject mapping, and replication revocation propagation | Local account matching by name/email, server-admin authority, or ordinary scope authorization policy |

### Z140 client contract handoff

The following server-confirmed surfaces are prerequisites for Z140. They are
consumed by the client and are not decisions left to a UI implementer:

| Surface | Server contract | Client rule |
|---|---|---|
| accessible scopes | Return only active scopes for which the authenticated principal has the required view capability; do not expose inaccessible scope IDs or names | Show the returned set, including an explicit no-membership/empty state; never enumerate or guess additional scopes |
| default scope | Resolve the human principal's server-owned `default_scope_id`; use it only when no explicit scope is selected | Treat the default as a convenience, not proof of access to another scope; show unavailable/default errors explicitly |
| effective capabilities | Return the server's effective permission set for the selected scope, including the v1 `manage_members` and `manage_scope_settings` owner grants | Gate controls on server capabilities, not on role-name assumptions; a hidden control is not an authorization check |
| principal/member directory | Membership endpoints are scope-qualified and expose only data allowed to an authorized scope reader; there is no general account directory for invitations | Distinguish server accounts from scope members; never search or display credentials, invitation bearers, or unrelated accounts |
| item scope context | Item/search/planner responses preserve the authorized scope selection and return privacy-safe denial for inaccessible content | Display scope context where useful, especially for shared scopes and cross-scope links; do not retry another scope to infer existence |

The current Z181 HTTP routes already implement the scoped membership and
invitation portion of this handoff. Z180 must provide the server-account and
policy surfaces before Z140 adds server-administration UI.

### Z181 reconciliation record

The merged Z181 implementation matches the policy boundary above: its
membership and invitation commands require the scope-local owner preset for
management, bound roles prevent escalation, final-owner removal is rejected,
anonymous acceptance becomes `pending_admission`, active-principal acceptance
consumes the bearer once, and unauthorized target failures remain privacy-safe.
The implementation records redacted lifecycle metadata and never returns the
bearer from list/read operations.

Z181 does not provision accounts, verify identities, apply server-admin grants,
manage server credentials, or implement break-glass recovery. Those remain
Z180 obligations. Its existing scoped service-key path is the bounded
compatibility exception described in the service-principal section; it does
not widen Z181 into server administration or create a server-wide account
directory.

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

## 10. Acceptance gates

Z180 and Z181 are ready to implement only when their command/API surfaces can
be derived from this document without making a new policy decision. Their
verification must also show that personal-scope bootstrap is distinct from
collaboration invitations, that ordinary scope authorization remains
scope-local, and that no local authentication or credential data crosses the
Z141 replication boundary. Z140 is ready to consume the client contract above
only when its server-owned accessible-scope, default-scope, effective
capability, principal-directory, and item-context responses are available.
