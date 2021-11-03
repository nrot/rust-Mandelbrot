use image::{DynamicImage, ImageOutputFormat, Rgb, RgbImage};
use std::fs::File;
use std::io;
use std::process::exit;
use std::str::FromStr;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

extern crate num_cpus;

struct Pixel {
    x: u32,
    y: u32,
    clr: Rgb<u8>,
}

struct Offset{
    x: f64,
    y: f64
}

struct Job {
    start: u32,
    stop: u32,
    dw: u32,
    ht: u32,
    scale: f64,
    snd: Sender<Pixel>,
    worker: u32,
    offset: Offset
}

macro_rules! SQR {
    ($x: tt) => {
        $x * $x
    };
    ($x: expr) => {
        ($x) * ($x)
    };
}


fn calculation(jb: Job) {
    let max_i: u32 = 0x1FF;
    let white: u32 = 0xFFFFFF;
    let offsetx = (jb.dw as f64) * jb.scale * 0.5;
    let offsety = (jb.ht as f64) * jb.scale * 0.5;
    // let mut cnt = Vec::new();
    for i in jb.start..jb.stop {
        if false && cfg!(debug_assertions) && i % 1000 == 0{
            println!("Worker: {}; Pixel calc: {};", jb.worker, i.clone());
        }
        let (px, py) = (i % jb.ht, i / jb.dw);
        let x = px as f64 * jb.scale - offsetx + jb.offset.x;
        let y = py as f64 * jb.scale - offsety + jb.offset.y;

        let p = SQR!(x - 0.25) + SQR!(y);
        let pc = 0.5 - 0.5 * y.atan2(x - 0.25).cos();

        if p < pc || px >= jb.dw || py >= jb.ht { // p < pc || 
            match jb.snd.send(Pixel {
                x: px,
                y: py,
                clr: Rgb([255, 255, 255]),
            }) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: by send data: {}", e);
                    exit(0x10);
                }
            }
            continue;
        }

        let (mut xp, mut yp) = (0.0, 0.0);
        let (mut xn, mut yn) = (0.0, 0.0);
        let mut n = 0;
        while n < max_i {
            xn = SQR!(xp) - SQR!(yp) + x;
            yn = 2.0 * xp * yp + y;
            if SQR!(xn) + SQR!(yn) > 4.0 {
                break;
            }
            n += 1;
            xp = xn;
            yp = yn;
        }
        // cnt.push(n);
        // let clr = n / max_i * white;
        match jb.snd.send(Pixel {
            x: px,
            y: py,
            clr: Rgb([
                0, //(clr >> 3 & 0xFF) as u8,
                (n >> 2 & 0xFF) as u8,
                (n & 0xFF) as u8,
            ]),
        }) {
            Ok(_) => {}
            Err(e) => {
                println!("Error: by send data: {}", e);
                exit(0x10);
            }
        };
    }
    // println!("Worker: {}", jb.worker);
    // for i in cnt{
    //     print!("{}, ", i);
    // }
}


fn read_tp<F: FromStr>()->F{
    let mut input_text = String::new();
    loop{
        println!("Please write: ");
        match io::stdin().read_line(&mut input_text){
            Ok(_)=>{
                match input_text.trim().parse::<F>(){
                    Ok(res)=>{
                        return res;
                    }
                    Err(_)=>{
                        println!("Failed to parse from stdin. Try again.")
                    }
                }
            },
            Err(_)=>{
                println!("Failed to read from stdin. Try again.");
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    println!("CPU: {}", num_cpus::get());
    println!("Physical CPU: {}", num_cpus::get_physical());
    let cpu = num_cpus::get() as u32;

    println!("Write image width px: ");
    let dw = read_tp::<u32>();

    println!("Write image heigh px: ");
    let ht = read_tp::<u32>();

    println!("Write scale in float: ");
    let scale = read_tp::<f64>();

    println!("Write x offset in float: ");
    let offsetx = read_tp::<f64>();

    println!("Write y offset in float: ");
    let offsety = read_tp::<f64>();

    let isize = dw * ht;

    let mut img = RgbImage::new(dw.clone(), ht.clone());
    let (tx, rx): (Sender<Pixel>, Receiver<Pixel>) = mpsc::channel();
    let chunk = isize / cpu;

    for i in 0..cpu {
        let snd = tx.clone();
        thread::spawn(move || {
            calculation(Job {
                start: i * chunk,
                stop: if i + 1 == cpu { isize } else { (i + 1) * chunk },
                dw: dw.clone(),
                ht: ht.clone(),
                scale: scale.clone(),
                snd: snd,
                worker: i.clone(),
                offset: Offset{
                    x: offsetx,
                    y: offsety
                }
            })
        });
    }
    let mut n = 0;
    let step = isize / 100;
    while let Ok(msg) = rx.recv() {
        if n % step == 0{
            println!("Draw pixel: {}/{}; {}%", n.clone(), isize.clone(), (n as f64)/(isize as f64)*100.0);
        }
        n += 1;
        if n >= isize{
            break;
        }
        if msg.x >= dw || msg.y >= ht{
            continue;
        }
        img.put_pixel(msg.x, msg.y, msg.clr);
        
    }
    let fimg = DynamicImage::ImageRgb8(img);
    match File::create("fract.pn") {
        Ok(mut f)=>{
            match fimg.write_to(&mut f, ImageOutputFormat::Png){
                Ok(_)=>{
                    println!("Success save");
                }
                Err(e)=>{
                    println!("Can`t save image: {}", e);
                    exit(0x02);
                }
            }
        },
        Err(e) => {
            println!("Can`t save image: {}", e);
            exit(0x01);
        }
    }
}
