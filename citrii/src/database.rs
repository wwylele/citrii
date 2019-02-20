use crate::head_renderer;
use byte_struct::*;

bitfields!(
    ProfileHeader: u32 {
        catagory: 8,
        allow_copying: 1,
        a: 1,
        b: 2,
        c: 2,
        d: 2,

        page: 4,
        slot: 4,
        g: 4,
        h: 3,
        i: 1,
    }
);

bitfields!(
    ProfileIdLow: u32 {
        creation_date: 28,
        unknown: 1,
        temporary: 1,
        ntr: 1,
        special: 1,
    }
);

#[derive(ByteStructBE, Debug, Default, Copy, Clone)]
pub struct ProfileId {
    pub low: ProfileIdLow,
    pub mac: [u8; 6]
}

bitfields!(
    ProfileGeneral: u16 { //?
        sex: 1,
        birth_day: 4,
        birth_month: 5,
        wearing_color: 4,
        favourate: 1,
        padding: 1,
    }
);

bitfields!(
    ProfileFace: u16 { // 0x30
        disable_sharing: 1, // ?
        style: 4,
        color: 3,
        wrinkle: 4,
        makeup: 4,
    }
);

bitfields!(
    ProfileHair: u16 { // 0x32
        style: 8,
        color: 4,
        flip: 1,
        padding: 3,
    }
);

bitfields!(
    ProfileEye: u32 { // 0x34
        style: 6,
        color: 3,
        scale: 4,
        y_scale: 3,
        rotation: 5,
        x: 4,
        y: 5,
        padding: 2,
    }
);

bitfields!(
    ProfileEyebrow: u32 { // 0x38
        style: 5,
        color: 3,
        scale: 4,
        y_scale: 3,
        padding: 1,
        rotation: 5,
        x: 4,
        y: 5,
        padding2: 2,
    }
);

bitfields!(
    ProfileNose: u16 { // 0x3C
        style: 5,
        scale: 4,
        y: 5,
        padding: 2,
    }
);

bitfields!(
    ProfileLip: u16 { // 0x3E
        style: 6,
        color: 3,
        scale: 4,
        y_scale: 3,
    }
);

bitfields!(
    ProfileMisc: u16 { // 0x40
        lip_y: 5,
        mustache_style: 3,
        padding: 8
    }
);

bitfields!(
    ProfileBeard: u16 { // 0x42
        style: 3,
        color: 3,
        mustach_scale: 4,
        mustach_y: 5,
        padding: 1
    }
);

bitfields!(
    ProfileGlass: u16 { // 0x44
        style: 4,
        color: 3,
        scale: 4,
        y: 5,
    }
);

bitfields!(
    Mole: u16 { // 0x46
        style: 1,
        scale: 4,
        x: 5,
        y: 5,
        padding: 1
    }
);

#[derive(ByteStructLE, Debug, Default, Copy, Clone)]
pub struct Profile {
    pub header: ProfileHeader,
    pub system_id: [u8; 8],
    pub char_id: ProfileId,
    pub padding: u16,
    pub general: ProfileGeneral,
    pub name: [u16; 10],
    pub width: u8,
    pub height: u8,
    pub face: ProfileFace,
    pub hair: ProfileHair,
    pub eye: ProfileEye,
    pub eyebrow: ProfileEyebrow,
    pub nose: ProfileNose,
    pub lip: ProfileLip,
    pub misc: ProfileMisc,
    pub beard: ProfileBeard,
    pub glass: ProfileGlass,
    pub mole: Mole,
    pub author: [u16; 10],
}

const SKIN_COLOR_TABLE: [(u8, u8, u8); 6] = [
    (255, 211, 173),
    (255, 182, 107),
    (222, 121, 66),
    (255, 170, 140),
    (173, 81, 41),
    (99, 44, 24),
];

const SKIHAIR_COLOR_TABLE: [(u8, u8, u8); 8] = [
    (30, 26, 24),
    (71, 38, 22),
    (104, 24, 20),
    (124, 58, 20),
    (120, 128, 128),
    (78, 62, 16),
    (132, 85, 23),
    (208, 160, 74),
];

const EYE_COLOR_TABLE: [(u8, u8, u8); 6] = [
    (0, 0, 0),
    (108, 112, 112),
    (102, 60, 44),
    (93, 91, 46),
    (70, 84, 168),
    (56, 112, 88),
];

const LIP_COLOR_TABLE: [(u8, u8, u8); 5] = [
    (216, 82, 8),
    (239, 12, 8),
    (245, 72, 72),
    (240, 154, 116),
    (140, 80, 64),
];

const GLASS_COLOR_TABLE: [(u8, u8, u8); 6] = [
    (24, 24, 24),
    (96, 56, 16),
    (168, 16, 8),
    (32, 48, 104),
    (168, 96, 0),
    (120, 112, 104),
];

const WEARING_COLOR_TABLE: [(u8, u8, u8); 12] = [
    (210, 30, 30),
    (255, 110, 25),
    (255, 216, 32),
    (120, 210, 32),
    (0, 120, 48),
    (32, 72, 152),
    (60, 170, 222),
    (245, 90, 125),
    (115, 40, 173),
    (72, 56, 24),
    (224, 224, 224),
    (24, 24, 20),
];

const INVALID_COLOR: (u8, u8, u8) = (255, 255, 255);

fn convert_color((r, g, b): &(u8, u8, u8)) -> (f32, f32, f32) {
    (*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
}

fn get_color(table: &[(u8, u8, u8)], index: usize) -> (f32, f32, f32) {
    convert_color(table.get(index).unwrap_or(&INVALID_COLOR))
}

const EYE_ROTATION_OFFSETS: [u32; 62] = [
    3,
    4,
    4,
    4,
    3,
    4,
    4,
    4,
    3,
    4,
    4,
    4,
    4,
    3,
    3,
    4,
    4,
    4,
    3,
    3,
    4,
    3,
    4,
    3,
    3,
    4,
    3,
    4,
    4,
    3,
    4,
    4,
    4,
    3,
    3,
    3,
    4,
    4,
    3,
    3,
    3,
    4,
    4,
    3,
    3,
    3,
    3,
    3,
    3,
    3,
    3,
    3,
    4,
    4,
    4,
    4,
    3,
    4,
    4,
    3,
    4,
    4,
];

const EYEBROW_ROTATION_OFFSETS: [u32; 24] = [
    6,
    6,
    5,
    7,
    6,
    7,
    6,
    7,
    4,
    7,
    6,
    8,
    5,
    5,
    6,
    6,
    7,
    7,
    6,
    6,
    5,
    6,
    7,
    5,
];

impl Profile {
    pub fn to_render_info(&self) -> head_renderer::HeadRenderInfo {
        const FACE_SCALE: f32 = 1.0 / 64.0;
        const Y_STEP: f32 = 1.0761;
        const ROTATION_STEP: f32 = 360.0 / 32.0;
        fn scaling(code: f32) -> f32 {
            1.0 + 0.4 * code
        }
        fn y_scaling(code: f32) -> f32 {
            0.64 + 0.12 * code
        }
        head_renderer::HeadRenderInfo {
            hair: self.hair.style as usize,
            face: self.face.style as usize,
            nose: self.nose.style as usize,
            beard: if self.beard.style <= 3 {self.beard.style} else {0} as usize,
            glass: self.glass.style as usize,
            eye: self.eye.style as usize,
            eyebrow: self.eyebrow.style as usize,
            beard_plain: if self.beard.style > 3 {self.beard.style - 3} else {0} as usize,
            wrinkle: self.face.wrinkle as usize,
            makeup: self.face.makeup as usize,
            mole: self.mole.style as usize,
            lip: self.lip.style as usize,
            mustache: self.misc.mustache_style as usize,

            full_hair: true,

            hair_color: get_color(&SKIHAIR_COLOR_TABLE, self.hair.color as usize),
            wearing_color: get_color(&WEARING_COLOR_TABLE, self.general.wearing_color as usize),
            face_color: get_color(&SKIN_COLOR_TABLE, self.face.color as usize),
            beard_color: get_color(&SKIHAIR_COLOR_TABLE, self.beard.color as usize),
            glass_color: get_color(&GLASS_COLOR_TABLE, self.glass.color as usize),
            eye_color: get_color(&EYE_COLOR_TABLE, self.eye.color as usize),
            eyebrow_color: get_color(&SKIHAIR_COLOR_TABLE, self.eyebrow.color as usize),
            lip_color: get_color(&LIP_COLOR_TABLE, self.lip.color as usize),

            nose_scale: 0.4 + 0.175 * self.nose.scale as f32,
            nose_y: 12.0 - 1.5 * self.nose.y as f32,
            glass_y: 21.5 - 1.5 * self.glass.y as f32,
            glass_scale: 0.4 + 0.15 * self.glass.scale as f32,
            mole_x: (17.766 + 1.7792 * self.mole.x as f32) * FACE_SCALE,
            mole_y: 1.0 - (17.96 + Y_STEP * self.mole.y as f32) * FACE_SCALE,
            mole_width: scaling(self.mole.scale as f32) * FACE_SCALE,
            lip_y: 1.0 - (29.259 + Y_STEP * self.misc.lip_y as f32) * FACE_SCALE,
            lip_width: 6.1875 * scaling(self.lip.scale as f32) * FACE_SCALE,
            lip_height: 4.5 * scaling(self.lip.scale as f32) * y_scaling(self.lip.y_scale as f32) * FACE_SCALE,
            mustache_y: 1.0 - (31.764 + Y_STEP * self.beard.mustach_y as f32) * FACE_SCALE,
            mustache_width: 4.5 * scaling(self.beard.mustach_scale as f32) * FACE_SCALE,
            mustache_height: 9.0 * scaling(self.beard.mustach_scale as f32) * FACE_SCALE,
            eye_x: 0.88961 * self.eye.x as f32 * FACE_SCALE,
            eye_y: 1.0 - (18.452 + Y_STEP * self.eye.y as f32) * FACE_SCALE,
            eye_width: 5.3438 * scaling(self.eye.scale as f32) * FACE_SCALE,
            eye_height: 4.5 * scaling(self.eye.scale as f32) * y_scaling(self.eye.y_scale as f32) * FACE_SCALE,
            eye_rotation: ROTATION_STEP *
                (self.eye.rotation as f32 - *EYE_ROTATION_OFFSETS.get(self.eye.style as usize).unwrap_or(&0) as f32),
            eyebrow_x: 0.88961 * self.eyebrow.x as f32 * FACE_SCALE,
            eyebrow_y: 1.0 - (16.55 + Y_STEP * self.eyebrow.y as f32) * FACE_SCALE,
            eyebrow_width: 5.0625 * scaling(self.eyebrow.scale as f32) * FACE_SCALE,
            eyebrow_height: 4.5 * scaling(self.eyebrow.scale as f32) * y_scaling(self.eyebrow.y_scale as f32) * FACE_SCALE,
            eyebrow_rotation: ROTATION_STEP *
                (self.eyebrow.rotation as f32 - *EYEBROW_ROTATION_OFFSETS.get(self.eyebrow.style as usize).unwrap_or(&0) as f32),
        }
    }
}

#[derive(ByteStructLE)]
pub struct Database {
    pub cfog: [u8; 4],
    pub magic: u32,
    pub owned: [Profile; 100],
    pub cfhe: [u8; 4],
    pub magic2: u32,
    pub unk: [[u8; 14]; 3001],
    pub crc: u16,
}

#[test]
fn struct_size_test() {
    assert_eq!(Database::byte_len(), 0xC820);
}
