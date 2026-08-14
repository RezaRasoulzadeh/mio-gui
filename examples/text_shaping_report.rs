// text_shaping_report.rs

use mio_gui::{TextDirection, TextStyle, TextSystem};

struct Fixture {
    name: &'static str,
    text: &'static str,
    direction: TextDirection,
}

fn main() {
    let fixtures = [
        Fixture {
            name: "persian",
            text: "رابط کاربری راست‌به‌چپ",
            direction: TextDirection::Auto,
        },
        Fixture {
            name: "arabic",
            text: "واجهة مستخدم من اليمين إلى اليسار",
            direction: TextDirection::Auto,
        },
        Fixture {
            name: "mixed-persian",
            text: "نسخه Mio-GUI 2.0 (آماده)",
            direction: TextDirection::Auto,
        },
        Fixture {
            name: "mixed-arabic",
            text: "الإصدار Mio-GUI 2.0 (جاهز)",
            direction: TextDirection::Auto,
        },
        Fixture {
            name: "forced-ltr",
            text: "Mio-GUI نسخه 2",
            direction: TextDirection::Ltr,
        },
        Fixture {
            name: "forced-rtl",
            text: "Mio-GUI نسخه 2",
            direction: TextDirection::Rtl,
        },
    ];
    let mut text_system = TextSystem::new();

    println!("fixture\tline_rtl\twidth\tstart\tend\tglyph\tglyph_rtl\tx\tadvance\tsource");
    for fixture in fixtures {
        let line = text_system.shape_line_with_style(
            fixture.text,
            &TextStyle {
                font_size: 20.0,
                line_height: 28.0,
                direction: fixture.direction,
                ..TextStyle::default()
            },
        );
        for glyph in line.glyphs {
            println!(
                "{}\t{}\t{:.3}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:?}",
                fixture.name,
                line.rtl,
                line.width,
                glyph.start,
                glyph.end,
                glyph.glyph_id,
                glyph.rtl,
                glyph.x,
                glyph.width,
                &fixture.text[glyph.start..glyph.end]
            );
        }
    }
}
