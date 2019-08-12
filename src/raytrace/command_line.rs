// -----------------------------------------------------------------------------------------

use clap::{App, Arg};

// -----------------------------------------------------------------------------------------

pub fn parse<'a>() -> clap::ArgMatches<'a> {
	App::new("Ray Tracer")
		.version("0.0.0")
		.author("Thomas Sampson <tmsampson@gmail.com>")
		.arg(
			Arg::with_name("quality")
				.long("quality")
				.takes_value(true)
				.help("Quality preset")
				.default_value("default"),
		)
		.arg(
			Arg::with_name("output-file")
				.long("output-file")
				.takes_value(true)
				.help("Output image filename")
				.default_value("output.bmp"),
		)
		.arg(
			Arg::with_name("debug-normals")
				.long("debug-normals")
				.takes_value(false)
				.help("Debug render normals"),
		)
		.arg(
			Arg::with_name("seed")
				.long("seed")
				.takes_value(true)
				.help("Seed value for random number generator")
				.default_value("0"),
		)
		.get_matches()
}

// -----------------------------------------------------------------------------------------
