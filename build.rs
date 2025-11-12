use anyhow::{bail, Context, Result};
use glob::glob;
use naga::back::wgsl;
use naga::front::glsl::Options;
use naga::front::glsl::Frontend;
use std::{fs::read_to_string, path::PathBuf};

pub fn load_shader(src_path: PathBuf) -> anyhow::Result<()> {
    let extension = src_path
        .extension()
        .context("File has no extension")?
        .to_str()
        .context("Extension cannot be converted to &str")?;
    let kind = match extension {
        "vert" => naga::ShaderStage::Vertex,
        "frag" => naga::ShaderStage::Fragment,
        "comp" => naga::ShaderStage::Compute,
        _ => bail!("Unsupported shader: {}", src_path.display()),
    };

    let src = read_to_string(src_path.clone())?;
    let wgsl_path = src_path.with_extension(format!("{}.wgsl", extension));

    let mut frontend = Frontend::default();
    let options = Options::from(kind);
    let module = match frontend.parse(&options, &src) {
        Ok(it) => it,
        Err(errors) => {
            bail!(
                "Failed to compile shader: {}\nErrors:\n{:#?}",
                src_path.display(),
                errors
            );
        }
    };

    let flags = naga::valid::ValidationFlags::all();
    let info =
        naga::valid::Validator::new(flags, naga::valid::Capabilities::empty()).validate(&module)?;
    std::fs::write(
        wgsl_path,
        wgsl::write_string(&module, &info, wgsl::WriterFlags::all())?,
    )?;

    Ok(())
}

fn main() -> Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/");

    // Collect all shaders recursively within /res/
    let shader_paths = {
        let mut data = Vec::new();
        data.extend(glob("./res/**/*.vert")?);
        data.extend(glob("./res/**/*.frag")?);
        data.extend(glob("./res/**/*.comp")?);
        data
    };

    // Process shaders
    for glob_result in shader_paths {
        let shader_path = glob_result?;
        load_shader(shader_path)?;
    }

    Ok(())
}
