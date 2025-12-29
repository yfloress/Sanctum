use plotters::prelude::*;
use plotters::series::{AreaSeries, LineSeries};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use slint::Image;
use std::fs::{self, OpenOptions};
use std::path::Path;
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

pub struct ChartsService;

impl Default for ChartsService {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartsService {
    pub fn new() -> Self {
        Self
    }

    pub fn render_habit_radar_chart(
        &self,
        categories: &[(String, String, f32)],
    ) -> Option<Image> {
        if categories.is_empty() {
            return None;
        }

        let temp_svg = create_secure_temp_svg("sanctum_habits_radar")?;
        let root = SVGBackend::new(&temp_svg, (1400, 900)).into_drawing_area();
        root.fill(&RGBAColor(0, 0, 0, 0.0)).ok()?;

        let (root_w, root_h) = root.dim_in_pixel();
        let center = (root_w as i32 / 2, root_h as i32 / 2);
        let radius = (root_w.min(root_h) as f64) * 0.32;
        let axis_count = categories.len() as f64;
        let base_angle = -std::f64::consts::FRAC_PI_2;

        let grid_color = RGBColor(46, 46, 60);
        let axis_color = RGBColor(72, 84, 102);

        for level in 1..=4 {
            let r = radius * level as f64 / 4.0;
            let mut points: Vec<(i32, i32)> = Vec::new();
            for idx in 0..categories.len() {
                let angle = base_angle + (idx as f64) * (2.0 * std::f64::consts::PI / axis_count);
                let x = center.0 as f64 + r * angle.cos();
                let y = center.1 as f64 + r * angle.sin();
                points.push((x.round() as i32, y.round() as i32));
            }
            if let Some(first) = points.first().copied() {
                points.push(first);
            }
            root.draw(&PathElement::new(
                points,
                ShapeStyle::from(&grid_color).stroke_width(1),
            ))
            .ok()?;
        }

        for idx in 0..categories.len() {
            let angle = base_angle + (idx as f64) * (2.0 * std::f64::consts::PI / axis_count);
            let x = center.0 as f64 + radius * angle.cos();
            let y = center.1 as f64 + radius * angle.sin();
            root.draw(&PathElement::new(
                vec![center, (x.round() as i32, y.round() as i32)],
                ShapeStyle::from(&axis_color).stroke_width(2),
            ))
            .ok()?;
        }

        let mut data_points: Vec<(i32, i32)> = Vec::new();
        for (idx, (_, _, value)) in categories.iter().enumerate() {
            let angle = base_angle + (idx as f64) * (2.0 * std::f64::consts::PI / axis_count);
            let v = value.clamp(0.0, 1.0) as f64;
            let x = center.0 as f64 + radius * v * angle.cos();
            let y = center.1 as f64 + radius * v * angle.sin();
            data_points.push((x.round() as i32, y.round() as i32));
        }

        if data_points.len() >= 3 {
            let mut filled_points = data_points.clone();
            if let Some(first) = filled_points.first().copied() {
                filled_points.push(first);
            }
            root.draw(&Polygon::new(
                filled_points.clone(),
                RGBAColor(139, 92, 246, 0.25).filled(),
            ))
            .ok()?;
            root.draw(&PathElement::new(
                filled_points,
                ShapeStyle::from(&RGBColor(139, 92, 246)).stroke_width(3),
            ))
            .ok()?;
        }

        for ((_, color, _), point) in categories.iter().zip(data_points.iter()) {
            let rgb = rgb_from_hex(color);
            root.draw(&Circle::new(*point, 8, rgb.filled())).ok()?;
        }

        let label_color = RGBColor(148, 163, 184);
        for (idx, (label, _, _)) in categories.iter().enumerate() {
            let angle = base_angle + (idx as f64) * (2.0 * std::f64::consts::PI / axis_count);
            let x = center.0 as f64 + radius * 1.15 * angle.cos();
            let y = center.1 as f64 + radius * 1.15 * angle.sin();
            let cos = angle.cos();
            let sin = angle.sin();
            let hpos = if cos > 0.2 {
                HPos::Left
            } else if cos < -0.2 {
                HPos::Right
            } else {
                HPos::Center
            };
            let vpos = if sin > 0.2 {
                VPos::Top
            } else if sin < -0.2 {
                VPos::Bottom
            } else {
                VPos::Center
            };
            let style = ("sans-serif", 48)
                .into_font()
                .color(&label_color)
                .pos(Pos::new(hpos, vpos));
            root.draw(&Text::new(
                label.clone(),
                (x.round() as i32, y.round() as i32),
                style,
            ))
            .ok()?;
        }

        root.present().ok()?;
        render_svg_image(&temp_svg)
    }

    pub fn render_weekday_efficiency_chart(
        &self,
        weekdays: &[(String, f32, bool)],
    ) -> Option<Image> {
        if weekdays.is_empty() {
            return None;
        }

        let temp_svg = create_secure_temp_svg("sanctum_weekday_efficiency")?;
        let root = SVGBackend::new(&temp_svg, (1400, 600)).into_drawing_area();
        root.fill(&RGBAColor(0, 0, 0, 0.0)).ok()?;

        let padding: i32 = 80;
        let chart_width: i32 = 1400 - (padding * 2);
        let chart_height: i32 = 600 - (padding * 2);
        let bar_spacing: i32 = 20;
        let num_bars = weekdays.len() as i32;
        let bar_width: i32 = (chart_width - (bar_spacing * (num_bars - 1))) / num_bars;

        let max_avg = weekdays
            .iter()
            .map(|(_, avg, _)| *avg)
            .fold(0.0_f32, f32::max);
        if max_avg <= 0.0 {
            return None;
        }

        let accent_color = RGBColor(139, 92, 246);
        let gray_color = RGBColor(72, 84, 102);
        let text_color = RGBColor(148, 163, 184);
        let grid_color = RGBColor(46, 46, 60);

        for i in 0..5 {
            let y = padding + (chart_height * i / 4);
            root.draw(&PathElement::new(
                vec![(padding, y), (padding + chart_width, y)],
                ShapeStyle::from(&grid_color).stroke_width(1),
            ))
            .ok()?;
        }

        for (idx, (day_label, avg_count, is_best)) in weekdays.iter().enumerate() {
            let x = padding + (idx as i32 * (bar_width + bar_spacing));
            let bar_height = ((*avg_count / max_avg) * chart_height as f32) as i32;
            let y = padding + chart_height - bar_height;

            if bar_height > 0 {
                let bar_color = if *is_best { accent_color } else { gray_color };

                root.draw(&Rectangle::new(
                    [(x, y), (x + bar_width, padding + chart_height)],
                    bar_color.filled(),
                ))
                .ok()?;

                if *is_best {
                    root.draw(&Rectangle::new(
                        [
                            (x - 2, y - 2),
                            (x + bar_width + 2, padding + chart_height + 2),
                        ],
                        RGBAColor(139, 92, 246, 0.3).filled(),
                    ))
                    .ok()?;
                }
            }

            let label_y = padding + chart_height + 35;
            let label_x = x + bar_width / 2;
            let label_style = ("sans-serif", 42)
                .into_font()
                .color(&text_color)
                .pos(Pos::new(HPos::Center, VPos::Top));
            root.draw(&Text::new(
                day_label.clone(),
                (label_x, label_y),
                label_style,
            ))
            .ok()?;

            let value_text = format!("{:.1}", avg_count);
            let value_y = y - 12;
            let value_style = ("sans-serif", 36)
                .into_font()
                .color(if *is_best { &accent_color } else { &text_color })
                .pos(Pos::new(HPos::Center, VPos::Bottom));
            root.draw(&Text::new(value_text, (label_x, value_y), value_style))
                .ok()?;
        }

        root.present().ok()?;
        render_svg_image(&temp_svg)
    }

    pub fn render_portfolio_distribution_chart(&self, data: &[(String, f64)]) -> Option<Image> {
        if data.is_empty() {
            return None;
        }

        let total: f64 = data.iter().map(|(_, value)| *value).sum();
        if total <= 0.0 {
            return None;
        }

        let temp_svg = create_secure_temp_svg("sanctum_portfolio_dist")?;
        let root = SVGBackend::new(&temp_svg, (600, 600)).into_drawing_area();

        let sizes: Vec<f64> = data.iter().map(|(_, value)| *value).collect();
        let labels_empty: Vec<String> = vec![String::new(); data.len()];
        let colors: Vec<RGBColor> = data
            .iter()
            .enumerate()
            .map(|(idx, (label, _))| {
                let (r, g, b) = symbol_chart_color(label, idx);
                RGBColor(r, g, b)
            })
            .collect();

        let center = (300, 300);
        let radius = 220.0;
        let mut pie = Pie::new(&center, &radius, &sizes, &colors, &labels_empty);
        pie.start_angle(-90.0);
        pie.donut_hole(radius * 0.6);

        root.draw(&pie).ok()?;
        root.present().ok()?;

        render_svg_image(&temp_svg)
    }

    pub fn render_portfolio_trend_chart(&self, data: &[(String, f64, f64)]) -> Option<Image> {
        if data.len() < 2 {
            return None;
        }

        let mut min_val = f64::MAX;
        let mut max_val = 0.0_f64;

        for (_, total_value, total_cost) in data {
            if *total_value > max_val {
                max_val = *total_value;
            }
            if *total_cost > max_val {
                max_val = *total_cost;
            }
            if *total_value < min_val {
                min_val = *total_value;
            }
            if *total_cost < min_val {
                min_val = *total_cost;
            }
        }

        if max_val <= 0.0 {
            return None;
        }

        let padding = ((max_val - min_val) * 0.1).max(max_val * 0.05);
        let lower = (min_val - padding).max(0.0);
        let upper = max_val + padding;

        let temp_svg = create_secure_temp_svg("sanctum_portfolio_trend")?;
        let root = SVGBackend::new(&temp_svg, (1800, 520)).into_drawing_area();
        root.fill(&RGBColor(10, 10, 15)).ok()?;

        let x_max = (data.len() - 1) as i32;
        let mut chart = ChartBuilder::on(&root)
            .margin(18)
            .build_cartesian_2d(0..x_max, lower..upper)
            .ok()?;

        chart
            .configure_mesh()
            .disable_mesh()
            .disable_x_axis()
            .disable_y_axis()
            .draw()
            .ok()?;

        let value_points: Vec<(i32, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, total_value, _))| (i as i32, *total_value))
            .collect();
        let cost_points: Vec<(i32, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, (_, _, total_cost))| (i as i32, *total_cost))
            .collect();

        chart
            .draw_series(AreaSeries::new(
                value_points.iter().copied(),
                lower,
                RGBColor(139, 92, 246).mix(0.2),
            ))
            .ok()?;

        chart
            .draw_series(LineSeries::new(
                value_points.iter().copied(),
                ShapeStyle::from(&RGBColor(139, 92, 246)).stroke_width(4),
            ))
            .ok()?;

        chart
            .draw_series(LineSeries::new(
                cost_points.iter().copied(),
                ShapeStyle::from(&RGBColor(148, 163, 184)).stroke_width(2),
            ))
            .ok()?;

        root.present().ok()?;

        render_svg_image(&temp_svg)
    }

    pub fn chart_color_for_symbol(&self, symbol: &str, index: usize) -> (u8, u8, u8) {
        symbol_chart_color(symbol, index)
    }
}

fn rgb_from_hex(hex: &str) -> RGBColor {
    if let Some(stripped) = hex.strip_prefix('#')
        && stripped.len() == 6
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&stripped[0..2], 16),
            u8::from_str_radix(&stripped[2..4], 16),
            u8::from_str_radix(&stripped[4..6], 16),
        )
    {
        return RGBColor(r, g, b);
    }
    RGBColor(139, 92, 246)
}

fn create_secure_temp_svg(prefix: &str) -> Option<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    for _ in 0..8 {
        let name = format!("{}_{}.svg", prefix, Uuid::new_v4());
        let path = temp_dir.join(name);

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);

        match options.open(&path) {
            Ok(_) => return Some(path),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return None,
        }
    }
    None
}

fn render_svg_image(temp_svg: &Path) -> Option<Image> {
    let mut fontdb = fontdb::Database::new();
    let font_path = std::path::PathBuf::from("ui/fonts/DejaVuSans.ttf");
    if font_path.exists() {
        fontdb.load_font_file(&font_path).ok()?;
    } else {
        fontdb.load_system_fonts();
    }

    fontdb.set_serif_family("DejaVu Sans");
    fontdb.set_sans_serif_family("DejaVu Sans");
    fontdb.set_monospace_family("DejaVu Sans");

    let svg_data = match fs::read_to_string(temp_svg) {
        Ok(data) => data,
        Err(_) => {
            let _ = fs::remove_file(temp_svg);
            return None;
        }
    };
    let opt = usvg::Options {
        fontdb: std::sync::Arc::new(fontdb),
        ..Default::default()
    };
    let tree = match usvg::Tree::from_str(&svg_data, &opt) {
        Ok(tree) => tree,
        Err(_) => {
            let _ = fs::remove_file(temp_svg);
            return None;
        }
    };

    let final_svg = match create_secure_temp_svg("sanctum_chart_render") {
        Some(path) => path,
        None => {
            let _ = fs::remove_file(temp_svg);
            return None;
        }
    };

    let write_result = fs::write(&final_svg, tree.to_string(&usvg::WriteOptions::default()));
    let image = if write_result.is_ok() {
        Image::load_from_path(&final_svg).ok()
    } else {
        None
    };

    let _ = fs::remove_file(temp_svg);
    let _ = fs::remove_file(&final_svg);
    image
}

fn fallback_chart_color(index: usize) -> (u8, u8, u8) {
    match index % 6 {
        0 => (139, 92, 246),
        1 => (236, 72, 153),
        2 => (56, 189, 248),
        3 => (34, 197, 94),
        4 => (245, 158, 11),
        _ => (168, 85, 247),
    }
}

fn symbol_chart_color(symbol: &str, index: usize) -> (u8, u8, u8) {
    match symbol.to_uppercase().as_str() {
        "BTC" => (247, 147, 26),
        "ETH" => (98, 126, 234),
        "USDT" => (38, 161, 123),
        "USDC" => (39, 117, 202),
        "BNB" => (243, 186, 47),
        "SOL" => (20, 241, 149),
        "XMR" => (255, 102, 0),
        "LTC" => (191, 191, 191),
        "ADA" => (0, 51, 173),
        "DOGE" => (194, 166, 51),
        "XRP" => (0, 136, 204),
        "MATIC" => (130, 71, 229),
        "DOT" => (230, 0, 122),
        "AVAX" => (232, 65, 66),
        _ => fallback_chart_color(index),
    }
}
