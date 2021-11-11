use anyhow::{bail, ensure, Context, Result};
use clap::App;
use image::{GenericImageView, ImageBuffer, Luma, Pixel, Rgb};
use show_image::{create_window, event};
use std::path::Path;

fn task1<T: AsRef<Path>>(src: T, dst: T) -> Result<()> {
    let im = image::open(src.as_ref())?;
    im.save(dst.as_ref())?;
    let window = create_window("image", Default::default())?;
    window.set_image("source", im)?;

    for event in window.event_channel()? {
        if let event::WindowEvent::KeyboardInput(event) = event {
            if event.input.key_code == Some(event::VirtualKeyCode::Escape)
                && event.input.state.is_pressed()
            {
                break;
            }
        }
    }

    Ok(())
}

fn task2<T: AsRef<Path>>(src: T, dst: T, swap: &str) -> Result<()> {
    let im = image::open(src.as_ref())?;

    let buf = ImageBuffer::from_fn(im.width(), im.height(), |x, y| {
        let [r, g, b] = im.get_pixel(x, y).to_rgb().0;
        match swap {
            "r,b" | "b,r" => Rgb([b, g, r]),
            "r,g" | "g,r" => Rgb([g, r, b]),
            "b,g" | "g,b" => Rgb([r, b, g]),
            _ => unreachable!(),
        }
    });
    buf.save(dst)?;

    Ok(())
}

fn task3<T: AsRef<Path>>(src1: T, src2: T, dst: T, ratio: f64) -> Result<()> {
    let im1 = image::open(src1.as_ref())?;
    let im2 = image::open(src2.as_ref())?;

    ensure!(im1.width() == im2.width(), "寸法が異なります");
    ensure!(im1.height() == im2.height(), "寸法が異なります");

    let buf = ImageBuffer::from_fn(im1.width(), im1.height(), |x, y| {
        let px1 = im1.get_pixel(x, y).to_luma().0[0] as f64;
        let px2 = im2.get_pixel(x, y).to_luma().0[0] as f64;

        Luma([(px1 * ratio + px2 * (1.0 - ratio)) as u8])
    });
    buf.save(dst)?;

    Ok(())
}

#[show_image::main]
fn main() -> Result<()> {
    let matches = App::new("kadai1")
        .version("0.0.1")
        .args_from_usage(
            "--task=[number] '課題の番号(1-3)'
             --src=[path] '入力画像のパス'
             --dst=[path] '出力画像のパス'
             --swap=[color] '課題2で入れ替える色(r,b など)'
             --src2=[path] '課題3での2枚目の入力画像のパス'
             --ratio=[ratio] '課題3で合成の割合'",
        )
        .get_matches();

    let task = matches
        .value_of("task")
        .context("課題番号を指定してください")?;
    let src = matches
        .value_of("src")
        .context("入力画像を指定してください")?;
    let dst = matches
        .value_of("dst")
        .context("出力画像を指定してください")?;

    match task {
        "1" => task1(src, dst)?,
        "2" => {
            let swap = matches
                .value_of("swap")
                .context("入れ替える色を指定してください")?;

            task2(src, dst, swap)?
        }
        "3" => {
            let ratio = matches
                .value_of("ratio")
                .context("割合を指定してください")?
                .parse::<f64>()?;
            let src2 = matches
                .value_of("src2")
                .context("2枚目の入力画像を指定してください")?;

            task3(src, src2, dst, ratio)?
        }
        _ => bail!("課題番号は1-3で指定してください"),
    }

    Ok(())
}
