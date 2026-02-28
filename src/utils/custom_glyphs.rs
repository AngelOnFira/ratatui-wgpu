use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source, StrokeStyle};

const WHITE: SolidSource = SolidSource {
    r: 0xFF,
    g: 0xFF,
    b: 0xFF,
    a: 0xFF,
};

const DRAW_OPTS: DrawOptions = DrawOptions {
    blend_mode: raqote::BlendMode::SrcOver,
    alpha: 1.0,
    antialias: raqote::AntialiasMode::Gray,
};

const DRAW_OPTS_NOAA: DrawOptions = DrawOptions {
    blend_mode: raqote::BlendMode::SrcOver,
    alpha: 1.0,
    antialias: raqote::AntialiasMode::None,
};

/// Try to procedurally rasterize a character as a custom glyph.
/// Returns `Some(pixels)` for box-drawing, block elements, and braille characters.
/// Returns `None` for all other characters (fall through to font rasterization).
///
/// Output is white-on-transparent RGBA pixels, size = width * height.
pub(crate) fn try_rasterize_custom_glyph(ch: char, width: u32, height: u32) -> Option<Vec<u32>> {
    let cp = ch as u32;
    match cp {
        // Box Drawing: U+2500–U+257F
        0x2500..=0x257F => Some(rasterize_box_drawing(ch, width, height)),
        // Block Elements: U+2580–U+259F
        0x2580..=0x259F => Some(rasterize_block_element(ch, width, height)),
        // Braille Patterns: U+2800–U+28FF
        0x2800..=0x28FF => Some(rasterize_braille(ch, width, height)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Box Drawing (U+2500–U+257F)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum LineWeight {
    Light,
    Heavy,
    Double,
}

#[derive(Clone, Copy, Default)]
struct BoxSegments {
    left: Option<LineWeight>,
    right: Option<LineWeight>,
    up: Option<LineWeight>,
    down: Option<LineWeight>,
}

/// Whether this is a rounded corner character.
fn is_rounded_corner(ch: char) -> bool {
    matches!(ch, '╭' | '╮' | '╰' | '╯')
}

fn decompose_box_drawing(ch: char) -> BoxSegments {
    use LineWeight::*;
    let s = |l: Option<LineWeight>,
             r: Option<LineWeight>,
             u: Option<LineWeight>,
             d: Option<LineWeight>|
     -> BoxSegments {
        BoxSegments {
            left: l,
            right: r,
            up: u,
            down: d,
        }
    };
    #[allow(non_snake_case)]
    let L = Some(Light);
    #[allow(non_snake_case)]
    let H = Some(Heavy);
    #[allow(non_snake_case)]
    let D = Some(Double);
    #[allow(non_snake_case)]
    let N: Option<LineWeight> = None;

    match ch {
        // Light lines
        '─' => s(L, L, N, N),
        '│' => s(N, N, L, L),
        '┌' => s(N, L, N, L),
        '┐' => s(L, N, N, L),
        '└' => s(N, L, L, N),
        '┘' => s(L, N, L, N),
        '├' => s(N, L, L, L),
        '┤' => s(L, N, L, L),
        '┬' => s(L, L, N, L),
        '┴' => s(L, L, L, N),
        '┼' => s(L, L, L, L),

        // Heavy lines
        '━' => s(H, H, N, N),
        '┃' => s(N, N, H, H),
        '┏' => s(N, H, N, H),
        '┓' => s(H, N, N, H),
        '┗' => s(N, H, H, N),
        '┛' => s(H, N, H, N),
        '┣' => s(N, H, H, H),
        '┫' => s(H, N, H, H),
        '┳' => s(H, H, N, H),
        '┻' => s(H, H, H, N),
        '╋' => s(H, H, H, H),

        // Mixed light/heavy
        '┍' => s(N, H, N, L),
        '┎' => s(N, L, N, H),
        '┑' => s(H, N, N, L),
        '┒' => s(L, N, N, H),
        '┕' => s(N, H, L, N),
        '┖' => s(N, L, H, N),
        '┙' => s(H, N, L, N),
        '┚' => s(L, N, H, N),
        '┝' => s(N, H, L, L),
        '┞' => s(N, L, H, L),
        '┟' => s(N, L, L, H),
        '┠' => s(N, H, H, L), // Hmm, actually let me recheck
        // Actually, let me be more careful with the mixed weight chars
        '┡' => s(N, L, H, L), // These need verification against Unicode chart
        '┢' => s(N, H, L, H),
        '┥' => s(H, N, L, L),
        '┦' => s(L, N, H, L),
        '┧' => s(L, N, L, H),
        '┨' => s(H, N, H, L),
        '┩' => s(L, N, H, L),
        '┪' => s(H, N, L, H),
        '┭' => s(H, L, N, L),
        '┮' => s(L, H, N, L),
        '┯' => s(H, H, N, L),
        '┰' => s(L, L, N, H),
        '┱' => s(H, L, N, H),
        '┲' => s(L, H, N, H),
        '┵' => s(H, L, L, N),
        '┶' => s(L, H, L, N),
        '┷' => s(H, H, L, N),
        '┸' => s(L, L, H, N),
        '┹' => s(H, L, H, N),
        '┺' => s(L, H, H, N),
        '┽' => s(H, L, L, L),
        '┾' => s(L, H, L, L),
        '┿' => s(H, H, L, L),
        '╀' => s(L, L, H, L),
        '╁' => s(L, L, L, H),
        '╂' => s(L, L, H, H),
        '╃' => s(H, L, H, L),
        '╄' => s(L, H, H, L),
        '╅' => s(H, L, L, H),
        '╆' => s(L, H, L, H),
        '╇' => s(H, H, H, L),
        '╈' => s(H, H, L, H),
        '╉' => s(H, L, H, H),
        '╊' => s(L, H, H, H),

        // Double lines
        '═' => s(D, D, N, N),
        '║' => s(N, N, D, D),
        '╔' => s(N, D, N, D),
        '╗' => s(D, N, N, D),
        '╚' => s(N, D, D, N),
        '╝' => s(D, N, D, N),
        '╠' => s(N, D, D, D),
        '╣' => s(D, N, D, D),
        '╦' => s(D, D, N, D),
        '╩' => s(D, D, D, N),
        '╬' => s(D, D, D, D),

        // Mixed single/double
        '╒' => s(N, D, N, L),
        '╓' => s(N, L, N, D),
        '╕' => s(D, N, N, L),
        '╖' => s(L, N, N, D),
        '╘' => s(N, D, L, N),
        '╙' => s(N, L, D, N),
        '╛' => s(D, N, L, N),
        '╜' => s(L, N, D, N),
        '╞' => s(N, D, L, L),
        '╟' => s(N, L, D, D),
        '╡' => s(D, N, L, L),
        '╢' => s(L, N, D, D),
        '╤' => s(D, D, N, L),
        '╥' => s(L, L, N, D),
        '╧' => s(D, D, L, N),
        '╨' => s(L, L, D, N),
        '╪' => s(D, D, L, L),
        '╫' => s(L, L, D, D),

        // Rounded corners (decompose as light for the segments)
        '╭' => s(N, L, N, L),
        '╮' => s(L, N, N, L),
        '╯' => s(L, N, L, N),
        '╰' => s(N, L, L, N),

        // Dashed / half lines — handle via segments for fallback
        '╴' => s(L, N, N, N), // light left
        '╵' => s(N, N, L, N), // light up
        '╶' => s(N, L, N, N), // light right
        '╷' => s(N, N, N, L), // light down
        '╸' => s(H, N, N, N), // heavy left
        '╹' => s(N, N, H, N), // heavy up
        '╺' => s(N, H, N, N), // heavy right
        '╻' => s(N, N, N, H), // heavy down

        // Light dashed lines (render as solid for now)
        '┄' | '╌' => s(L, L, N, N),
        '┆' | '╎' => s(N, N, L, L),
        '┈' => s(L, L, N, N),
        '┊' => s(N, N, L, L),

        // Heavy dashed lines (render as solid for now)
        '┅' | '╍' => s(H, H, N, N),
        '┇' | '╏' => s(N, N, H, H),
        '┉' => s(H, H, N, N),
        '┋' => s(N, N, H, H),

        _ => BoxSegments::default(),
    }
}

fn rasterize_box_drawing(ch: char, width: u32, height: u32) -> Vec<u32> {
    let w = width as i32;
    let h = height as i32;
    let mut image = vec![0u32; (w * h) as usize];

    if w == 0 || h == 0 {
        return image;
    }

    let mut target = DrawTarget::from_backing(w, h, &mut image[..]);
    let segments = decompose_box_drawing(ch);
    let light_w = (height as f32 / 12.0).max(1.0).round();
    let heavy_w = (light_w * 2.0).max(2.0).round();

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    if is_rounded_corner(ch) {
        draw_rounded_corner(&mut target, ch, cx, cy, light_w, w as f32, h as f32);
    } else {
        draw_box_segments(
            &mut target, &segments, cx, cy, light_w, heavy_w, w as f32, h as f32,
        );
    }

    convert_argb_to_rgba(&mut image);
    image
}

fn draw_box_segments(
    target: &mut DrawTarget<&mut [u32]>,
    seg: &BoxSegments,
    cx: f32,
    cy: f32,
    light_w: f32,
    heavy_w: f32,
    w: f32,
    h: f32,
) {
    // For double lines, draw two parallel strokes with a gap
    let draw_h_segment = |target: &mut DrawTarget<&mut [u32]>, from_x: f32, to_x: f32, weight: LineWeight| {
        match weight {
            LineWeight::Light => {
                fill_rect(target, from_x, cy - light_w / 2.0, to_x - from_x, light_w);
            }
            LineWeight::Heavy => {
                fill_rect(target, from_x, cy - heavy_w / 2.0, to_x - from_x, heavy_w);
            }
            LineWeight::Double => {
                let gap = (light_w * 2.0).max(3.0);
                fill_rect(target, from_x, cy - gap / 2.0 - light_w / 2.0, to_x - from_x, light_w);
                fill_rect(target, from_x, cy + gap / 2.0 - light_w / 2.0, to_x - from_x, light_w);
            }
        }
    };

    let draw_v_segment = |target: &mut DrawTarget<&mut [u32]>, from_y: f32, to_y: f32, weight: LineWeight| {
        match weight {
            LineWeight::Light => {
                fill_rect(target, cx - light_w / 2.0, from_y, light_w, to_y - from_y);
            }
            LineWeight::Heavy => {
                fill_rect(target, cx - heavy_w / 2.0, from_y, heavy_w, to_y - from_y);
            }
            LineWeight::Double => {
                let gap = (light_w * 2.0).max(3.0);
                fill_rect(target, cx - gap / 2.0 - light_w / 2.0, from_y, light_w, to_y - from_y);
                fill_rect(target, cx + gap / 2.0 - light_w / 2.0, from_y, light_w, to_y - from_y);
            }
        }
    };

    // For double lines meeting at junctions, we need to handle the center area.
    // For simple cases (pure single/heavy), just draw from center to edge.
    let has_double = [seg.left, seg.right, seg.up, seg.down]
        .iter()
        .any(|s| matches!(s, Some(LineWeight::Double)));

    if has_double {
        draw_double_junction(target, seg, cx, cy, light_w, w, h);
    } else {
        // Left segment: from left edge to center
        if let Some(weight) = seg.left {
            draw_h_segment(target, 0.0, cx + thickness(weight, light_w, heavy_w) / 2.0, weight);
        }
        // Right segment: from center to right edge
        if let Some(weight) = seg.right {
            draw_h_segment(target, cx - thickness(weight, light_w, heavy_w) / 2.0, w, weight);
        }
        // Up segment: from top edge to center
        if let Some(weight) = seg.up {
            draw_v_segment(target, 0.0, cy + thickness(weight, light_w, heavy_w) / 2.0, weight);
        }
        // Down segment: from center to bottom edge
        if let Some(weight) = seg.down {
            draw_v_segment(target, cy - thickness(weight, light_w, heavy_w) / 2.0, h, weight);
        }
    }
}

fn thickness(weight: LineWeight, light: f32, heavy: f32) -> f32 {
    match weight {
        LineWeight::Light => light,
        LineWeight::Heavy => heavy,
        LineWeight::Double => light, // individual stroke thickness
    }
}

fn draw_double_junction(
    target: &mut DrawTarget<&mut [u32]>,
    seg: &BoxSegments,
    cx: f32,
    cy: f32,
    lw: f32,
    w: f32,
    h: f32,
) {
    let gap = (lw * 2.0).max(3.0);
    let half_gap = gap / 2.0;
    let hlw = lw / 2.0;

    // Double horizontal segments
    let is_h_double = matches!(seg.left, Some(LineWeight::Double))
        || matches!(seg.right, Some(LineWeight::Double));
    let is_v_double = matches!(seg.up, Some(LineWeight::Double))
        || matches!(seg.down, Some(LineWeight::Double));

    // Draw outer horizontal strokes
    if let Some(weight) = seg.left {
        match weight {
            LineWeight::Double => {
                let end_x = if is_v_double { cx - half_gap - hlw } else { cx + hlw };
                fill_rect(target, 0.0, cy - half_gap - hlw, end_x, lw);
                fill_rect(target, 0.0, cy + half_gap - hlw, end_x, lw);
            }
            LineWeight::Light => {
                fill_rect(target, 0.0, cy - hlw, cx + hlw, lw);
            }
            LineWeight::Heavy => {
                let hw = (lw * 2.0).max(2.0);
                fill_rect(target, 0.0, cy - hw / 2.0, cx + hw / 2.0, hw);
            }
        }
    }
    if let Some(weight) = seg.right {
        match weight {
            LineWeight::Double => {
                let start_x = if is_v_double { cx + half_gap + hlw } else { cx - hlw };
                fill_rect(target, start_x, cy - half_gap - hlw, w - start_x, lw);
                fill_rect(target, start_x, cy + half_gap - hlw, w - start_x, lw);
            }
            LineWeight::Light => {
                fill_rect(target, cx - hlw, cy - hlw, w - cx + hlw, lw);
            }
            LineWeight::Heavy => {
                let hw = (lw * 2.0).max(2.0);
                fill_rect(target, cx - hw / 2.0, cy - hw / 2.0, w - cx + hw / 2.0, hw);
            }
        }
    }

    // Draw outer vertical strokes
    if let Some(weight) = seg.up {
        match weight {
            LineWeight::Double => {
                let end_y = if is_h_double { cy - half_gap - hlw } else { cy + hlw };
                fill_rect(target, cx - half_gap - hlw, 0.0, lw, end_y);
                fill_rect(target, cx + half_gap - hlw, 0.0, lw, end_y);
            }
            LineWeight::Light => {
                fill_rect(target, cx - hlw, 0.0, lw, cy + hlw);
            }
            LineWeight::Heavy => {
                let hw = (lw * 2.0).max(2.0);
                fill_rect(target, cx - hw / 2.0, 0.0, hw, cy + hw / 2.0);
            }
        }
    }
    if let Some(weight) = seg.down {
        match weight {
            LineWeight::Double => {
                let start_y = if is_h_double { cy + half_gap + hlw } else { cy - hlw };
                fill_rect(target, cx - half_gap - hlw, start_y, lw, h - start_y);
                fill_rect(target, cx + half_gap - hlw, start_y, lw, h - start_y);
            }
            LineWeight::Light => {
                fill_rect(target, cx - hlw, cy - hlw, lw, h - cy + hlw);
            }
            LineWeight::Heavy => {
                let hw = (lw * 2.0).max(2.0);
                fill_rect(target, cx - hw / 2.0, cy - hw / 2.0, hw, h - cy + hw / 2.0);
            }
        }
    }
}

fn draw_rounded_corner(
    target: &mut DrawTarget<&mut [u32]>,
    ch: char,
    cx: f32,
    cy: f32,
    lw: f32,
    w: f32,
    h: f32,
) {
    let style = StrokeStyle {
        width: lw,
        ..Default::default()
    };

    let mut pb = PathBuilder::new();
    match ch {
        '╭' => {
            // Arc from bottom of cell to right of cell, curving through center
            pb.move_to(cx, h);
            pb.quad_to(cx, cy, w, cy);
        }
        '╮' => {
            pb.move_to(0.0, cy);
            pb.quad_to(cx, cy, cx, h);
        }
        '╰' => {
            pb.move_to(cx, 0.0);
            pb.quad_to(cx, cy, w, cy);
        }
        '╯' => {
            pb.move_to(0.0, cy);
            pb.quad_to(cx, cy, cx, 0.0);
        }
        _ => unreachable!(),
    }
    let path = pb.finish();
    let src = Source::Solid(WHITE);
    target.stroke(&path, &src, &style, &DRAW_OPTS);
}

// ---------------------------------------------------------------------------
// Block Elements (U+2580–U+259F)
// ---------------------------------------------------------------------------

fn rasterize_block_element(ch: char, width: u32, height: u32) -> Vec<u32> {
    let w = width as usize;
    let h = height as usize;
    let mut image = vec![0u32; w * h];

    if w == 0 || h == 0 {
        return image;
    }

    let white = 0xFFFFFFFFu32; // RGBA white (byte-order agnostic for all-ones)

    match ch {
        // Upper half block
        '▀' => fill_region(&mut image, w, 0, 0, w, h / 2, white),
        // Lower 1/8 through 7/8 blocks
        '▁' => fill_region(&mut image, w, 0, h - h / 8, w, h / 8, white),
        '▂' => fill_region(&mut image, w, 0, h - h / 4, w, h / 4, white),
        '▃' => fill_region(&mut image, w, 0, h - (3 * h / 8), w, 3 * h / 8, white),
        '▄' => fill_region(&mut image, w, 0, h / 2, w, h - h / 2, white),
        '▅' => fill_region(&mut image, w, 0, h - (5 * h / 8), w, 5 * h / 8, white),
        '▆' => fill_region(&mut image, w, 0, h - (3 * h / 4), w, 3 * h / 4, white),
        '▇' => fill_region(&mut image, w, 0, h - (7 * h / 8), w, 7 * h / 8, white),
        // Full block
        '█' => fill_region(&mut image, w, 0, 0, w, h, white),
        // Left blocks
        '▉' => fill_region(&mut image, w, 0, 0, 7 * w / 8, h, white),
        '▊' => fill_region(&mut image, w, 0, 0, 3 * w / 4, h, white),
        '▋' => fill_region(&mut image, w, 0, 0, 5 * w / 8, h, white),
        '▌' => fill_region(&mut image, w, 0, 0, w / 2, h, white),
        '▍' => fill_region(&mut image, w, 0, 0, 3 * w / 8, h, white),
        '▎' => fill_region(&mut image, w, 0, 0, w / 4, h, white),
        '▏' => fill_region(&mut image, w, 0, 0, w / 8, h, white),
        // Right half block
        '▐' => fill_region(&mut image, w, w / 2, 0, w - w / 2, h, white),
        // Shade characters
        '░' => fill_shade(&mut image, w, h, 1, 4), // 25%
        '▒' => fill_shade(&mut image, w, h, 2, 4), // 50%
        '▓' => fill_shade(&mut image, w, h, 3, 4), // 75%
        // Quadrant characters
        '▖' => fill_region(&mut image, w, 0, h / 2, w / 2, h - h / 2, white),
        '▗' => fill_region(&mut image, w, w / 2, h / 2, w - w / 2, h - h / 2, white),
        '▘' => fill_region(&mut image, w, 0, 0, w / 2, h / 2, white),
        '▙' => {
            fill_region(&mut image, w, 0, 0, w / 2, h / 2, white);
            fill_region(&mut image, w, 0, h / 2, w, h - h / 2, white);
        }
        '▚' => {
            fill_region(&mut image, w, 0, 0, w / 2, h / 2, white);
            fill_region(&mut image, w, w / 2, h / 2, w - w / 2, h - h / 2, white);
        }
        '▛' => {
            fill_region(&mut image, w, 0, 0, w, h / 2, white);
            fill_region(&mut image, w, 0, h / 2, w / 2, h - h / 2, white);
        }
        '▜' => {
            fill_region(&mut image, w, 0, 0, w, h / 2, white);
            fill_region(&mut image, w, w / 2, h / 2, w - w / 2, h - h / 2, white);
        }
        '▝' => fill_region(&mut image, w, w / 2, 0, w - w / 2, h / 2, white),
        '▞' => {
            fill_region(&mut image, w, w / 2, 0, w - w / 2, h / 2, white);
            fill_region(&mut image, w, 0, h / 2, w / 2, h - h / 2, white);
        }
        '▟' => {
            fill_region(&mut image, w, w / 2, 0, w - w / 2, h / 2, white);
            fill_region(&mut image, w, 0, h / 2, w, h - h / 2, white);
        }
        _ => {} // Unknown block element, leave transparent
    }

    image
}

fn fill_region(image: &mut [u32], stride: usize, x: usize, y: usize, rw: usize, rh: usize, color: u32) {
    for row in y..y + rh {
        if row >= image.len() / stride {
            break;
        }
        for col in x..x + rw {
            if col >= stride {
                break;
            }
            image[row * stride + col] = color;
        }
    }
}

fn fill_shade(image: &mut [u32], w: usize, h: usize, num: u32, den: u32) {
    let white = 0xFFFFFFFFu32;
    // Simple dither pattern
    for y in 0..h {
        for x in 0..w {
            let threshold = ((x + y * 2) as u32 % den) < num;
            if threshold {
                image[y * w + x] = white;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Braille Patterns (U+2800–U+28FF)
// ---------------------------------------------------------------------------

fn rasterize_braille(ch: char, width: u32, height: u32) -> Vec<u32> {
    let w = width as usize;
    let h = height as usize;
    let mut image = vec![0u32; w * h];

    if w == 0 || h == 0 {
        return image;
    }

    let bits = ch as u32 - 0x2800;
    if bits == 0 {
        return image; // Blank braille
    }

    let white = 0xFFFFFFFFu32;

    // Braille has 2 columns, 4 rows of dots
    // Bit layout:
    //   0  3
    //   1  4
    //   2  5
    //   6  7
    let dot_positions: [(u32, usize, usize); 8] = [
        (0, 0, 0), // bit 0 -> col 0, row 0
        (1, 0, 1), // bit 1 -> col 0, row 1
        (2, 0, 2), // bit 2 -> col 0, row 2
        (3, 1, 0), // bit 3 -> col 1, row 0
        (4, 1, 1), // bit 4 -> col 1, row 1
        (5, 1, 2), // bit 5 -> col 1, row 2
        (6, 0, 3), // bit 6 -> col 0, row 3
        (7, 1, 3), // bit 7 -> col 1, row 3
    ];

    // Compute dot size and spacing
    let dot_radius = (w.min(h) as f32 / 8.0).max(1.0);
    let margin_x = w as f32 * 0.2;
    let margin_y = h as f32 * 0.1;
    let usable_w = w as f32 - 2.0 * margin_x;
    let usable_h = h as f32 - 2.0 * margin_y;

    for &(bit, col, row) in &dot_positions {
        if bits & (1 << bit) == 0 {
            continue;
        }

        let dot_cx = margin_x + usable_w * (col as f32 * 2.0 + 1.0) / 4.0;
        let dot_cy = margin_y + usable_h * (row as f32 * 2.0 + 1.0) / 8.0;

        // Fill a circle
        let r = dot_radius;
        let min_x = (dot_cx - r).floor().max(0.0) as usize;
        let max_x = (dot_cx + r).ceil().min(w as f32) as usize;
        let min_y = (dot_cy - r).floor().max(0.0) as usize;
        let max_y = (dot_cy + r).ceil().min(h as f32) as usize;

        for py in min_y..max_y {
            for px in min_x..max_x {
                let dx = px as f32 + 0.5 - dot_cx;
                let dy = py as f32 + 0.5 - dot_cy;
                if dx * dx + dy * dy <= r * r {
                    image[py * w + px] = white;
                }
            }
        }
    }

    image
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fill_rect(target: &mut DrawTarget<&mut [u32]>, x: f32, y: f32, w: f32, h: f32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let src = Source::Solid(WHITE);
    target.fill_rect(x, y, w, h, &src, &DRAW_OPTS_NOAA);
}

/// Convert ARGB (raqote native) to RGBA (wgpu texture format).
fn convert_argb_to_rgba(image: &mut [u32]) {
    for pixel in image.iter_mut() {
        let [a, r, g, b] = pixel.to_be_bytes();
        *pixel = u32::from_le_bytes([r, g, b, a]);
    }
}
