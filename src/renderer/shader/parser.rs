use nom::alphanumeric;
use std::str;

enum ShaderStage<'a> {
    Vertex(&'a str),
    Fragment(&'a str),
    Geometry(&'a str),
}

pub struct ShaderSrc<'a> {
    pub vertex: &'a str,
    pub fragment: &'a str,
    pub geometry: Option<&'a str>
}

pub fn parser(slice: &[u8]) -> ShaderSrc {
    let mut vertex = "";
    let mut fragment = "";
    let mut geometry = None;
    let shader_stages = shader_stages(slice).unwrap().1;
    for stage in &shader_stages {
        match stage {
            &ShaderStage::Vertex(src) => vertex = src,
            &ShaderStage::Fragment(src) => fragment = src,
            &ShaderStage::Geometry(src) => geometry = Some(src),
        }
    }
    ShaderSrc {vertex, fragment, geometry}
}

named!(word<&str>, map_res!(
        ws!(alphanumeric),
        str::from_utf8
));

named!(shader_stage<ShaderStage>, do_parse!(
    stage: word >>
    src: bracketed >>
    (match stage {
        "Vertex" => ShaderStage::Vertex(str::from_utf8(src).unwrap()),
        "Fragment" => ShaderStage::Fragment(str::from_utf8(src).unwrap()),
        "Geometry" => ShaderStage::Geometry(str::from_utf8(src).unwrap()),
        _ => panic!("")
    })
));

named!(shader_stages<Vec<ShaderStage>>, many1!(shader_stage));

named!(bracketed,
    delimited!(
        ws!(tag!("<")),
        take_until!(">"),
        ws!(tag!(">"))
    )
);