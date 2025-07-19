use crate::vector::Vector3;

pub mod vector;

fn main() {
    let image_width = 256;
    let image_height = 256;

    println!("P3");
    println!("{image_width} {image_height}");
    println!("{}", 255);

    for row in 0..image_height {
        eprint!("\rScanlines remaining: {}", image_height - row);
        for col in 0..image_width {
            let color = Vector3 {
                x: col as f64 / f64::from(image_width - 1),
                y: row as f64 / f64::from(image_height - 1),
                z: 0 as f64,
            };

            println!("{}", ppm_pixel(color))
        }
    }
}

fn ppm_pixel(color: Vector3) -> String {
    let ir = (255.999 * color.x) as u8;
    let ig = (255.999 * color.y) as u8;
    let ib = (255.999 * color.z) as u8;

    format!("{ir} {ig} {ib}")
}
