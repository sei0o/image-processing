use anyhow::{bail, Context, Result};
use clap::App;
use fft2d::slice::dcst::{dct_2d, idct_2d};
use image::Luma;
use std::path::Path;

fn filter<T: AsRef<Path>, F: Fn(Vec<f64>) -> Vec<f64>>(src: T, dst: T, f: F) -> Result<()> {
    let im = image::open(src.as_ref())?.into_luma8();
    let dim = im.dimensions();
    let (width, height) = (dim.0 as usize, dim.1 as usize);
    assert!(width % 8 == 0 && height % 8 == 0);
    let width_in_block = width / 8;

    // 0.0~1.0に正規化
    let buf: Vec<f64> = im.as_raw().iter().map(|pix| *pix as f64 / 255.0).collect();

    let mut blocks = vec![];
    for block_row in buf.chunks_exact(width * 8) {
        let mut blocks_in_row = vec![vec![]; width_in_block];
        for (i, chunk) in block_row.chunks_exact(8).enumerate() {
            chunk
                .iter()
                .for_each(|x| blocks_in_row[i % width_in_block].push(*x));
        }
        blocks_in_row
            .into_iter()
            .for_each(|block| blocks.push(block));
    }
    assert!(blocks[0].len() == 64);

    blocks.iter_mut().for_each(|block| dct_2d(8, 8, block));

    let mut filtered_blocks: Vec<Vec<f64>> = blocks
        .into_iter()
        .map(|block| block.into_iter().map(|px| px * 64.0).collect()) // 正規化
        .map(f)
        .collect();

    filtered_blocks
        .iter_mut()
        .for_each(|block| idct_2d(8, 8, block));
    let filtered_blocks: Vec<Vec<f64>> = filtered_blocks
        .into_iter()
        .map(|block| block.into_iter().map(|px| px / 4.0).collect())
        .collect();

    let mut filtered_image = image::GrayImage::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let block_pos = width_in_block * (y / 8) + x / 8;
            let pos_in_block = (y % 8) * 8 + (x % 8);
            let luma = filtered_blocks[block_pos][pos_in_block];
            filtered_image.put_pixel(x as u32, y as u32, Luma([luma as u8]));
        }
    }

    filtered_image.save(dst.as_ref())?;

    // SN比を計算
    let sum = (0..height as u32).into_iter().fold(0.0, |sum, y| {
        (0..width as u32).into_iter().fold(0.0, |s, x| {
            let diff = im.get_pixel(x, y).0[0] as f64 - filtered_image.get_pixel(x, y).0[0] as f64;
            s + diff * diff
        }) + sum
    });
    let mse = sum / (width * height) as f64;
    let psnr = 10.0 * (255.0 * 255.0 / mse).log(10.0);
    println!("MSE: {}, PSNR: {}", mse, psnr);

    Ok(())
}

fn task1<T: AsRef<Path>>(src: T, dst: T) -> Result<()> {
    filter(src, dst, |block| {
        block
            .iter()
            .enumerate()
            .map(|(i, px)| {
                let x = i % 8;
                let y = i / 8;
                if (x + y) >= 8 {
                    0.0
                } else {
                    *px
                }
            })
            .collect()
    })
}

fn task2<T: AsRef<Path>>(src: T, dst: T, threshold: u8) -> Result<()> {
    filter(src, dst, |block| {
        block
            .iter()
            .map(|&px| if px.abs() < threshold as f64 { 0.0 } else { px })
            .collect()
    })
}

fn task3<T: AsRef<Path>>(src: T, dst: T, threshold: u8) -> Result<()> {
    let bound = (64.0 * threshold as f64 / 100.0).floor() as usize;
    filter(src, dst, |block| {
        let mut b = block.clone();
        b.sort_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap());
        let smalls = &b[0..=bound];
        block
            .iter()
            .map(|px| if smalls.contains(&px.abs()) { 0.0 } else { *px })
            .collect()
    })
}

const ZIGZAG: [usize; 64] = [
    0, 8, 1, 2, 9, 16, 24, 17, 10, 3, 4, 11, 18, 25, 32, 40, 33, 26, 19, 21, 5, 6, 13, 20, 27, 34,
    41, 48, 56, 49, 42, 35, 28, 21, 14, 7, 15, 22, 29, 36, 43, 50, 57, 58, 51, 44, 37, 30, 23, 31,
    38, 45, 52, 59, 60, 53, 46, 39, 47, 54, 61, 62, 55, 63,
];

fn zigzag(block: &[f64]) -> Vec<f64> {
    (0..64).map(|x| block[ZIGZAG[x]]).collect()
}

fn task4<T: AsRef<Path>>(src: T, dst: T, threshold: u8) -> Result<()> {
    let bound = (64.0 * threshold as f64 / 100.0).floor() as usize;
    filter(src, dst, |block| {
        let mut b = zigzag(&block);
        b.reverse();
        let tail = &b[0..=bound];
        block
            .iter()
            .map(|px| if tail.contains(&px.abs()) { 0.0 } else { *px })
            .collect()
    })
}

fn main() -> Result<()> {
    let matches = App::new("kadai3")
        .version("0.0.1")
        .args_from_usage(
            "--task=[number] '課題の番号(1, 2, 3, 4)'
             --src=[path] '入力画像のパス'
             --dst=[path] '出力画像のパス'
             --t=[val]    'しきい値'",
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
    let threshold = matches.value_of("t").unwrap_or("128").parse::<u8>()?;

    match task {
        "1" => task1(src, dst)?,
        "2" => task2(src, dst, threshold)?,
        "3" => task3(src, dst, threshold)?,
        "4" => task4(src, dst, threshold)?,
        _ => bail!("課題番号は1, 2, 3, 4のいずれかで指定してください"),
    }

    Ok(())
}
