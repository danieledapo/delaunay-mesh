use std::env;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

use rand::prelude::*;

use delaunay_mesh::geo::{Bbox, Vec2};
use delaunay_mesh::DelaunayMesh;

pub fn main() -> io::Result<()> {
    let npoints = env::args()
        .skip(1)
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(50);

    let mut bbox = Bbox::new(Vec2::zero());
    bbox.expand(Vec2::new(800.0, 800.0));

    let mut rng = thread_rng();
    let mut mesh = DelaunayMesh::new(bbox);

    for i in 0..npoints {
        if (i + 1) % (npoints / 100).max(1) == 0 {
            print!("\rprogress: {}%", i * 100 / npoints);
            io::stdout().flush()?;
        }

        // don't spam too much
        if npoints <= 100 {
            let mut out = BufWriter::new(File::create(format!("triangulation-{}.svg", i))?);
            delaunay_mesh::mesh::dump_svg(&mut out, &mesh)?;
        }

        let x = rng.gen_range(bbox.min().x, bbox.max().x);
        let y = rng.gen_range(bbox.min().y, bbox.max().y);

        mesh.insert(Vec2::new(x, y));
    }
    println!("\rprogress: 100%");

    // don't create huge files
    if npoints <= 1_000 {
        let mut out = BufWriter::new(File::create("triangulation.svg")?);
        delaunay_mesh::mesh::dump_svg(&mut out, &mesh)?;
    }

    Ok(())
}
