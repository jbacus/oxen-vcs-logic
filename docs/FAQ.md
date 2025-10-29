# Frequently Asked Questions (FAQ)

**Last Updated:** 2025-10-29
**Version:** 0.1-beta

---

## Table of Contents

- [General Questions](#general-questions)
- [Technical Questions](#technical-questions)
- [Workflow Questions](#workflow-questions)
- [Collaboration Questions](#collaboration-questions)
- [Performance & Storage](#performance--storage)
- [Troubleshooting](#troubleshooting)
- [Comparison to Other Tools](#comparison-to-other-tools)

---

## General Questions

### What is Oxen-VCS?

Oxen-VCS is a version control system designed specifically for Apple Logic Pro projects. It tracks changes to your music productions, enables safe experimentation, and supports team collaboration without the data loss and conflicts common with traditional version control systems like Git.

### Do I need to know how to use Git or version control?

No! Oxen-VCS is designed to be intuitive for music producers, not just software developers. The app provides a user-friendly interface, and most versioning happens automatically in the background.

### Is Oxen-VCS free?

Yes, Oxen-VCS is open-source and free to use. The underlying Oxen.ai tool is also free for local use. Remote collaboration via Oxen Hub has free and paid tiers depending on storage needs.

### What versions of Logic Pro are supported?

**Supported:**
- Logic Pro 11.x (folder-based `.logicx` projects)
- Logic Pro 10.8+ (folder-based projects)

**Not Supported:**
- Package-based projects (`.logic` or packaged `.logicx` files)
- Logic Pro 9 and earlier

To check your project format:
- Right-click project in Finder
- If you see "Show Package Contents", it's packaged (not supported)
- If it opens as a folder, it's folder-based (supported)

### Does this work with other DAWs?

Currently, only Logic Pro is supported. The architecture could be extended to:
- Ableton Live (`.als` projects)
- Pro Tools (`.ptx` sessions)
- Cubase (`.cpr` projects)

These may be added in future versions based on user demand.

### Does Oxen-VCS replace backups?

**No!** Oxen-VCS tracks changes and enables rollback, but it is **not a backup solution**.

**You still need:**
- Time Machine or other backup system
- Cloud storage (iCloud, Dropbox, etc.)
- Off-site backups for disaster recovery

**What Oxen-VCS provides:**
- Version history (rollback to any commit)
- Experimental safety (undo changes)
- Collaboration support (multi-user workflows)

**What it doesn't replace:**
- Backup against hard drive failure
- Backup against ransomware
- Off-site disaster recovery

**Recommendation:** Use both Oxen-VCS (for versioning) and Time Machine (for backups).

---

## Technical Questions

### How is Oxen-VCS different from Git with Git-LFS?

| Feature | Git + Git-LFS | Oxen-VCS |
|---------|---------------|----------|
| **Storage Method** | Stores entire files on change | Block-level deduplication |
| **Storage Efficiency** | 10-100x bloat common | Minimal overhead (~10-20%) |
| **Binary Merge** | Unresolvable conflicts | Prevented via pessimistic locking |
| **DAW Understanding** | None | Native Logic Pro support |
| **Large File Speed** | Slow (full file transfers) | Fast (block-level sync) |
| **Audio Optimization** | No | Yes (designed for media files) |

**Key Difference:** Oxen only stores changed blocks, not entire files. For audio/binary projects, this is 10-100x more efficient than Git-LFS.

### How much disk space does versioning use?

**Typical Overhead:**
- Small projects (<1GB): +100-200MB for full history
- Medium projects (1-5GB): +500MB-1GB
- Large projects (10-50GB): +2-5GB

**Why so efficient?**
- Block-level deduplication: Only changed data stored
- Audio files rarely change entirely (only edits stored)
- .oxenignore excludes large regenerable files (bounces, freezes)

**Example:**
- 5GB Logic Pro project
- 50 commits over 3 months
- Total repository size: ~6.5GB (1.5GB overhead = 30%)

Compare to Git-LFS:
- Same project
- 50 commits
- Would be ~25-50GB (5-10x bloat)

### Do I need an internet connection?

**Local Use:** No internet required
- All versioning works offline
- Daemon runs locally
- Commits stored on your machine

**Remote Sync:** Internet required for:
- Pushing commits to Oxen Hub
- Pulling collaborator changes
- Initial clone of shared project

### Does Oxen-VCS slow down Logic Pro?

**No.** Oxen-VCS runs as a background daemon and:
- Monitors file changes without impacting Logic Pro
- Waits 30-60 seconds after you stop editing before committing
- Uses minimal CPU (<1% when idle, <5% during commit)
- Does not interfere with audio playback or recording

**Performance Impact:**
- Save operations: No noticeable difference
- Project opening: No difference
- Playback/Recording: Zero impact
- Export/Bounce: No impact (unless committing simultaneously)

### Can I use Oxen-VCS with iCloud Drive or Dropbox?

**Not Recommended.**

**Problems:**
- Cloud sync conflicts with Oxen's versioning
- iCloud/Dropbox try to version the `.oxen/` directory
- Can cause corruption or conflicts
- Performance degradation

**Better Approach:**
1. Store projects **locally** (not in iCloud/Dropbox)
2. Use **Oxen Hub** for remote sync (if needed)
3. Backup to iCloud/Dropbox separately (via Time Machine or copy)

**If you must use cloud storage:**
- Exclude `.oxen/` directory from sync
- Expect potential issues

---

## Workflow Questions

### What's the difference between draft commits and milestone commits?

| Feature | Draft Commits | Milestone Commits |
|---------|---------------|-------------------|
| **Creation** | Automatic (after 30-60s inactivity) | Manual (via UI) |
| **Purpose** | Safety net, granular history | Mark important versions |
| **Branch** | `draft` branch | `main` branch (or feature branches) |
| **Metadata** | Timestamp only | BPM, sample rate, key, tags |
| **Lifespan** | Pruned after milestone commits | Permanent |
| **Use Case** | Recover from mistakes | Share with team, client deliverables |

**Analogy:**
- **Draft Commits:** Like Cmd+S (save frequently, automatic)
- **Milestone Commits:** Like "Save As..." with a meaningful name

### How often should I create milestone commits?

**Recommended Frequency:**
- **After significant progress:** New section, tracking session complete
- **Before major changes:** About to rearrange, try new idea
- **End of work session:** Before closing Logic Pro for the day
- **Client deliverables:** Mix versions, master versions
- **Collaboration handoffs:** Before passing project to teammate

**Anti-Patterns:**
- Every 5 minutes (let drafts handle this)
- Never (defeats the purpose)
- Only when project is "done" (too infrequent)

**Good Rhythm:**
- 2-5 milestone commits per 4-hour session
- 10-20 milestones per project (from start to completion)

### Can I exclude certain files from version control?

Yes! Edit `.oxenignore` in your project folder.

**Already Excluded by Default:**
```
Bounces/
Freeze Files/
Autosave/
.DS_Store
*.nosync
```

**To Add Custom Exclusions:**
1. Open `MyProject.logicx/.oxenignore` in a text editor
2. Add patterns (one per line):
   ```
   # Exclude large sample library
   Samples/Orchestra/

   # Exclude video files
   *.mov

   # Exclude personal notes
   NOTES_PRIVATE.md
   ```
3. Save and close

**Note:** Existing tracked files won't be automatically un-tracked. To remove from tracking:
```bash
cd MyProject.logicx
oxen rm --cached Samples/Orchestra/
# Then add to .oxenignore
```

### What happens if Logic Pro crashes while Oxen-VCS is running?

**Good News:** Your work is protected.

**What Happens:**
1. Last draft commit (before crash) is preserved
2. Any changes since last commit are in project file
3. macOS may have AutoSave backup in `Autosave/` folder
4. On reopen, Oxen-VCS resumes monitoring
5. Next save triggers new draft commit

**Recovery Steps:**
1. Reopen Logic Pro
2. Logic Pro may offer to recover from AutoSave
3. Check Oxen-VCS history for latest commit
4. If recent work is missing, rollback to last draft
5. Redo lost work (usually minimal)

**Prevention:**
- Enable Logic Pro AutoSave (Preferences → General → AutoSave)
- Create milestone commits frequently (before risky operations)

### Can I version control just part of a project?

**No.** Logic Pro projects are atomic - you cannot selectively version individual tracks or regions.

**Why:**
- ProjectData file is binary and indivisible
- All tracks, automation, plugins are in one file
- Partial versioning would corrupt the project

**Workarounds:**
- **Feature Branches:** Work on different aspects in separate branches
- **Track Export:** Export individual tracks, version those separately
- **Project Variants:** Duplicate project for major variations

---

## Collaboration Questions

### How many people can work on a project simultaneously?

**One active editor at a time** (pessimistic locking).

**Why Only One:**
- Logic Pro projects cannot be automatically merged
- Multiple editors would cause conflicts and data loss
- One lock holder edits, others wait their turn

**Workaround for Larger Teams:**
- Use **feature branches** - each person works on their own branch
- Merge branches manually when done (FCP XML workflow)
- Allows parallel work on different aspects

**Example:**
- Engineer A: Works on drums (on `drums` branch)
- Engineer B: Works on vocals (on `vocals` branch)
- Engineer C: Works on mix (on `main` branch)
- Merge all three branches when ready

### What if someone forgets to release the lock?

**Lock Timeout:** 4 hours by default

After 4 hours:
- Lock automatically expires
- Others can acquire lock
- Original holder gets warning notification

**If you need it sooner:**
1. Contact the lock holder (email/chat shown in UI)
2. If no response, **force-break** the lock (admin only)
3. Confirm action (their uncommitted work may be lost)

**Best Practice:**
- Always release lock when done
- Set calendar reminder for long sessions
- Use Slack/Discord to coordinate

### Can we work on different sections at the same time?

**Sort of**, via **feature branches**:

1. **Person A:**
   ```bash
   oxen checkout -b feature/intro
   # Work on intro
   ```

2. **Person B:**
   ```bash
   oxen checkout -b feature/outro
   # Work on outro
   ```

3. **Merge Later:** Use FCP XML export/import to combine work

**Limitation:**
- Merging is manual (not automatic)
- Works best for distinct sections
- Plugin settings must be manually reconciled

**When It Works Well:**
- Different song sections (intro, verse, chorus, outro)
- Different instrument groups (drums, bass, vocals, strings)
- Different mix versions (radio edit, album version, instrumental)

**When It Doesn't Work:**
- Overlapping edits (both editing same chorus)
- Plugin changes (automation conflicts)
- Structural changes (arrangement differences)

### How do I share my project with a collaborator?

**Option 1: Oxen Hub (Recommended)**

1. **Create Account:** [https://oxen.ai/signup](https://oxen.ai/signup)
2. **Push Project:**
   ```bash
   cd MyProject.logicx
   oxen push origin main
   ```
3. **Share Access:**
   - Invite collaborator via Oxen Hub UI
   - They clone project:
     ```bash
     oxen clone https://hub.oxen.ai/yourname/myproject myproject.logicx
     ```

**Option 2: Self-Hosted**

1. Set up Oxen server on your network
2. Configure remote:
   ```bash
   oxen remote add origin ssh://server/path/to/repo
   ```
3. Push/pull as above

**Option 3: File Sharing (Not Recommended)**

- Zip entire project folder (including `.oxen/`)
- Transfer via Dropbox, WeTransfer, etc.
- Collaborator extracts and opens
- **Problem:** No automatic sync, conflicts possible

---

## Performance & Storage

### Why do commits take 10-30 seconds?

**Commit Time Depends On:**
- **Project Size:** 1GB = ~5s, 10GB = ~30s
- **Changed Files:** More changes = longer
- **Disk Speed:** SSD vs HDD (SSD is 10x faster)
- **CPU Load:** Other apps running

**What Happens During Commit:**
1. Scan changed files (~1-5s)
2. Compute block hashes (~2-10s)
3. Store deduplicated blocks (~2-10s)
4. Update references and metadata (~1s)

**Optimization Tips:**
- Use SSD (not HDD)
- Ensure Bounces/ and Freeze Files/ are ignored
- Close other heavy apps during commit
- Upgrade to faster CPU if consistently slow

**Expected Times:**
- Small project (<1GB): 3-8 seconds
- Medium project (1-10GB): 10-30 seconds
- Large project (10-50GB): 30-120 seconds

### How do I clean up old commits to free space?

**Automatic Pruning:**
- Draft commits: Automatically pruned (keep last 50)
- Milestone commits: Never pruned (permanent)

**Manual Pruning (Advanced):**
```bash
cd MyProject.logicx

# Remove commits older than 6 months
oxen gc --prune-older-than 6m

# Remove unreferenced objects
oxen gc --aggressive
```

**Remote Offloading:**
```bash
# Push old commits to remote
oxen push origin main

# Remove local copies (keep remote)
oxen gc --remote-only
```

**Warning:** Pruning deletes history permanently. Ensure you have backups or remote copies.

### Can I move my project to a different drive?

**Yes, but follow these steps:**

1. **Close Logic Pro and Oxen-VCS.app**

2. **Copy Entire Project Folder:**
   ```bash
   cp -R /Volumes/OldDrive/MyProject.logicx /Volumes/NewDrive/
   ```

3. **Verify Copy:**
   ```bash
   diff -r /Volumes/OldDrive/MyProject.logicx /Volumes/NewDrive/MyProject.logicx
   ```

4. **Remove Old Project from Oxen-VCS Monitoring:**
   - Open Oxen-VCS.app
   - Select old project
   - Click "Remove from Monitoring"

5. **Add New Location:**
   - Click "Add Project"
   - Navigate to `/Volumes/NewDrive/MyProject.logicx`
   - Select (no need to re-initialize)

6. **Test in Logic Pro:**
   - Open project from new location
   - Save, verify auto-commit works

**Important:** Don't just move the folder in Finder - Oxen-VCS won't find it.

---

## Troubleshooting

### Auto-commits stopped working. What do I do?

**Check Daemon Status:**
```bash
launchctl list | grep oxenvcs
```

If empty, daemon crashed. Restart:
```bash
launchctl load ~/Library/LaunchAgents/com.oxenvcs.agent.plist
```

**Check Logs:**
```bash
log show --predicate 'process == "OxVCS-LaunchAgent"' --last 30m
```

Look for errors like:
- "Permission denied"
- "Project not found"
- "Oxen command failed"

**Common Fixes:**
1. Grant Full Disk Access (System Preferences → Security & Privacy)
2. Reinstall oxen CLI: `pip3 install --upgrade oxen-ai`
3. Restart daemon (above)
4. Check disk space: `df -h`

### "oxen command not found" error

**Problem:** Oxen CLI is not installed or not in PATH.

**Solution 1: Install Oxen:**
```bash
pip3 install oxen-ai
```

**Solution 2: Verify PATH:**
```bash
which oxen
# Should output: /usr/local/bin/oxen or similar
```

If empty:
```bash
# Add to PATH (add to ~/.zshrc or ~/.bash_profile)
export PATH="/usr/local/bin:$PATH"
```

**Solution 3: Reinstall:**
```bash
pip3 uninstall oxen-ai
pip3 install oxen-ai
oxen --version  # Verify
```

### Project initialization failed

**Common Causes:**

1. **Not a folder-based .logicx:**
   - Right-click project in Finder
   - If "Show Package Contents" appears, it's packaged (not supported)
   - Convert: File → Save As → Folder (in Logic Pro)

2. **No ProjectData file:**
   - Project not yet saved in Logic Pro
   - Open, make change, save (Cmd+S)
   - Try initialization again

3. **Permissions error:**
   - Check disk permissions
   - Ensure not on read-only volume

4. **Oxen CLI issues:**
   - Verify oxen installed: `oxen --version`
   - Check oxen works: `oxen init /tmp/test`

**Detailed Error Info:**
```bash
# Run initialization manually to see full error
cd MyProject.logicx
oxen init
```

### For More Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for comprehensive solutions.

---

## Comparison to Other Tools

### Oxen-VCS vs. Git

**Git Is Better For:**
- Text-based code (merging, diffing)
- Widely-used standard
- Massive ecosystem (GitHub, GitLab, etc.)

**Oxen-VCS Is Better For:**
- Large binary files (audio, video)
- DAW projects (Logic Pro, etc.)
- Storage efficiency (block-level dedup)
- Preventing merge conflicts (pessimistic locking)

**When to Use Git:**
- Software development
- Text documents, markdown
- Small projects (<100MB)

**When to Use Oxen-VCS:**
- Logic Pro projects
- Audio/video production
- Large binary assets
- Collaboration on un-mergeable files

### Oxen-VCS vs. Splice

**Splice Is:**
- Cloud-based (requires internet)
- Subscription service
- Automated backups
- Multipl DAW support

**Oxen-VCS Is:**
- Local-first (offline capable)
- Open-source (free)
- Self-hosted option
- Currently Logic Pro only

**Splice Advantages:**
- Easier setup (no CLI required)
- Cloud storage included
- Mobile access
- Supports Ableton, FL Studio, Logic

**Oxen-VCS Advantages:**
- Free and open-source
- More efficient storage (block-level dedup)
- Full control over data
- Privacy (local-only option)
- No subscription fees

**Recommendation:** Use both if needed - Splice for cloud backup, Oxen-VCS for local versioning and collaboration.

### Oxen-VCS vs. Perforce Helix Core

**Perforce Is:**
- Enterprise version control
- Used in game development, film
- Supports massive projects (100GB+)
- Pessimistic locking (like Oxen-VCS)

**Oxen-VCS Is:**
- Designed for music production
- macOS-native
- Logic Pro-specific features
- Easier setup and use

**Perforce Advantages:**
- Proven at scale (AAA games, films)
- Advanced features (streams, edge servers)
- Enterprise support

**Oxen-VCS Advantages:**
- DAW-specific (Logic Pro understanding)
- Free and open-source
- Simpler workflow (less overhead)
- Native macOS UI

**When to Use Perforce:**
- Large studios (10+ users)
- Massive projects (100GB+)
- Need enterprise support

**When to Use Oxen-VCS:**
- Music production teams
- Small-medium projects
- Budget-conscious users
- Want Logic Pro integration

---

## Getting More Help

**Documentation:**
- [User Guide](USER_GUIDE.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Quick Start](QUICKSTART_GUIDE.md)

**Support:**
- GitHub Issues: [https://github.com/jbacus/oxen-vcs-logic/issues](https://github.com/jbacus/oxen-vcs-logic/issues)
- Discord Community: [Join Server](#) (Coming soon)
- Email: support@oxen-vcs.com

**Contributing:**
- Report bugs via GitHub Issues
- Feature requests welcome
- Code contributions: [CONTRIBUTING.md](../CONTRIBUTING.md)

---

**Last Updated:** 2025-10-29
**Questions not answered here?** [Open an issue](https://github.com/jbacus/oxen-vcs-logic/issues) or check [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
