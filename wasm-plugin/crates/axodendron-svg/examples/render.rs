use std::env;
use std::fs;
use std::process::ExitCode;

use axodendron_core::{ValidationProfile, Vec3, parse_swc};
use axodendron_svg::{
    ColorMode, GeometryMode, RadiusMode, RenderOptions, SomaMode, View, render_svg,
};

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.len() < 2 || arguments.len() > 7 {
        eprintln!(
            "usage: cargo run -p axodendron-svg --example render -- \
             INPUT.swc OUTPUT.svg [xy|xz|yz|iso] [tapered|skeleton] [type|mono] \
             [readable|physical] [equivalent|encoded|raw]"
        );
        return ExitCode::from(2);
    }
    let source = match fs::read_to_string(&arguments[0]) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{}: {error}", arguments[0]);
            return ExitCode::FAILURE;
        }
    };
    let parsed = parse_swc(&source, ValidationProfile::IncfStrict);
    let Some(morphology) = parsed.morphology else {
        for diagnostic in parsed.diagnostics {
            eprintln!("{}: {}", diagnostic.code, diagnostic.message);
        }
        return ExitCode::FAILURE;
    };
    let view = match arguments.get(2).map(String::as_str).unwrap_or("xy") {
        "xy" => View::Xy,
        "xz" => View::Xz,
        "yz" => View::Yz,
        "iso" => View::Orthographic {
            direction: Vec3::new(1.0, 1.0, 1.0),
            up: Vec3::new(0.0, 0.0, 1.0),
        },
        value => {
            eprintln!("unknown view {value:?}; expected xy, xz, yz, or iso");
            return ExitCode::from(2);
        }
    };
    let geometry = match arguments.get(3).map(String::as_str).unwrap_or("tapered") {
        "tapered" => GeometryMode::Tapered,
        "skeleton" => GeometryMode::Skeleton,
        value => {
            eprintln!("unknown geometry {value:?}; expected tapered or skeleton");
            return ExitCode::from(2);
        }
    };
    let color = match arguments.get(4).map(String::as_str).unwrap_or("type") {
        "type" => ColorMode::ByType,
        "mono" => ColorMode::Uniform {
            color: "#111827".to_owned(),
        },
        value => {
            eprintln!("unknown color mode {value:?}; expected type or mono");
            return ExitCode::from(2);
        }
    };
    let radius_mode = match arguments.get(5).map(String::as_str).unwrap_or("readable") {
        "readable" => RadiusMode::Readable,
        "physical" => RadiusMode::Physical,
        value => {
            eprintln!("unknown radius mode {value:?}; expected readable or physical");
            return ExitCode::from(2);
        }
    };
    let soma_mode = match arguments.get(6).map(String::as_str).unwrap_or("equivalent") {
        "equivalent" => SomaMode::EquivalentSphere,
        "encoded" => SomaMode::Encoded,
        "raw" => SomaMode::RawPoints,
        value => {
            eprintln!("unknown soma mode {value:?}; expected equivalent, encoded, or raw");
            return ExitCode::from(2);
        }
    };
    let options = RenderOptions {
        width: 1200.0,
        height: 1200.0,
        padding: 36.0,
        background: Some("#ffffff".to_owned()),
        view,
        geometry,
        radius_mode,
        soma_mode,
        color,
        ..Default::default()
    };
    let document = match render_svg(&morphology, &options) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("render failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = fs::write(&arguments[1], document.svg) {
        eprintln!("{}: {error}", arguments[1]);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
