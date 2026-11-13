# UBE MIT License Review Outcome

**Resolved: MIT is active through the owner-provided root repository license.**

When the local project was integrated with `https://github.com/eOnoes/UBE`, the remote initial commit already contained:

```text
MIT License
Copyright (c) 2026 eOnoes
```

That `LICENSE` was preserved unchanged. No new license grant or replacement holder wording was invented during integration. The earlier proposal to use “Unbiased Bob Engine contributors” is superseded by the owner-provided license already present in the target repository.

All workspace packages declare `license.workspace = true`, inheriting `license = "MIT"` from the root Cargo workspace. Strict third-party notice generation with cargo-about `--fail` passes. See `licensing.md` and the root `THIRD_PARTY_NOTICES.html` for verification and distribution requirements.

MIT permits free private and commercial use, modification, distribution, and sublicensing without royalties, subject to preserving the copyright/license notice and its warranty/liability terms. Third-party dependency and bundled asset terms remain separate. Future contributor agreements, asset/branding decisions, and public-release legal review remain team responsibilities.
