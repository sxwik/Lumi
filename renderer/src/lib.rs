use egui::{Color32, FontId, RichText, Ui, Vec2};
use lumi_parser::{ElementType, LumiNode};

#[derive(Default)]
pub struct RenderOptions {
    pub pending_navigation: Option<String>,
}

pub fn render_page(ui: &mut Ui, root: &LumiNode, options: &mut RenderOptions) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);
        render_node(ui, root, options);
    });
}

fn render_node(ui: &mut Ui, node: &LumiNode, options: &mut RenderOptions) {
    match &node.element_type {
        ElementType::Page => {
            for child in &node.children {
                render_node(ui, child, options);
            }
        }
        ElementType::Title => {
            let text = extract_text(node);
            ui.heading(
                RichText::new(text)
                    .size(28.0)
                    .strong()
                    .color(Color32::from_rgb(90, 160, 255)),
            );
            ui.add_space(8.0);
        }
        ElementType::Heading => {
            let text = extract_text(node);
            let size = node
                .get_attr("size")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(22.0);
            ui.label(
                RichText::new(text)
                    .size(size)
                    .strong()
                    .color(Color32::from_rgb(220, 225, 235)),
            );
        }
        ElementType::Paragraph => {
            let text = extract_text(node);
            ui.label(
                RichText::new(text)
                    .size(15.0)
                    .color(Color32::from_rgb(180, 190, 205)),
            );
        }
        ElementType::Text => {
            let text = node.value.as_deref().unwrap_or("");
            ui.label(RichText::new(text).size(15.0));
        }
        ElementType::Button => {
            let text = extract_text(node);
            let goto_url = node
                .get_attr("goto")
                .or_else(|| {
                    node.children.iter().find_map(|c| c.get_attr("goto"))
                });

            let btn = egui::Button::new(
                RichText::new(if text.is_empty() { "Click" } else { &text })
                    .size(15.0)
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(60, 110, 220))
            .rounding(6.0);

            if ui.add(btn).clicked() {
                if let Some(target) = goto_url {
                    options.pending_navigation = Some(target.to_string());
                }
            }
        }
        ElementType::List => {
            ui.indent("lumi_list", |ui| {
                for child in &node.children {
                    render_node(ui, child, options);
                }
            });
        }
        ElementType::Item => {
            let text = extract_text(node);
            ui.horizontal(|ui| {
                ui.label(RichText::new("•").color(Color32::from_rgb(100, 180, 255)));
                ui.label(RichText::new(text).size(15.0));
            });
        }
        ElementType::Container => {
            egui::Frame::none()
                .fill(Color32::from_rgb(25, 30, 42))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    for child in &node.children {
                        render_node(ui, child, options);
                    }
                });
        }
        ElementType::Row => {
            ui.horizontal(|ui| {
                for child in &node.children {
                    render_node(ui, child, options);
                }
            });
        }
        ElementType::Column => {
            ui.vertical(|ui| {
                for child in &node.children {
                    render_node(ui, child, options);
                }
            });
        }
        ElementType::Divider => {
            ui.separator();
        }
        ElementType::Badge => {
            let text = extract_text(node);
            egui::Frame::none()
                .fill(Color32::from_rgb(40, 60, 90))
                .rounding(4.0)
                .inner_margin(Vec2::new(6.0, 3.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(text).size(12.0).color(Color32::from_rgb(120, 200, 255)));
                });
        }
        ElementType::CodeBlock => {
            let text = extract_text(node);
            egui::Frame::none()
                .fill(Color32::from_rgb(18, 20, 28))
                .rounding(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(text)
                            .font(FontId::monospace(13.0))
                            .color(Color32::from_rgb(150, 220, 160)),
                    );
                });
        }
        _ => {
            for child in &node.children {
                render_node(ui, child, options);
            }
        }
    }
}

fn extract_text(node: &LumiNode) -> String {
    if let Some(ref val) = node.value {
        return val.clone();
    }
    for child in &node.children {
        if child.element_type == ElementType::Text {
            if let Some(ref val) = child.value {
                return val.clone();
            }
        }
        if let Some(ref val) = child.value {
            return val.clone();
        }
    }
    String::new()
}
