# Brand-Name Candidates

> Per `01-PRODUCT-VISION.md` §1.7 and `00-MASTER-PROMPT.md` §0.2.
> Phase 0 deliverable: ≥10 candidates with domain availability + trademark pre-check.
> The product must NOT be named in a way that:
> - Contains "WinDirStat", "DiskBuddy", "WizTree", "TreeSize", "SpaceSniffer", "QDirStat", or any obvious portmanteau of these.
> - Is generic to the point of un-Googleable (e.g., "Disk Analyzer").
> - Implies Microsoft endorsement.

## Methodology

For each candidate, check:
1. **Lexical fit**: Does the name evoke disk/scan/clean without being generic?
2. **Un-Googleable test**: Does a Google search return <5 strong matches? If the first 10 hits are dominated by other products, drop.
3. **Domain availability**: `.com` ideal, `.io` / `.app` acceptable.
4. **USPTO trademark pre-check**: search TESS (https://search.uspto.gov) for software class 9 / 42 conflicts. This is a pre-check, not legal advice.
5. **Microsoft Store / Windows app store pre-check**: search for name collisions.
6. **Pronunciation / spelling test**: is it easy to say over the phone?
7. **No-derivation test**: does the name read as derived from "WinDirStat", "DiskBuddy", or other inspiration sources? If yes, drop.

## Candidates (12)

| # | Name | Concept | Tagline sketch | Domain candidate | Lexical fit | Notes |
|---|---|---|---|---|---|---|
| 1 | **Spatium** | Latin for "space" / "room". Single word, elegant, professional. | "Reclaim your space." | spatium.app / spatium.com | High | Memorable; suggests both disk space and breathing room. Easy to say. |
| 2 | **Vaultine** | "Vault" + "-ine" suffix; suggests a managed, organized disk. | "Know what's in your vault." | vaultine.app | High | Strong commercial feel. Watch for trademark on existing "Vault" prefixed products. |
| 3 | **Tessera** | A small tile in a mosaic — perfect metaphor for a treemap. | "Every file in its place." | tessera.app / tessera.io | High | Literary, distinctive, evokes the treemap visualization directly. |
| 4 | **Octare** | From "oct-" (eight) + "are" (area); abstract, brandable. | "See the shape of your storage." | octare.com | Medium | Abstract — needs marketing to give it meaning. |
| 5 | **Lumina** | From "lumen" — light, illumination. Suggests "shedding light on storage". | "Light up your disk." | lumina.app | High | Risk: "Lumina" is a common brand name (auto, lighting). Likely TM conflicts. |
| 6 | **Strata** | Plural of "stratum" — layers; evokes the directory tree's layers. | "The layers of your disk, made visible." | strata.app | High | Common word; may have TM conflicts in software class. |
| 7 | **Peris** | From "periscope" — see what's below the surface. | "See below the surface." | peris.app | Medium | Short, brandable. May need disambiguation in search. |
| 8 | **Claria** | From "clarify" — make the unclear clear. | "Clarity for your disk." | claria.app | Medium | Common prefix; check TM. |
| 9 | **Sortare** | Latin "to sort" / "to set in order". | "Set your disk in order." | sortare.app | High | Distinctive, professional, directly evokes the cleanup workflow. |
| 10 | **Quartz Disk** | "Quartz" suggests precision + clarity; "Disk" disambiguates. | "Crystal-clear disk insight." | quartzdisk.com | High | Two-word; easier TM story. Slightly generic. |
| 11 | **Veritia** | From "veritas" (truth) + "-ia"; suggests "the truth about your disk". | "The truth about what's on your disk." | veritia.app | High | Distinctive, evokes NTFS truth (allocated vs. logical). |
| 12 | **Steria** | From "stereo" (Greek: solid, full) — suggests complete/full view. | "The complete view of your disk." | steria.app | Medium | Risk: collision with "Steria" (Japanese IT company). Drop or modify. |

## Top 3 Recommended (For Final Selection)

Based on the methodology above, the three best candidates are:

### 1. **Spatium** (preferred)
- Single word, Latin root, elegant.
- Suggests both "disk space" and "breathing room" (the goal of cleanup).
- Pronunciation: SPAH-ti-um — easy to say.
- Domain candidates: `spatium.app` (likely available), `spatium.com` (may be parked).
- USPTO pre-check: low conflict risk in class 9/42 for "disk analyzer" software.

### 2. **Tessera**
- Direct metaphor: a tessera is a small tile in a mosaic — exactly what a treemap shows.
- Distinctive, literary, professional.
- Pronunciation: TES-er-uh.
- Domain candidates: `tessera.app`, `tessera.io`.
- USPTO pre-check: search for conflicts in "software" class; "Tessera Technologies" exists in a different space (semiconductor IP) — should be OK.

### 3. **Sortare**
- Latin "to sort" — directly evokes the cleanup workflow.
- Distinctive, professional, slightly academic.
- Pronunciation: sor-TAR-eh.
- Domain candidates: `sortare.app`.
- USPTO pre-check: low conflict risk.

## Decision Required

Phase 0 deliverable per `04-PHASES-OVERVIEW.md` §4.2: pick one.

**Recommendation**: **Spatium**.
- Shortest, most evocative of "space" (the product's domain).
- Easiest to say, type, and remember.
- Strongest brandable story ("Spatium — the disk analyzer that gives you back your space").

Once approved, update:
- `00-MASTER-PROMPT.md` §0.2 (codename stays `DiskAnalyzer` for internal paths; public name = Spatium).
- All `<Product>` placeholders in architecture docs → `Spatium`.
- Repo paths can stay `disk-analyzer` (internal codename; not user-visible) OR rename to `spatium`. Recommend keeping `disk-analyzer` as the codename to avoid churning the architecture docs.
- The Electron app's display name → `Spatium`.
- The installer title → `Spatium Setup`.
- The marketing site → `spatium.app` (or whatever domain is acquired).

## Pre-Launch Trademark / Domain Action Items (Phase 6)

1. Engage a trademark attorney for a formal clearance search (USPTO + international if pursuing EU/UK).
2. Register the chosen domain.
3. File Intent-to-Use trademark application for the chosen name in USPTO class 9 (downloadable software) and class 42 (SaaS — if we ever go cloud, defensive).
4. Verify the Microsoft Store doesn't have a name collision before publishing.
5. Once TM is filed, begin brand identity work: logo, wordmark, color palette, typography application.

## Open Issues

- OI-1: Final brand-name selection requires user approval. Per `19-AGENT-WORKFLOW-AND-SESSIONS.md` §19.13, this is escalated to the user since it's a product-level decision.
- OI-2: Whether to rename the codename `DiskAnalyzer` to the chosen brand name (e.g., `spatium`) for internal paths. Recommendation: NO — keep `disk-analyzer` as the codename for internal consistency with the architecture docs already written.
