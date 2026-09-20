# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.2.x   | :white_check_mark: (development pre-release) |
| < 0.2   | :x:                |

## Reporting a Vulnerability

Spatium takes security vulnerabilities seriously. We use a multi-layer security architecture per `15-SECURITY-ARCHITECTURE.md`, and we want to know if you find a way around it.

**How to report**:

1. **DO NOT open a public GitHub issue** for security vulnerabilities.
2. Email `4011umar@gmail.com` with the subject line `[SECURITY] Spatium — <short description>`.
3. Include:
   - Affected version (run `Spatium --version` or check Help → About).
   - Steps to reproduce (be as detailed as possible).
   - Impact assessment (what an attacker could achieve).
   - Suggested fix (if any).
4. You will receive an acknowledgement within 48 hours.
5. We will coordinate a fix and disclosure timeline with you.

**In scope**:

- Bypass of license capability checks (per `14-LICENSING-CLIENT.md` §14.12).
- Forged Ed25519 entitlements (per `13-LICENSING-SERVER.md` §13.8).
- Update channel MITM (per `15-SECURITY-ARCHITECTURE.md` §15.6).
- Renderer sandbox escape (per `15-SECURITY-ARCHITECTURE.md` §15.4).
- Privileged helper IPC abuse (per `15-SECURITY-ARCHITECTURE.md` §15.5).
- Path-injection / XSS in user-visible UI.

**Out of scope** (per `00-MASTER-PROMPT.md` §0.5 P5 — "casual bypass, not determined reverse engineer"):

- A user modifying their own binary to bypass the verifyEntitlement function (we accept this risk; the architecture is layered so a single patch doesn't unlock server-side capabilities).
- A user extracting their own entitlement file (it's their data).
- Brute-forcing the demo key (it's dev-only; the prod server rejects it).
- Memory-corruption exploits in third-party code (file upstream with the upstream project).

## Security Architecture

See [`15-SECURITY-ARCHITECTURE.md`](https://github.com/tensorkernel/spatium/blob/main/docs/architecture/15-SECURITY-ARCHITECTURE.md) for the full threat model and the 10-layer mitigation map.

The trusted computing base is:
1. The Ed25519 private signing key (server-side only, never in the client binary).
2. The licensing server (we operate it).
3. The Authenticode signing key (CI/CD only, in Azure Trusted Signing or a hardware token).
4. The Rust core's `verify_entitlement` function (audited).
5. The embedded public key in the client binary.
6. The OS-level process isolation.
7. The updater's signature verification.

If any of these is compromised, see `apps/licensing-server/RUNBOOK.md` (Phase 5 production) for the rotation procedure.
