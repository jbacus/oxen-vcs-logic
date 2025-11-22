# Getting Started with Auxin

This guide walks you through setting up Auxin from scratch and using it with your first Logic Pro project.

**Time Required**: 10-15 minutes

---

## Step 1: Check Your Environment

Before using Auxin, run the doctor command to verify everything is set up correctly:

```bash
auxin doctor
```

This will check:
- Oxen CLI installation
- Repository status
- Remote configuration
- Authentication status

### If Oxen CLI is Not Found

Install the Oxen CLI (required for Auxin to work):

```bash
# Option 1: Using pip (recommended)
pip3 install oxen-ai

# Option 2: Using Homebrew
brew install oxen-ai/tap/oxen

# Option 3: From source
cargo install oxen
```

Verify installation:

```bash
oxen --version
```

Then run `auxin doctor` again to confirm.

---

## Step 2: Initialize Your Project

Navigate to your Logic Pro project folder and initialize it as an Auxin repository:

```bash
cd "/path/to/My Song.logicx"
auxin init .
```

This creates:
- An `.oxen` directory for version history
- A `.oxenignore` file configured for Logic Pro projects (excludes bounces, freeze files, autosave)

### Verify Initialization

```bash
auxin status
```

You should see your project files listed as untracked.

---

## Step 3: Make Your First Commit

Add all your project files and create your initial commit:

```bash
# Stage all files
auxin add --all

# Create your first commit with project metadata
auxin commit -m "Initial commit" \
  --bpm 120 \
  --sample-rate 48000 \
  --key "C Major"
```

### Commit Metadata Options

- `--bpm <number>`: Project tempo (e.g., 120)
- `--sample-rate <number>`: Sample rate in Hz (e.g., 48000)
- `--key <string>`: Musical key (e.g., "C Major", "G Minor")
- `--tags <comma,separated>`: Optional tags for organization

---

## Step 4: Set Up Remote Collaboration (Optional)

To share your project with collaborators or sync between machines, configure a remote repository.

### Option A: Using Oxen Hub (Recommended)

1. **Create account**: Sign up at [oxen.ai](https://oxen.ai)

2. **Authenticate**:
   ```bash
   auxin auth login
   ```

3. **Create remote repository** on Oxen Hub web interface

4. **Add remote**:
   ```bash
   auxin remote add origin https://hub.oxen.ai/username/my-project
   ```

5. **Push your commits**:
   ```bash
   auxin push
   ```

### Option B: Self-Hosted Auxin Server

1. **Start the server** (on your server machine):
   ```bash
   cd auxin-server
   ./run-local.sh
   ```

2. **Add remote** (on your client machine):
   ```bash
   auxin remote add origin http://your-server:3000/username/my-project
   ```

3. **Push**:
   ```bash
   auxin push
   ```

### Verify Remote Configuration

```bash
auxin remote list
```

---

## Step 5: Daily Workflow

### When Starting a Session

1. **Pull latest changes** (if collaborating):
   ```bash
   auxin pull
   ```

2. **Check status**:
   ```bash
   auxin status
   ```

3. **Open your project in Logic Pro and work normally**

### When Finishing a Session

1. **Stage your changes**:
   ```bash
   auxin add --all
   ```

2. **Commit with a descriptive message**:
   ```bash
   auxin commit -m "Added bass track and adjusted EQ on drums" \
     --bpm 128 \
     --tags "mixing,bass"
   ```

3. **Push to remote** (if collaborating):
   ```bash
   auxin push
   ```

### Useful Commands During Work

```bash
# See what's changed
auxin status

# View commit history
auxin log

# Compare changes in metadata
auxin diff

# See project info
auxin info
```

---

## Step 6: Working with Bounce Files

Auxin has built-in support for managing audio bounces (mixdowns, stems, etc.).

### Add a Bounce

```bash
auxin bounce add "Mix_v3_Final.wav" \
  --commit abc1234 \
  --notes "Final client-approved mix"
```

### List All Bounces

```bash
auxin bounce list
```

### Play a Bounce

```bash
auxin bounce play abc1234
```

### Compare Two Bounces

```bash
auxin bounce compare abc1234 def5678
```

---

## Step 7: Restoring Previous Versions

One of Auxin's most powerful features is the ability to restore any previous version.

### View History

```bash
auxin log
```

### Restore a Specific Commit

```bash
# Restore entire project to a previous state
auxin restore abc1234

# Restore a specific file
auxin restore abc1234 --path "Audio Files/drums.wav"
```

### Create a Branch for Experimentation

```bash
# Create and switch to a new branch
auxin branch create experiment-remix
auxin branch checkout experiment-remix

# Do your work...

# Switch back to main
auxin branch checkout main
```

---

## Troubleshooting

### "Command not found: auxin"

The Auxin CLI isn't in your PATH. Either:
- Run the install script: `./install.sh`
- Or add it manually: `export PATH="$PATH:/path/to/Auxin-CLI-Wrapper/target/release"`

### "Command not found: oxen"

The Oxen CLI isn't installed. See [Step 1](#step-1-check-your-environment) for installation options.

### "Not in an Oxen repository"

You need to initialize the repository first:
```bash
auxin init .
```

### "Failed to push: remote not configured"

Add a remote repository:
```bash
auxin remote add origin <URL>
```

### Large Initial Commit Takes Too Long

Logic Pro projects can be large. For the initial commit:
1. Consider excluding sample libraries from your project folder
2. Use `.oxenignore` to exclude unnecessary files
3. Oxen's block-level deduplication will make subsequent commits faster

### Authentication Errors

Re-authenticate with:
```bash
auxin auth login
```

---

## Next Steps

- **[For Musicians](user/for-musicians.md)**: Understanding version control concepts
- **[CLI Reference](user/cli-reference.md)**: Complete CLI reference with examples
- **[Cloud Sharing Guide](user/cloud-sharing.md)**: Advanced collaboration workflows
- **[Troubleshooting](user/troubleshooting.md)**: Common issues and solutions

---

## Quick Reference Card

```bash
# Check environment
auxin doctor

# Initialize repository
auxin init .

# Daily workflow
auxin pull                    # Get latest
auxin add --all               # Stage changes
auxin commit -m "message"     # Save snapshot
auxin push                    # Share changes

# History & navigation
auxin log                     # View history
auxin status                  # See changes
auxin restore <commit>        # Go back in time

# Remotes
auxin remote add origin <url>
auxin remote list
auxin remote remove <name>

# Bounces
auxin bounce add <file>
auxin bounce list
auxin bounce play <commit>

# Help
auxin --help
auxin <command> --help
```

---

*Last Updated: November 2025*
