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
    bbox.expand(Vec2::new(f64::from(npoints), f64::from(npoints)));

    let mut rng = thread_rng();
    let mut mesh = DelaunayMesh::new(bbox);

    for i in 0..npoints {
        if (i + 1) % (npoints / 100).max(1) == 0 {
            let perc = i * 100 / npoints;
            print!("\rprogress: {}%", perc);
            io::stdout().flush()?;
        }

        let x: f64 = rng
            .gen_range(bbox.min().y as u32, bbox.max().x as u32)
            .into();
        let y: f64 = rng
            .gen_range(bbox.min().y as u32, bbox.max().y as u32)
            .into();

        assert!(x.is_finite());
        assert!(y.is_finite());

        mesh.insert(Vec2::new(x, y));
    }

    println!("\rprogress: 100% vertices: {}", mesh.vertices().count());

    let mut out = BufWriter::new(File::create("triangulation.svg")?);
    dump_svg(&mut out, &mesh)?;

    Ok(())
}

pub fn dump_svg(out: &mut impl Write, dmesh: &DelaunayMesh) -> io::Result<()> {
    let bbox = dmesh.bbox();
    let d = bbox.dimensions();

    writeln!(
        out,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="{x} {y} {w} {h}">
<rect x="{x}" y="{y}" width="{w}" height="{h}" stroke="none" fill="white" />
             "#,
        x = bbox.min().x,
        y = bbox.min().y,
        w = d.x,
        h = d.y,
    )?;

    for (tri, _) in dmesh.triangles() {
        let [a, b, c] = dmesh.triangle_vertices(tri);

        writeln!(
            out,
            r#"<polygon points="{},{} {},{} {},{}" fill="none" stroke="black" />"#,
            a.x, a.y, b.x, b.y, c.x, c.y
        )?;
    }

    writeln!(out, "</svg>")
}
