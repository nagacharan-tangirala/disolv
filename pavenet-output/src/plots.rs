use krabmaga::{addplot, PlotData, DATA};

pub struct PlotConfig {
    plot_name: String,
    x_label: String,
    y_label: String,
    is_save: bool,
}

struct SimplePlotter {
    plot_config: Vec<PlotConfig>,
}

impl SimplePlotter {}
pub fn define_plot(plot_name: &str, x_label: &str, y_label: &str, is_save: bool) {
    let plot_name_str = plot_name.to_string();
    let x_label = x_label.to_string();
    let y_label = y_label.to_string();
    addplot! {
        plot_name_str.clone(),
        x_label,
        y_label,
        is_save
    }
}
