//! Splash UI declaration. Runtime behavior belongs to the Rust adapters.

use super::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.DsElevatedPanel
    use mod.widgets.DsMenuBar
    use mod.widgets.DsMenuButton
    use mod.widgets.DsMenuItem
    use mod.widgets.DsMenuSeparator
    use mod.widgets.DsMenuSurface
    use mod.widgets.DsIconChevronRight
    use mod.widgets.DsPanel
    use mod.widgets.DsStatusBar
    use mod.widgets.DsToolbar
    use mod.widgets.DsWindow
    use mod.widgets.GlyfOverlayHost
    use mod.widgets.ClipPopup

    // SVG icons cannot currently render on the pinned Windows D3D11 backend
    // (DrawSvg's shader uses an unsupported unpack2f16 intrinsic). Keep the
    // primary toolbar legible with text controls until that backend is fixed.
    mod.widgets.GlyfToolbarAction = mod.widgets.ButtonFlatter {
        width: 38 height: 34
        padding: Inset{left: 0 right: 0 top: 0 bottom: 0}
        draw_bg +: { border_radius: 6.0 border_size: 1.0 }
        draw_text.text_style.font_size: 19.0
    }
    use mod.widgets.GlyfToolbarAction
    mod.widgets.GlyfToolbarIconSlot = View {
        width: 38 height: 34 flow: Overlay align: Align{x: 0.5 y: 0.5}
    }
    use mod.widgets.GlyfToolbarIconSlot

    startup() do #(App::script_component(vm)) {
        ui: Root {
            main_window := Window {
                window.inner_size: vec2(1440, 900)
                window.title: "GLYF Editor"
                pass.clear_color: theme.color_bg_app
                // On Windows, Window's built-in caption region owns every
                // pointer event for dragging.  This in-client title bar keeps
                // tabs interactive while App maps drag/min/max/close to the
                // corresponding native window actions.
                show_caption_bar: false
                body +: {
                    width: Fill height: Fill flow: Overlay
                    main_layout := DsWindow {
                    width: Fill height: Fill flow: Down spacing: 2
                    padding: Inset{left: 0 right: 0 top: 0 bottom: 2} show_bg: true draw_bg.color: theme.color_bg_app

                    chrome_bar := DsPanel {
                        width: Fill height: 42 flow: Right spacing: 4 padding: Inset{left: 8 right: 0 top: 2 bottom: 2}
                        new_batch: true draw_bg.color: theme.color_bg_even
                        chrome_drag := View {
                            width: 196 height: Fill flow: Right align: Align {y: 0.5}
                            chrome_title := Label { text: "GLYF Editor" draw_text.text_style.font_size: theme.font_size_p }
                        }
                        tabs := mod.widgets.DocumentTabStrip { width: Fill height: 38 }
                        tab_overflow := ButtonFlatter { width: 30 text: "⌄" }
                        tab_new := ButtonFlatter { width: 30 text: "+" }
                        chrome_minimize := ButtonFlatter { width: 46 height: 38 text: "—" }
                        chrome_maximize := ButtonFlatter { width: 46 height: 38 text: "□" }
                        chrome_close := ButtonFlatter { width: 46 height: 38 text: "×" }
                    }

                    menu_bar := DsMenuBar {
                        width: Fill height: 36 flow: Right spacing: 2 padding: Inset{left: 8 right: 8 top: 2 bottom: 2}
                        new_batch: true draw_bg.color: theme.color_bg_container
                        menu_file := DsMenuButton { text: "File" }
                        menu_edit := DsMenuButton { text: "Edit" }
                        menu_view := DsMenuButton { text: "View" }
                        menu_window := DsMenuButton { text: "Window" }
                        menu_language := DsMenuButton { text: "Language" }
                        menu_help := DsMenuButton { text: "Help" }
                    }

                    toolbar := DsToolbar {
                        width: Fill height: 48 flow: Right spacing: 6 padding: Inset{left: 10 right: 10 top: 6 bottom: 6} align: Align { y: 0.5 }
                        new_batch: true draw_bg.color: theme.color_fg_app
                        GlyfToolbarIconSlot {
                            new_project := GlyfToolbarAction { text: "" }
                            Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/file-plus.png") }
                        }
                        GlyfToolbarIconSlot {
                            save_project := GlyfToolbarAction { text: "" }
                            Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/save.png") }
                        }
                        GlyfToolbarIconSlot {
                            sidebar_images := GlyfToolbarAction { text: "" }
                            Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/image.png") }
                        }
                        GlyfToolbarIconSlot {
                            sidebar_layers := GlyfToolbarAction { text: "" }
                            Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/layers.png") }
                        }
                        View { width: 1 height: 24 show_bg: true draw_bg.color: theme.color_bevel }
                        annotation_toolbar := View { width: Fit height: Fit flow: Right spacing: 6
                            GlyfToolbarIconSlot {
                                tool_select := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/mouse-pointer-2.png") }
                            }
                            GlyfToolbarIconSlot {
                                tool_point := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/crosshair.png") }
                            }
                            GlyfToolbarIconSlot {
                                tool_region := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/square-dashed-mouse-pointer.png") }
                            }
                            GlyfToolbarIconSlot {
                                canvas_fit := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/maximize.png") }
                            }
                            GlyfToolbarIconSlot {
                                canvas_100 := GlyfToolbarAction { text: "" }
                                Image { width: 24 height: 20 src: crate_resource("self:resources/icons/toolbar/actual-size.png") }
                            }
                            GlyfToolbarIconSlot {
                                canvas_zoom_out := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/zoom-out.png") }
                            }
                            canvas_zoom_label := Label { width: 54 height: Fit text: "100%" align: Center draw_text.color: theme.color_label_inner_inactive }
                            GlyfToolbarIconSlot {
                                canvas_zoom_in := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/zoom-in.png") }
                            }
                        }
                        source_toolbar := View { width: Fit height: Fit flow: Right spacing: 6 visible: false
                            GlyfToolbarIconSlot {
                                source_back := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/chevron-left.png") }
                            }
                            GlyfToolbarIconSlot {
                                source_select := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/mouse-pointer-2.png") }
                            }
                            GlyfToolbarIconSlot {
                                source_clip := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/crop.png") }
                            }
                            GlyfToolbarIconSlot {
                                source_fit := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/maximize.png") }
                            }
                            GlyfToolbarIconSlot {
                                source_100 := GlyfToolbarAction { text: "" }
                                Image { width: 24 height: 20 src: crate_resource("self:resources/icons/toolbar/actual-size.png") }
                            }
                            GlyfToolbarIconSlot {
                                source_zoom_out := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/zoom-out.png") }
                            }
                            source_zoom_label := Label { width: 54 height: Fit text: "100%" align: Center draw_text.color: theme.color_label_inner_inactive }
                            GlyfToolbarIconSlot {
                                source_zoom_in := GlyfToolbarAction { text: "" }
                                Image { width: 20 height: 20 src: crate_resource("self:resources/icons/toolbar/zoom-in.png") }
                            }
                            source_status := Label { width: Fit height: Fit text: "Source: —" draw_text.color: theme.color_label_inner_inactive }
                        }
                        Filler {}
                        mode_badge := Label { text: "COMPOSITION" draw_text.color: theme.color_highlight }
                        dirty_badge := Label { text: "SAVED" draw_text.color: theme.color_label_inner_inactive }
                    }

                    workspace := View {
                        width: Fill height: Fill flow: Right spacing: 4

                        sidebar := DsPanel {
                            width: 260 height: Fill flow: Overlay padding: 0 visible: false
                            new_batch: true draw_bg.color: theme.color_fg_app
                            resources_panel := View {
                                width: Fill height: Fill flow: Down spacing: 6 padding: 10 visible: false
                                resources_heading := Label { text: "IMAGES" draw_text.text_style.font_size: theme.font_size_3 }
                                images_list := Label { width: Fill height: Fit text: "No images yet" draw_text.color: theme.color_label_inner_inactive }
                                image_dropdown := DropDown { width: Fill labels: ["No images"] }
                                image_path_input := TextInput { width: Fill height: 36 empty_text: "PNG / JPG / WEBP / BMP / ICO path" }
                                View { width: Fill height: Fit flow: Right spacing: 6
                                    add_image := Button { text: "+ Add" }
                                    remove_image := Button { text: "Remove" }
                                }
                                Hr {}
                                metadata_heading := Label { text: "Resource metadata" draw_text.text_style.font_size: theme.font_size_3 }
                                name_input := TextInput { width: Fill height: 36 empty_text: "image name" }
                                source_label := Label { width: Fill height: Fit text: "Source: —" draw_text.color: theme.color_label_inner_inactive }
                                comment_input := TextInput { width: Fill height: 82 empty_text: "resource comment" }
                                update_image := Button { width: Fill text: "Update resource" }
                                edit_source := Button { width: Fill text: "Open Source" }
                            }

                            layers_panel := View {
                                width: Fill height: Fill flow: Down spacing: 6 padding: 10 visible: false
                                layers_heading := Label { text: "LAYERS · Z-ORDER" draw_text.text_style.font_size: theme.font_size_3 }
                                layers_list := Label { width: Fill height: Fit text: "No parts yet" draw_text.color: theme.color_label_inner_inactive }
                                part_dropdown := DropDown { width: Fill labels: ["No parts"] }
                                View { width: Fill height: Fit flow: Right spacing: 5
                                    part_add := Button { text: "+ Part" }
                                    part_add_clip := Button { text: "+ Clip" }
                                }
                                View { width: Fill height: Fit flow: Right spacing: 5
                                    part_remove := Button { text: "Delete" }
                                    part_edit_clip := Button { width: Fill text: "Edit source crop" }
                                }
                                View { width: Fill height: Fit flow: Right spacing: 5
                                    part_up := Button { width: Fill text: "Up" }
                                    part_down := Button { width: Fill text: "Down" }
                                }
                                View { width: Fill height: Fit flow: Right spacing: 5
                                    part_back := Button { width: Fill text: "Back" }
                                    part_front := Button { width: Fill text: "Front" }
                                }
                                layers_hint := Label { width: Fill height: Fit text: "Drag a part on Canvas to place it. Double-click it to edit name and comment." draw_text.color: theme.color_label_inner_inactive }
                                Hr {}
                                annotations_heading := Label { text: "ANNOTATIONS" draw_text.text_style.font_size: theme.font_size_3 }
                                annotations_list := Label { width: Fill height: Fit text: "No annotations yet" draw_text.color: theme.color_label_inner_inactive }
                                annotation_dropdown := DropDown { width: Fill labels: ["No annotations"] }
                                View { width: Fill height: Fit flow: Right spacing: 5
                                    annotation_edit := Button { text: "Edit" }
                                    annotation_delete := Button { text: "Delete" }
                                }
                                annotations_hint := Label { width: Fill height: Fit text: "Point + Region comments live on Canvas" draw_text.color: theme.color_label_inner_inactive }
                                annotation_geometry := Label { width: Fill height: Fit text: "No selection" draw_text.color: theme.color_label_inner_inactive }
                                annotation_role := Label { width: Fill height: Fit text: "Role: —" draw_text.color: theme.color_label_inner_inactive }
                                annotation_rel := Label { width: Fill height: Fit text: "Coordinates: —" draw_text.color: theme.color_label_inner_inactive }
                                annotation_comment := Label { width: Fill height: Fit text: "Comment: —" draw_text.color: theme.color_label_inner_inactive }
                            }
                        }

                        preview := DsElevatedPanel {
                            width: Fill height: Fill flow: Overlay padding: 8
                            new_batch: true draw_bg.color: theme.color_bg_app
                            preview_image := Image { width: Fill height: Fill fit: ImageFit.Smallest visible: false }
                            canvas_overlay := mod.widgets.SpatialCanvas { width: Fill height: Fill }
                            source_canvas := mod.widgets.SourceCanvas { width: Fill height: Fill visible: false }
                            annotation_popup := DsElevatedPanel {
                                width: 310 height: Fit flow: Down spacing: 8 padding: 10 visible: false
                                draw_bg +: { color: theme.color_bg_app border_size: 2.0 border_color: theme.color_highlight border_radius: 8.0 }
                                title := Label {
                                    width: Fill height: Fit text: "Add annotation" draw_text.color: theme.color_label_inner
                                }
                                comment_input := TextInput {
                                    width: Fill height: 44 empty_text: "What is here?" return_key_type: Done
                                }
                                actions := View {
                                    width: Fill height: Fit flow: Right spacing: 8 align: Align {x: 1.0 y: 0.5}
                                    cancel := Button { text: "Cancel" }
                                    commit := Button { text: "Done" }
                                }
                            }
                            clip_popup := ClipPopup { width: Fill height: Fill }
                            preview_placeholder := Label { width: Fill height: Fill text: "No parts yet\n\nAdd an image, then create a part\nfrom the whole image or a clipped region." align: Center draw_text.color: theme.color_label_inner_inactive }
                        }
                    }

                    // Coordinates are now manipulated on the canvas.  The hidden cache keeps
                    // old semantic automation IDs harmless while the visible inspector is gone.
                    part_edit_cache := View {
                        width: 0 height: 0 visible: false
                        part_name_input := TextInput {}
                        part_source_input := TextInput {}
                        part_parent_input := TextInput {}
                        clip_x_input := TextInput {}
                        clip_y_input := TextInput {}
                        clip_w_input := TextInput {}
                        clip_h_input := TextInput {}
                        rel1_x_input := TextInput {}
                        rel1_y_input := TextInput {}
                        rel2_x_input := TextInput {}
                        rel2_y_input := TextInput {}
                        part_comment_input := TextInput {}
                        part_apply := Button {}
                    }

                    status_bar := DsStatusBar {
                        width: Fill height: 28
                        status := Label { width: Fill height: Fit text: "Ready" draw_text.color: theme.color_label_inner_inactive }
                    }
                    }

                    shell_overlay := GlyfOverlayHost {
                        file_menu := DsMenuSurface {
                            width: 360 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            menu_file_new := DsMenuItem { text: "New" }
                            menu_file_open := DsMenuItem { text: "Open..." }
                            menu_file_recent_row := View {
                                width: Fill height: 29 flow: Overlay align: Align{x: 1.0 y: 0.5}
                                menu_file_recent := DsMenuItem { text: "Open Recent" }
                                DsIconChevronRight { width: 16 height: 16 margin: Inset{left: 0 right: 10 top: 0 bottom: 0} }
                            }
                            DsMenuSeparator {}
                            menu_file_save := DsMenuItem { text: "Save" }
                            menu_file_save_as := DsMenuItem { text: "Save As..." }
                            menu_file_save_pack := DsMenuItem { text: "Save & Pack" }
                            DsMenuSeparator {}
                            menu_file_close := DsMenuItem { text: "Close" }
                            menu_file_exit := DsMenuItem { text: "Exit" }
                        }
                        file_recent_submenu := DsMenuSurface {
                            width: 420 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            recent_empty := Label { width: Fill height: Fit text: "No recent files" draw_text.color: theme.color_label_inner_inactive }
                            recent_file_0 := DsMenuItem { text: "" visible: false }
                            recent_file_1 := DsMenuItem { text: "" visible: false }
                            recent_file_2 := DsMenuItem { text: "" visible: false }
                            recent_file_3 := DsMenuItem { text: "" visible: false }
                            recent_file_4 := DsMenuItem { text: "" visible: false }
                            recent_file_5 := DsMenuItem { text: "" visible: false }
                            recent_file_6 := DsMenuItem { text: "" visible: false }
                            recent_file_7 := DsMenuItem { text: "" visible: false }
                            recent_file_8 := DsMenuItem { text: "" visible: false }
                            recent_file_9 := DsMenuItem { text: "" visible: false }
                            recent_separator := DsMenuSeparator { visible: false }
                            recent_clear := DsMenuItem { text: "Clear Recent" visible: false }
                        }
                        edit_menu := DsMenuSurface {
                            width: 320 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            menu_edit_undo := DsMenuItem { text: "Undo" }
                            menu_edit_redo := DsMenuItem { text: "Redo" }
                            DsMenuSeparator {}
                            menu_edit_cut := DsMenuItem { text: "Cut" }
                            menu_edit_copy := DsMenuItem { text: "Copy" }
                            menu_edit_paste := DsMenuItem { text: "Paste" }
                            DsMenuSeparator {}
                            menu_edit_duplicate := DsMenuItem { text: "Duplicate" }
                            menu_edit_delete := DsMenuItem { text: "Delete" }
                            menu_edit_select_all := DsMenuItem { text: "Select All" }
                            DsMenuSeparator {}
                            menu_settings := DsMenuItem { text: "Settings" }
                        }
                        view_menu := DsMenuSurface {
                            width: 290 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            menu_view_zoom_in := DsMenuItem { text: "Zoom In" }
                            menu_view_zoom_out := DsMenuItem { text: "Zoom Out" }
                            menu_view_fit := DsMenuItem { text: "Fit" }
                            DsMenuSeparator {}
                            menu_view_resources := DsMenuItem { text: "Resources" }
                            menu_view_layers := DsMenuItem { text: "Layers" }
                            }
                        window_menu := DsMenuSurface {
                            width: 300 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            menu_window_next := DsMenuItem { text: "Next Tab" }
                            menu_window_previous := DsMenuItem { text: "Previous Tab" }
                            DsMenuSeparator {}
                            menu_window_close := DsMenuItem { text: "Close Tab" }
                            menu_window_close_others := DsMenuItem { text: "Close Other Tabs" }
                            menu_window_close_all := DsMenuItem { text: "Close All Tabs" }
                        }
                        language_menu := DsMenuSurface {
                            width: 220 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            menu_language_english := DsMenuItem { text: "English" }
                            menu_language_russian := DsMenuItem { text: "Русский" }
                        }
                        help_menu := DsMenuSurface {
                            width: 260 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            menu_help_about := DsMenuItem { text: "About Glyf Editor" }
                            menu_help_shortcuts := DsMenuItem { text: "Keyboard Shortcuts" }
                        }
                        context_menu := DsMenuSurface {
                            width: 260 height: Fit flow: Down spacing: 4 padding: 10 visible: false
                            new_batch: true
                            context_caption := Label { width: Fill height: Fit text: "Context menu" }
                            DsMenuSeparator {}
                            context_copy := DsMenuItem { text: "Copy" }
                            context_cut := DsMenuItem { text: "Cut" }
                            context_paste := DsMenuItem { text: "Paste" }
                            DsMenuSeparator {}
                            context_duplicate := DsMenuItem { text: "Duplicate" }
                            context_delete := DsMenuItem { text: "Delete" }
                        }
                        settings_panel := DsElevatedPanel {
                            width: 760 height: 510 flow: Down spacing: 0 padding: 0 visible: false
                            new_batch: true
                            draw_bg.color: theme.color_fg_app
                            settings_header := View {
                                width: Fill height: 52 flow: Right align: Align{y: 0.5}
                                padding: Inset{left: 20 right: 16 top: 8 bottom: 8}
                                settings_title := Label { width: Fill height: Fit text: "Settings" draw_text.text_style.font_size: theme.font_size_3 }
                                settings_close := ButtonFlatter { width: 84 height: 32 text: "Close" }
                            }
                            Hr { width: Fill height: 1 }
                            settings_body := View {
                                width: Fill height: Fill flow: Right spacing: 0
                                settings_nav := DsPanel {
                                    width: 174 height: Fill flow: Down spacing: 5 padding: 10
                                    settings_category_general := ButtonFlat { width: Fill height: 38 text: "General" }
                                    settings_category_appearance := ButtonFlat { width: Fill height: 38 text: "Appearance" }
                                    settings_category_editor := ButtonFlat { width: Fill height: 38 text: "Editor" }
                                    settings_category_files := ButtonFlat { width: Fill height: 38 text: "Files" }
                                    settings_category_accessibility := ButtonFlat { width: Fill height: 38 text: "Accessibility" }
                                }
                                settings_scroll := ScrollYView {
                                    width: Fill height: Fill flow: Down spacing: 0
                                    settings_general_view := View {
                                        width: Fill height: Fit flow: Down spacing: 12 padding: 18
                                        settings_general_heading := Label { text: "General" draw_text.text_style.font_size: theme.font_size_3 }
                                        settings_language_label := Label { text: "Language" }
                                        settings_language_select := DropDown { width: Fill height: 36 labels: ["English", "Русский"] }
                                        settings_restore := ButtonFlat { width: Fill height: 36 text: "Restore previous session" }
                                        settings_recent_label := Label { text: "Recent files count" }
                                        settings_recent_count := DropDown { width: Fill height: 36 labels: ["5", "10"] }
                                    }
                                    settings_appearance_view := View {
                                        width: Fill height: Fit flow: Down spacing: 9 padding: 18 visible: false
                                        settings_appearance_heading := Label { text: "Appearance" draw_text.text_style.font_size: theme.font_size_3 }
                                        settings_preset_label := Label { text: "Visual preset" }
                                        settings_preset := DropDown { width: Fill height: 34 labels: ["Classic Light", "Midnight", "Neon", "Glass Dark", "Soft Light", "Sharp Dark", "Custom"] }
                                        settings_palette_label := Label { text: "Palette" }
                                        settings_palette := DropDown { width: Fill height: 34 labels: ["Dark", "Light", "Neon"] }
                                        settings_shape_label := Label { text: "Shape" }
                                        settings_shape := DropDown { width: Fill height: 34 labels: ["Rounded", "Soft", "Sharp", "Angular"] }
                                        settings_surface_label := Label { text: "Surface" }
                                        settings_surface := DropDown { width: Fill height: 34 labels: ["Flat", "Elevated", "Glass", "Neumorphic"] }
                                        settings_motion_label := Label { text: "Motion" }
                                        settings_motion := DropDown { width: Fill height: 34 labels: ["None", "Reduced", "Subtle", "Smooth"] }
                                        settings_font_label := Label { text: "Interface font size" }
                                        settings_font_size := DropDown { width: Fill height: 34 labels: ["Small", "Normal", "Large"] }
                                        settings_scale_label := Label { text: "UI Scale" }
                                        settings_ui_scale := DropDown { width: Fill height: 34 labels: ["100%", "110%", "125%", "150%"] }
                                        settings_density_label := Label { text: "Density" }
                                        settings_density := DropDown { width: Fill height: 34 labels: ["Compact", "Comfortable"] }
                                        settings_style_profile := Label { width: Fill height: Fit text: "Dark · Rounded · Elevated · Subtle" draw_text.color: theme.color_label_inner_inactive }
                                    }
                                    settings_editor_view := View {
                                        width: Fill height: Fit flow: Down spacing: 12 padding: 18 visible: false
                                        settings_editor_heading := Label { text: "Editor" draw_text.text_style.font_size: theme.font_size_3 }
                                        settings_zoom_label := Label { text: "Default zoom" }
                                        settings_default_zoom := DropDown { width: Fill height: 36 labels: ["50%", "75%", "100%", "125%", "150%"] }
                                        settings_resources := ButtonFlat { width: Fill height: 36 text: "Show Resources" }
                                        settings_layers := ButtonFlat { width: Fill height: 36 text: "Show Layers" }
                                        settings_inspector := ButtonFlat { width: Fill height: 36 text: "Show Inspector" }
                                        settings_status_bar := ButtonFlat { width: Fill height: 36 text: "Show Status Bar" }
                                        settings_confirm_destructive := ButtonFlat { width: Fill height: 36 text: "Confirm destructive actions" }
                                    }
                                    settings_files_view := View {
                                        width: Fill height: Fit flow: Down spacing: 12 padding: 18 visible: false
                                        settings_files_heading := Label { text: "Files" draw_text.text_style.font_size: theme.font_size_3 }
                                        settings_reopen_last := ButtonFlat { width: Fill height: 36 text: "Reopen last documents" }
                                        settings_project_location_label := Label { width: Fill height: Fit text: "Default project location: Documents" }
                                        settings_glyp_behavior := ButtonFlat { width: Fill height: 36 text: "GLYP: manual pack" }
                                    }
                                    settings_accessibility_view := View {
                                        width: Fill height: Fit flow: Down spacing: 12 padding: 18 visible: false
                                        settings_accessibility_heading := Label { text: "Accessibility" draw_text.text_style.font_size: theme.font_size_3 }
                                        settings_reduced_motion := ButtonFlat { width: Fill height: 36 text: "Reduced Motion" }
                                        settings_large_font := ButtonFlat { width: Fill height: 36 text: "Large interface font" }
                                        settings_high_contrast_note := Label { width: Fill height: Fit text: "High Contrast: under investigation" draw_text.color: theme.color_label_inner_inactive }
                                    }
                                }
                            }
                            Hr { width: Fill height: 1 }
                            settings_footer := View {
                                width: Fill height: 48 flow: Right spacing: 8 align: Align{y: 0.5}
                                padding: Inset{left: 14 right: 14 top: 6 bottom: 6}
                                settings_reset_appearance := ButtonFlatter { width: Fit height: 32 text: "Reset Appearance" }
                                settings_reset_all := ButtonFlatter { width: Fit height: 32 text: "Reset All Settings" }
                                settings_reset_confirm := View {
                                    width: Fill height: Fit flow: Right spacing: 6 visible: false
                                    settings_reset_confirm_label := Label { text: "Reset all?" }
                                    settings_reset_yes := Button { text: "Reset" }
                                    settings_reset_cancel := ButtonFlat { text: "Cancel" }
                                }
                            }
                        }
                        about_panel := DsElevatedPanel {
                            width: 340 height: 180 flow: Down spacing: 6 padding: 14 visible: false
                            new_batch: false
                            draw_bg.color: theme.color_fg_app
                            about_title := Label { width: Fill height: Fit text: "About Glyf Editor" draw_text.text_style.font_size: theme.font_size_3 }
                            about_body := Label { width: Fill height: Fit text: "GLYF Editor
GLYF format v1
Desktop shell" draw_text.color: theme.color_label_inner_inactive }
                            about_close := ButtonFlatter { width: Fill text: "Close" }
                        }
                        shortcuts_panel := DsElevatedPanel {
                            width: 430 height: 220 flow: Down spacing: 5 padding: 14 visible: false
                            new_batch: false
                            draw_bg.color: theme.color_fg_app
                            shortcuts_title := Label { width: Fill height: Fit text: "Keyboard Shortcuts" draw_text.text_style.font_size: theme.font_size_3 }
                            shortcuts_body := Label { width: Fill height: Fit text: "Ctrl+N New · Ctrl+O Open · Ctrl+S Save
Ctrl+Shift+S Save As · Ctrl+W Close tab
Ctrl+Z/Y Undo/Redo · Ctrl+C/V Copy/Paste
Ctrl+Tab Switch tab · Alt+F4 Exit" draw_text.color: theme.color_label_inner_inactive }
                            shortcuts_close := ButtonFlatter { width: Fill text: "Close" }
                        }
                        close_confirm_panel := DsElevatedPanel {
                            width: 380 height: 190 flow: Down spacing: 8 padding: 14 visible: false
                            new_batch: false
                            draw_bg.color: theme.color_fg_app
                            close_confirm_title := Label { width: Fill height: Fit text: "Unsaved changes" draw_text.text_style.font_size: theme.font_size_3 }
                            close_confirm_body := Label { width: Fill height: Fit text: "Save changes before closing this tab?" draw_text.color: theme.color_label_inner_inactive }
                            close_confirm_actions := View {
                                width: Fill height: Fit flow: Right spacing: 8 align: Align {x: 1.0 y: 0.5}
                                close_confirm_cancel := ButtonFlatter { text: "Cancel" }
                                close_confirm_dont_save := ButtonFlatter { text: "Don't Save" }
                                close_confirm_save := Button { text: "Save" }
                            }
                        }
                    }
                    toolbar_tooltip := Tooltip {
                        content +: {
                            padding: Inset{left: 10 right: 10 top: 7 bottom: 7}
                            draw_bg +: { color: theme.color_bg_container border_color: theme.color_bevel radius: 5.0 }
                            tooltip_label +: {
                                width: Fit
                                draw_text +: { color: theme.color_label_inner text_style: theme.font_regular{font_size: 11} }
                            }
                        }
                    }
                }
            }
        }
    }
}
