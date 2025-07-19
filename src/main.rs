fn main() {
    let image_width = 256;
    let image_height = 256;

    println!("P3\n{image_width} {image_height}\n255");

    for row in 0..image_height {
        for col in 0..image_width {
            let r = col as f64 / f64::from(image_width - 1);
            let g = row as f64 / f64::from(image_height - 1);
            let b = 0 as f64;

            let ir = (255.999 * r) as i32;
            let ig = (255.999 * g) as i32;
            let ib = (255.999 * b) as i32;

            println!("{ir} {ig} {ib}");
        }
    }
}
