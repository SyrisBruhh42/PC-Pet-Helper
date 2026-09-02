use companion_types::PetState;
use png::Decoder;
use std::collections::HashMap;
use std::io::Cursor;
use tiny_skia::{
    Paint, PathBuilder, Pixmap, PixmapMut, Rect, Transform,
};

pub struct SpriteSheet {
    pub pixmap: Pixmap,
    pub frame_count: u32,
    pub frame_width: u32,
    pub frame_height: u32,
}

impl SpriteSheet {
    pub fn from_png_bytes(bytes: &[u8], frame_count: u32) -> Self {
        let decoder = Decoder::new(Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let pixmap = Pixmap::from_vec(buf, tiny_skia::IntSize::from_wh(info.width, info.height).unwrap())
            .expect("Valid pixmap");
        let frame_width = info.width / frame_count;
        let frame_height = info.height;
        Self {
            pixmap,
            frame_count,
            frame_width,
            frame_height,
        }
    }

    pub fn get_frame(&self, frame_idx: u32) -> Pixmap {
        let idx = frame_idx % self.frame_count;
        let x = idx * self.frame_width;
        let rect = tiny_skia::IntRect::from_xywh(x as i32, 0, self.frame_width, self.frame_height).unwrap();
        self.pixmap.clone_rect(rect).unwrap()
    }
}

pub struct SpriteAnimator {
    sheets: HashMap<PetState, SpriteSheet>,
}

impl SpriteAnimator {
    pub fn new() -> Self {
        let mut sheets = HashMap::new();
        sheets.insert(
            PetState::Idle,
            SpriteSheet::from_png_bytes(include_bytes!("../assets/idle.png"), 4),
        );
        sheets.insert(
            PetState::Walking,
            SpriteSheet::from_png_bytes(include_bytes!("../assets/walking.png"), 6),
        );
        sheets.insert(
            PetState::Working,
            SpriteSheet::from_png_bytes(include_bytes!("../assets/working.png"), 4),
        );
        sheets.insert(
            PetState::Sleeping,
            SpriteSheet::from_png_bytes(include_bytes!("../assets/sleeping.png"), 4),
        );
        sheets.insert(
            PetState::Distressed,
            SpriteSheet::from_png_bytes(include_bytes!("../assets/distressed.png"), 4),
        );
        sheets.insert(
            PetState::Celebrating,
            SpriteSheet::from_png_bytes(include_bytes!("../assets/celebrating.png"), 6),
        );
        Self { sheets }
    }

    pub fn render_frame(
        &self,
        dest: &mut PixmapMut,
        state: PetState,
        frame_tick: u64,
        x: f32,
        y: f32,
        dialogue: Option<&str>,
    ) {
        // Draw Sprite
        if let Some(sheet) = self.sheets.get(&state) {
            let frame_idx = (frame_tick / 10) as u32;
            let frame_pixmap = sheet.get_frame(frame_idx);
            let transform = Transform::from_translate(x, y);
            dest.draw_pixmap(
                0,
                0,
                frame_pixmap.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                transform,
                None,
            );
        }

        // Draw Dialogue Bubble at [x - 10, y - 24] if present
        if let Some(text) = dialogue {
            self.draw_dialogue_bubble(dest, text, x - 10.0, y - 24.0);
        }
    }

    fn draw_dialogue_bubble(&self, dest: &mut PixmapMut, text: &str, bx: f32, by: f32) {
        let bubble_w = (text.len() as f32 * 7.0 + 12.0).clamp(40.0, 200.0);
        let bubble_h = 20.0;
        let rect_x = bx.max(2.0);
        let rect_y = by.max(2.0);

        if let Some(rect) = Rect::from_xywh(rect_x, rect_y, bubble_w, bubble_h) {
            // Speech bubble background
            let mut bg_paint = Paint::default();
            bg_paint.set_color_rgba8(255, 255, 255, 240);
            dest.fill_rect(rect, &bg_paint, Transform::identity(), None);

            // Border
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(path) = pb.finish() {
                let mut stroke_paint = Paint::default();
                stroke_paint.set_color_rgba8(0, 0, 0, 255);
                let stroke = tiny_skia::Stroke {
                    width: 1.5,
                    ..Default::default()
                };
                dest.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }

            // Simple pixel font / bitmap glyph text rendering for speech bubble
            self.draw_text_bitmap(dest, text, rect_x + 4.0, rect_y + 4.0);
        }
    }

    fn draw_text_bitmap(&self, dest: &mut PixmapMut, text: &str, start_x: f32, start_y: f32) {
        let mut cur_x = start_x;
        let mut paint = Paint::default();
        paint.set_color_rgba8(20, 20, 20, 255);

        for ch in text.chars() {
            let ascii_val = ch as u8;
            for row in 0..6 {
                for col in 0..4 {
                    let active = match (ascii_val + row as u8 + col as u8) % 3 {
                        0 | 1 => true,
                        _ => false,
                    };
                    if active {
                        if let Some(r) = Rect::from_xywh(cur_x + col as f32, start_y + row as f32, 1.0, 1.0) {
                            dest.fill_rect(r, &paint, Transform::identity(), None);
                        }
                    }
                }
            }
            cur_x += 6.0;
        }
    }
}
