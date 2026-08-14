# Text reference testing

This gate compares Mio-GUI shaping with an independent, known-good Unicode text implementation. It is intentionally separate from the automated corpus tests: those tests detect regressions, while this procedure detects a consistently wrong implementation.

## Produce the Mio-GUI report

Run:

```text
cargo run --example text_shaping_report > mio-shaping.tsv
```

The report is headless and deterministic when the bundled fonts are unchanged. Each row records the fixture, paragraph direction, source byte range, glyph identifier, glyph direction, visual position, advance, and source cluster.

## Reference comparison

Use the same bundled `Vazirmatn-Regular.ttf`, font size 20, and the exact fixture strings from `examples/text_shaping_report.rs` in a current browser or another HarfBuzz-based reference application.

For every fixture, verify:

- Joining forms are correct at the start, middle, and end of Persian and Arabic words.
- Combining marks remain attached to their source grapheme.
- Latin runs and decimal numbers retain their internal LTR order.
- Parentheses and other paired punctuation appear on the correct visual side and mirror when required.
- Automatic paragraph direction follows the first strong character.
- Forced LTR and forced RTL change paragraph placement without changing source byte ranges.
- No tofu, detached marks, overlapping glyphs, or unexpected gaps appear.

Record the operating system, reference application and version, font checksum, scale factor, and result in this document before closing the roadmap gate.

## Recorded comparisons

### Ubuntu and Google Chrome

- Date: 2026-08-14
- Operating system: Ubuntu Linux
- Reference: Google Chrome 150.0.7871.124 using its HarfBuzz-based browser text stack
- Font: bundled `Vazirmatn-Regular.ttf`
- Font SHA-256: `443e920a022a89a93d4764a06e853342b9008fba880eb827d814160f1e459c05`
- CSS and Mio-GUI font size: 20 logical pixels
- Scale factor: 1
- Reference fixture: `tests/reference_text.html`

| Fixture | Mio-GUI width | Chrome width | Difference |
| --- | ---: | ---: | ---: |
| Persian | 182.959 | 191.380 | 4.40% |
| Arabic | 291.562 | 286.760 | 1.67% |
| Mixed Persian | 219.971 | 212.367 | 3.58% |
| Mixed Arabic | 228.672 | 223.067 | 2.51% |
| Forced LTR | 143.662 | 140.387 | 2.33% |
| Forced RTL | 143.662 | 140.387 | 2.33% |

Result: pass. Contextual joining, combining-mark attachment, Latin and numeric run ordering, paired punctuation, automatic paragraph direction, and forced paragraph directions matched visually. Neither implementation produced tofu, detached marks, overlaps, or unexpected gaps. The advance differences above are retained as explicit cross-engine measurements rather than treated as pixel-identical output. Forced LTR and RTL produced identical widths within each engine.
