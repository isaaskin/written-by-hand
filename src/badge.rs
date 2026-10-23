//! Self-contained SVG badges. No fonts, images, scripts or external references;
//! text widths are estimated so labels aren't clipped.

use crate::config::{BadgeConfig, BadgeStyle};
use crate::scan::{Stats, format_pct, tenths};

const SANS: &str = "Verdana,Geneva,'DejaVu Sans',sans-serif";
const MONO: &str = "ui-monospace,SFMono-Regular,Menlo,Consolas,'DejaVu Sans Mono',monospace";
const STAMP_FONT: &str = "'Courier New',Courier,'Nimbus Mono PS',monospace";
const UI: &str = "-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif";
const SERIF: &str = "Georgia,'Times New Roman',serif";
const DESC: &str = "Self-declared handwritten coverage, reported by the project maintainer. Not verified, and not AI detection.";

pub fn render(cfg: &BadgeConfig, stats: &Stats) -> String {
    let pct = format_pct(tenths(stats.marked_lines, stats.eligible_lines));
    let value = cfg.show_percentage.then_some(pct.as_str());
    let title = match value {
        Some(v) => format!("{}: {v}", cfg.text),
        None => cfg.text.clone(),
    };
    let (w, h, body) = match cfg.style {
        BadgeStyle::Stamp => stamp(&cfg.text, value, cfg.compact),
        BadgeStyle::Classic => classic(&cfg.text, value, cfg.compact),
        BadgeStyle::Terminal => terminal(&cfg.text, value, cfg.compact),
        BadgeStyle::Pixel => pixel(&cfg.text, value, cfg.compact),
        BadgeStyle::Minimal => minimal(&cfg.text, value, cfg.compact),
        BadgeStyle::Coffee => coffee(&cfg.text, value, cfg.compact),
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" role=\"img\" aria-labelledby=\"wbh-title wbh-desc\">\n<title id=\"wbh-title\">{}</title>\n<desc id=\"wbh-desc\">{DESC}</desc>\n{body}</svg>\n",
        esc(&title)
    )
}

/// Rubber stamp: paper, ink, double border with a slightly broken inner line.
fn stamp(label: &str, value: Option<&str>, compact: bool) -> (u32, u32, String) {
    let (fs, h, pad, spacing) = if compact {
        (10.0, 20, 8.0, 1.0)
    } else {
        (13.0, 34, 15.0, 1.5)
    };
    let text = match value {
        Some(v) => format!("{} · {v}", label.to_uppercase()),
        None => label.to_uppercase(),
    };
    let w = (mono_width(&text, fs, spacing) + 2.0 * pad).ceil() as u32;
    let ink = "#a8432a";
    let mut b = format!(
        "<rect x=\"1\" y=\"1\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"#f7f0e1\" stroke=\"{ink}\" stroke-width=\"2\"/>\n",
        w - 2,
        h - 2
    );
    if !compact {
        b += &format!(
            "<rect x=\"4.5\" y=\"4.5\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"none\" stroke=\"{ink}\" stroke-width=\"1\" stroke-dasharray=\"9 1.5 4 1.5\"/>\n",
            w - 9,
            h - 9
        );
    }
    b += &format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"{STAMP_FONT}\" font-size=\"{fs}\" font-weight=\"bold\" letter-spacing=\"{spacing}\" fill=\"{ink}\">{}</text>\n",
        fmt(w as f64 / 2.0),
        baseline(h, fs),
        esc(&text)
    );
    (w, h, b)
}

/// Two-piece shield like the badges GitHub READMEs already use.
fn classic(label: &str, value: Option<&str>, compact: bool) -> (u32, u32, String) {
    let (fs, h, pad) = if compact {
        (10.0, 18, 5.0)
    } else {
        (11.0, 20, 6.0)
    };
    let lw = (sans_width(label, fs) + 2.0 * pad).ceil() as u32;
    let vw = value.map_or(0, |v| (sans_width(v, fs) + 2.0 * pad).ceil() as u32);
    let w = lw + vw;
    let y = baseline(h, fs);
    let mut b = format!(
        "<clipPath id=\"wbh-r\"><rect width=\"{w}\" height=\"{h}\" rx=\"3\"/></clipPath>\n<g clip-path=\"url(#wbh-r)\">\n<rect width=\"{lw}\" height=\"{h}\" fill=\"#555\"/>\n"
    );
    if vw > 0 {
        b += &format!("<rect x=\"{lw}\" width=\"{vw}\" height=\"{h}\" fill=\"#b5532c\"/>\n");
    }
    b += &format!(
        "</g>\n<g fill=\"#fff\" text-anchor=\"middle\" font-family=\"{SANS}\" font-size=\"{fs}\">\n<text x=\"{}\" y=\"{y}\">{}</text>\n",
        fmt(lw as f64 / 2.0),
        esc(label)
    );
    if let Some(v) = value {
        b += &format!(
            "<text x=\"{}\" y=\"{y}\">{}</text>\n",
            fmt(lw as f64 + vw as f64 / 2.0),
            esc(v)
        );
    }
    b += "</g>\n";
    (w, h, b)
}

/// Green-on-dark prompt with a block cursor.
fn terminal(label: &str, value: Option<&str>, compact: bool) -> (u32, u32, String) {
    let (fs, h, pad) = if compact {
        (10.0, 18, 6.0)
    } else {
        (11.0, 22, 8.0)
    };
    let prompt = "~$";
    let mut parts = vec![(prompt, "#6e9f74"), (label, "#7ee787")];
    if let Some(v) = value {
        parts.push((v, "#e6edf3"));
    }
    let space = mono_width(" ", fs, 0.0);
    let y = baseline(h, fs);
    let mut x = pad;
    let mut b = format!(
        "<rect width=\"{{W}}\" height=\"{h}\" rx=\"3\" fill=\"#0d1117\"/>\n<g font-family=\"{MONO}\" font-size=\"{fs}\">\n"
    );
    for (text, color) in parts {
        b += &format!(
            "<text x=\"{}\" y=\"{y}\" fill=\"{color}\">{}</text>\n",
            fmt(x),
            esc(text)
        );
        x += mono_width(text, fs, 0.0) + space;
    }
    let cursor_h = fs;
    b += &format!(
        "</g>\n<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#7ee787\" opacity=\"0.8\"/>\n",
        fmt(x),
        fmt((h as f64 - cursor_h) / 2.0),
        fmt(space),
        fmt(cursor_h)
    );
    let w = (x + space + pad).ceil() as u32;
    (w, h, b.replace("{W}", &w.to_string()))
}

/// Retro 8-bit: lettering drawn from a built-in 5×7 bitmap font, so it looks the
/// same everywhere. Falls back to monospace text for characters the font lacks.
fn pixel(label: &str, value: Option<&str>, compact: bool) -> (u32, u32, String) {
    let (p, pad) = if compact { (1.5, 5.0) } else { (2.0, 7.0) };
    let label = label.to_uppercase();
    let parts: Vec<(&str, &str)> = std::iter::once((label.as_str(), "#ffd84a"))
        .chain(value.map(|v| (v, "#7ce0ff")))
        .collect();
    let chars = parts.iter().map(|(t, _)| t.chars().count()).sum::<usize>() + parts.len() - 1;
    let all_drawable = parts
        .iter()
        .all(|(t, _)| t.chars().all(|c| glyph(c).is_some()));
    let text_w = if all_drawable {
        (chars * 6 - 1) as f64 * p
    } else {
        // Bold monospace runs ~10% wider than regular.
        let joined: Vec<&str> = parts.iter().map(|(t, _)| *t).collect();
        mono_width(&joined.join(" "), 7.0 * p, 0.0) * 1.1
    };
    let w = (text_w + 2.0 * pad).ceil() as u32;
    let h = (7.0 * p + 2.0 * pad).ceil() as u32;
    let (wf, hf) = (w as f64, h as f64);
    // Background with stepped (pixelated) corners.
    let mut b = format!(
        "<path d=\"M{p} 0H{}V{p}H{wf}V{}H{}V{hf}H{p}V{}H0V{p}H{p}Z\" fill=\"#26224f\"/>\n",
        fmt(wf - p),
        fmt(hf - p),
        fmt(wf - p),
        fmt(hf - p)
    );
    let top = (hf - 7.0 * p) / 2.0;
    let mut x = pad;
    for (text, color) in parts {
        if all_drawable {
            let mut d = String::new();
            for c in text.chars() {
                for (row, bits) in glyph(c).unwrap().iter().enumerate() {
                    let mut col = 0;
                    while col < 5 {
                        if bits.as_bytes()[col] == b'#' {
                            let start = col;
                            while col < 5 && bits.as_bytes()[col] == b'#' {
                                col += 1;
                            }
                            d += &format!(
                                "M{} {}h{}v{p}h-{}z",
                                fmt(x + start as f64 * p),
                                fmt(top + row as f64 * p),
                                fmt((col - start) as f64 * p),
                                fmt((col - start) as f64 * p)
                            );
                        }
                        col += 1;
                    }
                }
                x += 6.0 * p;
            }
            b += &format!("<path d=\"{d}\" fill=\"{color}\" shape-rendering=\"crispEdges\"/>\n");
            x += 6.0 * p;
        } else {
            let fs = 7.0 * p;
            b += &format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"{MONO}\" font-size=\"{fs}\" font-weight=\"bold\" fill=\"{color}\">{}</text>\n",
                fmt(x),
                baseline(h, fs),
                esc(text)
            );
            x += (mono_width(text, fs, 0.0) + mono_width(" ", fs, 0.0)) * 1.1;
        }
    }
    (w, h, b)
}

/// Quiet: white card, hairline border, a drawn pencil, label and a lighter number.
fn minimal(label: &str, value: Option<&str>, compact: bool) -> (u32, u32, String) {
    let (fs, h, pad, icon) = if compact {
        (10.0, 18, 6.0, 10.0)
    } else {
        (11.0, 22, 8.0, 12.0)
    };
    let gap = 5.0;
    let lw = sans_width(label, fs);
    let vw = value.map_or(0.0, |v| sans_width(v, fs) + 2.0 * gap + 1.0);
    let w = (pad + icon + gap + lw + vw + pad).ceil() as u32;
    let y = baseline(h, fs);
    let s = icon / 12.0;
    let mut b = format!(
        "<rect x=\"0.5\" y=\"0.5\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"#fff\" stroke=\"#d0d7de\"/>\n\
         <path transform=\"translate({} {}) scale({})\" d=\"M2 10l.6-2.4L8.4 1.8a.9.9 0 0 1 1.3 0l.5.5a.9.9 0 0 1 0 1.3L4.4 9.4zM7.4 2.8l1.8 1.8\" fill=\"none\" stroke=\"#57606a\" stroke-width=\"1.1\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>\n\
         <g font-family=\"{UI}\" font-size=\"{fs}\">\n<text x=\"{}\" y=\"{y}\" fill=\"#24292f\">{}</text>\n",
        w - 1,
        h - 1,
        fmt(pad),
        fmt((h as f64 - icon) / 2.0),
        fmt(s),
        fmt(pad + icon + gap),
        esc(label)
    );
    if let Some(v) = value {
        let sep = pad + icon + gap + lw + gap;
        b += &format!(
            "<text x=\"{}\" y=\"{y}\" fill=\"#57606a\">{}</text>\n",
            fmt(sep + 1.0 + gap),
            esc(v)
        );
        b += &format!(
            "</g>\n<path d=\"M{} 5V{}\" stroke=\"#d0d7de\"/>\n",
            fmt(sep + 0.5),
            h - 5
        );
    } else {
        b += "</g>\n";
    }
    (w, h, b)
}

/// Coffee-shop seal: espresso pill, cream lettering, a steaming cup.
fn coffee(label: &str, value: Option<&str>, compact: bool) -> (u32, u32, String) {
    let (fs, h, spacing, cup) = if compact {
        (9.0, 20, 0.8, 11.0)
    } else {
        (11.0, 30, 1.2, 15.0)
    };
    let hf = h as f64;
    let pad = hf / 2.0;
    let text = match value {
        Some(v) => format!("{} · {v}", label.to_uppercase()),
        None => label.to_uppercase(),
    };
    // Georgia runs ~5% narrower than the Verdana-based width table.
    let tw = sans_width(&text, fs) * 0.95 + spacing * text.chars().count() as f64;
    let w = (pad + cup + 6.0 + tw + pad).ceil() as u32;
    let cream = "#f3e3c3";
    let mut b = format!(
        "<rect width=\"{w}\" height=\"{h}\" rx=\"{}\" fill=\"#4a2c1d\"/>\n",
        fmt(pad)
    );
    if !compact {
        b += &format!(
            "<rect x=\"3\" y=\"3\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"none\" stroke=\"{cream}\" stroke-width=\"0.8\" stroke-dasharray=\"1 2\"/>\n",
            w - 6,
            h - 6,
            fmt(pad - 3.0)
        );
    }
    // Cup: body, handle, saucer and two wisps of steam, in a 15×15 box.
    let s = cup / 15.0;
    b += &format!(
        "<g transform=\"translate({} {}) scale({})\" fill=\"none\" stroke=\"{cream}\" stroke-width=\"1.3\" stroke-linecap=\"round\">\n\
         <path d=\"M2 7h8v3a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3z\" fill=\"{cream}\"/>\n\
         <path d=\"M10 8h1.2a1.6 1.6 0 0 1 0 3.2H10M1 14.2h10M4.5 1.5q1 1.2 0 2.4M7.5 1.5q1 1.2 0 2.4\"/>\n</g>\n",
        fmt(pad - 2.0),
        fmt((hf - cup) / 2.0),
        fmt(s)
    );
    b += &format!(
        "<text x=\"{}\" y=\"{}\" font-family=\"{SERIF}\" font-size=\"{fs}\" font-weight=\"bold\" letter-spacing=\"{spacing}\" fill=\"{cream}\">{}</text>\n",
        fmt(pad - 2.0 + cup + 6.0),
        baseline(h, fs),
        esc(&text)
    );
    (w, h, b)
}

/// 5×7 bitmap glyphs for the pixel style.
fn glyph(c: char) -> Option<[&'static str; 7]> {
    Some(match c {
        'A' => [
            ".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        'B' => [
            "####.", "#...#", "#...#", "####.", "#...#", "#...#", "####.",
        ],
        'C' => [
            ".###.", "#...#", "#....", "#....", "#....", "#...#", ".###.",
        ],
        'D' => [
            "####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####.",
        ],
        'E' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#####",
        ],
        'F' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#....",
        ],
        'G' => [
            ".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".####",
        ],
        'H' => [
            "#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        'I' => [
            ".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###.",
        ],
        'J' => [
            "..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##..",
        ],
        'K' => [
            "#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#",
        ],
        'L' => [
            "#....", "#....", "#....", "#....", "#....", "#....", "#####",
        ],
        'M' => [
            "#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#",
        ],
        'N' => [
            "#...#", "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#",
        ],
        'O' => [
            ".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        'P' => [
            "####.", "#...#", "#...#", "####.", "#....", "#....", "#....",
        ],
        'Q' => [
            ".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#",
        ],
        'R' => [
            "####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#",
        ],
        'S' => [
            ".####", "#....", "#....", ".###.", "....#", "....#", "####.",
        ],
        'T' => [
            "#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#..",
        ],
        'U' => [
            "#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        'V' => [
            "#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#..",
        ],
        'W' => [
            "#...#", "#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", ".#.#.",
        ],
        'X' => [
            "#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#",
        ],
        'Y' => [
            "#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#..",
        ],
        'Z' => [
            "#####", "....#", "...#.", "..#..", ".#...", "#....", "#####",
        ],
        '0' => [
            ".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###.",
        ],
        '1' => [
            "..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###.",
        ],
        '2' => [
            ".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####",
        ],
        '3' => [
            "#####", "...#.", "..#..", "...#.", "....#", "#...#", ".###.",
        ],
        '4' => [
            "...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#.",
        ],
        '5' => [
            "#####", "#....", "####.", "....#", "....#", "#...#", ".###.",
        ],
        '6' => [
            "..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###.",
        ],
        '7' => [
            "#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#...",
        ],
        '8' => [
            ".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###.",
        ],
        '9' => [
            ".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##..",
        ],
        ' ' => [
            ".....", ".....", ".....", ".....", ".....", ".....", ".....",
        ],
        '.' => [
            ".....", ".....", ".....", ".....", ".....", ".##..", ".##..",
        ],
        ',' => [
            ".....", ".....", ".....", ".....", ".##..", "..#..", ".#...",
        ],
        ':' => [
            ".....", ".##..", ".##..", ".....", ".##..", ".##..", ".....",
        ],
        '!' => [
            "..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#..",
        ],
        '?' => [
            ".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#..",
        ],
        '\'' => [
            "..#..", "..#..", ".#...", ".....", ".....", ".....", ".....",
        ],
        '-' => [
            ".....", ".....", ".....", "#####", ".....", ".....", ".....",
        ],
        '_' => [
            ".....", ".....", ".....", ".....", ".....", ".....", "#####",
        ],
        '/' => [
            "....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#....",
        ],
        '&' => [
            ".##..", "#..#.", "#.#..", ".#...", "#.#.#", "#..#.", ".##.#",
        ],
        '%' => [
            "##..#", "##..#", "...#.", "..#..", ".#...", "#..##", "#..##",
        ],
        '(' => [
            "...#.", "..#..", ".#...", ".#...", ".#...", "..#..", "...#.",
        ],
        ')' => [
            ".#...", "..#..", "...#.", "...#.", "...#.", "..#..", ".#...",
        ],
        '+' => [
            ".....", "..#..", "..#..", "#####", "..#..", "..#..", ".....",
        ],
        '#' => [
            ".#.#.", ".#.#.", "#####", ".#.#.", "#####", ".#.#.", ".#.#.",
        ],
        '*' => [
            ".....", "..#..", "#.#.#", ".###.", "#.#.#", "..#..", ".....",
        ],
        '@' => [
            ".###.", "#...#", "#.###", "#.#.#", "#.###", "#....", ".####",
        ],
        '=' => [
            ".....", ".....", "#####", ".....", "#####", ".....", ".....",
        ],
        _ => return None,
    })
}

/// Estimated advance width for a proportional sans font, in px.
fn sans_width(s: &str, fs: f64) -> f64 {
    s.chars()
        .map(|c| match c {
            'i' | 'l' | 'I' | '|' | '.' | ',' | ':' | ';' | '!' | '\'' => 0.35,
            'f' | 'j' | 'r' | 't' | '(' | ')' | '[' | ']' | ' ' | '-' => 0.45,
            'm' | 'w' | 'M' | 'W' | '%' | '@' => 1.0,
            'A'..='Z' | '0'..='9' => 0.72,
            c if c.is_ascii() => 0.62,
            // Non-ASCII may be CJK or emoji: assume full width so nothing clips.
            _ => 1.0,
        })
        .sum::<f64>()
        * fs
}

fn mono_width(s: &str, fs: f64, spacing: f64) -> f64 {
    s.chars()
        .map(|c| if c.is_ascii() { 0.6 * fs } else { fs } + spacing)
        .sum()
}

fn baseline(h: u32, fs: f64) -> String {
    fmt(h as f64 / 2.0 + fs * 0.35)
}

/// One decimal place, without a trailing `.0`, for stable compact output.
fn fmt(v: f64) -> String {
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(marked: usize, eligible: usize) -> Stats {
        Stats {
            marked_lines: marked,
            eligible_lines: eligible,
            ..Stats::default()
        }
    }

    /// Minimal well-formedness check: tags balance and attributes are quoted.
    fn assert_well_formed(svg: &str) {
        let mut stack = Vec::new();
        let mut rest = svg;
        while let Some(i) = rest.find('<') {
            let end = rest[i..].find('>').expect("unclosed tag") + i;
            let tag = &rest[i + 1..end];
            if let Some(name) = tag.strip_prefix('/') {
                assert_eq!(stack.pop(), Some(name.to_string()), "{svg}");
            } else if !tag.ends_with('/') {
                stack.push(tag.split_whitespace().next().unwrap().to_string());
            }
            assert_eq!(tag.matches('"').count() % 2, 0, "{tag}");
            rest = &rest[end + 1..];
        }
        assert!(stack.is_empty(), "unclosed: {stack:?}");
    }

    #[test]
    fn all_styles_and_variants_render_safely() {
        let evil = "<script>alert(\"x\")</script> & 'é' 漢字";
        for style in BadgeStyle::ALL {
            for compact in [false, true] {
                for show_percentage in [false, true] {
                    for text in ["Written by Hand", evil, "x", &"W".repeat(60)] {
                        for (m, e) in [(0, 0), (0, 10), (51, 200), (10, 10)] {
                            let cfg = BadgeConfig {
                                style,
                                compact,
                                show_percentage,
                                text: text.into(),
                            };
                            let svg = render(&cfg, &stats(m, e));
                            assert_well_formed(&svg);
                            assert!(!svg.contains("<script"), "{svg}");
                            assert_eq!(svg.matches("http").count(), 1, "only the xmlns: {svg}");
                            assert!(svg.contains("<title") && svg.contains("<desc"));
                            if e == 0 && show_percentage {
                                assert!(svg.contains("N/A") && !svg.contains("100.0%"), "{svg}");
                            }
                            if (m, e) == (51, 200) && show_percentage {
                                assert!(svg.contains("25.5%"));
                            }
                            assert_eq!(svg, render(&cfg, &stats(m, e)), "deterministic");
                        }
                    }
                }
            }
        }
    }

    /// The README gallery is rendered by this renderer, not drawn by hand.
    #[test]
    fn gallery_is_current() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/badges");
        let bless = std::env::var_os("WBH_BLESS").is_some();
        for style in BadgeStyle::ALL {
            for (suffix, compact, show_percentage) in [
                ("", false, false),
                ("-percentage", false, true),
                ("-compact", true, false),
            ] {
                let cfg = BadgeConfig {
                    style,
                    compact,
                    show_percentage,
                    ..BadgeConfig::default()
                };
                let svg = render(&cfg, &stats(3, 4));
                let name = format!("{}{suffix}.svg", style.key());
                let path = dir.join(&name);
                if bless {
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(&path, &svg).unwrap();
                } else {
                    let committed = std::fs::read_to_string(&path)
                        .unwrap_or_default()
                        .replace('\r', "");
                    assert!(
                        committed == svg,
                        "docs/badges/{name} is stale; run `WBH_BLESS=1 cargo test`"
                    );
                }
            }
        }
    }

    #[test]
    fn width_grows_with_text() {
        for style in BadgeStyle::ALL {
            let width = |text: &str| {
                let cfg = BadgeConfig {
                    style,
                    text: text.into(),
                    ..BadgeConfig::default()
                };
                let svg = render(&cfg, &stats(1, 2));
                let w = svg
                    .split("width=\"")
                    .nth(1)
                    .unwrap()
                    .split('"')
                    .next()
                    .unwrap();
                w.parse::<u32>().unwrap()
            };
            assert!(width(&"W".repeat(60)) > width("Hand") + 300, "{style:?}");
        }
    }
}
