use crate::window::Graphics;

impl Graphics {
    fn draw_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let size = self.pixels.texture().size();

        if x >= size.width || y >= size.height {
            return;
        }

        let index = ((y * size.width + x) << 2) as usize;

        self.pixels.frame_mut()[index..index + 4].copy_from_slice(&color)
    }
}

pub fn draw_fn(graphics: &mut Graphics) {
    for i in 0..200 {
        for j in 0..200 {
            graphics.draw_pixel(i, j, [200, 100, 50, 100]);
        }
    }
}
