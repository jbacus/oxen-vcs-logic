# BONEYARD - Archived Documentation

**Last Updated**: 2025-11-20
**Purpose**: Historical reference for deprecated or superseded documentation

---

## What Is This?

The BONEYARD contains documentation that has been:
- **Superseded** by new, reorganized docs
- **Deprecated** but kept for historical reference
- **Planned features** not yet implemented

These files are kept for:
- Historical context and design decisions
- Reference during future development
- Avoiding loss of detailed specifications

---

## Archived Files

### Superseded by Reorganization (November 2025)

These files were replaced when documentation was restructured into `user/`, `developer/`, and `system/` directories:

| Old File | Replaced By | Notes |
|----------|-------------|-------|
| `FOR_MUSICIANS.md` | `user/for-musicians.md` | User guide for audio producers |
| `FOR_DEVELOPERS.md` | `developer/architecture.md`, `developer/development-setup.md` | Developer documentation |
| `CLI_EXAMPLES.md` | `user/cli-reference.md` | CLI usage examples |
| `TROUBLESHOOTING.md` | `user/troubleshooting.md` | Common issues and solutions |
| `TESTING_STRATEGY.md` | `developer/testing.md` | Testing approach |
| `CLOUD_SHARING_GUIDE.md` | `user/cloud-sharing.md` | Team collaboration |
| `EXTENSIBILITY.md` | `developer/extensibility.md` | Adding new applications |
| `SKETCHUP_EXAMPLES.md` | `user/for-modelers.md` | SketchUp-specific guide |
| `SKETCHUP_CONFIGURATION.md` | `user/for-modelers.md` | SketchUp setup |

### Future Features (Phase 8+)

These contain design specifications for features not yet implemented:

| File | Feature | Target Phase |
|------|---------|--------------|
| `FCP_XML_DIFF_TOOL_PLAN.md` | Final Cut Pro XML diffing | Phase 8 |
| `SEMANTIC_AUDIO_DIFF_IMPLEMENTATION.md` | AI-powered audio comparison | Phase 8 |
| `SEMANTIC_DIFF_EXECUTIVE_SUMMARY.md` | Semantic diffing overview | Phase 8 |

### Technical Reference

Design docs and analysis kept for reference:

| File | Purpose |
|------|---------|
| `OXEN_INTEGRATION_ANALYSIS.md` | Analysis of Oxen backend integration |
| `LIBOXEN_MIGRATION.md` | Migration plan for liboxen dependency |
| `MERGE_PROTOCOL.md` | Protocol design for binary merge handling |
| `MIGRATION.md` | Data migration strategies |
| `AUXIN_SERVER.md` | Server architecture notes |

---

## Using Archived Docs

### For Historical Context

When investigating why something was designed a certain way:

```bash
# Find discussions of a topic
grep -l "topic" docs/BONEYARD/*.md

# Compare old and new versions
diff docs/BONEYARD/FOR_MUSICIANS.md docs/user/for-musicians.md
```

### For Future Development

When implementing Phase 8+ features, these specs provide starting points:

- `FCP_XML_DIFF_TOOL_PLAN.md` - Complete implementation plan
- `SEMANTIC_AUDIO_DIFF_IMPLEMENTATION.md` - Technical specifications
- Use as reference but update for current architecture

---

## Cleanup Policy

Files remain in BONEYARD for **6 months** after archival. After that:

1. **Superseded docs**: Can be deleted (content lives in active docs)
2. **Future feature specs**: Keep until feature is implemented
3. **Technical reference**: Keep indefinitely (historical value)

To propose removal, create an issue discussing which files and why.

---

## Do Not Use Directly

These files may contain:
- Outdated paths and references
- Deprecated API calls
- Old project names (Oxen-VCS instead of Auxin)
- Incorrect directory structures

**Always use the active documentation** in:
- `docs/user/` - User guides
- `docs/developer/` - Developer guides
- `docs/system/` - AI assistant context

---

*Last Updated: 2025-11-20*
