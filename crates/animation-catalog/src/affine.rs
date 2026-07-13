#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub rows: [[f32; 4]; 3],
}

impl Affine {
    pub const IDENTITY: Self = Self {
        rows: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ],
    };

    pub fn translation(value: [f32; 3]) -> Self {
        let mut result = Self::IDENTITY;
        result.rows[0][3] = value[0];
        result.rows[1][3] = value[1];
        result.rows[2][3] = value[2];
        result
    }

    pub fn rotation_translation(axis: u32, angle: f32, translation: [f32; 3]) -> Self {
        let (sine, cosine) = angle.sin_cos();
        let mut result = Self::translation(translation);
        match axis % 3 {
            0 => {
                result.rows[1][1] = cosine;
                result.rows[1][2] = -sine;
                result.rows[2][1] = sine;
                result.rows[2][2] = cosine;
            }
            1 => {
                result.rows[0][0] = cosine;
                result.rows[0][2] = sine;
                result.rows[2][0] = -sine;
                result.rows[2][2] = cosine;
            }
            _ => {
                result.rows[0][0] = cosine;
                result.rows[0][1] = -sine;
                result.rows[1][0] = sine;
                result.rows[1][1] = cosine;
            }
        }
        result
    }

    pub fn compose(self, child: Self) -> Self {
        let mut result = Self::IDENTITY;
        for row in 0..3 {
            for column in 0..3 {
                result.rows[row][column] = self.rows[row][0] * child.rows[0][column]
                    + self.rows[row][1] * child.rows[1][column]
                    + self.rows[row][2] * child.rows[2][column];
            }
            result.rows[row][3] = self.rows[row][0] * child.rows[0][3]
                + self.rows[row][1] * child.rows[1][3]
                + self.rows[row][2] * child.rows[2][3]
                + self.rows[row][3];
        }
        result
    }

    pub fn transform_point(self, point: [f32; 3]) -> [f32; 3] {
        [
            dot_point(self.rows[0], point),
            dot_point(self.rows[1], point),
            dot_point(self.rows[2], point),
        ]
    }

    pub fn with_variant(mut self, seed: u32, bone: u32) -> Self {
        if seed == 0 {
            return self;
        }
        let first = seed
            .wrapping_mul(747_796_405)
            .wrapping_add(bone.wrapping_mul(2_891_336_453));
        let second = first.rotate_left(13).wrapping_mul(2_246_822_519);
        self.rows[0][3] += centered_byte(first) * 0.012;
        self.rows[2][3] += centered_byte(second) * 0.012;
        self
    }

    pub fn bytes(self, output: &mut Vec<u8>) {
        for row in self.rows {
            for value in row {
                output.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }
    }
}

fn centered_byte(value: u32) -> f32 {
    (value & 255) as f32 / 255.0 - 0.5
}

fn dot_point(row: [f32; 4], point: [f32; 3]) -> f32 {
    row[0] * point[0] + row[1] * point[1] + row[2] * point[2] + row[3]
}
