use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Image dimensions
    let image_width = 256;
    let image_height = 256;

    // Lock stdout/stderr for performance
    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();
    let stderr = io::stderr();
    let mut stderr_handle = stderr.lock();

    // PPM header
    writeln!(stdout_handle, "P3\n{} {}\n255", image_width, image_height)?;

    // Render pixels with progress
    for j in 0..image_height {
        // Update progress (stderr)
        write!(
            stderr_handle,
            "\rScanlines remaining: {} ",
            image_height - j
        )?;
        stderr_handle.flush()?;

        for i in 0..image_width {
            let r = i as f64 / (image_width - 1) as f64;
            let g = j as f64 / (image_height - 1) as f64;
            let b = 0.0;

            let ir = (255.999 * r) as i32;
            let ig = (255.999 * g) as i32;
            let ib = (255.999 * b) as i32;

            writeln!(stdout_handle, "{} {} {}", ir, ig, ib)?;
        }
    }

    // Clear progress and print "Done"
    writeln!(stderr_handle, "\rDone.                 ")?;

    Ok(())
}