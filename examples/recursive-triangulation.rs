use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

use rand::prelude::*;

use delaunay_mesh::geo::{Bbox, Vec2};
use delaunay_mesh::DelaunayMesh;

fn main() -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let nshapes = rng.gen_range(2, 7);

    let mut bbox = Bbox::new(Vec2::zero());
    bbox.expand(Vec2::new(1920.0, 1080.0));

    let mut mesh = DelaunayMesh::new(bbox);

    for _ in 0..nshapes {
        mesh.insert(rand_vec2(&mut rng, bbox));
        mesh.insert(rand_vec2(&mut rng, bbox));
        mesh.insert(rand_vec2(&mut rng, bbox));
        mesh.insert(rand_vec2(&mut rng, bbox));

        let npoints = rng.gen_range(10, 1000);
        for _ in 0..npoints {
            let vertices = mesh
                .triangles()
                .map(|(t, _)| mesh.triangle_vertices(t))
                .max_by_key(|[a, b, c]| {
                    let mut bbox = Bbox::new(*a);
                    bbox.expand(*b);
                    bbox.expand(*c);

                    bbox.area().round() as u64
                });

            if let Some([a, b, c]) = vertices {
                mesh.insert((a + b + c) / 3.0);
            }
        }
    }

    let mut out = BufWriter::new(File::create("recursive-triangulation.svg")?);
    dump_svg(&mut out, &mesh)
}

pub fn rand_vec2(rng: &mut impl Rng, bbox: Bbox) -> Vec2 {
    let x: f64 = rng
        .gen_range(bbox.min().x as u32, bbox.max().x as u32)
        .into();
    let y: f64 = rng
        .gen_range(bbox.min().y as u32, bbox.max().y as u32)
        .into();

    Vec2::new(x, y)
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
