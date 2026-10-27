use glyf_core::{GlyfAnnotationGeometry, GlyfPoint, parse, serialize};

fn parse_serialize_parse(input: &str) -> glyf_core::GlyfDocument {
    let parsed = parse(input).expect("parse fixture manifest");
    let serialized = serialize(&parsed).expect("serialize parsed manifest");
    let reparsed = parse(&serialized).expect("parse serialized manifest");
    assert_eq!(reparsed, parsed, "manifest changed during round trip");
    reparsed
}

#[test]
fn image_only_manifest_round_trips_its_comment() {
    let document = parse_serialize_parse(
        r#"
        glyf {
            version: 1;
            // A document can contain an image without annotations or groups.
            image {
                name: "background";
                source: "art/background.png";
                comment: "Base artwork // keep this text.";
            }
        }
        "#,
    );

    assert_eq!(document.images.len(), 1);
    assert_eq!(document.images[0].name, "background");
    assert_eq!(document.images[0].source, "art/background.png");
    assert_eq!(document.images[0].comment, "Base artwork // keep this text.");
    assert!(document.groups.is_empty());
    assert!(document.annotations.is_empty());
}

#[test]
fn point_annotation_round_trips_role_coordinates_and_comment() {
    let document = parse_serialize_parse(
        r#"
        glyf {
            version: 1;
            annotation {
                name: "origin";
                role: "pivot";
                rel: 0.125 0.75;
                comment: "Anchor point.";
            }
        }
        "#,
    );

    assert_eq!(document.annotations.len(), 1);
    let point = &document.annotations[0];
    assert_eq!(point.name, "origin");
    assert_eq!(point.role, "pivot");
    assert_eq!(point.comment, "Anchor point.");
    assert_eq!(
        point.geometry,
        GlyfAnnotationGeometry::Point {
            rel: GlyfPoint { x: 0.125, y: 0.75 }
        }
    );
}

#[test]
fn region_annotation_round_trips_role_coordinates_and_comment() {
    let document = parse_serialize_parse(
        r#"
        glyf {
            version: 1;
            annotation {
                name: "toolbar";
                role: "interactive_region";
                rel1: 0.125 0.25;
                rel2: 0.875 0.75;
                comment: "Toolbar bounds.";
            }
        }
        "#,
    );

    assert_eq!(document.annotations.len(), 1);
    let region = &document.annotations[0];
    assert_eq!(region.name, "toolbar");
    assert_eq!(region.role, "interactive_region");
    assert_eq!(region.comment, "Toolbar bounds.");
    assert_eq!(
        region.geometry,
        GlyfAnnotationGeometry::Region {
            rel1: GlyfPoint { x: 0.125, y: 0.25 },
            rel2: GlyfPoint { x: 0.875, y: 0.75 },
        }
    );
}

#[test]
fn shared_source_parts_keep_clips_comments_coordinates_and_layer_order() {
    let document = parse_serialize_parse(
        r#"
        glyf {
            version: 1;
            image {
                name: "atlas";
                source: "sprites.png";
                comment: "Shared sprite sheet.";
            }
            group {
                name: "layers";
                part {
                    name: "underlay";
                    source: "atlas";
                    clip: 0 0 128 64;
                    rel1: 0.125 0.25;
                    rel2: 0.875 0.75;
                    comment: "First layer — base.";
                }
                part {
                    name: "highlight";
                    source: "atlas";
                    clip: 128 0 32 16;
                    rel1: 0.25 0.375;
                    rel2: 0.5 0.625;
                    parent: "underlay";
                    comment: "Second layer.\nPreserve this order.";
                }
            }
        }
        "#,
    );

    assert_eq!(document.groups.len(), 1);
    let parts = &document.groups[0].parts;
    assert_eq!(
        parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>(),
        vec!["underlay", "highlight"]
    );
    assert!(parts.iter().all(|part| part.source == "atlas"));
    assert_eq!(
        parts[0].clip,
        Some(glyf_core::GlyfClip {
            x: 0.0,
            y: 0.0,
            w: 128.0,
            h: 64.0,
        })
    );
    assert_eq!(
        parts[1].clip,
        Some(glyf_core::GlyfClip {
            x: 128.0,
            y: 0.0,
            w: 32.0,
            h: 16.0,
        })
    );
    assert_eq!(parts[0].rel1, GlyfPoint { x: 0.125, y: 0.25 });
    assert_eq!(parts[0].rel2, GlyfPoint { x: 0.875, y: 0.75 });
    assert_eq!(parts[1].rel1, GlyfPoint { x: 0.25, y: 0.375 });
    assert_eq!(parts[1].rel2, GlyfPoint { x: 0.5, y: 0.625 });
    assert_eq!(parts[0].comment, "First layer — base.");
    assert_eq!(parts[1].comment, "Second layer.\nPreserve this order.");
    assert_eq!(parts[1].parent.as_deref(), Some("underlay"));
}
