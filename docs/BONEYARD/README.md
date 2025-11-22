# BONEYARD - Archived Documentation

**Last Updated**: 2025-11-22
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

### Superseded by Consolidation (November 2025)

**Phase 1** (Nov 20): Documentation restructured into `user/`, `developer/`, and `system/` directories

**Phase 2** (Nov 22): Further consolidation into two master guides: `USER_GUIDE.md` and `DEVELOPER_GUIDE.md`

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

#### Recently Archived (November 22, 2025)

Narrow/scattered docs consolidated into `USER_GUIDE.md` and `DEVELOPER_GUIDE.md`:

**Root-level planning docs** → `BONEYARD/root/`:
- `BUSINESS_MODEL.md`, `COMPETITIVE_POSITIONING.md`, `CAPABILITY_ASSESSMENT.md`
- `GAP_ANALYSIS_2025-11-20.md`, `MISSING_FEATURES_ANALYSIS.md`
- `V0.3_RELEASE_PLAN.md`, `DEMO_SCRIPT.md`
- `CODE_QUALITY_MAINTENANCE_PLAN.md` (→ CI/CD Guide)
- `DIFF_FEATURES_IMPLEMENTATION.md`, `INSTALLER.md`
- `THUMBNAIL_BOUNCE_FEATURE_SUMMARY.md`, `THUMBNAIL_BOUNCE_QUICKSTART.md`

**Server docs** → `BONEYARD/auxin-server/`:
- `DEPLOYMENT.md`, `QUICKSTART.md`, `INSTALL.md` (→ DEVELOPER_GUIDE.md deployment section)
- `FRONTEND_SETUP.md`, `TESTING.md` (→ DEVELOPER_GUIDE.md)
- `BUILD_MACOS_26.md`, `STATUS.md`, `RELEASE_NOTES.md`, `MANUAL_TEST_RESULTS.md`

**GitHub CI docs** → `BONEYARD/github/`:
- `CI_SETUP.md`, `TESTING_STRATEGY.md` (→ `.github/CI_CD_GUIDE.md`)

**Component docs** → `BONEYARD/components/`:
- `Auxin-App/BUILD_AND_TEST.md`, `APP_BUNDLE.md`, `UI_IMPLEMENTATION_SUMMARY.md`
- `Auxin-CLI-Wrapper/USAGE.md`, `CLI_HELP.md`, `VERBOSE_MODE.md`

**Production docs** → `BONEYARD/production/`:
- `PRODUCTION_QUICKSTART.md`, `PRODUCTION_SIGNING.md` (→ DEVELOPER_GUIDE.md)

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

**Always use the active documentation**:

**For Users**:
- **[USER_GUIDE.md](../../USER_GUIDE.md)** - Complete user guide
- `docs/user/` - Detailed topic guides

**For Developers**:
- **[DEVELOPER_GUIDE.md](../../DEVELOPER_GUIDE.md)** - Development & deployment
- `docs/developer/` - Technical reference
- `.github/CI_CD_GUIDE.md` - CI/CD workflows

**For AI Assistants**:
- `docs/system/CLAUDE.md` - System prompt

---

*Last Updated: 2025-11-22*
