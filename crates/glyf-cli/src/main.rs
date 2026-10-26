use glyf_core::{GlyfAnnotationGeometry, GlyfDocument, parse_file, serialize_file, validate};
use glyf_pack::{GlypReader, GlypWriter};
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("glyf: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage());
    };
    match command {
        "new" => new_project(args.get(1).map(String::as_str).ok_or_else(usage)?),
        "info" => info_project(
            args.get(1).map(String::as_str).ok_or_else(usage)?,
            args.get(2).is_some_and(|arg| arg == "--json"),
        ),
        "validate" => validate_project(args.get(1).map(String::as_str).ok_or_else(usage)?),
        "pack" => pack_file(
            args.get(1).map(String::as_str).ok_or_else(usage)?,
            args.get(2).map(String::as_str).ok_or_else(usage)?,
        ),
        "list" => list_files(args.get(1).map(String::as_str).ok_or_else(usage)?),
        "extract" => extract_file(
            args.get(1).map(String::as_str).ok_or_else(usage)?,
            args.get(2).map(String::as_str).ok_or_else(usage)?,
            args.get(3).map(String::as_str),
        ),
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(format!("unknown command '{command}'\n\n{}", usage())),
    }
}

fn new_project(raw_path: &str) -> Result<(), String> {
    let glyf_path = with_extension(raw_path, "glyf");
    let glyp_path = glyf_path.with_extension("glyp");
    let document = GlyfDocument {
        version: 1,
        images: Vec::new(),
        groups: Vec::new(),
        annotations: Vec::new(),
    };
    serialize_file(&document, &glyf_path).map_err(|error| error.to_string())?;
    GlypWriter::new()
        .write(&glyp_path)
        .map_err(|error| error.to_string())?;
    println!(
        "created {} and {}",
        glyf_path.display(),
        glyp_path.display()
    );
    Ok(())
}

fn info_project(raw_path: &str, json: bool) -> Result<(), String> {
    let document = parse_file(raw_path).map_err(|error| error.to_string())?;
    if json {
        println!("{}", document_json(&document));
        return Ok(());
    }
    println!("GLYF v{}", document.version);
    for image in &document.images {
        println!(
            "\nIMAGE: {}\nSOURCE: {}\nDESCRIPTION:\n{}",
            image.name, image.source, image.comment
        );
    }
    for annotation in &document.annotations {
        match &annotation.geometry {
            GlyfAnnotationGeometry::Point { rel } => println!(
                "\nANNOTATION: {}\n    ROLE: {}\n    GEOMETRY: Point\n    RELATIVE: {:.4} {:.4}\n    COMMENT: {}",
                annotation.name, annotation.role, rel.x, rel.y, annotation.comment
            ),
            GlyfAnnotationGeometry::Region { rel1, rel2 } => println!(
                "\nANNOTATION: {}\n    ROLE: {}\n    GEOMETRY: Region\n    RELATIVE 1: {:.4} {:.4}\n    RELATIVE 2: {:.4} {:.4}\n    COMMENT: {}",
                annotation.name,
                annotation.role,
                rel1.x,
                rel1.y,
                rel2.x,
                rel2.y,
                annotation.comment
            ),
        }
    }
    for group in &document.groups {
        println!("\nGROUP: {}", group.name);
        for (index, part) in group.parts.iter().enumerate() {
            println!(
                "  PART {}: {}\n    SOURCE: {}\n    CLIP: {}\n    RELATIVE: {:.4} {:.4} → {:.4} {:.4}\n    PARENT: {}\n    COMMENT: {}",
                index,
                part.name,
                part.source,
                part.clip
                    .map(|clip| format!("{:.0} {:.0} {:.0} {:.0}", clip.x, clip.y, clip.w, clip.h))
                    .unwrap_or_else(|| "full image".into()),
                part.rel1.x,
                part.rel1.y,
                part.rel2.x,
                part.rel2.y,
                part.parent.as_deref().unwrap_or("<none>"),
                part.comment,
            );
        }
    }
    Ok(())
}

fn validate_project(path: &str) -> Result<(), String> {
    let document = parse_file(path).map_err(|error| error.to_string())?;
    validate(&document).map_err(|error| error.to_string())?;
    let part_count: usize = document.groups.iter().map(|group| group.parts.len()).sum();
    println!(
        "valid GLYF v{}: {} image(s), {} group(s), {} part(s), {} annotation(s)",
        document.version,
        document.images.len(),
        document.groups.len(),
        part_count,
        document.annotations.len(),
    );
    Ok(())
}

fn pack_file(container: &str, image_path: &str) -> Result<(), String> {
    let image_path = Path::new(image_path);
    let bytes =
        std::fs::read(image_path).map_err(|error| format!("{}: {error}", image_path.display()))?;
    let name = image_path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| "image path has no valid file name".to_string())?;
    if !supported_image(name) {
        return Err(format!("unsupported image extension for '{name}'"));
    }
    let mut writer = GlypWriter::new();
    writer
        .add_file(name, bytes)
        .map_err(|error| error.to_string())?;
    writer.write(container).map_err(|error| error.to_string())?;
    println!("packed {name} into {container}");
    Ok(())
}

fn list_files(path: &str) -> Result<(), String> {
    let reader = GlypReader::open(path).map_err(|error| error.to_string())?;
    for file in reader.list_files() {
        println!("{file}");
    }
    Ok(())
}

fn extract_file(container: &str, name: &str, output: Option<&str>) -> Result<(), String> {
    let mut reader = GlypReader::open(container).map_err(|error| error.to_string())?;
    let bytes = reader.read_file(name).map_err(|error| error.to_string())?;
    let output = output.unwrap_or(name);
    std::fs::write(output, bytes).map_err(|error| format!("{output}: {error}"))?;
    println!("extracted {name} to {output}");
    Ok(())
}

fn with_extension(path: &str, extension: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.extension().is_some_and(|value| value == extension) {
        path
    } else {
        path.with_extension(extension)
    }
}

fn supported_image(name: &str) -> bool {
    matches!(
        Path::new(name)
            .extension()
            .and_then(OsStr::to_str)
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "bmp" | "ico")
    )
}

fn document_json(document: &GlyfDocument) -> String {
    let images = document
        .images
        .iter()
        .map(|image| {
            format!(
                r#"{{"name":{},"source":{},"comment":{}}}"#,
                json_string(&image.name),
                json_string(&image.source),
                json_string(&image.comment)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let groups = document
        .groups
        .iter()
        .map(|group| {
            let parts = group
                .parts
                .iter()
                .map(|part| {
                    let clip = part.clip.map(|clip| {
                        format!(
                            r#"{{"x":{},"y":{},"w":{},"h":{}}}"#,
                            clip.x, clip.y, clip.w, clip.h
                        )
                    });
                    format!(
                        r#"{{"name":{},"source":{},"clip":{},"rel1":{{"x":{},"y":{}}},"rel2":{{"x":{},"y":{}}},"comment":{},"parent":{}}}"#,
                        json_string(&part.name),
                        json_string(&part.source),
                        clip.as_deref().unwrap_or("null"),
                        part.rel1.x,
                        part.rel1.y,
                        part.rel2.x,
                        part.rel2.y,
                        json_string(&part.comment),
                        part.parent
                            .as_deref()
                            .map(json_string)
                            .unwrap_or_else(|| "null".into()),
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"name":{},"parts":[{}]}}"#,
                json_string(&group.name),
                parts
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let annotations = document
        .annotations
        .iter()
        .map(|annotation| {
            match &annotation.geometry {
                GlyfAnnotationGeometry::Point { rel } => format!(
                    r#"{{"name":{},"role":{},"geometry":"point","rel":{{"x":{},"y":{}}},"comment":{}}}"#,
                    json_string(&annotation.name),
                    json_string(&annotation.role),
                    rel.x,
                    rel.y,
                    json_string(&annotation.comment)
                ),
                GlyfAnnotationGeometry::Region { rel1, rel2 } => format!(
                    r#"{{"name":{},"role":{},"geometry":"region","rel1":{{"x":{},"y":{}}},"rel2":{{"x":{},"y":{}}},"comment":{}}}"#,
                    json_string(&annotation.name),
                    json_string(&annotation.role),
                    rel1.x,
                    rel1.y,
                    rel2.x,
                    rel2.y,
                    json_string(&annotation.comment)
                ),
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"version":{},"images":[{}],"groups":[{}],"annotations":[{}]}}"#,
        document.version, images, groups, annotations
    )
}
fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other if other.is_control() => output.push_str(&format!("\\u{:04x}", other as u32)),
            other => output.push(other),
        }
    }
    output.push('"');
    output
}

fn usage() -> String {
    "usage:\n  glyf new project\n  glyf info project.glyf [--json]\n  glyf validate project.glyf\n  glyf pack project.glyp image.png\n  glyf list project.glyp\n  glyf extract project.glyp image.png [output-path]".into()
}
