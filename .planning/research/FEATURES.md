# Features Research: Macro/Text Expander Apps

**Domain:** Macro playback / text expansion utilities
**Researched:** 2026-01-16
**Focus:** Features for KeyBlast (hotkey-triggered keystroke injection)

## Comparable Products

Survey of the landscape to understand feature expectations.

### Direct Competitors (Hotkey-Triggered)

| Product | Platform | Trigger Type | Pricing | Notes |
|---------|----------|--------------|---------|-------|
| **AutoHotkey** | Windows | Hotkeys + abbreviations | Free/OSS | Scripting-focused, steep learning curve |
| **Keyboard Maestro** | macOS | Hotkeys + abbreviations + palettes | $36 one-time | Feature-rich, complex UI |
| **Beeftext** | Windows | Abbreviations | Free/OSS | Lightweight, system tray, simple |
| **FastKeys** | Windows | Hotkeys + abbreviations | $20 one-time | Text expander + automation combined |

### Text Expanders (Abbreviation-Triggered)

| Product | Platform | Trigger Type | Pricing | Notes |
|---------|----------|--------------|---------|-------|
| **TextExpander** | Mac/Win/iOS | Abbreviations | $4.16/mo subscription | Team features, cloud sync |
| **Espanso** | Mac/Win/Linux | Abbreviations | Free/OSS (Rust) | YAML config, shell scripts, no GUI |
| **aText** | Mac/Win | Abbreviations | $5 one-time | Simple, affordable |
| **TypeIt4Me** | macOS | Abbreviations | $20 one-time | Original Mac text expander (since 1989) |
| **PhraseExpress** | Mac/Win | Abbreviations | Freemium | Feature-heavy, portable mode |

### Key Observations

1. **Trigger dichotomy:** Most tools use either hotkeys (Ctrl+Alt+X) OR abbreviations (type `:sig` and it expands). KeyBlast uses hotkeys only, which is the less common approach.

2. **Abbreviation expanders dominate:** The market is heavily tilted toward abbreviation-triggered tools. Hotkey-only tools like KeyBlast are rarer.

3. **Complexity gradient:** Tools range from "edit a YAML file" (Espanso) to "full GUI with scripting" (Keyboard Maestro). KeyBlast targets the simple end.

4. **System tray is standard:** All lightweight tools live in the system tray. This is expected behavior.

---

## Table Stakes

Features users expect from any macro/text injection tool. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Global hotkey registration** | Core mechanic - must work across all apps | Medium | Platform-specific APIs required |
| **Reliable keystroke injection** | If text doesn't appear, tool is useless | Medium | Must work in terminal, browser, desktop apps |
| **Persistent configuration** | Macros must survive restarts | Low | File-based config is standard |
| **System tray presence** | Invisible until needed, always accessible | Low | Expected for this category |
| **Create/edit/delete macros** | Basic CRUD for macros | Low | Via menu or config file |
| **Hotkey conflict detection** | Warn when assigning already-used hotkey | Low | Prevents frustration and confusion |
| **Visual feedback on trigger** | User needs to know macro fired | Low | Subtle tray icon change or sound |
| **Enable/disable toggle** | Temporarily pause all macros | Low | Prevents interference during certain tasks |

### Rationale

These are non-negotiable because:
- Without global hotkeys, tool is useless
- Without persistence, users lose work on restart
- Without CRUD, users cannot customize
- Without conflict detection, users waste time debugging

---

## Nice-to-Have (Differentiators)

Features that add value but are not critical for a v1 release. Consider for future versions.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Macro groups/categories** | Organization for 15-50 macros | Low | Already in PROJECT.md scope |
| **Per-macro delay settings** | Different use cases need different speeds | Low | Already in PROJECT.md scope |
| **Import/export config** | Backup, share, migrate | Low | JSON/YAML export |
| **Search/filter macros** | Find macros quickly in large collections | Medium | Useful at 30+ macros |
| **Auto-start at login** | Always available without manual launch | Low | Already in PROJECT.md scope |
| **Clipboard preservation** | Don't clobber user's clipboard | Low | Only relevant if using clipboard method |
| **Pause character in sequences** | Insert deliberate delays mid-sequence | Low | `{PAUSE:500}` syntax |
| **Unicode/emoji support** | Modern text includes emoji | Low | Platform-dependent |

### Differentiators for KeyBlast Specifically

Given KeyBlast's positioning (hotkey-triggered, not abbreviation-triggered):

| Feature | Why Differentiating | Complexity |
|---------|---------------------|------------|
| **Hotkey-only triggers** | Simpler mental model than abbreviations | N/A (core design) |
| **No learning curve** | Press hotkey, get text - no syntax to learn | N/A (core design) |
| **No clipboard involvement** | Direct keystroke injection = works everywhere | Medium |
| **Offline-only** | Privacy-first, no cloud dependency | N/A (core design) |

---

## Anti-Features (Deliberate Exclusions)

Things KeyBlast should NOT build. These align with PROJECT.md scope.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Abbreviation triggers** | Different product category, adds complexity | Stick to hotkey-only triggers |
| **Modifier combos as output** | Platform-specific edge cases, security concerns | Plain text + special keys only |
| **Mouse automation** | Scope creep, different tool category | Keyboard-only focus |
| **Macro recording** | Adds significant complexity, error-prone | Manual configuration only |
| **Scripting/conditionals** | Transforms into programming tool | Simple sequence playback |
| **Cloud sync** | Adds network dependency, privacy concerns | Local config file only |
| **Team/sharing features** | Enterprise feature, adds complexity | Personal tool focus |
| **Rich text/images** | Keystroke injection is plain text by nature | Plain text only |
| **Fill-in forms/prompts** | Adds UI complexity | Static sequences only |
| **App-specific hotkeys** | Adds configuration complexity | Global hotkeys only |

### Rationale for Exclusions

1. **Abbreviation triggers:** This is the biggest architectural decision. Abbreviation-based expanders like Espanso listen to ALL keystrokes to detect patterns. Hotkey-based tools only listen for specific key combos. The latter is simpler, more predictable, and requires less invasive system access.

2. **Scripting:** The moment you add conditionals or variables, you need an execution engine, error handling, a syntax to learn, and documentation. KeyBlast is "press hotkey, get text" - no learning required.

3. **Cloud sync:** Every comparable tool that went subscription-based (TextExpander) faced user backlash. Offline-only is a feature, not a limitation.

---

## Feature Complexity Assessment

Summary of all features with complexity ratings.

### Simple (1-2 days implementation)

| Feature | Rationale |
|---------|-----------|
| Persistent config file | Standard file I/O |
| System tray presence | Platform crates exist |
| Create/edit/delete macros | CRUD operations |
| Enable/disable toggle | Boolean flag |
| Macro groups/categories | Data structure change |
| Import/export config | Serialize/deserialize |
| Auto-start at login | Platform-specific but well-documented |

### Medium (3-5 days implementation)

| Feature | Rationale |
|---------|-----------|
| Global hotkey registration | Platform-specific APIs, needs testing |
| Keystroke injection | Platform-specific, timing-sensitive |
| Hotkey conflict detection | Query system for registered hotkeys |
| Visual feedback on trigger | Tray icon animation or notification |
| Per-macro delay settings | Timing logic in playback |
| Search/filter macros | UI addition (menu-based search) |

### Complex (1+ weeks implementation)

| Feature | Rationale | Recommendation |
|---------|-----------|----------------|
| Cross-platform support | Different APIs for Mac vs Windows | Phase 2+ |
| Abbreviation triggers | Requires keylogger-style input monitoring | Do not build |
| Scripting engine | Parser, runtime, error handling | Do not build |
| GUI config window | Full window management | Out of scope per PROJECT.md |

---

## MVP Feature Recommendations

Based on research, the MVP (v1.0) should include:

### Must Have (Table Stakes)
1. Global hotkey registration
2. Keystroke injection (plain text + special keys)
3. Persistent configuration (load on startup, save on change)
4. System tray presence with menu
5. Create/edit/delete macros via menu
6. Hotkey conflict detection
7. Enable/disable toggle

### Should Have (Improves UX significantly)
1. Macro groups/categories (needed for 15-50 macros)
2. Per-macro delay settings
3. Visual feedback on trigger (tray icon flash)

### Could Have (Defer to v1.1+)
1. Import/export configuration
2. Auto-start at login
3. Search within macro list

### Will Not Have (By design)
- Abbreviation triggers
- Modifier key output
- Mouse automation
- Recording
- Scripting
- Cloud anything

---

## Sources

### Comparable Products
- [Ergonis: Top 9 Text Expanders in 2025](https://ergonis.com/blog/top-9-text-expanders-2025)
- [TheSweetBits: Best Text Expanders for Mac 2025](https://thesweetbits.com/best-text-expansion-mac/)
- [AlternativeTo: Espanso Alternatives](https://alternativeto.net/software/espanso/)
- [Blaze: TextExpander Alternatives](https://blaze.today/blog/text_expander_alternatives/)
- [TextExpander Official](https://textexpander.com/)
- [Espanso Official](https://espanso.org/)
- [Espanso GitHub](https://github.com/espanso/espanso)

### Feature Research
- [TextExpander Blog: Typing Efficiency Tools](https://textexpander.com/blog/typing-efficiency-tools)
- [TextExpander Blog: Best Hotkeys Software](https://textexpander.com/blog/best-hotkeys-software)
- [NC Bar Association: Text Expanders for Efficiency](https://www.ncbar.org/2025/05/12/text-expanders-improving-efficiency-with-automation/)
- [GetMagical: What is a Text Expander](https://www.getmagical.com/blog/what-is-a-text-expander)
- [Slashit: Best Free Text Expander Tools 2025](https://www.slashit.app/blog/best-free-text-expander-tools-in-2025)

### Technical Research
- [MakeUseOf: Beeftext Open Source App](https://www.makeuseof.com/save-hours-of-typing-on-windows-with-tiny-open-source-app/)
- [Tom's Hardware: Hotkey Conflict Resolution](https://www.tomshardware.com/software/windows/how-to-resolve-hotkey-conflicts-in-windows)
- [BetaNews: PowerToys Shortcut Conflict Detection](https://betanews.com/article/microsoft-powertoys-0-94-adds-shortcut-conflict-detection-fuzzy-search-and-more/)
- [Keyboard Maestro Forum: Clipboard vs Typing](https://forum.keyboardmaestro.com/t/in-which-situations-would-you-use-a-named-clipboard-as-opposed-to-a-sophisticated-text-expander-like-typinator/18128)
- [TextExpander: Import/Export](https://textexpander.com/blog/textexpander-import-export)
- [Beeftext: Import and Export](https://github.com/xmichelo/Beeftext/wiki/Import-and-export)

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Table stakes features | HIGH | Consistent across all surveyed tools |
| Complexity estimates | MEDIUM | Based on similar projects, not KeyBlast-specific |
| Anti-features rationale | HIGH | Aligns with PROJECT.md, well-reasoned |
| Differentiator value | MEDIUM | Hotkey-only is less common, market validation unclear |

---

*Research conducted for KeyBlast roadmap planning. Features aligned with PROJECT.md constraints.*
