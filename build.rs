extern crate protoc_rust;

use shaderc;

use protoc_rust::Customize;
use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;

pub fn glsl_to_spirv(name: &str, source: &str, kind: shaderc::ShaderKind) -> Vec<u8> {
    Vec::from(
        shaderc::Compiler::new()
            .expect("compiler creation failed")
            .compile_into_spirv(source, kind, name, "main", None)
            .expect("compilation failed")
            .as_binary_u8(),
    )
}

fn main() {
    remove_dir_all("src/spirv").unwrap();
    create_dir("src/spirv").unwrap();
    File::create("src/spirv/vert.spirv")
        .unwrap()
        .write(&glsl_to_spirv(
            "graphics.vert",
            include_str!("src/glsl/graphics.vert"),
            shaderc::ShaderKind::Vertex,
        ))
        .unwrap();
    File::create("src/spirv/frag.spirv")
        .unwrap()
        .write(&glsl_to_spirv(
            "graphics.frag",
            include_str!("src/glsl/graphics.frag"),
            shaderc::ShaderKind::Fragment,
        ))
        .unwrap();

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["protos/fileformat.proto", "protos/osmformat.proto"],
        includes: &["protos"],
        customize: Customize {
            ..Default::default()
        },
    })
    .expect("protoc");
}
