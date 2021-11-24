use anyhow::{bail, Context, Result};
use clap::App;
use image::{GenericImageView, Pixel};
use plotters::prelude::*;
use std::{ops::Range, path::Path};

fn task1<T: AsRef<Path>>(src: T, dst: T) -> Result<()> {
    let im = image::open(src.as_ref())?;

    let mut count = [0usize; 256];

    for (_, _, px) in im.pixels() {
        // pxはグレースケールが暗黙的にRgba<u8>型に変換されたもの。
        // lumaに戻そうとすると[計算誤差](https://docs.rs/image/0.23.14/src/image/color.rs.html#420)
        // が生じてヒストグラムが櫛形になってしまうので、ここではredをそのまま使う。
        // let lu = px.to_luma().0[0] as usize;
        let red = px.0[0] as usize;
        //println!("lu: {}, red: {}", lu, red);
        count[red] += 1;
    }

    let (avg, var) = calc_stat(&count, im.width() * im.height());

    println!("平均: {}, 分散: {}", avg, var);
    draw_histogram(&count, dst, 0..256, |x, y| (x as i64, y))?;

    Ok(())
}

fn task2a<T: AsRef<Path>>(src: T, dst: T) -> Result<()> {
    let im = image::open(src.as_ref())?;

    // -255..255の範囲を0..511に保存しておく
    let mut count = [0usize; 512];

    for (x, y, px) in im.pixels() {
        if x == 0 {
            continue;
        }

        let lu = px.to_luma().0[0] as i16;
        let prediction = im.get_pixel(x - 1, y).0[0] as i16;
        let error = prediction - lu + 255;
        assert!(error >= 0);

        count[error as usize] += 1;
    }

    let (_, var) = calc_stat(&count, im.width() * im.height());

    println!("分散: {}", var);
    draw_histogram(&count, dst, -255..256, |x, y| (x as i64 - 255, y))?;

    Ok(())
}

fn task2b<T: AsRef<Path>>(src: T, dst: T) -> Result<()> {
    let im = image::open(src.as_ref())?;

    // -255..255の範囲を0..511に保存しておく
    let mut count = [0usize; 512];

    for (x, y, px) in im.pixels() {
        if x == 0 || y == 0 {
            continue;
        }

        let lu = px.to_luma().0[0] as i16;
        let left_top = im.get_pixel(x - 1, y - 1).0[0] as i16;
        let left = im.get_pixel(x - 1, y).0[0] as i16;
        let top = im.get_pixel(x, y - 1).0[0] as i16;
        let prediction = left + top - left_top;
        let error = prediction - lu + 255;
        assert!(error >= 0);

        count[error as usize] += 1;
    }

    let (_, var) = calc_stat(&count, im.width() * im.height());
    println!("分散: {}", var);
    draw_histogram(&count, dst, -255..256, |x, y| (x as i64 - 255, y))?;

    Ok(())
}

fn task2c<T: AsRef<Path>>(src: T, dst: T) -> Result<()> {
    let im = image::open(src.as_ref())?;

    let mut count = [0usize; 512];

    for (x, y, px) in im.pixels() {
        if x == 0 || y == 0 {
            continue;
        }

        let lu = px.to_luma().0[0] as i16;
        let left = im.get_pixel(x - 1, y).0[0] as i16;
        let top = im.get_pixel(x, y - 1).0[0] as i16;
        let prediction = f64::round((left + top) as f64 / 2.0) as i16;
        let error = prediction - lu + 255;
        assert!(error >= 0);

        count[error as usize] += 1;
    }

    let (_, var) = calc_stat(&count, im.width() * im.height());
    println!("分散: {}", var);
    draw_histogram(&count, dst, -255..256, |x, y| (x as i64 - 255, y))?;

    Ok(())
}

fn calc_stat(data: &[usize], pixels: u32) -> (f64, f64) {
    let sum = data
        .iter()
        .enumerate()
        .fold(0.0, |sum, (i, cnt)| sum + (i * *cnt) as f64);
    let avg = sum / pixels as f64;
    let sqsum: f64 = data.iter().enumerate().fold(0.0, |s, (i, cnt)| {
        s + (*cnt as f64) * ((i as f64) - avg) * ((i as f64) - avg)
    });
    let var = sqsum / pixels as f64;

    (avg, var)
}

fn draw_histogram<T: AsRef<Path>>(
    data: &[usize],
    dst: T,
    xrange: Range<i64>,
    mapper: fn(usize, usize) -> (i64, usize),
) -> Result<()> {
    // Consult plotters' issue #182 on GitHub
    //let root = BitMapBackend::new(dst.as_ref(), (1280, 720)).into_drawing_area();
    let root = SVGBackend::new(dst.as_ref(), (1280, 720)).into_drawing_area();

    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .margin(10)
        .build_cartesian_2d(xrange, 0usize..*data.iter().max().unwrap())?;

    chart.configure_mesh().disable_x_mesh().draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(BLACK.filled())
            .data(data.iter().enumerate().map(|(x, y)| mapper(x, *y))),
    )?;

    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("kadai2")
        .version("0.0.1")
        .args_from_usage(
            "--task=[number] '課題の番号(1, 2a, 2b, 2c)'
             --src=[path] '入力画像のパス'
             --dst=[path] '出力画像のパス'",
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
        "2a" => task2a(src, dst)?,
        "2b" => task2b(src, dst)?,
        "2c" => task2c(src, dst)?,
        _ => bail!("課題番号は1, 2a, 2b, 2cのいずれかで指定してください"),
    }

    Ok(())
}
