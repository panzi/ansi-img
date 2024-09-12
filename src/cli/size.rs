#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Scale (i32),
    Width (u32),
    Height (u32),
    Exact (u32, u32),
}

impl Size {
    pub fn to_size(&self, image_width: u32, image_height: u32) -> (u32, u32) {
        match *self {
            Self::Scale(z) => {
                let width;
                let height;
                if z > 1 {
                    if image_width > image_height {
                        if image_width > (u32::MAX / z as u32) {
                            width  = u32::MAX;
                            height = (u32::MAX as u64 * image_height as u64 / image_width as u64) as u32;
                        } else {
                            width  = image_width  * z as u32;
                            height = image_height * z as u32;
                        }
                    } else {
                        if image_height > (u32::MAX / z as u32) {
                            width  = (u32::MAX as u64 * image_width as u64 / image_height as u64) as u32;
                            height = u32::MAX;
                        } else {
                            width  = image_width  * z as u32;
                            height = image_height * z as u32;
                        }
                    }
                } else if z == 1 {
                    width  = image_width;
                    height = image_height;
                } else {
                    width  = image_width  / (-z as u32);
                    height = image_height / (-z as u32);
                }

                (width, height)
            },
            Self::Width(w) => {
                (w, w * image_height / image_width)
            },
            Self::Height(h) => {
                (h, h * image_width / image_height)
            },
            Self::Exact(w, h) => (w, h),
        }
    }
}
