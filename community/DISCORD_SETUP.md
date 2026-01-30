# Vais Language Discord Server Setup Guide

Welcome! This guide provides comprehensive instructions for setting up and maintaining the official Vais Language Discord server. Vais is an AI-optimized systems programming language designed to minimize token usage while maximizing code expressiveness for LLM-assisted development.

## Table of Contents

1. [Server Basic Configuration](#server-basic-configuration)
2. [Channel Structure](#channel-structure)
3. [Role Structure](#role-structure)
4. [Bot Configuration](#bot-configuration)
5. [Welcome Message Templates](#welcome-message-templates)
6. [Server Rules & Code of Conduct](#server-rules--code-of-conduct)
7. [Moderation Guidelines](#moderation-guidelines)
8. [Best Practices](#best-practices)

---

## Server Basic Configuration

### Server Details

**Server Name:** `Vais Language`

**Description:**
```
AI-optimized systems programming language. Join us to discuss Vais, share projects,
and collaborate on compiler development. Minimal token syntax, maximum expressiveness.
```

### Server Icon

The server icon should feature:
- **Option 1:** The letter "V" in a modern, clean sans-serif font (e.g., Helvetica Neue Bold)
- **Option 2:** A stylized lambda (λ) or gear icon representing both programming and optimization
- **Option 3:** A combination of "V" with circuit patterns suggesting AI/optimization

**Recommended colors:**
- Primary: #3498DB (Professional Blue) or #2C3E50 (Dark Blue-Gray)
- Accent: #27AE60 (Green) or #E74C3C (Red)

**Icon specifications:**
- Format: PNG or JPG, 512x512 pixels minimum
- Transparent background recommended
- Ensure clarity at 48x48 pixels (Discord server list size)

### Server Settings

1. **Explicit Content Filter:** `Medium` (scan media)
2. **Default Notification Level:** `@mentions only`
3. **Verification Level:** `Low` (to prevent spam)
4. **Community Features:** Enable to promote server discovery
5. **Audit Log Retention:** Keep maximum (90 days)
6. **Language:** English (or set to match primary audience)

---

## Channel Structure

### 📌 INFORMATION (Category)

Essential server information and guidelines.

#### #welcome (Text Channel)
- **Permissions:** Everyone can view and read, no one can send messages (read-only)
- **Purpose:** First impression for new members
- **Content:** Welcome message with server overview and links to key channels

#### #announcements (Text Channel)
- **Permissions:** Only admins/moderators can post, everyone can read
- **Purpose:** Important updates, releases, and major announcements
- **Webhook integration:** GitHub releases (optional)

#### #rules (Text Channel)
- **Permissions:** Read-only for all members
- **Purpose:** Server Code of Conduct and community guidelines
- **Content:** Clear, numbered rules with explanations

#### #roles (Text Channel)
- **Permissions:** Everyone can view and use reactions
- **Purpose:** Role selection using emoji reactions
- **Integration:** Use MEE6 or Carl-bot for automatic role assignment

---

### 💬 COMMUNITY (Category)

General community engagement and member interaction.

#### #general (Text Channel)
- **Purpose:** Off-topic conversation related to programming or technology
- **Default channel:** Where new members are directed
- **Throttle:** Reasonable activity expected

#### #introductions (Text Channel)
- **Purpose:** New members introduce themselves
- **Suggested template:** Name, experience level with Vais, primary interests
- **Moderation:** Friendly and welcoming environment

#### #off-topic (Text Channel)
- **Purpose:** Non-programming discussions, memes, random chat
- **Rules:** Keep it respectful and relevant to the community

---

### 🔧 DEVELOPMENT (Category)

Technical discussion and project development.

#### #help (Text Channel)
- **Purpose:** Questions and troubleshooting
- **Best practices:**
  - Use threads for conversations
  - Include error messages and minimal reproducible examples
  - Mark solved issues with appropriate reactions
- **Pin important solutions**

#### #showcase (Text Channel)
- **Purpose:** Share completed projects, demos, and creative works
- **Encouraged content:**
  - Finished programs and applications
  - Benchmarks and performance comparisons
  - Interesting use cases
  - Tutorial content

#### #bugs (Text Channel)
- **Purpose:** Bug reports and issue tracking
- **Required format:**
  ```
  - Vais version: (output of `vaisc --version`)
  - OS: (macOS/Linux/Windows)
  - Steps to reproduce:
  - Expected behavior:
  - Actual behavior:
  - Minimal example: (code block)
  ```
- **Linked to GitHub Issues:** Reference issue numbers (#123)

#### #feature-requests (Text Channel)
- **Purpose:** Discuss and propose new language features
- **Process:**
  1. Post initial idea
  2. Community discusses pros/cons
  3. Advanced ideas may be escalated to formal RFC process

---

### 📚 LEARNING (Category)

Educational resources and knowledge sharing.

#### #tutorials (Text Channel)
- **Purpose:** Links and discussions about tutorials, blog posts, guides
- **Encouraged content:**
  - Official documentation announcements
  - Community tutorials
  - Getting started guides
  - Language feature deep-dives

#### #code-review (Text Channel)
- **Purpose:** Request and provide code feedback
- **Best practices:**
  - Post code in threads
  - Use code blocks with syntax highlighting (```vais)
  - Be constructive and specific in feedback
  - Ask questions to help others learn

#### #tips-and-tricks (Text Channel)
- **Purpose:** Share optimization techniques and clever solutions
- **Content examples:**
  - Performance optimization tips
  - Idiomatic Vais patterns
  - Tool usage and productivity hacks
  - Common pitfalls and solutions

---

### 🛠️ CONTRIBUTORS (Category)

*Channel category for active project contributors. May be role-restricted.*

#### #compiler-dev (Text Channel)
- **Purpose:** Compiler implementation, optimization, and architecture discussions
- **Audience:** Contributors working on `vais-lexer`, `vais-parser`, `vais-types`, `vais-codegen`
- **Discussion focus:**
  - Algorithm design
  - Performance optimization
  - Bug fixes and refactoring
  - Code review coordination

#### #stdlib-dev (Text Channel)
- **Purpose:** Standard library development and design
- **Audience:** Contributors working on standard library modules (Vec, HashMap, String, File, Net, etc.)
- **Discussion focus:**
  - API design
  - Performance considerations
  - Module organization
  - Documentation improvements

#### #tooling-dev (Text Channel)
- **Purpose:** IDE/editor integration, LSP, REPL, and tooling
- **Audience:** Contributors working on VSCode extension, LSP, REPL, formatter, debugger
- **Discussion focus:**
  - Feature implementations
  - User experience improvements
  - Integration with external tools
  - Testing and bug reports

---

## Role Structure

### Role Hierarchy (from top to bottom)

```
1. @everyone (default for all members)
2. Community Member (auto-assigned after verification)
3. Contributor (manual assignment)
4. Moderator (manual assignment)
5. Admin (careful assignment)
```

### Role Definitions

#### Everyone (@everyone)
- **Permissions:**
  - View channels in #welcome category
  - Send messages in #general, #introductions, #off-topic
  - Read all public channels
  - React to messages and create emoji
  - Default permissions sufficient for browsing

#### Community Member
- **Auto-assigned:** After member spends 5+ minutes or admin manual assignment
- **Permissions:**
  - Post in all public channels
  - Create threads
  - Upload media (images, code files)
  - Add reactions
  - Use voice channels (if any)

#### Contributor
- **Manual assignment:** Requires demonstrated contributions or application
- **Permissions:**
  - All Community Member permissions
  - Access to #compiler-dev, #stdlib-dev, #tooling-dev
  - Create persistent threads
  - Pin messages (with moderation)
  - Edit member information in profiles

#### Moderator
- **Manual assignment:** Trusted community members or staff
- **Responsibilities:**
  - Monitor all channels for violations
  - Warn or mute disruptive members
  - Remove inappropriate content
  - Manage roles and respond to appeals
  - Coordinate with other moderators
- **Permissions:**
  - All previous permissions
  - Kick members
  - Timeout/mute members
  - Manage channels
  - Manage messages
  - View audit log

#### Admin
- **Manual assignment:** Project maintainers only
- **Responsibilities:**
  - Full server management
  - Strategic decisions about community
  - Onboarding moderators
  - Managing bot configurations
  - Crisis management
- **Permissions:**
  - Full administrative access
  - Create and delete channels
  - Create and manage roles
  - Manage webhooks
  - View audit log
  - Ban/unban members

### Role Color Assignments

- **Community Member:** #95A5A6 (Light Gray)
- **Contributor:** #3498DB (Blue)
- **Moderator:** #E67E22 (Orange)
- **Admin:** #E74C3C (Red)

---

## Bot Configuration

### Recommended Bots

#### 1. **MEE6** (Primary - Free tier)

**Purpose:** Moderation, welcome messages, role assignment, and engagement

**Setup Steps:**
1. Visit [mee6.xyz](https://mee6.xyz)
2. Click "Invite" and authorize with permissions
3. In Discord settings → Integrations, configure:

**Key Features to Configure:**

- **Welcome Messages**
  - Enable custom welcome DM to new members
  - Message: See [Welcome Message Templates](#welcome-message-templates)
  - Channel: #welcome

- **Moderation**
  - Anti-spam protection (enabled)
  - Enable auto-moderation for bad words
  - Set slowmode for #general (1 message per 5 seconds during peak)

- **Role Assignment**
  - Create reaction role for #roles channel
  - Emoji assignments:
    - 📚 → Learning Resources
    - 💻 → Development
    - 🎮 → Games & Creative
    - 🔔 → Announcements

- **Leveling** (Optional)
  - Enable XP for messages and reactions
  - Disable XP farming checks
  - Set level-up message to thread only

#### 2. **Reminder Bot** (Optional - Improve engagement)

**Purpose:** Schedule reminders for recurring community events

**Setup:**
```
!remind "Weekly Q&A Session" Thursday 10:00 AM recurring
!remind "Code Review Session" Wednesday 3:00 PM recurring
```

#### 3. **GitHub Integration** (Optional)

**Purpose:** Notify channel when repo issues/PRs are created

**Setup:**
1. Repository settings → Webhooks
2. Create webhook for Discord
3. URL format: `https://discordapp.com/api/webhooks/<WEBHOOK_ID>/<WEBHOOK_TOKEN>`
4. Trigger on: Issues, Pull Requests
5. Post to #announcements channel

---

## Welcome Message Templates

### Welcome DM (MEE6 Custom Welcome)

```
🎉 Welcome to Vais Language Discord!

Hello {username}! We're excited to have you join our community dedicated to
Vais, an AI-optimized systems programming language.

Vais is designed to minimize token usage while maximizing code expressiveness,
making it ideal for AI-assisted development and LLM code generation.

📚 Quick Start:
1. Read #rules for community guidelines
2. Check #welcome for server overview
3. Introduce yourself in #introductions
4. Browse #tutorials for getting started guides
5. Ask questions in #help whenever you need

🔗 Important Links:
- GitHub: https://github.com/sswoo88/vais
- Documentation: https://sswoo.github.io/vais/
- Getting Started: https://sswoo.github.io/vais/tutorial/

We're here to help! Don't hesitate to ask questions in #help.

Happy coding with Vais! 🚀
```

### #welcome Channel Pinned Message

```
# Welcome to Vais Language

🎯 **Vais** is an AI-optimized systems programming language with token-efficient syntax.

**Key Features:**
- Single-letter keywords (F, S, E, I, L, M) for minimal token usage
- Self-recursion operator (@)
- Expression-oriented design
- LLVM backend with native performance
- Type inference system
- Excellent compiler speed and runtime performance

**Server Channels Overview:**
- 📌 **INFORMATION**: Server rules and guidelines
- 💬 **COMMUNITY**: General discussion and introductions
- 🔧 **DEVELOPMENT**: Technical questions and project showcase
- 📚 **LEARNING**: Tutorials, code reviews, and tips
- 🛠️ **CONTRIBUTORS**: Dedicated spaces for compiler and stdlib development

**Getting Started:**
1. Read #rules
2. Select roles in #roles using emoji reactions
3. Introduce yourself in #introductions
4. Check #tutorials for learning resources
5. Ask questions in #help

**Important Links:**
- [Official Repository](https://github.com/sswoo88/vais)
- [Online Documentation](https://sswoo.github.io/vais/)
- [Language Specification](https://sswoo.github.io/vais/language-spec/)
- [Standard Library Reference](https://sswoo.github.io/vais/stdlib/)

**Example Vais Code:**
```vais
# Fibonacci with self-recursion operator
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# Struct definition with type inference
S Point { x:f64, y:f64 }

# Loop with pattern matching
F sum(arr:[i64])->i64 {
    s := 0
    L x:arr { s += x }
    s
}
```

Welcome! We're excited to have you here. 🚀
```

### #roles Channel Message

```
React to this message to select your interests:

📚 **Learning Resources** - Get notified about tutorials and guides
💻 **Development** - Interested in compiler and tooling development
🎮 **Creative Projects** - Share creative/game development projects
🔔 **Announcements Only** - Receive only major announcements
📖 **Documentation** - Interested in documentation improvements
```

---

## Server Rules & Code of Conduct

### #rules Channel Content

```
# Vais Language Community Rules & Code of Conduct

## 1. Be Respectful

- Treat all members with respect and dignity
- No harassment, bullying, or personal attacks
- Respectful disagreement on technical topics is encouraged
- Avoid inflammatory language or provocative statements

**Examples of unacceptable behavior:**
- Name-calling or insults
- Threats or intimidation
- Discrimination based on race, gender, ethnicity, religion, age, disability, or sexual orientation
- Sustained derailment of conversations

## 2. Keep Discussions On-Topic

- #general and #off-topic are for non-Vais conversation
- #development channels should focus on technical content
- Use threads for extended conversations to avoid channel clutter
- Avoid excessive cross-posting the same message in multiple channels

## 3. No Spam or Advertising

- No unsolicited promotion of products, services, or other communities
- No mass-mentions (@everyone, @here) without moderator approval
- No repeated duplicate messages
- No farming of reactions or messages for engagement metrics

**Acceptable self-promotion:**
- Linking your Vais project in #showcase
- Mentioning relevant tools in #help discussions
- Sharing tutorial links in #tutorials with description

## 4. Code Standards

When sharing code:
- Use code blocks with proper syntax highlighting: \`\`\`vais
- Keep code snippets reasonable size (use Gists/Pastebin for large files)
- Provide context: what problem you're solving, what you've tried
- Include minimal reproducible examples for bugs

## 5. No Harmful Content

- No malware, viruses, exploits, or security vulnerabilities
- No illegal content
- No NSFW content, links, or discussions
- No misinformation about Vais or programming concepts

## 6. Respect Privacy

- Don't share personal information without consent
- Don't screenshot private conversations
- Don't share others' code without attribution

## 7. English Language

- Communication primarily in English
- Translations of non-English content appreciated but not required
- Respect language learners with patience

## 8. Use Threads Effectively

- Reply to questions in threads to keep main channels readable
- Use threads for extended help conversations
- Mark resolved threads with ✅ reaction

## 9. Search Before Asking

Before asking a question:
- Search in #help for similar questions
- Check pinned messages in relevant channels
- Review documentation at https://sswoo.github.io/vais/

## 10. Constructive Criticism

- When reviewing code or ideas, be constructive
- Ask clarifying questions before criticizing
- Acknowledge good efforts even when suggesting improvements
- Avoid "X is better than Y" debates without context

---

## Moderation

### Warnings & Actions

**Level 1: Informal Warning**
- First-time minor violations
- Private message with explanation
- No channel message needed

**Level 2: Formal Warning**
- Repeated minor violations or single major violation
- Public warning in channel or private message
- May include timeout (5 minutes - 1 hour)

**Level 3: Timeout**
- Significant or repeated violations
- Temporary mute (1 hour - 24 hours)
- Moderator discretion

**Level 4: Kick or Ban**
- Severe violations, harassment, spam
- Member removed from server
- May be re-invited after discussion with moderators

**Immediate Ban:**
- Threats or harassment
- Hate speech or discrimination
- Illegal content
- Repeated ban evasion

### Escalation

- Moderators discuss cases in private channel
- Admin makes final decision on bans
- Appeal process available for bans (DM an Admin)

---

## Best Practices

### For Members

1. **Read before asking** - Check pinned messages and search existing conversations
2. **Be specific** - Provide context, error messages, and minimal reproducible examples
3. **Be patient** - Maintainers and helpers are volunteers
4. **Share knowledge** - Help others when you can
5. **Use threads** - Keep main channels organized and readable
6. **Format code properly** - Use code blocks for readability
7. **Mark solutions** - React with ✅ when your issue is resolved

### For Moderators

1. **Stay neutral** - Enforce rules consistently regardless of personal opinions
2. **Communicate clearly** - Explain why an action was taken
3. **Document decisions** - Keep notes on warnings and bans
4. **Be proportionate** - Match severity of action to violation
5. **Protect privacy** - Don't expose reasons for removals publicly unless necessary
6. **Seek consensus** - Discuss major decisions with other moderators
7. **Lead by example** - Follow all community rules yourself

### For Maintainers

1. **Be accessible** - Respond to important questions in public channels
2. **Credit contributors** - Acknowledge help in project communications
3. **Provide updates** - Share progress on issues in #announcements
4. **Set tone** - Model respectful, constructive discussion
5. **Trust moderators** - Support moderation decisions publicly
6. **Maintain inclusivity** - Welcome all skill levels and perspectives

### Creating a Healthy Community

- Celebrate member achievements and contributions
- Highlight helpful answers and good questions
- Share interesting projects and learning resources
- Organize occasional community events (code reviews, AMAs, etc.)
- Remain open to feedback about server structure and policies
- Review rules periodically and update as needed

---

## Quick Reference Checklist

### Initial Server Setup
- [ ] Set server name to "Vais Language"
- [ ] Upload server icon
- [ ] Set server description
- [ ] Configure verification and content filter settings
- [ ] Create all channel categories and channels
- [ ] Set channel permissions correctly
- [ ] Create all roles with proper hierarchy

### Bot Configuration
- [ ] Invite MEE6 bot
- [ ] Configure welcome message
- [ ] Set up moderation rules
- [ ] Create reaction role in #roles
- [ ] Test all bot functions

### Content Setup
- [ ] Post #welcome message
- [ ] Post #rules message
- [ ] Post #roles message
- [ ] Create welcome DM template
- [ ] Pin important messages in each channel

### Team Setup
- [ ] Assign moderators
- [ ] Brief moderators on rules and procedures
- [ ] Set up private moderator channel
- [ ] Create initial pinned messages
- [ ] Announce server in relevant communities

---

## Additional Resources

- **Discord Server Management Guide**: https://support.discord.com/hc/articles/204849977
- **Vais GitHub**: https://github.com/sswoo88/vais
- **Official Documentation**: https://sswoo.github.io/vais/
- **Community Guidelines**: See #rules in Discord server

---

**Last Updated:** January 2026
**Maintained by:** Vais Language Community Team

For questions about this guide or server administration, please contact the Admin team.
