use crate::model::*;
use crate::texture::*;
use nom::*;

#[derive(Debug)]
pub enum TextureFormat {
    I4,
    I8,
    A4,
    A8,
    IA4,
    IA8,
    RG8,
    RGB565,
    RGB8,
    RGB5A1,
    RGBA4,
    RGBA8,
}

impl TextureFormat {
    fn from_code(code: u8) -> Option<TextureFormat> {
        match code {
            0 => Some(TextureFormat::I4),
            1 => Some(TextureFormat::I8),
            2 => Some(TextureFormat::A4),
            3 => Some(TextureFormat::A8),
            4 => Some(TextureFormat::IA4),
            5 => Some(TextureFormat::IA8),
            6 => Some(TextureFormat::RG8),
            7 => Some(TextureFormat::RGB565),
            8 => Some(TextureFormat::RGB8),
            9 => Some(TextureFormat::RGB5A1),
            10 => Some(TextureFormat::RGBA4),
            11 => Some(TextureFormat::RGBA8),
            _ => None
        }
    }

    fn bit_per_pixel(&self) -> usize {
        match self {
            TextureFormat::I4 => 4,
            TextureFormat::I8 => 8,
            TextureFormat::A4 => 4,
            TextureFormat::A8 => 8,
            TextureFormat::IA4 => 8,
            TextureFormat::IA8 => 16,
            TextureFormat::RG8 => 16,
            TextureFormat::RGB565 => 16,
            TextureFormat::RGB8 => 24,
            TextureFormat::RGB5A1 => 16,
            TextureFormat::RGBA4 => 16,
            TextureFormat::RGBA8 => 32,
        }
    }

    fn decode_pixel_pair(&self, source: &[u8]) -> [u8; 8] {
        let convert1 = |v: u8| -> u8 {v * 255};
        let convert4 = |v: u8| -> u8 {(v << 4) | v};
        let convert5 = |v: u8| -> u8 {(v << 3) | (v >> 2)};
        let convert6 = |v: u8| -> u8 {(v << 2) | (v >> 4)};
        let o = 0u8;
        let i = 255u8;
        match self {
            TextureFormat::I4 => {
                let x = convert4(source[0] & 0xF);
                let y = convert4(source[0] >> 4);
                [x, x, x, i, y, y, y, i]
            },
            TextureFormat::I8 => {
                let x = source[0];
                let y = source[1];
                [x, x, x, i, y, y, y, i]
            },
            TextureFormat::A4 => {
                let x = convert4(source[0] & 0xF);
                let y = convert4(source[0] >> 4);
                [o, o, o, x, o, o, o, y]
            },
            TextureFormat::A8 => {
                let x = source[0];
                let y = source[1];
                [o, o, o, x, o, o, o, y]
            },
            TextureFormat::IA4 => {
                let xa = convert4(source[0] & 0xF);
                let xi = convert4(source[0] >> 4);
                let ya = convert4(source[1] & 0xF);
                let yi = convert4(source[1] >> 4);
                [xi, xi, xi, xa, yi, yi, yi, ya]
            },
            TextureFormat::IA8 => {
                let xa = source[0];
                let xi = source[1];
                let ya = source[2];
                let yi = source[3];
                [xi, xi, xi, xa, yi, yi, yi, ya]
            },
            TextureFormat::RG8 => {
                let xg = source[0];
                let xr = source[1];
                let yg = source[2];
                let yr = source[3];
                [xr, xg, o, i, yr, yg, o, i]
            },
            TextureFormat::RGB565 => {
                let mut dest = [0, 0, 0, 0, 0, 0, 0, 0];
                let decode = |s: &[u8], d: &mut[u8]| {
                    let r = convert5(s[1] >> 3);
                    let g = convert6(((s[1] & 0b111) << 3) | (s[0] >> 5));
                    let b = convert5(s[0] & 0b11111);
                    d.copy_from_slice(&[r, g, b, i]);
                };
                decode(&source[0 .. 2], &mut dest[0 .. 4]);
                decode(&source[2 .. 4], &mut dest[4 .. 8]);
                dest
            },
            TextureFormat::RGB8 => {
                [source[2], source[1], source[0], i,
                    source[5], source[4], source[3], i]
            },
            TextureFormat::RGB5A1 => {
                let mut dest = [0, 0, 0, 0, 0, 0, 0, 0];
                let decode = |s: &[u8], d: &mut[u8]| {
                    let r = convert5(s[1] >> 3);
                    let g = convert5(((s[1] & 0b111) << 2) | (s[0] >> 6));
                    let b = convert5((s[0] & 0b111110) >> 1);
                    let a = convert1(s[0] & 1);
                    d.copy_from_slice(&[r, g, b, a]);
                };
                decode(&source[0 .. 2], &mut dest[0 .. 4]);
                decode(&source[2 .. 4], &mut dest[4 .. 8]);
                dest
            }
            TextureFormat::RGBA4 => {
                let mut dest = [0, 0, 0, 0, 0, 0, 0, 0];
                let decode = |s: &[u8], d: &mut[u8]| {
                    let r = convert4(s[1] >> 4);
                    let g = convert4(s[1] & 0b1111);
                    let b = convert4(s[0] >> 4);
                    let a = convert4(s[0] & 0b1111);
                    d.copy_from_slice(&[r, g, b, a]);
                };
                decode(&source[0 .. 2], &mut dest[0 .. 4]);
                decode(&source[2 .. 4], &mut dest[4 .. 8]);
                dest
            }
            TextureFormat::RGBA8 => {
                [source[3], source[2], source[1], source[0],
                    source[7], source[6], source[5], source[4]]
            },
        }
    }
}

impl WrapMode {
    fn from_code(code: u8) -> Option<WrapMode> {
        match code {
            0 => Some(WrapMode::Edge),
            1 => Some(WrapMode::Repeat),
            2 => Some(WrapMode::Mirror),
            _ => None
        }
    }
}

fn padded_size(s: u16) -> usize {
    let result = s.next_power_of_two() as usize;
    if result < 8 {
        8
    } else {
        result
    }
}

#[derive(Debug)]
pub struct RawTexture {
    pub width: u16,
    pub height: u16,
    pub format: TextureFormat,
    pub wrap_u: WrapMode,
    pub wrap_v: WrapMode,
    pub pixels: Vec<u8>,
}

impl RawTexture {
    fn decode(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::with_capacity((self.width as usize) * (self.height as usize) * 4);
        let padded_width = padded_size(self.width);
        let pair_size = self.format.bit_per_pixel() * 2 / 8;
        let xlut = [0x00, 0x01, 0x04, 0x05, 0x10, 0x11, 0x14, 0x15];
        let ylut = [0x00, 0x02, 0x08, 0x0a, 0x20, 0x22, 0x28, 0x2a];
        for y in 0 .. self.height {
            for x in (0 .. self.width).step_by(2) {
                let my = self.height - y - 1;
                let cx = x / 8;
                let cy = my / 8;
                let fx = x % 8;
                let fy = my % 8;
                let fo = xlut[fx as usize] + ylut[fy as usize];
                let o = (((cx as usize) + (cy as usize) * padded_width / 8) * 64 + fo) / 2;
                result.extend_from_slice(&self.format.decode_pixel_pair(
                    &self.pixels[pair_size * o .. pair_size * (o + 1)])[..]);
            }
        }
        result
    }

    fn bake(&self) -> Texture {
        let decoded = self.decode();
        Texture::new(self.width as usize, self.height as usize, &decoded[..], &self.wrap_u, &self.wrap_v)
    }
}

named!(parse_texture<&[u8], RawTexture>,
    do_parse!(
        width: le_u16 >>
        height: le_u16 >>
        tag!(b"\x01") >>
        format: map_opt!(le_u8, TextureFormat::from_code) >>
        wrap_u: map_opt!(le_u8, WrapMode::from_code) >>
        wrap_v: map_opt!(le_u8, WrapMode::from_code) >>
        pixels_data: take!(padded_size(width) * padded_size(height) * format.bit_per_pixel() / 8) >>
        (RawTexture{width, height, format, wrap_u, wrap_v, pixels: pixels_data.to_vec()})
    )
);

#[derive(Debug)]
pub enum AttributeMode {
    None,
    Common,
    Individual,
}

impl AttributeMode {
    pub fn from_count(attribute_count: u16) -> AttributeMode {
        if attribute_count == 0 {
            AttributeMode::None
        } else if attribute_count == 1 {
            AttributeMode::Common
        } else {
            AttributeMode::Individual
        }
    }
}

#[derive(Debug)]
pub struct RawModel {
    pub normal_mode: AttributeMode,
    pub texcoord_mode: AttributeMode,
    pub vertex_list: Vec<u8>,
    pub default_normal: (i16, i16, i16),
    pub default_texcoord: (i16, i16),
    pub index_list: Vec<u8>,
}

impl RawModel {
    pub fn bake(&self) -> Model {
        let mut attribute_map = vec!((0u32, Attribute::Varying(VaryingAttribute{
            dimension: 3,
            data_type: AttributeType::Short,
            offset: 0
        })));
        let mut stride = 6;

        stride += match self.normal_mode {
            AttributeMode::None => 0,
            AttributeMode::Common => {
                attribute_map.push((1u32, Attribute::Short3(self.default_normal)));
                0
            },
            AttributeMode::Individual => {
                attribute_map.push((1u32, Attribute::Varying(VaryingAttribute{
                    dimension: 3,
                    data_type: AttributeType::Short,
                    offset: stride
                })));
                6
            },
        };

        stride += match self.texcoord_mode {
            AttributeMode::None => 0,
            AttributeMode::Common => {
                attribute_map.push((2u32, Attribute::Short2(self.default_texcoord)));
                0
            },
            AttributeMode::Individual => {
                attribute_map.push((2u32, Attribute::Varying(VaryingAttribute{
                    dimension: 2,
                    data_type: AttributeType::Short,
                    offset: stride
                })));
                4
            },
        };

        Model::new(&self.vertex_list[..], &self.index_list[..], attribute_map, stride)
    }
}

named!(parse_model<&[u8], RawModel>,
    do_parse!(
        vertex_count: le_u16 >>
        normal_count: le_u16 >>
        texcoord_count: le_u16 >>
        index_list_count: le_u16 >>
        vertex_list: count!(le_u8, (vertex_count * (
            6 +
            (if normal_count > 1 {6} else {0}) +
            (if texcoord_count > 1 {4} else {0}))) as usize
        ) >>
        default_normal: switch!(value!(normal_count == 1),
            true => tuple!(le_i16, le_i16, le_i16) |
            false => value!((0i16, 0i16, 0i16))
        ) >>
        default_texcoord: switch!(value!(texcoord_count == 1),
            true => tuple!(le_i16, le_i16) |
            false => value!((0i16, 0i16))
        ) >>
        index_list: switch!(value!(index_list_count == 1),
            true => do_parse!(
                tag!(b"\x04\x00") >>
                index_count: le_u16 >>
                index: count!(le_u8, index_count as usize) >>
                (index)
            )|
            false => value!(vec!())
        ) >>
        (RawModel{
            normal_mode: AttributeMode::from_count(normal_count),
            texcoord_mode: AttributeMode::from_count(texcoord_count),
            vertex_list,
            default_normal,
            default_texcoord,
            index_list
        })
    )
);

#[derive(Debug)]
pub struct FaceConfig {
    pub hair_pos: (f32, f32, f32),
    pub nose_pos: (f32, f32, f32),
    pub beard_pos: (f32, f32, f32),
}

named!(parse_face_config<&[u8], FaceConfig>,
    do_parse!(
        hair_pos: tuple!(le_f32, le_f32, le_f32) >>
        nose_pos: tuple!(le_f32, le_f32, le_f32) >>
        beard_pos: tuple!(le_f32, le_f32, le_f32) >>
        (FaceConfig{hair_pos, nose_pos, beard_pos})
    )
);

#[derive(Debug)]
struct AssetHeader {
    pub version: u16,
    pub section_offsets: Vec<u32>,
}

named!(parse_header<&[u8], AssetHeader>,
    do_parse!(
        section_count: le_u16 >>
        version: le_u16 >>
        section_offsets: count!(le_u32, section_count as usize) >>
        (AssetHeader{version, section_offsets})
    )
);

#[derive(Debug)]
struct SectionHeader {
    pub item_offsets: Vec<u32>,
}

named!(parse_section_header<&[u8], SectionHeader>,
    do_parse!(
        item_count: le_u16 >>
        take!(2) >> // skip buffer size
        item_offsets: count!(le_u32, (item_count + 1) as usize) >>
        (SectionHeader{item_offsets})
    )
);

#[derive(Debug)]
pub struct Asset {
    pub version: u16,

    pub beard_models: Vec<Option<Model>>,
    pub accessory_models: Vec<Option<Model>>,
    pub face_models: Vec<Option<Model>>,
    pub scalp_models: Vec<Option<Model>>,
    pub glass_models: Vec<Option<Model>>,
    pub hair_models: Vec<Option<Model>>,
    pub face_canvas_models: Vec<Option<Model>>,
    pub nose_canvas_models: Vec<Option<Model>>,
    pub nose_models: Vec<Option<Model>>,

    pub accessory_textures: Vec<Option<Texture>>,
    pub eye_textures: Vec<Option<Texture>>,
    pub eyebrow_textures: Vec<Option<Texture>>,
    pub beard_textures: Vec<Option<Texture>>,
    pub wrinkle_textures: Vec<Option<Texture>>,
    pub makeup_textures: Vec<Option<Texture>>,
    pub glass_textures: Vec<Option<Texture>>,
    pub mole_textures: Vec<Option<Texture>>,
    pub lip_textures: Vec<Option<Texture>>,
    pub mustache_textures: Vec<Option<Texture>>,
    pub nose_textures: Vec<Option<Texture>>,

    pub face_configs: Vec<Option<FaceConfig>>,
}

impl Asset {
    pub fn from_bytes(bytes: &[u8]) -> Option<Asset> {
        let header = match parse_header(bytes) {
            Err(_) => return None,
            Ok((_, header)) => header
        };

        #[derive(Debug)]
        enum Item {
            Model(Model),
            Texture(Texture),
        }

        let mut section_list = Vec::<Vec<Option<Item>>>::with_capacity(header.section_offsets.len());
        let mut face_configs = Vec::<Option<FaceConfig>>::new();
        for section in 0 .. header.section_offsets.len() {
            let (item_data_chunk, section_header) =
            match parse_section_header(&bytes[header.section_offsets[section] as usize ..]) {
                Err(_) => return None,
                Ok(result) => result
            };
            let item_count = section_header.item_offsets.len() - 1;
            let mut item_list = Vec::<Option<Item>>::with_capacity(item_count);
            for item in 0 .. item_count {
                let mut item_k = item;
                let mut begin = section_header.item_offsets[item_k];
                let redirect = begin >> 22;
                if redirect != 0 {
                    item_k = (redirect - 1) as usize;
                    begin = section_header.item_offsets[item_k];
                }
                begin &= 0x3FFFFF;
                let end = section_header.item_offsets[item_k + 1] & 0x3FFFFF;
                let item_data = &item_data_chunk[begin as usize .. end as usize];

                if item_data.is_empty() {
                    item_list.push(None);
                    if section == 2 {
                        face_configs.push(None);
                    }
                    continue;
                }

                if section >= 9 {
                    let (rest, texture) = match parse_texture(item_data) {
                        Err(_) => return None,
                        Ok(t) => t
                    };
                    if rest.len() >= 4 {
                        return None
                    }
                    item_list.push(Some(Item::Texture(texture.bake())));
                } else {
                    let model_data = match section {
                        2 => {
                            let (model_data, face_config) = match parse_face_config(item_data) {
                                Err(_) => return None,
                                Ok(c) => c
                            };
                            face_configs.push(Some(face_config));
                            model_data
                        }
                        5 => &item_data[0x48..],
                        _ => item_data
                    };
                    let (rest, model) = match parse_model(model_data) {
                        Err(_) => return None,
                        Ok(m) => m
                    };
                    if section == 6 {
                        let cover_data_len = model.index_list.len() / 3 * 2;
                        if rest.len() < cover_data_len || rest.len() >= cover_data_len + 4 {
                            return None
                        }
                    } else if rest.len() >= 4 {
                        return None
                    }
                    item_list.push(Some(Item::Model(model.bake())));
                }

            }
            section_list.push(item_list);
        }

        let unpack_model = |item: Option<Item>| -> Option<Model> {
            match item {
                None => None,
                Some(Item::Model(model)) => Some(model),
                _ => None
            }
        };

        let unpack_texture = |item: Option<Item>| -> Option<Texture> {
            match item {
                None => None,
                Some(Item::Texture(texture)) => Some(texture),
                _ => None
            }
        };

        let asset = Asset {
            version: header.version,
            beard_models: section_list[0].drain(..).map(unpack_model).collect(),
            accessory_models: section_list[1].drain(..).map(unpack_model).collect(),
            face_models: section_list[2].drain(..).map(unpack_model).collect(),
            scalp_models: section_list[3].drain(..).map(unpack_model).collect(),
            glass_models: section_list[4].drain(..).map(unpack_model).collect(),
            hair_models: section_list[5].drain(..).map(unpack_model).collect(),
            face_canvas_models: section_list[6].drain(..).map(unpack_model).collect(),
            nose_canvas_models: section_list[7].drain(..).map(unpack_model).collect(),
            nose_models: section_list[8].drain(..).map(unpack_model).collect(),

            accessory_textures: section_list[9].drain(..).map(unpack_texture).collect(),
            eye_textures: section_list[10].drain(..).map(unpack_texture).collect(),
            eyebrow_textures: section_list[11].drain(..).map(unpack_texture).collect(),
            beard_textures: section_list[12].drain(..).map(unpack_texture).collect(),
            wrinkle_textures: section_list[13].drain(..).map(unpack_texture).collect(),
            makeup_textures: section_list[14].drain(..).map(unpack_texture).collect(),
            glass_textures: section_list[15].drain(..).map(unpack_texture).collect(),
            mole_textures: section_list[16].drain(..).map(unpack_texture).collect(),
            lip_textures: section_list[17].drain(..).map(unpack_texture).collect(),
            mustache_textures: section_list[18].drain(..).map(unpack_texture).collect(),
            nose_textures: section_list[19].drain(..).map(unpack_texture).collect(),

            face_configs
        };
        Some(asset)
    }
}
