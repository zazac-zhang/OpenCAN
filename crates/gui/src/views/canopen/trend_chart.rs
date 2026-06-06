//! DS402 trend chart using iced Canvas.

use iced::widget::canvas::{self, Cache, Canvas, Geometry, LineCap, Path, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, mouse};

use crate::state::Message;

/// Trend chart data.
#[derive(Debug, Clone)]
pub struct TrendData {
    pub values: Vec<f32>,
    pub label: String,
    pub unit: String,
    pub min: f32,
    pub max: f32,
}

impl TrendData {
    pub fn new(label: String, unit: String) -> Self {
        Self {
            values: Vec::new(),
            label,
            unit,
            min: f32::MAX,
            max: f32::MIN,
        }
    }

    pub fn push(&mut self, value: f32) {
        self.values.push(value);
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        // Keep last 500 samples
        if self.values.len() > 500 {
            self.values.remove(0);
            // Recalculate min/max
            self.min = self.values.iter().cloned().fold(f32::MAX, f32::min);
            self.max = self.values.iter().cloned().fold(f32::MIN, f32::max);
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.min = f32::MAX;
        self.max = f32::MIN;
    }
}

/// Trend chart state.
#[derive(Debug)]
pub struct TrendChartState {
    pub position: TrendData,
    pub velocity: TrendData,
    pub torque: TrendData,
    pub show_position: bool,
    pub show_velocity: bool,
    pub show_torque: bool,
    cache: Cache,
}

impl Default for TrendChartState {
    fn default() -> Self {
        Self {
            position: TrendData::new("Position".to_string(), "counts".to_string()),
            velocity: TrendData::new("Velocity".to_string(), "counts/s".to_string()),
            torque: TrendData::new("Torque".to_string(), "‰".to_string()),
            show_position: true,
            show_velocity: true,
            show_torque: true,
            cache: Cache::new(),
        }
    }
}

impl TrendChartState {
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn push_position(&mut self, value: f32) {
        self.position.push(value);
        self.cache.clear();
    }

    pub fn push_velocity(&mut self, value: f32) {
        self.velocity.push(value);
        self.cache.clear();
    }

    pub fn push_torque(&mut self, value: f32) {
        self.torque.push(value);
        self.cache.clear();
    }
}

/// Trend chart program for iced Canvas.
pub struct TrendChart<'a> {
    state: &'a TrendChartState,
}

impl<'a> TrendChart<'a> {
    pub fn new(state: &'a TrendChartState) -> Self {
        Self { state }
    }
}

impl<'a, Message> canvas::Program<Message> for TrendChart<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.state.cache.draw(renderer, bounds.size(), |frame| {
            let width = bounds.width;
            let height = bounds.height;

            // Chart area (with margins for labels)
            let margin_left = 60.0;
            let margin_right = 20.0;
            let margin_top = 30.0;
            let margin_bottom = 30.0;

            let chart_width = width - margin_left - margin_right;
            let chart_height = height - margin_top - margin_bottom;

            if chart_width <= 0.0 || chart_height <= 0.0 {
                return;
            }

            let chart_origin = Point::new(margin_left, margin_top);

            // Background
            let bg = Path::rectangle(
                Point::new(0.0, 0.0),
                Size::new(width, height),
            );
            frame.fill(&bg, Color::from_rgb(0.1, 0.1, 0.12));

            // Chart background
            let chart_bg = Path::rectangle(
                chart_origin,
                Size::new(chart_width, chart_height),
            );
            frame.fill(&chart_bg, Color::from_rgb(0.15, 0.15, 0.18));

            // Grid lines
            let grid_stroke = Stroke::default()
                .with_width(0.5)
                .with_color(Color::from_rgba(0.5, 0.5, 0.5, 0.3));

            // Horizontal grid lines (5 lines)
            for i in 0..=4 {
                let y = margin_top + (i as f32 * chart_height / 4.0);
                let grid_line = Path::line(
                    Point::new(margin_left, y),
                    Point::new(width - margin_right, y),
                );
                frame.stroke(&grid_line, grid_stroke.clone());
            }

            // Vertical grid lines (10 lines)
            for i in 0..=10 {
                let x = margin_left + (i as f32 * chart_width / 10.0);
                let grid_line = Path::line(
                    Point::new(x, margin_top),
                    Point::new(x, height - margin_bottom),
                );
                frame.stroke(&grid_line, grid_stroke.clone());
            }

            // Draw data series
            let draw_series = |frame: &mut canvas::Frame, data: &[f32], color: Color, min: f32, max: f32| {
                if data.len() < 2 {
                    return;
                }

                let range = if max - min == 0.0 { 1.0 } else { max - min };
                let points: Vec<Point> = data
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        let x = margin_left + (i as f32 / (data.len() - 1).max(1) as f32) * chart_width;
                        let normalized = (v - min) / range;
                        let y = margin_top + chart_height - (normalized * chart_height);
                        Point::new(x, y)
                    })
                    .collect();

                if points.len() >= 2 {
                    let path = Path::new(|builder| {
                        if let Some(first) = points.first() {
                            builder.move_to(*first);
                            for point in points.iter().skip(1) {
                                builder.line_to(*point);
                            }
                        }
                    });
                    let stroke = Stroke::default()
                        .with_width(1.5)
                        .with_color(color)
                        .with_line_cap(LineCap::Round);
                    frame.stroke(&path, stroke);
                }
            };

            // Draw position (blue)
            if self.state.show_position && !self.state.position.values.is_empty() {
                draw_series(
                    frame,
                    &self.state.position.values,
                    Color::from_rgb(0.3, 0.5, 1.0),
                    self.state.position.min,
                    self.state.position.max,
                );
            }

            // Draw velocity (green)
            if self.state.show_velocity && !self.state.velocity.values.is_empty() {
                draw_series(
                    frame,
                    &self.state.velocity.values,
                    Color::from_rgb(0.3, 0.8, 0.3),
                    self.state.velocity.min,
                    self.state.velocity.max,
                );
            }

            // Draw torque (orange)
            if self.state.show_torque && !self.state.torque.values.is_empty() {
                draw_series(
                    frame,
                    &self.state.torque.values,
                    Color::from_rgb(1.0, 0.6, 0.2),
                    self.state.torque.min,
                    self.state.torque.max,
                );
            }

            // Y-axis labels
            let label_style = Text {
                size: 10.0.into(),
                color: Color::from_rgb(0.7, 0.7, 0.7),
                ..Text::default()
            };

            // Draw Y-axis labels for position (if shown)
            if self.state.show_position && self.state.position.values.len() > 0 {
                let min = self.state.position.min;
                let max = self.state.position.max;
                for i in 0..=4 {
                    let value = min + (i as f32 * (max - min) / 4.0);
                    let y = margin_top + chart_height - (i as f32 * chart_height / 4.0);
                    frame.fill_text(Text {
                        content: format!("{:.0}", value),
                        position: Point::new(5.0, y - 5.0),
                        ..label_style.clone()
                    });
                }
            }

            // X-axis label (time)
            frame.fill_text(Text {
                content: "Time →".to_string(),
                position: Point::new(width / 2.0 - 20.0, height - 5.0),
                ..label_style.clone()
            });

            // Legend
            let mut legend_x = margin_left + 10.0;
            let legend_y = 15.0;

            if self.state.show_position {
                frame.fill_text(Text {
                    content: format!("■ Position ({})", self.state.position.unit),
                    position: Point::new(legend_x, legend_y),
                    color: Color::from_rgb(0.3, 0.5, 1.0),
                    size: 11.0.into(),
                    ..Text::default()
                });
                legend_x += 100.0;
            }

            if self.state.show_velocity {
                frame.fill_text(Text {
                    content: format!("■ Velocity ({})", self.state.velocity.unit),
                    position: Point::new(legend_x, legend_y),
                    color: Color::from_rgb(0.3, 0.8, 0.3),
                    size: 11.0.into(),
                    ..Text::default()
                });
                legend_x += 100.0;
            }

            if self.state.show_torque {
                frame.fill_text(Text {
                    content: format!("■ Torque ({})", self.state.torque.unit),
                    position: Point::new(legend_x, legend_y),
                    color: Color::from_rgb(1.0, 0.6, 0.2),
                    size: 11.0.into(),
                    ..Text::default()
                });
            }
        });

        vec![geometry]
    }
}

/// Create a trend chart widget.
pub fn trend_chart<'a>(state: &'a TrendChartState) -> Element<'a, Message> {
    Canvas::new(TrendChart::new(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
