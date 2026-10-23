# Written by Hand — Growth, Adoption & Launch Plan

> **Implementation-ready companion document** for `WRITTEN_BY_HAND_IMPLEMENTATION_PLAN.md`  
> **Project:** Written by Hand (`wbh`)  
> **Positioning:** A tiny badge for code made the old-fashioned way.  
> **Audience:** GitHub maintainers, hobbyists, indie developers, Rust/open-source communities  
> **Objective:** Make the badge easy to discover, easy to adopt, and naturally shareable.  
> **Status:** Product/growth specification and launch checklist — *not* evidence of a completed launch.

---

## 0. Instructions for the coding AI agent

This document **extends**, but does not replace, `WRITTEN_BY_HAND_IMPLEMENTATION_PLAN.md`.

1. Read the primary engineering plan before implementing features.
2. Implement the working Rust CLI and accurate reporting first. The growth features in this document should not delay a reliable `v0.1` release.
3. Add an **Instant Badge** path that requires no installation, login, Rust, Node.js, or code scanning.
4. Build the public badge gallery as a **static site** with no backend, database, authentication, tracking, or required build-time web service.
5. Keep the distinction clear: *Instant Badge* = a decorative, self-declared label; *Advanced Badge* = CLI-generated mark coverage with report/Code Map.
6. Never imply handwritten claims have been independently verified. Never call unmarked code AI-written.
7. Treat all example domains and repository names in this document as placeholders until the real repo URL is configured. Do not publish broken example links.
8. Do not post to social networks or communities automatically. Prepare launch assets and copy for human review/publication.
9. Only report tests and publishing actions actually completed.

### Definition of success

**North-star adoption metric:** number of *independent public repositories* whose maintainers have chosen to display a Written by Hand badge.

GitHub stars, page views, and social impressions are secondary signals. The desired loop is:

`Developer discovers badge → adds it to README → someone else sees and clicks → visits gallery → copies their own badge → new README displays it.`

---

## 1. Core positioning

### 1.1 Product promise

> **Written by Hand**  
> *A tiny badge for code made the old-fashioned way.*
>
> It doesn't detect AI. It doesn't prove anything. It's just a little stamp of pride.

This is a playful symbol of craftsmanship, not an anti-AI campaign, moral judgement, security attestation, or certification service.

The product should feel:

- **Playful:** a light developer-culture joke, not an assessment of someone's development practices.
- **Proud but not preachy:** it celebrates handwritten portions of code without shaming other workflows.
- **Transparent:** all labels are self-declared.
- **Small:** optional, unobtrusive, trivial to remove.
- **Accessible:** an ordinary GitHub user can adopt a badge without knowing Rust.
- **Tasteful:** the badge looks good in both light and dark README themes.

### 1.2 Avoid these claims

- "Verified human code"
- "AI-free certified"
- "Detects ChatGPT-generated code"
- "Proof that every line was typed by a human"
- "Automatically recognizes handwritten source"

Instead prefer **"self-declared"**, **"marked handwritten"**, and **"unmarked"**.

### 1.3 Brand voice and possible copy

Primary copy:

- **Written by Hand**
- *Proudly written by hand.*
- *Some things are still better made by hand.*
- *Made with fingers and questionable decisions.*
- *Artisanal bugs since 2026.*

Humorous alternatives (optional, not all in the default UI):

- *No prompts were harmed in the making of this code.*
- *Made by a human, allegedly.*
- *Powered by coffee & fingers.*
- *Certified by Me™.*

If a joke suggests proof or certification, make the parody/self-declaration explicit nearby.

---

## 2. Two-tier adoption model — highest-priority product adjustment

### 2.1 Tier A: Instant Badge (zero-installation)

**Target:** the person who sees a funny badge in a README and wants it on their own repository in less than a minute.

User flow:

1. Click a badge in someone else's README.
2. Open the Written by Hand project or gallery.
3. Choose one of the badge styles.
4. Choose a supported static label variant (see below).
5. Click **Copy Markdown**.
6. Paste it into their `README.md`.
7. Done: there is **no CLI, scanner, account, or code map** involved.

Requirements:

- Public gallery hosted as a static site, e.g. via GitHub Pages.
- Actual ready-made, standalone SVG files for each style/variant.
- The generated Markdown has a stable, publicly accessible SVG URL.
- The clickable badge links to the public gallery or project landing page, **not** to a nonexistent repository-specific report.
- Badge gallery previews and copy action work without any network API calls once site assets are loaded.
- No arbitrary percentage input for instant badges. A self-selected percentage would look computed despite having no scan. **Static generic labels only** in the instant flow.
- An accessible copy fallback displays the Markdown in a selectable field if clipboard APIs are unavailable.

**Instant labels:**

- `Written by Hand` — default.
- `Handcrafted Code` — optional.
- `Made by a Human, Allegedly` — optional humorous style.

The label and style pair must correspond to a genuinely existing published SVG asset. Never construct unsupported asset URLs.

**Example README snippet (illustrative placeholders only):**

```md
[![Written by Hand](https://<ACTUAL-GALLERY-HOST>/badges/stamp/written-by-hand.svg)](https://<ACTUAL-GALLERY-HOST>/)
```

Do not copy this snippet verbatim into production: replace the host with the real public gallery URL.

### 2.2 Tier B: Advanced Badge (Rust CLI)

**Target:** developers who want to identify files/code blocks and show a generated code map.

User flow:

1. Download the native `wbh` executable.
2. Run `wbh init`, select a badge.
3. Mark files with `wbh mark path`, or put `@handwritten` markers in supported source comments.
4. Run `wbh scan`.
5. Commit `.wbh/badge.svg` and `.wbh/REPORT.md`.
6. README badge links to `.wbh/REPORT.md`, with source-line links and optionally an interactive Code Map.

All scanning, coverage, marker semantics, and schemas remain governed by the primary engineering plan.

**Key UX difference:**

| Capability | Instant | Advanced |
|---|---|---|
| Requires installation | No | Yes, one binary |
| Requires user account | No | No |
| Badge design selection | Yes | Yes |
| Handwritten percentage | No | Yes, computed from markers |
| File/block marking | No | Yes |
| Click-through destination | Project/gallery | Repository report or Code Map |
| Hosted application backend | No | No |
| Independent verification | No | No |

### 2.3 Upgrade path from Instant → Advanced

Offer a subtle line in the gallery:

> Want your badge to show *which code* was written by hand? Try the `wbh` CLI →

This is an optional enhancement, not a forced registration funnel.

---

## 3. Badge strategy

The badge itself is the distribution medium. Optimize for immediate visual recognition and aesthetic fit in READMEs.

### 3.1 Badge collection

Start with three fully finished styles, then expand to six:

| Style ID | Name | Visual direction | Role |
|---|---|---|---|
| `stamp` | Rubber Stamp | Warm paper/red-brown stamp, handcrafted impression | **Signature** look |
| `classic` | Classic | Compact GitHub/Shield-like two-part badge | Familiar and legible |
| `terminal` | Terminal | Dark background, green monospace command line | Developer niche |
| `pixel` | Pixel | Retro computing / pixel-art aesthetic | Playful personality |
| `minimal` | Minimal | Quiet type + pencil mark | Elegant READMEs |
| `coffee` | Coffee Edition | Warm vintage label with coffee motif | Humor/social sharing |

Every SVG:

- Must work embedded in GitHub Markdown.
- Must be self-contained; no external font, script, remote image, or interactive SVG behavior.
- Must render crisply at normal README scale.
- Must include meaningful accessible `title`/description where supported.
- Must maintain readable contrast and sensible light/dark presentation.
- Must be tested for XML correctness and safe text escaping.
- Should remain modest in height so it sits well near other badges.

### 3.2 Static vs generated variants

**Static (gallery):** `style + allowed label` → a pre-generated SVG. No user-specific numbers.

**Generated (CLI):** `style + optional scan percentage + optional custom text` → a local SVG from real marker coverage.

For CLI-generated percentages, ensure the README/report explains this is **self-declared marker coverage**, not verified human origin.

### 3.3 Click-through experience

- For Instant badges, destination = public gallery/landing page.
- For Advanced badges, destination = generated `REPORT.md` or the user's Code Map.
- Gallery page includes a visible **Get this badge** action near previews.
- If an Advanced Code Map contains a "Get this badge" link, it must be unobtrusive, clearly external, and removable/configurable.
- Never silently swap a repository-specific report link with a promotional landing page.

### 3.4 Viral loop constraints

The project should encourage sharing **through useful/desirable artifacts** rather than unsolicited marketing:

- No branded links in source comments.
- No automatic posting or tagging people.
- No hidden telemetry in SVGs.
- No requirement to keep a promotional backlink in the README for locally generated badges.
- Avoid any badge appearance that falsely suggests certification by a third party.

---

## 4. Static badge gallery — functional specification

### 4.1 Goal

Make it possible to choose and copy an attractive badge in approximately 15 seconds when the user already knows what they want.

### 4.2 Single-page layout

Suggested hierarchy:

1. Wordmark / logo: **Written by Hand**.
2. Hero: **Good code takes a human touch.**
3. One-sentence product explanation and the self-declared nature of the badge.
4. Badge style gallery (six tiles, three in first public version).
5. Large live preview of the selected SVG.
6. Optional label selector (only variants for which SVG assets exist).
7. **Copy Markdown** as the primary action.
8. Secondary action: download/save SVG locally (optional; avoid claiming that downloaded SVG auto-updates).
9. Short explanation of Instant vs Advanced.
10. GitHub source/documentation link and attribution/license.

Keep the site simple and elegant. Prefer static HTML, plain CSS, and a small amount of vanilla JavaScript.

### 4.3 Interaction details

- Selecting a style immediately updates the preview, copyable code, and accessible label.
- The copy control copies a complete, valid Markdown image link.
- Confirm copy success unobtrusively; display a useful fallback message on failure.
- Keyboard selection and visible focus states required.
- Support narrow mobile and desktop layouts; badges must never be clipped.
- Provide descriptive alt text for all SVG previews.
- A badge variant must not appear selectable if its asset was not built/published.
- Gallery URLs and link targets come from **one deployment config constant** to avoid inconsistent paths.
- A preview must not accidentally link to a user's private code, local filesystem, or placeholder host.

### 4.4 Hosting

**Default:** GitHub Pages from a published branch/folder or a workflow-based artifact. No custom backend.

Recommended output structure:

```text
site/
├── index.html
├── styles.css
├── app.js
├── robots.txt
├── favicon.svg
├── social-preview.png        # produced by design workflow
└── badges/
    ├── stamp/
    │   ├── written-by-hand.svg
    │   └── handcrafted-code.svg
    ├── classic/
    ├── terminal/
    ├── pixel/
    ├── minimal/
    └── coffee/
```

An alternative implementation can keep source assets under `assets/` and publish the transformed site under `dist/gallery/`. The important part is a stable public `badges/<style>/<variant>.svg` path.

**URL stability requirement:** treat published SVG links as an API. Once people paste an asset URL into a README, renaming or removing it breaks external repos. Keep old URLs working via retained files/aliases.

### 4.5 Static site security/privacy

- No accounts, cookies, trackers, or analytics scripts by default.
- No runtime form submissions or arbitrary HTML rendering.
- Construct asset lookups from a fixed allowlist of styles and variants.
- Never concatenate untrusted strings into HTML markup.
- Avoid third-party JS/CDN requirements.
- Do not promise meaningful copy analytics without an opt-in measurement method.

### 4.6 Gallery acceptance tests

- [ ] Gallery renders with JavaScript enabled and shows a useful static fallback without it.
- [ ] Every visible badge tile points to a valid SVG in the published asset directory.
- [ ] Copy Markdown produces correct URL-encoded, Markdown-safe output.
- [ ] Copied Markdown displays in a GitHub README and links to the live gallery.
- [ ] Layout works at narrow phone widths and on desktop.
- [ ] All options are operable by keyboard and announced clearly to screen readers.
- [ ] No tracker, account, backend, or sensitive data requirement.
- [ ] Re-publishing never invalidates previously issued badge URLs.

---

## 5. Repository as a landing page

The GitHub repository itself is the second conversion surface. README must sell the idea within one scroll.

Recommended top-to-bottom structure:

1. Large, tasteful logo/hero image or an illustrative badge strip.
2. `Written by Hand` + one-line tagline.
3. A short **"It doesn't detect AI, and it doesn't prove anything"** explanation.
4. Badge design gallery preview (three examples above the fold where practical).
5. **"Want a badge now?"** link to Instant Gallery.
6. **"Want a code map?"** 4-line CLI quick start.
7. Screenshot/GIF showing scan → badge → linked report.
8. Supported platforms and installation links.
9. Marker syntax example and basic report output.
10. Contributing instructions, license, and community guidelines.

Use applicable GitHub topics such as `rust`, `cli`, `developer-tools`, `github-readme`, `badge`, `open-source` where accurate.

### README conversion goals

A visitor should understand in **five seconds**:

- What the badge means.
- That using it is intentionally not verified.
- How to get one instantly.
- That a deeper CLI/code-map tool exists.

### Social preview

Prepare a GitHub social preview image around **1280 × 640 px**, featuring:

- Clear "Written by Hand" wordmark.
- Distinctive Rubber Stamp artwork.
- The one-line tagline.
- Minimal clutter and strong legibility at thumbnail scale.

Deliver the image as an actual project asset at launch; don't rely on an unbuilt mockup.

---

## 6. Primary distribution channels

### 6.1 Prioritized channels

| Channel | Priority | What to publish | Guardrails |
|---|---|---|---|
| GitHub | Critical | Beautiful README, badges, release, demo repo | Real working binaries/examples |
| Hacker News / Show HN | Very high | Working public tool with focused story | No vote requests; follow submission rules |
| Reddit | High | Technical demo, lessons, honest feedback request | Read each community's current rules; no repetitive promotion |
| Bluesky / Mastodon | High | Short GIF, joke, simple link | Avoid spam/automated outreach |
| DEV Community | Medium | Build story and technical design write-up | Educational substance over ads |
| Product Hunt | Later | Polished gallery + CLI + demo | Wait until onboarding really works |

Community rules and submission norms change. Check the current relevant rules at launch time rather than assuming older advice is valid.

### 6.2 Show HN approach

**Candidate title:**

> Show HN: Written by Hand – A tiny badge for human-written code

Landing URL should resolve to the working repository or live gallery with obvious source/download links.

First-comment structure:

- Why it was made (playful response to increasingly automated code writing).
- What it actually does (mark files/blocks, generate badge, optional Code Map).
- What it explicitly does **not** do (AI detection or provenance verification).
- Implementation basics (small Rust binary, local-first, static HTML/SVG).
- What feedback is useful (badge designs, integration friction, edge cases).

Do not ask people to upvote, coordinate votes, or flood comments.

### 6.3 Reddit approach

Publish only in appropriate communities where project sharing or feedback is allowed, respecting weekly threads and self-promotion restrictions. Potential audiences include Rust, programming, open-source, indie hacking, and side-project communities — **not** a claim that any specific subreddit currently permits promotional standalone posts.

Suggested question-led angle:

> I made a tiny Rust tool that lets you stamp the files you wrote by hand. It doesn't try to detect AI; it just generates a self-declared GitHub badge and code map. Would you use this, or is it a perfectly useless developer joke?

Include source, demo, and limitations. Reply genuinely to feedback. Don't repost identical promotional copy across communities.

### 6.4 Bluesky/Mastodon approach

Use a silent 5–12 second looping demo (no camera or narration required):

`Badge gallery → Choose style → Copy Markdown → Paste into README → Click badge → Code Map.`

Suggested post:

> I made a tiny badge for code written the old-fashioned way. ✍️  
> No AI detection. No verification. Just a little stamp of pride.  
> Pick a badge in seconds, or mark source files with a small Rust CLI.  
> [real project link]

Limit the post to a single clear action. Use platform-native formatting, alt text, and a readable visual.

### 6.5 DEV Community approach

Suggested article title:

**"I built a ridiculous Rust tool to stamp my handwritten code"**

Outline:

1. The joke and why it resonated.
2. What self-declared means and why verification was intentionally excluded.
3. The zero-install badge gallery.
4. CLI scanner/comment marker design.
5. SVG generation and README integration.
6. What worked/failed during launch.
7. Source/demo links.

No invented traction claims, benchmark numbers, or testimonials.

### 6.6 Product Hunt (optional second wave)

Consider only after:

- Gallery and CLI have stable onboarding.
- Screenshots/demo/GIF are polished.
- External users have tried it.
- Installation and README links work on supported platforms.

The project may resonate more on developer-focused channels than on broad product directories; do not over-invest early.

---

## 7. First-30-days launch execution plan

This is an **execution schedule**, not a prediction of adoption.

### Days 1–3: Prepare launch-quality surfaces

- [ ] Complete core CLI, valid releases, and end-to-end demo repo.
- [ ] Finish at least three polished SVG styles (`stamp`, `classic`, `terminal`).
- [ ] Publish Instant Badge gallery with working Copy Markdown.
- [ ] Ensure all outgoing links use actual final deployment/repository URLs.
- [ ] Create a GitHub social preview image.
- [ ] Record a short, silent onboarding/demo GIF or video.
- [ ] Add license, security/disclaimer copy, CONTRIBUTING, and issue templates.
- [ ] Verify badge rendering in real light/dark GitHub READMEs.

**Exit criteria:** a stranger can get a badge in under a minute, and a technical user can generate a Code Map/Markdown report from a sample repo.

### Days 4–7: Seed with authentic examples

- [ ] Add badges to 3–5 real repositories under the maintainer's control, where fitting.
- [ ] Use multiple styles so the gallery looks diverse.
- [ ] Ask a small group of developer acquaintances for usability feedback (not coordinated stars/votes).
- [ ] Observe where setup fails or documentation is unclear.
- [ ] Fix highest-friction issues before public amplification.
- [ ] Capture 1–2 genuine screenshots of badges in real README contexts.

**Exit criteria:** badge URLs, click-through, and quick start work in repos besides the tool's own repo.

### Week 2: Public introduction

- [ ] Publish Show HN (once the tool is usable, following current guidelines).
- [ ] Share the demo on Bluesky/Mastodon.
- [ ] Share in suitable Reddit project/feedback threads under their current rules.
- [ ] Respond to issues/comments and distinguish misunderstandings about "AI verification".
- [ ] Track questions and friction patterns; prioritize fixes, not vanity features.

**Exit criteria:** first public wave has a clear message and maintainers can adopt without help.

### Week 3: Follow-up and community contributions

- [ ] Publish a technical build-story article.
- [ ] Ship the remaining badge designs if quality-ready (`pixel`, `minimal`, `coffee`).
- [ ] Improve gallery accessibility and mobile experience from feedback.
- [ ] Add an `examples/` collection linking to public opt-in uses, with permission where appropriate.
- [ ] Label manageable first contributions (`good first issue`) after documenting them.

**Exit criteria:** the project has more than a one-day launch event and a clear contribution path.

### Week 4: Second wave and review

- [ ] Share Code Map demo and meaningful improvements, not an identical repeat announcement.
- [ ] Decide whether a Product Hunt launch would add meaningful reach.
- [ ] Review conversion and adoption metrics.
- [ ] Fix the highest-impact onboarding or link reliability issues.
- [ ] Decide what stays small vs. what merits a future release.

**Exit criteria:** documented lessons, real usage examples, prioritized next steps.

---

## 8. Success metrics and ethical measurement

### 8.1 Primary metric

**Independent repositories displaying the badge** (maintainer-controlled, public, counted once per repository).

Do not imply that a simple count can automatically be determined accurately from GitHub search. Search indexing, URL variations, copies, and private repositories make it incomplete. Manual/periodic sampling or clearly disclosed opt-in submissions are acceptable.

### 8.2 Initial experimental goals (not forecasts)

| Measure | First 30-day target | Rationale |
|---|---:|---|
| GitHub stars | 100+ | Discovery signal |
| Independent repos using badge | 25+ | **Adoption** signal |
| External pull requests/contributions | 3+ | Community signal |

Interpret failures as learning opportunities, not evidence the tool must become a monetized business.

### 8.3 Supporting funnel questions

- Do people understand what a badge means before copying it?
- How many can copy a badge without asking for help?
- Which styles attract voluntary adoption?
- How often do users want the report/Code Map versus a simple badge?
- Which setup step causes users to abandon the CLI?
- Do maintainers leave the badge in their READMEs after the initial experiment?

### 8.4 Measurement constraints

- Keep usage private and default to **no telemetry**.
- GitHub traffic analytics may offer aggregate repo visit/referrer data; availability and retention are platform-dependent.
- A public opt-in showcase issue/discussion can collect examples but is not a complete census.
- Avoid adding trackers to README badges or using badge request logs to profile developers.
- If adding optional anonymous gallery analytics in a later version, disclose clearly, respect consent/privacy requirements, and make a separate decision rather than silently including it.

---

## 9. Shareable story and sample launch copy

### 9.1 Core origin story

> In a world where AI can write thousands of lines of code in seconds, I built a tiny badge for those lines we still write ourselves.
>
> It doesn't detect AI. It doesn't prove anything. It's just a little stamp of pride.
>
> **Written by Hand.**

This story is intentionally concise and non-confrontational.

### 9.2 Short GitHub/project description

> A tiny, self-declared GitHub badge for handwritten code. Pick a badge instantly, or use the Rust CLI to mark files and generate an explorable Code Map. No AI detection. No backend.

### 9.3 Show HN submission draft

**Title:** Show HN: Written by Hand – A tiny badge for human-written code

**First comment:**

> I made Written by Hand as a small developer-culture joke. You can copy a badge straight into a GitHub README, or use the Rust CLI to mark files/blocks you wrote yourself and generate a report or Code Map.
>
> It doesn't detect AI and it doesn't prove code provenance. The declarations are voluntary — that's the point.
>
> I wanted it to be tiny: static SVG badges, an offline native CLI, and an optional static viewer. I'd particularly appreciate feedback on the badge designs and whether the setup is as simple as it should be.

Before publishing, remove any claims about features not yet implemented.

### 9.4 Demo script (silent, ~10 seconds)

1. Frame 1 (2 sec): Gallery with six badge tiles, Rubber Stamp selected.
2. Frame 2 (2 sec): "Copy Markdown" action succeeds.
3. Frame 3 (2 sec): Code snippet pasted in a GitHub README preview.
4. Frame 4 (2 sec): README shows the badge.
5. Frame 5 (2 sec): Click through to example `REPORT.md` or Code Map, with a note that this example uses the CLI.

Separate the Instant and Advanced flows visually so viewers do not think the instant badge computes percentages.

---

## 10. Suggested implementation workstream for growth features

This section converts marketing recommendations into discrete engineering tasks.

### G0 — Prerequisites from primary implementation plan

- [ ] CLI can scan and generate a correct report.
- [ ] At least 3 SVG templates produce valid assets.
- [ ] README insertion is correct and idempotent.
- [ ] Example repository demonstrates the Advanced flow.
- [ ] CI/tests confirm deterministic outputs.

### G1 — Zero-install Instant Badge

- [ ] Define a static badge manifest (`style`, `label variant`, `relative SVG path`, accessible label).
- [ ] Generate/publish actual SVGs for every manifest entry.
- [ ] Build single-page responsive gallery.
- [ ] Add preview selection and Copy Markdown interaction.
- [ ] Add copy fallback and test keyboard navigation.
- [ ] Configure production base URL in one place.
- [ ] Confirm all badge URLs remain stable across site redeployments.

### G2 — README and developer onboarding

- [ ] Restructure project README around Instant and Advanced choices.
- [ ] Add one concise CLI quick-start block.
- [ ] Create a demo repo or checked-in fixture with source markers, badge, and report.
- [ ] Add clear "self-declared" explanation near percentages.
- [ ] Create project social preview and gallery/favicon assets.
- [ ] Add contribution guide, issue labels/templates, and license.

### G3 — Advanced Code Map sharing

- [ ] Implement static viewer from the generated report schema as described in the primary spec.
- [ ] Show marked vs unmarked without attributing unmarked code to AI.
- [ ] Add "Open on GitHub" file/line links where repository URL/revision is known.
- [ ] Offer an unobtrusive optional "Get this badge" link to the gallery.
- [ ] Document GitHub Pages publishing; keep all default operations local/offline.

### G4 — Release communication assets

- [ ] Generate at least one silent GIF/video showing Instant use.
- [ ] Generate a second demo showing CLI/report click-through.
- [ ] Prepare the Show HN, social post, Reddit discussion, and technical article drafts above.
- [ ] Prepare screenshots for both GitHub themes and mobile gallery view.
- [ ] Confirm production URLs, license, download links, and commands.
- [ ] Conduct a human review before each external post.

### G5 — Feedback and iteration

- [ ] Make it easy to report broken SVGs and unsupported language markers.
- [ ] Maintain a small public CHANGELOG.
- [ ] Keep first-time setup friction bugs ahead of cosmetic feature requests.
- [ ] Document new requests without growing into an unrelated platform.

---

## 11. Launch gate / final QA checklist

**No public amplification until these are true:**

- [ ] A user can paste a gallery-provided badge into a real GitHub README and see the SVG render.
- [ ] The same badge links to a real, public destination.
- [ ] Gallery selection, copy, and mobile rendering work.
- [ ] CLI Advanced flow works from a clean repository checkout using documented commands.
- [ ] All percentage claims reflect marker coverage, not AI attribution or proof of authorship.
- [ ] The README and landing page clearly distinguish Instant from Advanced.
- [ ] The `wbh` binary is downloadable for the release's advertised platforms.
- [ ] No sample hostnames, missing SVG variants, broken paths, or placeholder screenshots remain in user-facing assets.
- [ ] Basic keyboard accessibility and readable contrast are validated.
- [ ] Project README, license, contribution docs, and demo examples are present.
- [ ] At least one external tester has followed the instructions without private help, or equivalent clean-room testing has been performed and clearly documented.
- [ ] Any launch copy accurately describes what is **currently shipped**, not just planned.

---

## 12. What we are intentionally not building

Do not convert this into a generic SaaS, badge authority, AI detector, or social platform.

Out of scope for the launch:

- Accounts, sign-in, OAuth, database, subscriptions.
- Hosted code scanning or source code uploads.
- AI-generated code classification, plagiarism detection, provenance certification.
- Tracking pixels or telemetry embedded in SVGs.
- Badge rankings/leaderboards or competitive human-vs-AI scoring.
- Automated social posting, spammy link insertion, or coordinated voting.
- Paid upgrades and commercial upsell.
- A heavyweight frontend toolchain when static HTML/CSS/JS is sufficient.

---

## 13. Release sequencing and priority decisions

| Milestone | Deliverable | Shipping criteria |
|---|---|---|
| **R1: Core tool** | Rust CLI, marker scan, badge, report | Primary implementation spec `v0.1` complete |
| **R2: Instant adoption** | Static badge gallery + copyable Markdown | New user can adopt without installing anything |
| **R3: Visual polish** | Six styles, social preview, demo assets | Badge presentation feels intentional and coherent |
| **R4: Code Map** | Optional static interactive viewer | Deeper exploration works with no backend |
| **R5: Public launch** | Show HN/social/community publication | R1–R3 stable; R4 only if ready |
| **R6: Iterate** | Docs, fixes, examples, contributions | Based on real user friction |

**Important sequencing note:** The Code Map is a differentiator, but do not block the first public release on it. A reliable CLI plus a zero-install gallery creates a working adoption loop sooner.

---

## 14. Final strategic decision

**Build Written by Hand as a badge people want to put in their READMEs, with a small Rust tool for those who want the deeper code map.**

This order reverses the typical developer-tool marketing approach:

- **Discovery starts with the badge**, not a command-line installer.
- **Adoption begins with copy/paste**, not a compilation environment.
- **Advanced functionality lives in `wbh`**, not in a cloud service.
- **Distribution happens organically** when badges appear in public repositories.
- **Success is measured in real repo adoption**, not just GitHub stars.

### Proposed early challenge

> **Get Written by Hand badges into 100 GitHub repositories.**

Treat this as a community aspiration, not a promise or a reason to pressure others into using the project.

---

## 15. Suggested next prompt to the coding agent

> Read `WRITTEN_BY_HAND_IMPLEMENTATION_PLAN.md` and this companion document, `WRITTEN_BY_HAND_GROWTH_LAUNCH_PLAN.md`. Implement the core Rust CLI first according to the primary spec. Then implement G1 (the zero-install static Instant Badge gallery) using the same badge artwork. Ensure Instant badges do not display fake computed percentages, copied Markdown points to real stable SVG URLs, the site remains backend-free, and Advanced badges keep linking to repository reports. Work milestone by milestone, run tests, and report what is implemented versus planned. Do not publish social posts automatically.

