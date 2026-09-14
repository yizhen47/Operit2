#![allow(non_snake_case)]

use operit_host_api::HostResult;

use crate::face::{rgb565, FaceRect};
use crate::robot_face::FaceCanvas;

/// Number of plugin tiles reserved on the minus-one screen.
pub const PLUGIN_SLOT_COUNT: usize = 4;

/// Paints the empty plugin shelf. Plugin packages fill these tiles later.
pub fn paintPluginShelf(canvas: &mut dyn FaceCanvas) -> HostResult<()> {
    let width = canvas.width();
    let height = canvas.height();
    canvas.fill(rgb565(8, 12, 24))?;
    canvas.fillRect(
        FaceRect {
            x: 0,
            y: 0,
            width,
            height: 28,
        },
        rgb565(15, 23, 42),
    )?;
    canvas.fillRect(
        FaceRect {
            x: 16,
            y: 10,
            width: 48,
            height: 8,
        },
        rgb565(59, 130, 246),
    )?;

    let gap = 12u16;
    let tileWidth = width.saturating_sub(gap * 3) / 2;
    let tileHeight = height.saturating_sub(28 + 36 + gap * 3) / 2;
    let originY = 28 + gap;
    for row in 0..2u16 {
        for col in 0..2u16 {
            let x = gap + col * (tileWidth + gap);
            let y = originY + row * (tileHeight + gap);
            canvas.fillRect(
                FaceRect {
                    x,
                    y,
                    width: tileWidth,
                    height: tileHeight,
                },
                rgb565(30, 64, 105),
            )?;
            if tileWidth > 8 && tileHeight > 8 {
                canvas.fillRect(
                    FaceRect {
                        x: x + 4,
                        y: y + 4,
                        width: tileWidth - 8,
                        height: tileHeight - 8,
                    },
                    rgb565(15, 23, 42),
                )?;
            }
        }
    }

    let pillWidth = 40u16;
    let pillHeight = 6u16;
    canvas.fillRect(
        FaceRect {
            x: width.saturating_sub(pillWidth) / 2,
            y: height.saturating_sub(16),
            width: pillWidth,
            height: pillHeight,
        },
        rgb565(148, 163, 184),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robot_face::MemoryFaceCanvas;

    #[test]
    fn paintsFourEmptyPluginTiles() {
        let mut canvas = MemoryFaceCanvas::new(240, 320);
        paintPluginShelf(&mut canvas).expect("plugin shelf must paint");
        assert_eq!(PLUGIN_SLOT_COUNT, 4);
        assert_eq!(canvas.pixel(0, 0), Some(rgb565(15, 23, 42)));
        assert_eq!(canvas.pixel(12, 40), Some(rgb565(30, 64, 105)));
        assert_eq!(canvas.pixel(16, 44), Some(rgb565(15, 23, 42)));
        assert_eq!(canvas.pixel(120, 308), Some(rgb565(148, 163, 184)));
    }
}
