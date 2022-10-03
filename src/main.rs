use std::fs::File;
use std::io::BufWriter;

fn main() {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 360;

    let mut buf = [Pixel::black(); WIDTH * HEIGHT];

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let pixel = &mut buf[x + y * WIDTH];

            pixel.a = 255;
            pixel.r = (x % 255) as u8;
            pixel.g = (y % 255) as u8;
        }
    }

    let buf = unsafe {
        assert_eq!(std::mem::size_of::<Pixel>(), 4 * std::mem::size_of::<u8>());
        std::mem::transmute::<[Pixel; WIDTH * HEIGHT], [u8; WIDTH * HEIGHT * 4]>(buf)
    };

    let file = File::create("out.png").unwrap();
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, WIDTH as u32, HEIGHT as u32);
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&buf).unwrap();
}

#[derive(Copy, Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    pub fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}
