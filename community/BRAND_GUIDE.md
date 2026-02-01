# Vais Brand Guidelines

**Version**: 1.0
**Last Updated**: 2026-01-31
**Status**: Official

---

## Introduction

This document establishes the visual identity and brand standards for **Vais**, the AI-optimized systems programming language. These guidelines ensure consistent representation across all platforms, materials, and communications.

### Brand Essence

**Vais** embodies:
- **Efficiency**: Token-optimized design for AI code generation
- **Innovation**: Modern approach to systems programming
- **Accessibility**: Developer-friendly syntax and tools
- **Performance**: LLVM-powered native speed

---

## 1. Logo Guidelines

### 1.1 Symbol Mark

The Vais logo consists of a geometric "V" design that represents velocity, efficiency, and the convergence of AI and systems programming.

#### Concept
- **Shape**: Angular "V" formed by two converging paths
- **Symbolism**: Speed (velocity), Precision (clean geometry), AI Integration (data flow)
- **Style**: Minimalist, modern, scalable

#### Primary Logo - Light Version (for dark backgrounds)

```svg
<svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
  <!-- Light version for dark backgrounds -->
  <defs>
    <linearGradient id="vais-gradient-light" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#48BB78;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#38A169;stop-opacity:1" />
    </linearGradient>
  </defs>

  <!-- Left path of V -->
  <path d="M 30 20 L 60 100 L 48 100 L 20 25 Z"
        fill="url(#vais-gradient-light)"
        stroke="none"/>

  <!-- Right path of V -->
  <path d="M 90 20 L 72 100 L 60 100 L 100 25 Z"
        fill="#667EEA"
        stroke="none"/>

  <!-- Center accent line -->
  <path d="M 56 80 L 64 80 L 60 100 L 56 80 Z"
        fill="#FFFFFF"
        opacity="0.9"/>
</svg>
```

#### Secondary Logo - Dark Version (for light backgrounds)

```svg
<svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
  <!-- Dark version for light backgrounds -->
  <defs>
    <linearGradient id="vais-gradient-dark" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#2F855A;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#276749;stop-opacity:1" />
    </linearGradient>
  </defs>

  <!-- Left path of V -->
  <path d="M 30 20 L 60 100 L 48 100 L 20 25 Z"
        fill="url(#vais-gradient-dark)"
        stroke="none"/>

  <!-- Right path of V -->
  <path d="M 90 20 L 72 100 L 60 100 L 100 25 Z"
        fill="#5A67D8"
        stroke="none"/>

  <!-- Center accent line -->
  <path d="M 56 80 L 64 80 L 60 100 L 56 80 Z"
        fill="#1A202C"
        opacity="0.8"/>
</svg>
```

#### Icon Version (Simplified for small sizes)

```svg
<svg width="48" height="48" viewBox="0 0 48 48" xmlns="http://www.w3.org/2000/svg">
  <!-- Simplified icon version -->
  <path d="M 12 8 L 24 40 L 20 40 L 8 10 Z"
        fill="#48BB78"/>
  <path d="M 36 8 L 28 40 L 24 40 L 40 10 Z"
        fill="#667EEA"/>
</svg>
```

### 1.2 Minimum Size

To ensure legibility and visual impact:

- **Digital**: 24px minimum height
- **Print**: 0.5 inches (12.7mm) minimum height
- **Social Media Profile**: 200x200px minimum

Below these sizes, use the simplified icon version.

### 1.3 Clear Space

Maintain clear space around the logo equal to **50% of the logo height** on all sides.

```
[Clear Space = 0.5h]
    ┌─────────────────┐
    │                 │
    │   ┌─────────┐   │
    │   │  VAIS   │   │  h = logo height
    │   │  LOGO   │   │
    │   └─────────┘   │
    │                 │
    └─────────────────┘
[Clear Space = 0.5h]
```

### 1.4 Word Mark

#### Typography
- **Font**: Inter Bold or JetBrains Mono Bold
- **Style**: "Vais" with capital V, lowercase ais
- **Letter Spacing**: -2% (tight)
- **Weight**: 700 (Bold)

#### Styling Options

**Primary Word Mark**
```
Vais
Font: Inter Bold
Size: 48pt
Color: #1A202C (dark) or #FFFFFF (light)
```

**Code-Style Word Mark**
```
vais
Font: JetBrains Mono Bold
Size: 42pt
Color: #48BB78
Use: Technical documentation, code examples
```

### 1.5 Logo + Word Mark Combination

**Horizontal Layout** (Preferred for headers)
```
┌────┐
│ V  │  Vais
└────┘
[Logo] [Word Mark]
Spacing: 1x logo width
```

**Vertical Layout** (For square spaces)
```
┌────┐
│ V  │
└────┘
 Vais
```

### 1.6 Usage Guidelines - Do's and Don'ts

#### DO
- Use official logo files provided
- Maintain aspect ratio when scaling
- Use appropriate version for background (light/dark)
- Place on solid backgrounds when possible
- Ensure sufficient contrast

#### DON'T
- Change logo colors (except approved versions)
- Rotate or skew the logo
- Add effects (shadows, glows, 3D)
- Place on busy or low-contrast backgrounds
- Stretch or compress disproportionately
- Recreate or modify the logo
- Use old or unofficial versions

---

## 2. Color Palette

### 2.1 Primary Colors

#### Vais Green
```
HEX:  #48BB78
RGB:  72, 187, 120
HSL:  145, 45%, 51%
CMYK: 61, 0, 36, 27

Usage: Primary brand color, CTAs, highlights
Symbolism: Growth, efficiency, sustainability
```

#### Vais Dark
```
HEX:  #1A202C
RGB:  26, 32, 44
HSL:  216, 25%, 14%
CMYK: 41, 27, 0, 83

Usage: Text, backgrounds, headers
Symbolism: Solidity, systems programming, reliability
```

### 2.2 Secondary Colors

#### Accent Blue
```
HEX:  #667EEA
RGB:  102, 126, 234
HSL:  229, 76%, 66%
CMYK: 56, 46, 0, 8

Usage: Interactive elements, links, AI-related features
Symbolism: Technology, intelligence, innovation
```

#### Accent Dark Green
```
HEX:  #38A169
RGB:  56, 161, 105
HSL:  148, 48%, 43%
CMYK: 65, 0, 35, 37

Usage: Secondary actions, success states
```

#### Warning Orange
```
HEX:  #ED8936
RGB:  237, 137, 54
HSL:  27, 83%, 57%
CMYK: 0, 42, 77, 7

Usage: Warnings, attention, important notices
```

#### Error Red
```
HEX:  #FC8181
RGB:  252, 129, 129
HSL:  0, 95%, 75%
CMYK: 0, 49, 49, 1

Usage: Errors, critical alerts, danger states
```

### 2.3 Neutral Colors

#### Light Neutrals (for light themes)
```
White:        #FFFFFF
Light Gray:   #F7FAFC
Medium Gray:  #EDF2F7
Border Gray:  #E2E8F0
Text Gray:    #718096
```

#### Dark Neutrals (for dark themes)
```
Dark:         #1A202C
Darker:       #2D3748
Medium Dark:  #4A5568
Light Dark:   #718096
```

### 2.4 Gradients

#### Primary Gradient (Green to Dark Green)
```
Linear Gradient: 135deg
From: #48BB78
To:   #38A169

Usage: Buttons, hero sections, featured elements
```

#### Hero Gradient (Blue to Green)
```
Linear Gradient: 135deg
From: #667EEA (Accent Blue)
To:   #48BB78 (Vais Green)

Usage: Hero backgrounds, splash screens, promotional materials
```

#### Dark Gradient (for backgrounds)
```
Linear Gradient: 180deg
From: #2D3748
To:   #1A202C

Usage: Dark mode backgrounds, code editors
```

### 2.5 Color Usage in Context

#### Light Mode
- **Background**: #FFFFFF, #F7FAFC
- **Primary Text**: #1A202C
- **Secondary Text**: #4A5568
- **Borders**: #E2E8F0
- **Links**: #667EEA
- **CTAs**: #48BB78

#### Dark Mode
- **Background**: #1A202C, #2D3748
- **Primary Text**: #FFFFFF, #F7FAFC
- **Secondary Text**: #CBD5E0
- **Borders**: #4A5568
- **Links**: #667EEA
- **CTAs**: #48BB78

### 2.6 Accessibility Guidelines

Ensure WCAG 2.1 Level AA compliance:

**Text Contrast Ratios** (minimum)
- Normal text: 4.5:1
- Large text (18pt+): 3:1
- UI components: 3:1

**Approved Combinations**
- #1A202C text on #FFFFFF background (16.1:1) ✓
- #FFFFFF text on #48BB78 background (2.7:1) - Large text only
- #FFFFFF text on #667EEA background (4.8:1) ✓
- #1A202C text on #48BB78 background (6.0:1) ✓

---

## 3. Typography

### 3.1 Font Families

#### Primary: Inter
```
Purpose: UI, marketing, documentation
Weights: 400 (Regular), 600 (Semibold), 700 (Bold)
Source: Google Fonts
License: Open Font License
Fallback: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif
```

#### Code: JetBrains Mono
```
Purpose: Code snippets, technical content, monospace
Weights: 400 (Regular), 500 (Medium), 700 (Bold)
Source: JetBrains
License: OFL
Fallback: "Courier New", Courier, monospace
```

### 3.2 Type Scale

#### Desktop Scale
```
h1: 48px / 3rem     - Line height: 1.2 - Weight: 700 (Bold)
h2: 36px / 2.25rem  - Line height: 1.3 - Weight: 700 (Bold)
h3: 30px / 1.875rem - Line height: 1.4 - Weight: 600 (Semibold)
h4: 24px / 1.5rem   - Line height: 1.4 - Weight: 600 (Semibold)
h5: 20px / 1.25rem  - Line height: 1.5 - Weight: 600 (Semibold)
h6: 18px / 1.125rem - Line height: 1.5 - Weight: 600 (Semibold)

Body Large:  18px / 1.125rem - Line height: 1.7 - Weight: 400 (Regular)
Body:        16px / 1rem     - Line height: 1.6 - Weight: 400 (Regular)
Body Small:  14px / 0.875rem - Line height: 1.6 - Weight: 400 (Regular)

Caption:     12px / 0.75rem  - Line height: 1.5 - Weight: 400 (Regular)
```

#### Mobile Scale
```
h1: 36px / 2.25rem
h2: 30px / 1.875rem
h3: 24px / 1.5rem
h4: 20px / 1.25rem
h5: 18px / 1.125rem
h6: 16px / 1rem

Body Large:  16px / 1rem
Body:        15px / 0.9375rem
Body Small:  14px / 0.875rem

Caption:     12px / 0.75rem
```

### 3.3 Code Typography

#### Inline Code
```
Font: JetBrains Mono
Size: 0.9em (relative to surrounding text)
Background: #F7FAFC (light mode) / #2D3748 (dark mode)
Padding: 2px 6px
Border Radius: 4px
Color: #ED8936 (orange accent)
```

#### Code Blocks
```
Font: JetBrains Mono
Size: 14px / 0.875rem
Line Height: 1.6
Background: #1A202C (dark)
Padding: 16px 20px
Border Radius: 8px
Border: 1px solid #2D3748
```

### 3.4 Usage Examples

#### Marketing Headlines
```
Font: Inter Bold
Size: h1 (48px)
Color: #1A202C
Letter Spacing: -0.02em (tight)
Line Height: 1.2
```

#### Technical Documentation Headers
```
Font: Inter Semibold
Size: h2-h4 (36px-24px)
Color: #1A202C (light) / #F7FAFC (dark)
Letter Spacing: -0.01em
```

#### Code Examples
```
Font: JetBrains Mono Regular
Size: 14px
Color: Syntax highlighting colors
Background: #1A202C
```

---

## 4. Tone & Voice

### 4.1 Brand Personality

**Core Attributes**
- **Efficient**: Clear, concise, no fluff
- **Innovative**: Forward-thinking, modern approaches
- **Accessible**: Welcoming to developers of all levels
- **Developer-friendly**: Understanding of real-world needs

### 4.2 Voice Characteristics

#### Technical Communication
- **Precise**: Use exact terminology
- **Clear**: Explain complex concepts simply
- **Structured**: Organize information logically
- **Example-driven**: Show, don't just tell

#### Marketing Communication
- **Enthusiastic**: Show genuine excitement
- **Confident**: Assert value without arrogance
- **Inclusive**: Welcome all developers
- **Educational**: Inform while promoting

### 4.3 Tone Guidelines by Context

#### Documentation
```
Tone: Professional, instructional, helpful
Voice: Active, direct, clear
Example: "Define functions with the F keyword. Use @ for recursive calls."
Avoid: Passive voice, jargon without explanation, condescension
```

#### Social Media
```
Tone: Friendly, energetic, conversational
Voice: Enthusiastic, approachable, community-focused
Example: "Check out this elegant recursive Fibonacci! The @ operator makes it beautifully concise."
Avoid: Corporate speak, excessive emojis, hype without substance
```

#### Error Messages
```
Tone: Helpful, constructive, clear
Voice: Direct, actionable, encouraging
Example: "Type mismatch: expected i64, found str. Try using to_i64() to convert."
Avoid: Blame, vagueness, technical jargon dumps
```

#### Release Notes
```
Tone: Informative, celebratory, technical
Voice: Detailed, appreciative, forward-looking
Example: "Phase 14 P2 introduces production-ready deployment. Thanks to all contributors!"
Avoid: Underselling improvements, omitting credits, vague descriptions
```

### 4.4 Writing Style Examples

#### DO ✓
- "Vais minimizes tokens while maximizing expressiveness"
- "The @ operator makes recursion elegant and efficient"
- "Get started in 5 minutes with our quick installation guide"
- "Join our community of developers building with Vais"
- "Competitive performance with Rust and C"

#### DON'T ✗
- "Vais is the only language that gets it right"
- "Other languages waste your tokens"
- "Why would anyone use [language] when Vais exists?"
- "Revolutionary paradigm-shifting blockchain AI quantum..."
- "Blazingly fast" (overused marketing cliche)

### 4.5 Vocabulary

#### Preferred Terms
- AI-optimized (not "AI-first" or "AI-native")
- Token-efficient (not "token-minimal")
- Systems programming (not "low-level")
- Developer-friendly (not "easy" or "simple")
- LLVM-powered (not "blazingly fast")

#### Avoid
- Revolutionary, disruptive (overused)
- Blockchain, quantum (unless actually relevant)
- Next-generation, cutting-edge (vague)
- Game-changing, paradigm-shifting (hyperbolic)

---

## 5. Marketing Assets

### 5.1 Social Media Specifications

#### Profile Images (All Platforms)
```
Format: PNG with transparent background
Size: 400x400px minimum
Content: Vais icon (simplified V logo)
Export: @1x, @2x, @3x for high-DPI displays
```

#### Twitter/X
```
Profile Image: 400x400px (displays as 200x200px circle)
Header Image: 1500x500px
Aspect Ratio: 3:1
Safe Area: Center 1500x350px (avoid edges)
```

#### Instagram
```
Profile Image: 320x320px (displays as 110x110px circle)
Feed Posts: 1080x1080px (square) or 1080x1350px (portrait)
Stories: 1080x1920px (9:16 aspect ratio)
Reels: 1080x1920px (9:16 aspect ratio)
```

#### GitHub
```
Profile Image: 460x460px
Social Preview (OG Image): 1280x640px (2:1 aspect ratio)
```

#### Discord
```
Server Icon: 512x512px (PNG)
Server Banner: 960x540px (16:9 aspect ratio)
```

#### LinkedIn
```
Profile Image: 400x400px
Cover Image: 1584x396px (4:1 aspect ratio)
```

### 5.2 Open Graph (OG) Images

#### Standard OG Image
```
Dimensions: 1200x630px (1.91:1 aspect ratio)
Format: PNG or JPEG
Max File Size: 8MB (aim for <300KB)
Safe Area: 1104x513px (avoid edges for cropping)

Content Layout:
- Background: Dark gradient (#2D3748 to #1A202C)
- Logo: Top left (60x60px)
- Headline: Inter Bold 48px, white
- Description: Inter Regular 24px, #CBD5E0
- Footer: "vaislang.dev" bottom right
```

#### Documentation OG Image
```
Same specs as standard, but include:
- Section name (e.g., "Language Spec", "Tutorial")
- Doc-specific accent color
```

### 5.3 Marketing Collateral

#### Business Card
```
Size: 3.5" x 2" (89mm x 51mm)
Front: Logo + Name + Title
Back: Website + GitHub + Email
Stock: Matte finish recommended
```

#### Stickers
```
Formats:
- 3" circle (logo only)
- 2" x 4" rectangle (logo + word mark)
- Die-cut (custom V shape)

Material: Vinyl, weather-resistant
Finish: Matte or glossy
```

#### T-Shirts
```
Print Areas:
- Front left chest: 4" logo
- Back center: 10" logo + "vais" word mark
- Sleeve: Small icon (2")

Colors:
- Black shirt, green/blue logo
- Gray shirt, full-color logo
- Green shirt, white logo
```

### 5.4 Presentation Templates

#### Title Slide
```
Background: Dark gradient
Logo: Top left
Title: Center, Inter Bold 64px
Subtitle: Center below, Inter Regular 32px
Footer: URL or event name
```

#### Content Slide
```
Title: Top, Inter Bold 36px, Vais Green
Body: Inter Regular 24px
Code: JetBrains Mono 18px in dark code block
Bullet points: 32px line height
```

#### Code Example Slide
```
Full-screen dark background
Code: JetBrains Mono 20px
Syntax highlighting using Vais Green and Accent Blue
Line numbers: Optional, gray
```

### 5.5 Banner Templates

#### GitHub Repository Banner
```
Size: 1280x640px
Content:
- Vais logo + word mark
- Tagline: "AI-optimized systems programming language"
- Key features (3-4 bullet points)
- Background: Hero gradient
```

#### Conference/Event Banner
```
Standard Web: 728x90px (leaderboard)
Large Web: 970x250px (billboard)
Mobile: 320x50px

Include:
- Logo
- Event name
- Call to action
- High contrast for visibility
```

---

## 6. Asset Checklist

### Brand Asset Library

- [ ] Logo files (SVG, PNG at multiple resolutions)
  - [ ] Primary light version
  - [ ] Primary dark version
  - [ ] Icon/simplified version
  - [ ] Monochrome versions (black, white)

- [ ] Color swatches
  - [ ] Sketch/Figma color palette file
  - [ ] Adobe ASE (Adobe Swatch Exchange) file
  - [ ] CSS/SCSS variables file

- [ ] Typography
  - [ ] Font files (Inter, JetBrains Mono)
  - [ ] Typography scale documentation
  - [ ] Web font CSS imports

- [ ] Templates
  - [ ] Social media post templates (Canva, Figma)
  - [ ] Presentation slide deck template
  - [ ] GitHub social preview template
  - [ ] Documentation banner template

- [ ] Marketing Materials
  - [ ] Business card design
  - [ ] Sticker designs
  - [ ] T-shirt mockups
  - [ ] Conference booth graphics

- [ ] Digital Assets
  - [ ] Favicon (16x16, 32x32, 64x64)
  - [ ] Apple Touch Icon (180x180)
  - [ ] OG Images (default, docs, blog)
  - [ ] Email signature template

---

## 7. Brand Applications

### 7.1 Website

#### Homepage Hero
```
Background: Hero gradient (#667EEA to #48BB78)
Headline: Inter Bold 56px, white
Subheadline: Inter Regular 24px, rgba(255,255,255,0.9)
CTA Button: Vais Green with white text
Secondary CTA: Outline button, white
```

#### Navigation
```
Background: White (light) / #1A202C (dark)
Logo: Vais icon + word mark (32px height)
Links: Inter Semibold 16px
Active state: Vais Green underline
Hover: Accent Blue color
```

#### Code Examples
```
Background: #1A202C
Border: 1px #2D3748
Border Radius: 8px
Code font: JetBrains Mono 14px
Syntax colors:
  - Keywords: #48BB78
  - Functions: #667EEA
  - Strings: #ED8936
  - Comments: #718096
  - Numbers: #FC8181
```

### 7.2 Documentation

#### Sidebar
```
Background: #F7FAFC (light) / #2D3748 (dark)
Active link: Vais Green background with white text
Hover: Light green background (#E6F7ED)
Font: Inter Regular 15px
```

#### Content Area
```
Max width: 800px
Font: Inter Regular 16px, line-height 1.7
Headings: Inter Bold with Vais Green accent
Code inline: JetBrains Mono with orange background
Code blocks: Dark theme with syntax highlighting
```

#### Callouts
```
Info: Blue left border, light blue background
Warning: Orange left border, light orange background
Success: Green left border, light green background
Error: Red left border, light red background
```

### 7.3 Command Line Interface (CLI)

#### Color Scheme
```
Success messages: Vais Green (#48BB78)
Errors: Error Red (#FC8181)
Warnings: Warning Orange (#ED8936)
Info: Accent Blue (#667EEA)
Hints: Light Gray (#718096)
```

#### Output Formatting
```
Headers: Bold
File paths: Cyan
Code snippets: JetBrains Mono style
Progress bars: Green fill
Spinners: Green animation
```

### 7.4 IDE Extensions

#### Syntax Highlighting
```
Keywords (F, S, E, I, L, M): #48BB78 (Vais Green)
Functions: #667EEA (Accent Blue)
Strings: #ED8936 (Warning Orange)
Numbers: #FC8181 (Error Red)
Comments: #718096 (Text Gray)
Operators: #CBD5E0 (Light text)
Variables: #E2E8F0 (Light Gray)
Types: #9F7AEA (Purple)
```

#### UI Theme
```
Background: #1A202C (dark)
Sidebar: #2D3748
Active file: #48BB78 accent
Selection: #667EEA with opacity
Cursor: #48BB78
Line numbers: #4A5568
```

---

## 8. Usage Rights & Licensing

### Logo Usage
- The Vais logo is a trademark of the Vais project
- May be used to refer to the Vais programming language
- May not be used to imply endorsement without permission
- May not be modified except for scaling and color (approved versions only)

### Community Use
Allowed:
- Educational materials and tutorials
- Open source projects built with Vais
- Blog posts and articles about Vais
- Conference presentations (with attribution)

Requires Permission:
- Commercial products featuring the Vais brand
- Merchandise for sale
- Marketing materials for commercial services
- Company logos incorporating Vais branding

### Attribution
When using Vais branding, include:
```
"Vais" and the Vais logo are associated with the Vais programming language project.
Visit https://github.com/vaislang/vais
```

---

## 9. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-31 | Initial brand guidelines release |

---

## 10. Contact & Support

For brand guideline questions or permission requests:
- GitHub Issues: https://github.com/vaislang/vais/issues
- Email: community@vaislang.dev (when available)
- Discussions: https://github.com/vaislang/vais/discussions

For brand asset requests:
- Check the official repository: `/community/brand-assets/`
- Request missing assets via GitHub issue with `brand` label

---

## Appendix A: Quick Reference

### Color Codes (Copy-Paste)
```css
/* Primary */
--vais-green: #48BB78;
--vais-dark: #1A202C;

/* Secondary */
--accent-blue: #667EEA;
--accent-dark-green: #38A169;
--warning-orange: #ED8936;
--error-red: #FC8181;

/* Neutrals Light */
--white: #FFFFFF;
--gray-50: #F7FAFC;
--gray-100: #EDF2F7;
--gray-200: #E2E8F0;
--gray-600: #718096;

/* Neutrals Dark */
--dark-800: #1A202C;
--dark-700: #2D3748;
--dark-600: #4A5568;

/* Gradients */
--gradient-primary: linear-gradient(135deg, #48BB78, #38A169);
--gradient-hero: linear-gradient(135deg, #667EEA, #48BB78);
--gradient-dark: linear-gradient(180deg, #2D3748, #1A202C);
```

### Font Imports
```css
/* Google Fonts - Inter */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');

/* JetBrains Mono */
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;700&display=swap');
```

---

**Document Maintained By**: Vais Community Team
**Last Review**: 2026-01-31
**Next Review**: 2026-07-31
