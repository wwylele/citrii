use crate::color::*;
use crate::head_renderer;
use byte_struct::*;
use chrono::*;

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileHeader: u32 {
        pub three: 8, // always 3?
        pub allow_copying: 1,
        pub private_name: 1,
        pub region_lock: 2, // 0 - No lock, 1 - JPN, 2 - USA, 3 - EUR
        pub char_set: 2, // 0 - Standard, 1 - CHN, 2 - KOR, 3 - TWN
        padding_a: 2,

        pub page: 4,
        pub slot: 4,
        pub version_minor: 4, // always 0?
        pub version_major: 3, // 1 - Wii, 2 - DSi, 3 - 3DS
        padding_b: 1,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileIdLow: u32 {
        pub creation_date: 28,
        pub unknown: 1,
        pub temporary: 1,
        pub ntr: 1,
        pub normal: 1,
    }
);

#[derive(ByteStruct, Debug, Default)]
#[byte_struct_be]
pub struct ProfileId {
    pub low: ProfileIdLow,
    pub mac: [u8; 6],
}

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileGeneral: u16 { //?
        pub sex: 1,
        pub birth_month: 4,
        pub birth_day: 5,
        pub favorite_color: 4,
        pub favorite: 1,
        padding: 1,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileFace: u16 { // 0x30
        pub disable_sharing: 1, // ?
        pub style: 4,
        pub color: 3,
        pub wrinkle: 4,
        pub makeup: 4,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileHair: u16 { // 0x32
        pub style: 8,
        pub color: 3,
        pub flip: 1,
        padding: 4,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileEye: u32 { // 0x34
        pub style: 6,
        pub color: 3,
        pub scale: 4,
        pub y_scale: 3,
        pub rotation: 5,
        pub x: 4,
        pub y: 5,
        padding: 2,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileEyebrow: u32 { // 0x38
        pub style: 5,
        pub color: 3,
        pub scale: 4,
        pub y_scale: 3,
        padding: 1,
        pub rotation: 5,
        pub x: 4,
        pub y: 5,
        padding2: 2,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileNose: u16 { // 0x3C
        pub style: 5,
        pub scale: 4,
        pub y: 5,
        padding: 2,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileLip: u16 { // 0x3E
        pub style: 6,
        pub color: 3,
        pub scale: 4,
        pub y_scale: 3,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileMisc: u16 { // 0x40
        pub lip_y: 5,
        pub mustache_style: 3,
        padding: 8
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileBeard: u16 { // 0x42
        pub style: 3,
        pub color: 3,
        pub mustache_scale: 4,
        pub mustache_y: 5,
        padding: 1
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileGlass: u16 { // 0x44
        pub style: 4,
        pub color: 3,
        pub scale: 4,
        pub y: 5,
    }
);

bitfields!(
    #[derive(Debug, Default)]
    pub ProfileMole: u16 { // 0x46
        pub style: 1,
        pub scale: 4,
        pub x: 5,
        pub y: 5,
        padding: 1
    }
);

#[derive(ByteStruct, Debug, Default)]
#[byte_struct_le]
pub struct Profile {
    pub header: ProfileHeader,
    pub system_id: [u8; 8],
    pub id: ProfileId,
    pub padding: u16,
    pub general: ProfileGeneral,
    pub name: [u16; 10],
    pub height: u8,
    pub width: u8,
    pub face: ProfileFace,
    pub hair: ProfileHair,
    pub eye: ProfileEye,
    pub eyebrow: ProfileEyebrow,
    pub nose: ProfileNose,
    pub lip: ProfileLip,
    pub misc: ProfileMisc,
    pub beard: ProfileBeard,
    pub glass: ProfileGlass,
    pub mole: ProfileMole,
}

impl Profile {
    pub fn get_slot(&self) -> usize {
        assert!(self.header.page < 10);
        assert!(self.header.slot < 10);
        (self.header.page * 10 + self.header.slot) as usize
    }

    pub fn set_slot(&mut self, slot: usize) {
        assert!(slot < 100);
        self.header.page = (slot / 10) as u32;
        self.header.slot = (slot % 10) as u32;
    }

    pub fn is_null(&self) -> bool {
        self.id.low.creation_date == 0
            && self.id.low.unknown == 0
            && self.id.low.temporary == 0
            && self.id.low.ntr == 0
            && self.id.low.normal == 0
            && self.id.mac == [0; 6]
    }

    pub fn new(mac: [u8; 6], system_id: [u8; 8], time: NaiveDateTime, slot: usize) -> Profile {
        let epoch = NaiveDateTime::new(
            NaiveDate::from_ymd(2010, 1, 1),
            NaiveTime::from_hms(0, 0, 0),
        );
        let creation_date = ((time - epoch).num_seconds() / 2) as u32;
        Profile {
            header: ProfileHeader {
                three: 3,
                allow_copying: 0,
                private_name: 0,
                region_lock: 0,
                char_set: 0,
                padding_a: 0,
                page: (slot / 10) as u32,
                slot: (slot % 10) as u32,
                version_minor: 0,
                version_major: 3,
                padding_b: 0,
            },
            system_id,
            id: ProfileId {
                low: ProfileIdLow {
                    creation_date,
                    unknown: 1,
                    temporary: 0,
                    ntr: 0,
                    normal: 1,
                },
                mac,
            },
            padding: 0,
            general: ProfileGeneral {
                sex: 1,
                birth_month: 0,
                birth_day: 0,
                favorite_color: 0,
                favorite: 0,
                padding: 0,
            },
            name: [b'?' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            height: 64,
            width: 64,
            face: ProfileFace::default(),
            hair: ProfileHair {
                style: 12,
                color: 1,
                flip: 0,
                padding: 0,
            },
            eye: ProfileEye {
                style: 4,
                color: 0,
                scale: 4,
                y_scale: 3,
                rotation: 3,
                x: 2,
                y: 12,
                padding: 0,
            },
            eyebrow: ProfileEyebrow {
                style: 0,
                color: 1,
                scale: 4,
                y_scale: 3,
                padding: 0,
                rotation: 6,
                x: 2,
                y: 10,
                padding2: 0,
            },
            nose: ProfileNose {
                style: 1,
                scale: 4,
                y: 9,
                padding: 0,
            },
            lip: ProfileLip {
                style: 23,
                color: 0,
                scale: 4,
                y_scale: 3,
            },
            misc: ProfileMisc {
                lip_y: 13,
                mustache_style: 0,
                padding: 0,
            },
            beard: ProfileBeard {
                style: 0,
                color: 0,
                mustache_scale: 4,
                mustache_y: 10,
                padding: 0,
            },
            glass: ProfileGlass {
                style: 0,
                color: 0,
                scale: 4,
                y: 10,
            },
            mole: ProfileMole {
                style: 0,
                scale: 4,
                x: 2,
                y: 20,
                padding: 0,
            },
        }
    }
}

#[derive(ByteStruct, Debug, Default)]
#[byte_struct_le]
pub struct ProfileFull {
    pub main: Profile,
    pub author: [u16; 10],
}

#[derive(ByteStruct, Debug, Default)]
#[byte_struct_le]
pub struct ProfileAlt {
    pub main: Profile,
    pub timestamp: u32, // seconds since 1/1/2000
    pub unk: [u8; 8],
}

const EYE_ROTATION_OFFSETS: [u32; 62] = [
    3, 4, 4, 4, 3, 4, 4, 4, 3, 4, 4, 4, 4, 3, 3, 4, 4, 4, 3, 3, 4, 3, 4, 3, 3, 4, 3, 4, 4, 3, 4, 4,
    4, 3, 3, 3, 4, 4, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 3, 4, 4, 3, 4, 4,
];

const EYEBROW_ROTATION_OFFSETS: [u32; 24] = [
    6, 6, 5, 7, 6, 7, 6, 7, 4, 7, 6, 8, 5, 5, 6, 6, 7, 7, 6, 6, 5, 6, 7, 5,
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
            beard: if self.beard.style <= 3 {
                self.beard.style
            } else {
                0
            } as usize,
            glass: self.glass.style as usize,
            eye: self.eye.style as usize,
            eyebrow: self.eyebrow.style as usize,
            beard_plain: if self.beard.style > 3 {
                self.beard.style - 3
            } else {
                0
            } as usize,
            wrinkle: self.face.wrinkle as usize,
            makeup: self.face.makeup as usize,
            mole: self.mole.style as usize,
            lip: self.lip.style as usize,
            mustache: self.misc.mustache_style as usize,

            full_hair: true,
            flip_hair: self.hair.flip != 0,

            hair_color: get_color(&HAIR_COLOR_TABLE, self.hair.color as usize),
            wearing_color: get_color(&WEARING_COLOR_TABLE, self.general.favorite_color as usize),
            face_color: get_color(&SKIN_COLOR_TABLE, self.face.color as usize),
            beard_color: get_color(&HAIR_COLOR_TABLE, self.beard.color as usize),
            glass_color: get_color(&GLASS_COLOR_TABLE, self.glass.color as usize),
            eye_color: get_color(&EYE_COLOR_TABLE, self.eye.color as usize),
            eyebrow_color: get_color(&HAIR_COLOR_TABLE, self.eyebrow.color as usize),
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
            lip_height: 4.5
                * scaling(self.lip.scale as f32)
                * y_scaling(self.lip.y_scale as f32)
                * FACE_SCALE,
            mustache_y: 1.0 - (31.764 + Y_STEP * self.beard.mustache_y as f32) * FACE_SCALE,
            mustache_width: 4.5 * scaling(self.beard.mustache_scale as f32) * FACE_SCALE,
            mustache_height: 9.0 * scaling(self.beard.mustache_scale as f32) * FACE_SCALE,
            eye_x: 0.88961 * self.eye.x as f32 * FACE_SCALE,
            eye_y: 1.0 - (18.452 + Y_STEP * self.eye.y as f32) * FACE_SCALE,
            eye_width: 5.3438 * scaling(self.eye.scale as f32) * FACE_SCALE,
            eye_height: 4.5
                * scaling(self.eye.scale as f32)
                * y_scaling(self.eye.y_scale as f32)
                * FACE_SCALE,
            eye_rotation: ROTATION_STEP
                * (self.eye.rotation as f32
                    - *EYE_ROTATION_OFFSETS
                        .get(self.eye.style as usize)
                        .unwrap_or(&0) as f32),
            eyebrow_x: 0.88961 * self.eyebrow.x as f32 * FACE_SCALE,
            eyebrow_y: 1.0 - (16.55 + Y_STEP * self.eyebrow.y as f32) * FACE_SCALE,
            eyebrow_width: 5.0625 * scaling(self.eyebrow.scale as f32) * FACE_SCALE,
            eyebrow_height: 4.5
                * scaling(self.eyebrow.scale as f32)
                * y_scaling(self.eyebrow.y_scale as f32)
                * FACE_SCALE,
            eyebrow_rotation: ROTATION_STEP
                * (self.eyebrow.rotation as f32
                    - *EYEBROW_ROTATION_OFFSETS
                        .get(self.eyebrow.style as usize)
                        .unwrap_or(&0) as f32),
        }
    }
}

bitfields!(
    #[derive(Debug, Default)]
    pub CFHEListNode: u32 {
        pub prev: 15,
        pub pf: 1,
        pub next: 15,
        pub nf: 1,
    }
);

#[derive(ByteStruct, Debug, Default)]
#[byte_struct_le]
pub struct CFHEObject {
    pub profile_id: ProfileId,
    pub list_node: CFHEListNode,
}

#[derive(ByteStruct)]
#[byte_struct_le]
pub struct Database {
    pub cfog: [u8; 4],
    pub magic: u32, // 0x00000100
    pub owned: [ProfileFull; 100],
    pub cfhe: [u8; 4],
    pub cfhe_tail: u16,
    pub cfhe_head: u16,
    pub cfhe_objects: [CFHEObject; 3000],
    pub unk: [u8; 0xE],
    pub crc_a: u16, // actually BE, but we access this field from raw bytes
    pub cfra: u32,
    pub invited_count: u32,
    pub invited_order: [u8; 100],
    pub invited: [Profile; 100],
    pub unk2: [u8; 0x12],
    pub crc_b: u16,
    pub cfhe_profiles: [ProfileAlt; 3000],
}

impl Database {
    pub fn owned_slot_to_index(&self, slot: usize) -> Option<usize> {
        for (i, profile) in self.owned.iter().enumerate() {
            if !profile.main.is_null() && profile.main.get_slot() == slot {
                return Some(i);
            }
        }
        None
    }
}

#[test]
fn struct_size_test() {
    assert_eq!(Database::BYTE_LEN, 0x4BD20);
}
