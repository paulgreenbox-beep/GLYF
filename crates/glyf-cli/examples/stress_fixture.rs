use glyf_core::{
    GlyfAnnotation, GlyfDocument, GlyfGroup, GlyfImage, GlyfPart, GlyfPoint, serialize_file,
    validate,
};
use glyf_pack::GlypWriter;
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        env::args()
            .nth(1)
            .ok_or("usage: stress_fixture <output-directory> <image-path>")?,
    );
    let image_path = PathBuf::from(
        env::args()
            .nth(2)
            .ok_or("usage: stress_fixture <output-directory> <image-path>")?,
    );
    fs::create_dir_all(&output)?;
    let image_bytes = fs::read(&image_path)?;
    let source = image_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("image path has no UTF-8 file name")?
        .to_owned();

    let parts = (0..50)
        .map(|index| {
            let column = (index % 10) as f32;
            let row = (index / 10) as f32;
            let rel1 = GlyfPoint {
                x: 0.01 + column * 0.095,
                y: 0.02 + row * 0.19,
            };
            GlyfPart {
                name: format!("stress_part_{index:02}"),
                source: "Stress Source".into(),
                clip: None,
                rel1,
                rel2: GlyfPoint {
                    x: rel1.x + 0.08,
                    y: rel1.y + 0.14,
                },
                comment: format!("Stress part {index}"),
                parent: None,
            }
        })
        .collect::<Vec<_>>();

    let annotations = (0..50)
        .map(|index| {
            let column = (index % 10) as f32;
            let row = (index / 10) as f32;
            if index % 2 == 0 {
                GlyfAnnotation::point(
                    format!("stress_annotation_{index:02}"),
                    GlyfPoint {
                        x: (column + 0.5) / 10.0,
                        y: (row + 0.5) / 5.0,
                    },
                    format!("Stress point {index}"),
                )
            } else {
                let rel1 = GlyfPoint {
                    x: column * 0.1 + 0.01,
                    y: row * 0.2 + 0.02,
                };
                GlyfAnnotation::region(
                    format!("stress_annotation_{index:02}"),
                    rel1,
                    GlyfPoint {
                        x: rel1.x + 0.06,
                        y: rel1.y + 0.10,
                    },
                    format!("Stress region {index}"),
                )
            }
        })
        .collect::<Vec<_>>();

    let document = GlyfDocument {
        version: 1,
        images: vec![GlyfImage {
            name: "Stress Source".into(),
            source: source.clone(),
            comment: "50 Part + 50 Annotation rendering stress fixture".into(),
        }],
        groups: vec![GlyfGroup {
            name: "main".into(),
            parts,
        }],
        annotations,
    };
    validate(&document)?;

    let glyf_path = output.join("interaction-stress.glyf");
    let glyp_path = output.join("interaction-stress.glyp");
    serialize_file(&document, &glyf_path)?;
    let mut writer = GlypWriter::new();
    writer.add_file("interaction-stress.glyf", fs::read(&glyf_path)?)?;
    writer.add_file(source, image_bytes)?;
    writer.write(&glyp_path)?;
    println!(
        "created {} with {} parts and {} annotations",
        glyf_path.display(),
        document.groups[0].parts.len(),
        document.annotations.len()
    );
    Ok(())
}
