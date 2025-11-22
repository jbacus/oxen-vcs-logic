# Auxin Business Model & Sustainability Strategy

**Last Updated**: 2025-11-21
**Status**: Open Source Project - Exploring Sustainability Options

---

## Current Model: Open Source (MIT License)

### What This Means
- ✅ **Free Forever**: Core software is free to use, modify, and distribute
- ✅ **No Vendor Lock-In**: Users own their data, can self-host, can fork
- ✅ **Community-Driven**: Open development, transparent roadmap, public issues
- ✅ **MIT License**: Permissive (can be used commercially, modified, etc.)

### Sustainability Challenge
**Question**: How do we fund ongoing development, maintenance, and support?

**Current Reality**:
- Development is volunteer/passion project
- No revenue to support full-time development
- No funding for infrastructure (servers, domains, CI/CD)
- No budget for marketing or user acquisition

---

## Strategic Decision: Hybrid Open-Core Model

### Recommended Approach: "Open Core + Paid Hosting"

**Open Source (Forever Free)**:
- ✅ Auxin CLI (Rust wrapper)
- ✅ Auxin LaunchAgent (background daemon)
- ✅ Auxin.app (macOS GUI)
- ✅ Auxin Server (self-hosted collaboration)
- ✅ All documentation and guides

**Optional Paid Services** (Future):
- 💰 **Auxin Cloud** - Managed Oxen Hub hosting (like GitHub for creative files)
- 💰 **Priority Support** - SLA-backed support contracts
- 💰 **Enterprise Features** - SSO, LDAP, audit logs (plugin system)
- 💰 **Training & Consulting** - Onboarding for studios

### Why This Model?

**Benefits**:
1. ✅ Stays true to open-source mission (core is free)
2. ✅ Provides monetization path for sustainability
3. ✅ Aligns incentives (better product = more cloud customers)
4. ✅ Proven model (GitLab, Sentry, Ghost, many others)

**Risks**:
1. ⚠️ Community may perceive as "bait and switch"
2. ⚠️ Requires careful messaging (what's free vs paid)
3. ⚠️ Temptation to move features from free to paid

**Mitigation**:
- Document "Core Promise": CLI, daemon, GUI, self-hosted server always free
- No features move from free to paid (only add new paid services)
- Transparent about what sustains the project

---

## Business Model Options Analysis

### Option 1: Pure Open Source (Current)

**How It Works**: Everything free, rely on donations/volunteers

**Pros**:
- ✅ Maximum community trust
- ✅ No ethical conflicts
- ✅ Fastest adoption

**Cons**:
- ❌ Not sustainable long-term
- ❌ Relies on unpaid labor
- ❌ Can't provide support or guarantees
- ❌ Project dies when maintainers burn out

**Examples**: Many abandoned OSS projects

**Verdict**: ❌ **Not Sustainable** - Works for hobby projects, not professional tools

---

### Option 2: Freemium SaaS (Splice Model)

**How It Works**: Core app free, cloud features require subscription

**Example Pricing**:
- Free: Local-only, manual sync
- Pro ($9.99/mo): Cloud backup, 100GB storage
- Studio ($19.99/mo): Unlimited storage, team features

**Pros**:
- ✅ Predictable recurring revenue
- ✅ Proven model (Splice does this)
- ✅ Can fund full-time development

**Cons**:
- ❌ Requires closing source (or dual-licensing)
- ❌ Vendor lock-in (users depend on cloud)
- ❌ Competes directly with Splice (they have head start)
- ❌ Betrays "local-first" philosophy

**Verdict**: ❌ **Not Aligned** - Conflicts with Auxin's core values

---

### Option 3: Open Core + Paid Hosting (Recommended)

**How It Works**: Self-hosted free, managed hosting paid

**Example Pricing**:
- Free: Self-host Auxin Server, unlimited projects
- Cloud Starter ($9/mo): 50GB storage, 5 projects, 2 users
- Cloud Team ($29/mo): 500GB storage, unlimited projects, 10 users
- Cloud Studio ($99/mo): 2TB storage, unlimited users, priority support
- Enterprise: Custom pricing, dedicated infrastructure, SLA

**Pros**:
- ✅ Core stays open source (community trust)
- ✅ Monetizes convenience, not features
- ✅ Users can always self-host (no lock-in)
- ✅ Proven model (GitLab, Sentry, Bitwarden)
- ✅ Aligns with values (local-first, data ownership)

**Cons**:
- ⚠️ Must compete with self-hosting (keep price compelling)
- ⚠️ Infrastructure costs (hosting, bandwidth)
- ⚠️ Support burden for cloud customers

**Verdict**: ✅ **Recommended** - Best balance of sustainability and values

---

### Option 4: Enterprise Licensing (Perforce Model)

**How It Works**: Open source for individuals, paid license for companies

**Example Pricing**:
- Free: Individual creators, open source projects
- Pro ($99/user/year): Commercial use in companies <50 employees
- Enterprise ($499/user/year): Companies 50+, support, training, SLA

**Pros**:
- ✅ Individuals always free
- ✅ Companies expect to pay
- ✅ High revenue per customer

**Cons**:
- ❌ Requires license enforcement (trust or technical)
- ❌ MIT license makes this hard to enforce
- ❌ Alienates commercial solo creators
- ❌ Complex to implement legally

**Verdict**: ⚠️ **Possible but Complex** - Consider for v2.0+ if needed

---

### Option 5: Sponsorship & Donations (GitHub Sponsors Model)

**How It Works**: Everything free, funded by generous users/companies

**Example Tiers**:
- $5/mo: Supporter badge
- $25/mo: Name in README
- $100/mo: Logo on website
- $500/mo: Priority bug fixes
- $2,000/mo: Corporate sponsor (logo prominent)

**Pros**:
- ✅ No strings attached (stays pure open source)
- ✅ Community feels good supporting
- ✅ No complex infrastructure needed

**Cons**:
- ❌ Unpredictable revenue
- ❌ Rarely covers full-time development
- ❌ Doesn't scale with usage
- ❌ Relies on goodwill, not value exchange

**Verdict**: ⚠️ **Supplemental Only** - Good addition to other models, not standalone

---

### Option 6: Paid Support & Training (Red Hat Model)

**How It Works**: Software free, charge for support/consulting/training

**Example Services**:
- Studio Onboarding: $2,000 (one-time)
- Support Contract: $500/mo (SLA-backed email support)
- Custom Development: $150/hr
- Training Workshop: $5,000 (full-day, up to 10 people)

**Pros**:
- ✅ Software stays free
- ✅ Targets enterprises with budget
- ✅ Builds relationships with key customers

**Cons**:
- ❌ Doesn't scale (time-for-money)
- ❌ Requires expertise in consulting/training
- ❌ Unpredictable workload

**Verdict**: ⚠️ **Supplemental** - Good for enterprise customers, not primary model

---

## Recommended Business Model: Hybrid Approach

### Phase 1 (v0.3 - v1.0): Pure Open Source + Donations
**Timeline**: Next 3-6 months
**Focus**: Grow user base, prove product-market fit

**Revenue Streams**:
- GitHub Sponsors (launch now)
- Open Collective (launch at v0.3)
- Corporate sponsorships (approach after 1,000+ users)

**Expected Revenue**: $0-500/month (covers hosting, domain)

**Goal**: Sustainability is NOT the priority - growth is

---

### Phase 2 (v1.0 - v2.0): Add Managed Hosting
**Timeline**: 6-12 months after v1.0
**Focus**: Offer "Auxin Cloud" as convenience option

**Revenue Streams**:
- Auxin Cloud subscriptions (primary)
- GitHub Sponsors (continues)
- Support contracts (enterprises)

**Infrastructure**:
- Use Oxen Hub API for storage backend (leverage existing infrastructure)
- Simple web dashboard (React + Auxin Server)
- Stripe for payments

**Pricing** (Draft):
```
Free Tier: Self-hosted (unlimited)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Starter:    $9/mo  - 50GB,  5 projects, 2 users
Team:      $29/mo  - 500GB, 20 projects, 10 users
Studio:    $99/mo  - 2TB,   unlimited,  25 users
Enterprise: Custom - Custom storage, SSO, SLA
```

**Expected Revenue**: $1,000-5,000/month (covers 1 part-time developer)

---

### Phase 3 (v2.0+): Enterprise Features
**Timeline**: 12+ months
**Focus**: Serve professional studios at scale

**Revenue Streams**:
- Auxin Cloud subscriptions (primary)
- Enterprise add-ons (LDAP, SSO, audit logs)
- Support contracts (SLA-backed)
- Training & consulting

**Expected Revenue**: $10,000+/month (covers 1-2 full-time developers)

---

## Financial Projections (Conservative)

### Year 1 (v0.3 → v1.0)
- **Users**: 500-1,000 active users
- **Sponsors**: 10-20 @ $5-100/mo
- **Revenue**: $100-500/month
- **Costs**: $50-100/month (hosting, domain)
- **Net**: Break-even or small surplus

### Year 2 (v1.0 → v2.0, Auxin Cloud Launch)
- **Users**: 2,000-5,000 active users
- **Cloud Customers**: 50-100 (5% conversion)
- **Avg Revenue/Customer**: $15/mo
- **Monthly Revenue**: $750-1,500
- **Annual Revenue**: $9,000-18,000
- **Costs**: $2,000-3,000/year (hosting, infrastructure)
- **Net**: $6,000-15,000/year (covers part-time development)

### Year 3 (v2.0+, Enterprise)
- **Users**: 5,000-10,000 active users
- **Cloud Customers**: 200-300
- **Enterprise Customers**: 5-10 @ $500-2,000/mo
- **Monthly Revenue**: $3,000-10,000
- **Annual Revenue**: $36,000-120,000
- **Costs**: $10,000-20,000/year
- **Net**: $26,000-100,000/year (covers 1-2 developers)

---

## Pricing Strategy

### Guiding Principles
1. **Free tier is generous** - Self-hosting unlimited forever
2. **Cloud pricing competes with Splice** - Underprice slightly ($9 vs their $7.99, but more features)
3. **Value-based pricing** - Charge for convenience, not features
4. **No artificial limits** - Don't cripple free tier (it's truly full-featured)

### Competitive Pricing Analysis

| Provider | Price/Month | Storage | Users | Notes |
|----------|-------------|---------|-------|-------|
| **Splice** | $7.99-19.99 | 2GB-Unlimited | 1 | Industry standard |
| **Dropbox** | $11.99 | 2TB | 1 | Not VCS, but used for backup |
| **GitHub** | $4-21/user | 500MB-50GB (LFS) | Unlimited | Not for binaries |
| **Auxin Cloud** (proposed) | $9-99 | 50GB-2TB | 2-25 | Better than Splice, cheaper |

### Proposed Tiers (Detailed)

#### Free (Self-Hosted)
- **Cost**: $0
- **Storage**: Unlimited (your own server)
- **Projects**: Unlimited
- **Users**: Unlimited
- **Features**: All core features (CLI, daemon, GUI, server)
- **Support**: Community (GitHub Issues)
- **Target**: Solo creators, hobbyists, small teams with technical skills

#### Starter ($9/month)
- **Cost**: $9/month or $90/year (save $18)
- **Storage**: 50GB
- **Projects**: 5 active projects
- **Users**: 2
- **Features**: All core + managed hosting
- **Support**: Email (48hr response)
- **Extras**: Automatic updates, web dashboard, mobile view (future)
- **Target**: Solo creator who wants "set and forget" cloud backup

#### Team ($29/month)
- **Cost**: $29/month or $290/year (save $58)
- **Storage**: 500GB
- **Projects**: 20 active projects
- **Users**: 10
- **Features**: All Starter + team activity feed, Slack/Discord webhooks
- **Support**: Email (24hr response)
- **Extras**: Advanced search, archive old projects
- **Target**: Small production teams (3-5 active users)

#### Studio ($99/month)
- **Cost**: $99/month or $990/year (save $198)
- **Storage**: 2TB
- **Projects**: Unlimited
- **Users**: 25
- **Features**: All Team + priority support, dedicated infrastructure
- **Support**: Email + chat (8hr response), phone (critical issues)
- **Extras**: Custom domain, white-label option
- **Target**: Professional studios, agencies

#### Enterprise (Custom)
- **Cost**: Custom pricing (starts at $500/month)
- **Storage**: Custom (5TB+)
- **Projects**: Unlimited
- **Users**: Unlimited
- **Features**: All Studio + SSO, LDAP, audit logs, dedicated account manager
- **Support**: 24/7 phone + email, SLA (4hr critical, 1hr emergency)
- **Extras**: On-premise option, custom development, training
- **Target**: Large studios, educational institutions, corporations

---

## Revenue Diversification

### Primary Revenue (80%)
- **Auxin Cloud subscriptions** - Managed hosting

### Secondary Revenue (15%)
- **Support contracts** - Enterprise SLA-backed support
- **Training & consulting** - Onboarding, custom workflows

### Tertiary Revenue (5%)
- **GitHub Sponsors** - Community donations
- **Affiliate partnerships** - Oxen Hub, audio software (ethical only)

---

## Cost Structure

### Fixed Costs (Monthly)
- **Infrastructure**: $200-1,000 (AWS/DigitalOcean, scales with users)
- **Domain & SSL**: $10
- **Email service** (transactional): $20
- **Payment processing** (Stripe): 2.9% + $0.30 per transaction
- **Legal & accounting**: $100-500 (annual, amortized)

### Variable Costs (Per Customer)
- **Storage**: $0.02-0.05/GB/month (S3/B2)
- **Bandwidth**: $0.01-0.05/GB transfer
- **Support**: $10-50/customer/month (time allocation)

### Example Cost Analysis (100 Cloud Customers)
- 50 Starter @ $9 = $450/mo
- 40 Team @ $29 = $1,160/mo
- 10 Studio @ $99 = $990/mo
- **Total Revenue**: $2,600/mo

**Costs**:
- Infrastructure: $300
- Storage (avg 200GB/customer): $400
- Bandwidth (10GB/customer/mo): $50
- Support (2hr/week @ $50/hr): $400
- Payment processing (2.9%): $75
- **Total Costs**: $1,225/mo

**Net Profit**: $1,375/mo (53% margin)

---

## Growth Strategy

### User Acquisition (Free Tier)
1. **Content Marketing**: Blog posts, tutorials, YouTube videos
2. **Community Building**: Discord/Slack, forums, Reddit
3. **SEO**: "Logic Pro version control", "SketchUp git alternative"
4. **Partnerships**: Music schools, architecture programs
5. **Open Source Visibility**: Hacker News, Product Hunt launch

### Conversion (Free → Paid)
1. **In-App Prompts**: "Backup to cloud with one click" (non-intrusive)
2. **Email Drip Campaign**: Tips for 7 days, then cloud offer
3. **Trial Period**: 14-day free trial of Team tier
4. **Referral Program**: Give 1 month free for each referral
5. **Annual Discount**: 2 months free if paying annually

### Retention
1. **Reliability**: 99.9% uptime SLA (Studio+)
2. **Support Quality**: Fast, helpful, friendly responses
3. **Feature Velocity**: Regular updates, user-requested features
4. **Community**: Make users feel heard and valued
5. **Transparency**: Public roadmap, honest communication

---

## Open Source Sustainability Principles

### Core Commitments (Non-Negotiable)
1. ✅ **CLI, daemon, GUI always free** - No paid tiers for core software
2. ✅ **Self-hosted server always free** - No artificial feature limits
3. ✅ **No moving features from free to paid** - Only add new paid services
4. ✅ **Source code always open** - MIT license forever
5. ✅ **Data export always free** - No lock-in, ever

### What Can Be Paid
1. ✅ **Managed hosting** - Convenience, not features
2. ✅ **Enterprise plugins** - SSO, LDAP, audit logs (optional)
3. ✅ **Support contracts** - SLA-backed response times
4. ✅ **Training & consulting** - Professional services
5. ✅ **Priority feature requests** - Influence roadmap (transparently)

### Transparency Commitments
1. **Public roadmap** - Free vs paid features clearly marked
2. **Open financials** - Annual revenue/cost reports (optional)
3. **User advisory board** - Community input on pricing/features
4. **No surprise price changes** - 6-month notice, grandfather existing customers

---

## Risk Assessment

### Risk 1: Insufficient Revenue
**Scenario**: Cloud adoption <5%, can't cover costs
**Likelihood**: Medium
**Impact**: High
**Mitigation**:
- Keep costs low (leverage Oxen Hub, don't build storage from scratch)
- Diversify revenue (sponsors, support, training)
- Grow free user base first (10,000+ before pushing paid)

### Risk 2: Community Backlash
**Scenario**: "Auxin sold out, moving to competitor"
**Likelihood**: Low (if communicated well)
**Impact**: High
**Mitigation**:
- Communicate early and often about sustainability
- Show financials (we need X to keep project alive)
- Emphasize core stays free (not a bait and switch)
- Involve community in pricing decisions

### Risk 3: Enterprise Doesn't Materialize
**Scenario**: Studios won't pay for open source tool
**Likelihood**: Medium
**Impact**: Medium
**Mitigation**:
- Don't rely on enterprise for v1 sustainability
- Focus on individual/team tiers first
- Provide clear ROI (time saved, no subscription vs Splice)

### Risk 4: Competitor Undercuts Pricing
**Scenario**: New competitor launches at $5/mo
**Likelihood**: Low
**Impact**: Medium
**Mitigation**:
- Differentiate on features, not just price
- Self-hosted option always available (competitor lock-in)
- Build loyal community (they won't switch for $4/mo)

---

## Decision Framework: When to Charge

### Charge for:
- ✅ **Convenience** - Managed hosting vs self-hosting
- ✅ **Support** - SLA-backed response times
- ✅ **Scale** - Enterprise features (SSO, LDAP)
- ✅ **Services** - Training, consulting, custom development

### Never Charge for:
- ❌ **Core VCS features** - init, commit, log, restore, etc.
- ❌ **Application support** - Logic, SketchUp, Blender, etc.
- ❌ **Collaboration basics** - Locks, activity, team discovery
- ❌ **Data access** - Export, backup, migration

---

## Recommended Next Steps

### Immediate (v0.3)
1. ✅ Add GitHub Sponsors button to README
2. ✅ Add BUSINESS_MODEL.md (this document) to repo
3. ✅ Create Open Collective page (alternative to GitHub Sponsors)
4. ✅ Write blog post: "How We'll Keep Auxin Sustainable"

### Short-Term (v1.0)
1. ⏳ Validate cloud demand (survey: "Would you pay $9/mo for managed hosting?")
2. ⏳ Design Auxin Cloud MVP (simple web dashboard)
3. ⏳ Partner with Oxen Hub for storage backend
4. ⏳ Set up Stripe account and pricing pages

### Medium-Term (v1.0 → v2.0)
1. ⏳ Launch Auxin Cloud beta (free for 3 months)
2. ⏳ Gather feedback, iterate pricing
3. ⏳ Announce general availability
4. ⏳ Measure conversion rate (target: 5-10%)

### Long-Term (v2.0+)
1. ⏳ Add enterprise features (SSO, LDAP)
2. ⏳ Hire first full-time maintainer (if revenue supports)
3. ⏳ Explore partnership with DAW vendors (bundling)
4. ⏳ Consider establishing legal entity (LLC, foundation)

---

## Summary: The Auxin Business Model

**Philosophy**: Sustainable open source through optional convenience services

**Core Promise**:
- Free forever: CLI, daemon, GUI, self-hosted server
- Open source forever: MIT license, community-driven
- No lock-in ever: Data export, self-hosting always available

**Revenue Model**:
- Primary: Managed cloud hosting (Auxin Cloud)
- Secondary: Enterprise support contracts
- Tertiary: Training, consulting, sponsorships

**Timeline**:
- Phase 1 (now): Pure open source + donations
- Phase 2 (v1.0+): Add managed hosting
- Phase 3 (v2.0+): Enterprise features

**Success Metrics**:
- Year 1: Break-even on hosting costs
- Year 2: $10,000+/year (part-time development)
- Year 3: $50,000+/year (full-time development)

**Guiding Principle**: Make money by making Auxin so good that users *want* to support it, not because they're forced to.

---

*This document should be reviewed annually and adjusted based on actual user adoption, revenue, and community feedback.*
