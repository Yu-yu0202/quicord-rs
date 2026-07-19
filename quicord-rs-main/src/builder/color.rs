/*
 * Copyright (c) 2026 Yu-yu0202
 *
 * This Source Code Form is subject to the terms of the
 * Mozilla Public License, v. 2.0. If a copy of the MPL
 * was not distributed with this file, You can obtain one at
 * https://mozilla.org/MPL/2.0/.
 */

#[derive(Clone, Copy, Eq, PartialEq, Hash, Default, Debug)]
pub struct Color(u32);

impl Color {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self((r as u32) << 16 | (g as u32) << 8 | (b as u32))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        value.value()
    }
}

#[allow(non_upper_case_globals)]
impl Color {
    pub const Default: Self = Self(0x00_00_00);

    pub const Aqua: Self = Self(0x1a_bc_9c);
    pub const Blue: Self = Self(0x34_98_db);
    pub const Blurple: Self = Self(0x58_65_f2);
    pub const Fuchsia: Self = Self(0xeb_45_9e);
    pub const Gold: Self = Self(0xf1_c4_0f);
    pub const Green: Self = Self(0x57_f2_87);
    pub const Grey: Self = Self(0x95_a5_a6);
    pub const Greyple: Self = Self(0x99_aa_b5);
    pub const LightGrey: Self = Self(0xbc_c0_c0);
    pub const LuminousVividPink: Self = Self(0xe9_1e_63);
    pub const Navy: Self = Self(0x34_49_5e);
    pub const NotQuiteBlack: Self = Self(0x23_27_2a);
    pub const Orange: Self = Self(0xe6_7e_22);
    pub const Purple: Self = Self(0x9b_59_b6);
    pub const Red: Self = Self(0xed_42_45);
    pub const White: Self = Self(0xff_ff_ff);
    pub const Yellow: Self = Self(0xfe_e7_5c);

    pub const DarkAqua: Self = Self(0x11_80_6a);
    pub const DarkBlue: Self = Self(0x20_66_94);
    pub const DarkButNotBlack: Self = Self(0x2c_2f_33);
    pub const DarkGold: Self = Self(0xc2_7c_0e);
    pub const DarkGreen: Self = Self(0x1f_8b_4c);
    pub const DarkGrey: Self = Self(0x97_9c_9f);
    pub const DarkNavy: Self = Self(0x2c_3e_50);
    pub const DarkOrange: Self = Self(0xa8_43_00);
    pub const DarkPurple: Self = Self(0x71_36_8a);
    pub const DarkRed: Self = Self(0x99_2d_22);
    pub const DarkVividPink: Self = Self(0xad_14_57);
    pub const DarkerGrey: Self = Self(0x7f_8c_8d);
}
