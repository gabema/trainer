//! The goal-progress chart, replacing Chart.js and `chart-helper.js`.
//!
//! # What the chart shows
//!
//! Each bar is one activity type's total for the selected window, as a
//! percentage of that type's goal, drawn as a hundred-unit-tall bar positioned
//! so that **the zero line marks the goal**:
//!
//! ```text
//! positive type    bar spans [pct - 100, pct]   at goal its bottom sits on zero
//! negative type    bar spans [-pct, 100 - pct]  at goal its top sits on zero
//! ```
//!
//! So a positive bar clearing the line is a goal met, and a negative bar
//! dipping below it is a limit exceeded. That is why the value axis carries no
//! labels — the single line is the whole reading, and the C# hid the y ticks
//! for the same reason.
//!
//! # Why SVG rather than a chart library
//!
//! Chart.js came from a CDN and was never in the service worker's precache
//! list, so the chart did not render offline — in an app whose stated promise
//! is full offline function. Thirty lines of `<rect>` have no such failure mode.
//!
//! Theme colors come from CSS custom properties, so light and dark work through
//! the cascade. `chart-helper.js` instead read `data-bs-theme`, computed hex
//! colors in JS, and carried a `MutationObserver` plus a `themechange` listener
//! to repaint every chart when the theme flipped. None of that survives.
//!
//! # Deliberate differences
//!
//! * Chart.js drew automatic horizontal grid lines at computed tick positions.
//!   With the tick labels hidden those positions conveyed nothing, so this
//!   draws a fixed grid every fifty percent plus the emphasized zero line.
//! * Hovering a bar shows a native SVG tooltip rather than a rendered Chart.js
//!   one, which removes the tooltip plugin without removing the reading.
//! * An empty series renders a message. The C# left a blank 400px canvas.

use dioxus::prelude::*;
use trainer_core::models::NetBenefit;
use trainer_core::services::goal::{GoalProgress, chart_axis_limit};

const VIEW_WIDTH: f64 = 800.0;
const VIEW_HEIGHT: f64 = 400.0;
const MARGIN_X: f64 = 12.0;
const MARGIN_TOP: f64 = 12.0;
/// Room under the plot for the category labels.
const LABEL_BAND: f64 = 44.0;
/// Chart.js's default `barPercentage` times `categoryPercentage`.
const BAR_FILL: f64 = 0.72;
/// Spacing of the faint horizontal rules, in percentage points.
const GRID_STEP: f64 = 50.0;

const PLOT_TOP: f64 = MARGIN_TOP;
const PLOT_HEIGHT: f64 = VIEW_HEIGHT - MARGIN_TOP - LABEL_BAND;
const PLOT_LEFT: f64 = MARGIN_X;
const PLOT_WIDTH: f64 = VIEW_WIDTH - 2.0 * MARGIN_X;

/// One bar's rectangle plus the text that describes it.
struct Bar {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: String,
    label_x: f64,
    tooltip: String,
    fill: &'static str,
    stroke: &'static str,
}

/// Maps a percentage on the value axis to a y coordinate.
fn to_y(value: f64, limit: f64) -> f64 {
    PLOT_TOP + (limit - value) / (2.0 * limit) * PLOT_HEIGHT
}

/// Lays out the bars.
///
/// `Neutral` types never reach here, but an out-of-range `NetBenefit` can:
/// `System.Text.Json` accepts any integer for an enum, so a hand-edited import
/// can carry one. The C# compared `netBenefit.ToString()` against `"Positive"`
/// and `"Negative"`, so such a value produced no bar while still widening the
/// axis. That is reproduced rather than corrected — the axis is computed over
/// the whole series and the layout skips what it cannot place.
fn layout(series: &[GoalProgress], limit: f64) -> Vec<Bar> {
    let category = PLOT_WIDTH / series.len() as f64;
    let width = category * BAR_FILL;

    series
        .iter()
        .enumerate()
        .filter_map(|(index, progress)| {
            let (low, high, fill, stroke) = match progress.net_benefit {
                NetBenefit::Positive => (
                    progress.percentage - 100.0,
                    progress.percentage,
                    "var(--chart-positive)",
                    "var(--chart-positive-border)",
                ),
                NetBenefit::Negative => (
                    -progress.percentage,
                    100.0 - progress.percentage,
                    "var(--chart-negative)",
                    "var(--chart-negative-border)",
                ),
                NetBenefit::Neutral | NetBenefit::Other(_) => return None,
            };

            let center = PLOT_LEFT + category * (index as f64 + 0.5);
            let top = to_y(high, limit);
            Some(Bar {
                x: center - width / 2.0,
                y: top,
                width,
                height: to_y(low, limit) - top,
                label: progress.label.clone(),
                label_x: center,
                tooltip: format!("{}: {:.1}% of goal", progress.label, progress.percentage),
                fill,
                stroke,
            })
        })
        .collect()
}

#[component]
pub fn GoalChart(series: Vec<GoalProgress>) -> Element {
    if series.is_empty() {
        return rsx! {
            p { class: "text-muted mb-0", "No activity types with goals in this period." }
        };
    }

    let limit = chart_axis_limit(&series);
    let bars = layout(&series, limit);
    let zero_y = to_y(0.0, limit);

    // Faint rules either side of zero. Drawn from the step outwards so the
    // spacing is the same above and below, whatever the limit works out to.
    let grid_lines: Vec<f64> = std::iter::successors(Some(GRID_STEP), |v| Some(v + GRID_STEP))
        .take_while(|value| *value < limit)
        .flat_map(|value| [value, -value])
        .map(|value| to_y(value, limit))
        .collect();

    rsx! {
        svg {
            class: "goal-chart",
            view_box: "0 0 {VIEW_WIDTH} {VIEW_HEIGHT}",
            width: "100%",
            height: "400",
            preserve_aspect_ratio: "xMidYMid meet",
            role: "img",
            "aria-label": "Activity totals as a percentage of each goal",

            for y in grid_lines {
                line {
                    x1: "{PLOT_LEFT}",
                    x2: "{PLOT_LEFT + PLOT_WIDTH}",
                    y1: "{y}",
                    y2: "{y}",
                    stroke: "var(--chart-grid)",
                    stroke_width: "1",
                }
            }

            // The goal line. Bold, because it is the only value on the axis
            // that means anything.
            line {
                x1: "{PLOT_LEFT}",
                x2: "{PLOT_LEFT + PLOT_WIDTH}",
                y1: "{zero_y}",
                y2: "{zero_y}",
                stroke: "var(--chart-axis)",
                stroke_width: "2",
            }

            for bar in bars {
                g { key: "{bar.label}",
                    rect {
                        x: "{bar.x}",
                        y: "{bar.y}",
                        width: "{bar.width}",
                        height: "{bar.height}",
                        fill: bar.fill,
                        stroke: bar.stroke,
                        stroke_width: "1",
                        title { "{bar.tooltip}" }
                    }
                    text {
                        x: "{bar.label_x}",
                        y: "{VIEW_HEIGHT - LABEL_BAND + 20.0}",
                        text_anchor: "middle",
                        font_size: "13",
                        fill: "var(--chart-axis)",
                        "{bar.label}"
                    }
                }
            }
        }
    }
}
