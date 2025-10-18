mod graph;
mod layout;
mod canvas;

pub use graph::{ErdGraph, TableNode, ColumnData, RelationshipEdge, RelationType};
pub use canvas::ErdCanvas;

// Window rendering entry point
pub fn render_window(erd_graph: ErdGraph, title: String) -> Result<(), Box<dyn std::error::Error>> {
    let window_title = if title.is_empty() {
        "FreeERD - ERD Viewer".to_string()
    } else {
        format!("FreeERD - {}", title)
    };
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title(window_title),
        ..Default::default()
    };
    
    let title_clone = title.clone();
    eframe::run_native(
        "FreeERD",
        options,
        Box::new(move |_cc| Ok(Box::new(ErdCanvas::new(erd_graph, title_clone)))),
    )?;
    
    Ok(())
}
