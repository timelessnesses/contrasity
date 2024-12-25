#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ContrastPasses {
    /// Minimum Acceptable Contrast
    AA(bool),
    /// Enhanced Contrast
    AAA(bool),
    /// Minimum Acceptable but it's large texts (?)
    AALarge(bool),
}

impl ContrastPasses {
    pub fn get_rating(score: f64) -> [ContrastPasses; 3] {
        let mut rating = [ContrastPasses::AA(false); 3];
        rating[0] = if score >= 4.5 {
            ContrastPasses::AA(true)
        } else {
            ContrastPasses::AA(false)
        };
        rating[1] = if score >= 7.0 {
            ContrastPasses::AAA(true)
        } else {
            ContrastPasses::AAA(false)
        };
        rating[2] = if score >= 3.0 {
            ContrastPasses::AALarge(true)
        } else {
            ContrastPasses::AALarge(false)
        };
        rating
    }

    pub fn get_contrast(bg: (f64, f64, f64), fg: (f64, f64, f64)) -> f64 {
        let bg_luminance = Self::get_luminance(bg);
        let fg_luminance = Self::get_luminance(fg);
        return if bg_luminance > fg_luminance {
            (bg_luminance + 0.05) / (fg_luminance + 0.05)
        } else {
            (fg_luminance + 0.05) / (bg_luminance + 0.05)
        };
    }

    pub fn get_luminance(color: (f64, f64, f64)) -> f64 {
        let r = color.0;
        let g = color.1;
        let b = color.2;
        return Self::luminance_x(r) * 0.2126
            + Self::luminance_x(g) * 0.7152
            + Self::luminance_x(b) * 0.0722;
    }

    fn luminance_x(mut color: f64) -> f64 {
        color /= 255.0;
        return if color <= 0.03928 {
            color / 12.92
        } else {
            ((color + 0.055) / 1.055).powf(2.4)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_luminance() {
        assert_eq!(ContrastPasses::get_luminance((255.0, 255.0, 255.0)), 1.0);
        assert_eq!(ContrastPasses::get_luminance((0.0, 0.0, 0.0)), 0.0);
    }

    #[test]
    fn test_get_contrast() {
        assert_eq!(
            ContrastPasses::get_contrast((255.0, 255.0, 255.0), (0.0, 0.0, 0.0)),
            21.0
        );
        assert_eq!(
            ContrastPasses::get_contrast((0.0, 0.0, 0.0), (255.0, 255.0, 255.0)),
            21.0
        );
        assert_eq!(
            ContrastPasses::get_contrast((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
            1.0
        );
        assert_eq!(
            ContrastPasses::get_contrast((255.0, 255.0, 255.0), (255.0, 255.0, 255.0)),
            1.0
        );
        assert_eq!(
            round_float(
                ContrastPasses::get_contrast((0.0, 0.0, 0.0), (0.0, 255.0, 255.0)),
                2
            ),
            16.75
        );
    }
}

pub fn round_float(num: f64, precision: u32) -> f64 {
    let factor = 10.0_f64.powi(precision as i32);
    (num * factor).round() / factor
}
