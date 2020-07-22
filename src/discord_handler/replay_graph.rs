use plotters::{prelude::*, style::text_anchor::{Pos, HPos, VPos} /*style::RGBAColor*/};

const MARVELOUS_THRESHOLD: f32 = 0.0225;
const MARVELOUS_COLOR: RGBColor = RGBColor(0x99, 0xCC, 0xFF);
const PERFECT_THRESHOLD: f32 = 0.045;
const PERFECT_COLOR: RGBColor = RGBColor(0xF2, 0xCB, 0x30);
const GREAT_THRESHOLD: f32 = 0.090;
const GREAT_COLOR: RGBColor = RGBColor(0x14, 0xCC, 0x8F);
const GOOD_THRESHOLD: f32 = 0.135;
const GOOD_COLOR: RGBColor = RGBColor(0x1A, 0xB2, 0xFF);
const BAD_THRESHOLD: f32 = 0.180;
const BAD_COLOR: RGBColor = RGBColor(0xFF, 0x1A, 0xB3);
const MISS_COLOR: RGBColor = RGBColor(0xCC, 0x29, 0x29);

// const WIFESCORE_LINE_COLOR: RGBColor = RGBColor(255, 128, 0);
// const WIFESCORE_LINE_COLOR: RGBColor = RGBColor(255, 192, 128);
const WIFESCORE_LINE_COLOR: RGBColor = WHITE;

/// Takes a deviation in seconds, positive or negative, and generates the appropriate judgement
/// color
fn deviation_to_color(deviation: f32) -> RGBColor {
	let dev_abs = deviation.abs();
	if dev_abs < MARVELOUS_THRESHOLD {
		MARVELOUS_COLOR
	} else if dev_abs < PERFECT_THRESHOLD {
		PERFECT_COLOR
	} else if dev_abs < GREAT_THRESHOLD {
		GREAT_COLOR
	} else if dev_abs < GOOD_THRESHOLD {
		GOOD_COLOR
	} else if dev_abs < BAD_THRESHOLD {
		BAD_COLOR
	} else {
		MISS_COLOR
	}//.to_rgba().mix(0.5)
}

pub fn inner(
	replay: etternaonline_api::v2::Replay,
	output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
	let notes = replay.notes;

	let mut wifescores = Vec::new();
	let mut hit_wifescore_sum = 0.0;
	let mut min_wifescore = f32::INFINITY;
	let mut max_wifescore = f32::NEG_INFINITY;
	for note in &notes {
		let hit_wifescore = crate::wife::wife3(note.deviation as f32) as f32 * 100.0;
		hit_wifescore_sum += hit_wifescore;

		let wifescore = hit_wifescore_sum / (wifescores.len() + 1) as f32;
		wifescores.push(wifescore);

		if wifescore < min_wifescore { min_wifescore = wifescore }
		if wifescore > max_wifescore { max_wifescore = wifescore }
	}

	let mut chart_length = 0.0;
	for note in &notes {
		if note.time as f32 > chart_length {
			chart_length = note.time as f32;
		}
	}

	let root = BitMapBackend::new(output_path, (1290, 400)).into_drawing_area();
	root.fill(&BLACK)?;
	
	let wifescore_chart_x_range = 0.0f32..chart_length;
	let wifescore_chart_y_range = (min_wifescore - 1.0)..(max_wifescore + 1.0);

	let mut wifescore_chart = ChartBuilder::on(&root)
		.build_ranged(wifescore_chart_x_range.clone(), wifescore_chart_y_range.clone())?;

	let draw_horizontal_line = |height: f32, color: &RGBColor| {
		let path = PathElement::new(vec![
			// these multipliers and offsets are complete fucking guesswork and I have no idea why I
			// have to apply these calculations to me coordinates in order for plotters to display
			// them correctly AAAAAAAAAAAAAAAAA
			(0.0, height * 17.8 + 97.56),
			(chart_length, height * 17.8 + 97.56)
		], ShapeStyle {
			color: color.to_rgba().mix(0.3),
			filled: false,
			stroke_width: 1,
		});
		wifescore_chart.plotting_area().draw(&path)
	};
	
	// JAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANK
	// draw_horizontal_line(MARVELOUS_THRESHOLD, &MARVELOUS_COLOR)?;
	// draw_horizontal_line(-MARVELOUS_THRESHOLD, &MARVELOUS_COLOR)?;
	// draw_horizontal_line(PERFECT_THRESHOLD, &PERFECT_COLOR)?;
	// draw_horizontal_line(-PERFECT_THRESHOLD, &PERFECT_COLOR)?;
	// draw_horizontal_line(GREAT_THRESHOLD, &GREAT_COLOR)?;
	// draw_horizontal_line(-GREAT_THRESHOLD, &GREAT_COLOR)?;
	// draw_horizontal_line(GOOD_THRESHOLD, &GOOD_COLOR)?;
	// draw_horizontal_line(-GOOD_THRESHOLD, &GOOD_COLOR)?;
	// draw_horizontal_line(BAD_THRESHOLD, &BAD_COLOR)?;
	// draw_horizontal_line(-BAD_THRESHOLD, &BAD_COLOR)?;
	
	ChartBuilder::on(&root)
		.build_ranged(0.0f32..chart_length, -0.19..0.19f32)?
		.draw_series(notes.iter().map(|n| {
			let x = n.time as f32;
			let y = n.deviation as f32;

			EmptyElement::at((x, y)) + Circle::new(
				(0, 0),
				2,
				ShapeStyle::from(&deviation_to_color(y)).filled()
			)
		}))?;
	
	wifescore_chart
	.draw_series(LineSeries::new(
		notes.iter().zip(&wifescores).map(|(n, &w)| (n.time as f32, w)),
		&WIFESCORE_LINE_COLOR,
	))?;

	ChartBuilder::on(&root)
		.y_label_area_size(50)
		.build_ranged(wifescore_chart_x_range, wifescore_chart_y_range)?
		.configure_mesh()
		.disable_mesh()
		// .disable_x_mesh()
		// .line_style_1(&WHITE.mix(0.5))
		// .line_style_2(&TRANSPARENT)
		.disable_x_axis()
		.axis_style(&WIFESCORE_LINE_COLOR.to_rgba().mix(0.5))
		.y_label_style(TextStyle {
			color: WIFESCORE_LINE_COLOR.to_rgba().mix(0.8),
			pos: Pos::new(HPos::Center, VPos::Center),
			font: ("sans-serif", 12).into(),
		})
		.y_label_formatter(&|y| format!("{:.1}%", y))
		.y_labels(5)
		.draw()?;

	Ok(())
}

/// plotters did a GREAT fucking JOB of hiding their error types so that I'm **unable** to handle
/// them. For that reason, this has a String as an error type.
pub fn generate_replay_graph(
	replay: etternaonline_api::v2::Replay,
	output_path: &str
) -> Result<(), String> {
	inner(replay, output_path).map_err(|e| e.to_string())
}